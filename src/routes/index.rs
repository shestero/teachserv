use std::arch::x86_64;
use std::fs::DirEntry;
use std::{fs, io};
use std::ffi::OsStr;
use std::path::Path;

use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest, HttpMessage};
use actix_identity::Identity;
//use actix_session::storage::RedisSessionStore;

use ::captcha::{Captcha, CaptchaName, Difficulty, Geometry};
use ::captcha::filters::{Cow, Noise, Wave};
use base64::Engine;

use actix_web::http::header::ContentType;
use tera::{Context, Tera};
use crate::attendance::Attendance;
use crate::routes::login::Login;
use crate::routes::user_agent_info;
use crate::teachrec::TeachRec;
use crate::wrong_pwd::{need_captcha, time_since_last_wrong_pwd, update_wrong_pwd_timestamp};

fn read_entity<'a>(th_id: &'a str) -> impl Fn(DirEntry) -> Option<Attendance> + 'a {
    move |entry| {
        let path = entry.path();
        if path.extension() != Some(OsStr::new("tsv")) {
            None
        } else {
            path.file_stem()
                .and_then(|name| name.to_str())
                .filter(|name| String::from("0000") == th_id || name.starts_with(th_id))
                .map(|_| Attendance::read(path.to_str().unwrap()))
                .and_then(|r| r.ok())
        }
    }
}

pub fn read_attendance_dir<'a>(th_id: &'a str) -> impl Fn(&str) -> io::Result<Vec<Attendance>> + 'a {
    |path| {
        let path = Path::new(path);
        Ok(
            fs::read_dir(path)?
                .filter_map(|r| r.ok())
                .filter_map(read_entity(th_id))
                .collect::<Vec<_>>()
        )
    }
}

#[get("/")]
async fn index(
    req: HttpRequest,
    user: Option<Identity>,
    tera: web::Data<Tera>
) -> impl Responder {
    user_agent_info(&req, "index");
    if let Some(user) = user {

        let (id, name) = TeachRec::split_id_and_name(user.id().unwrap());
        let id = id.parse().map_or(id, |id: i32| format!("{:04}", id));
        let is_admin: bool = id.parse() == Ok(0);
        let opens =
            read_attendance_dir(id.as_str())("attendance/inbox")
                .unwrap_or(Vec::new()); // todo: report errors

        let mut context = Context::new();
        context.insert("is_admin", &is_admin);
        context.insert("name", format!("{name} (номер {id})").as_str());
        context.insert("opens", &opens);

        let body =
            tera
                .render("index-teacher.html", &context)
                .expect("Cannot render index-teacher template!");

        HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body)

    } else {
        println!("no auth! redirect to login... Request: {:?}", &req);
        // HttpResponse::Ok().content_type("text/html; charset=utf-8").body("Welcome Anonymous!".to_owned())
        web::Redirect::to("/login").temporary().respond_to(&req).map_into_boxed_body()
    }
}

#[get("/login")]
async fn login_form(tera: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();

    println!("use captcha? {}, last wrong pwd: {:?}",
             need_captcha(),
             time_since_last_wrong_pwd().map(|res| res.map(humantime::format_duration))
    );

    if need_captcha() {
        println!("Captcha is required!");
        context.insert("captcha", captcha_internal(&tera).as_str());
    }

    let body =
        tera
            .render("login.html", &context)
            .expect("Cannot render login template!");

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body)
}

#[post("/login")]
async fn login(req: HttpRequest, form: web::Form<Login>) -> impl Responder {
    user_agent_info(&req, "login");
    //let teachers = rdr.deserialize().collect::<Vec<_>>();

    // Some kind of authentication should happen here
    // e.g. password-based, biometric, etc.
    // [...]

    if need_captcha() {
        println!("Captcha is required!");

        if !form.check_captcha() {
            return HttpResponse::Ok().body("Wrong captcha!");
        }
    }

    match TeachRec::find(form.into_inner()) {
        Some(rec) => {
            // attach a verified user identity to the active session
            println!("rec found: {:?}", rec);

            Identity::login(&req.extensions(), rec.id_and_name())
                .err()
                .iter()
                .for_each(|e| println!("login error: {:?}", e));

            web::Redirect::to("/")
                .see_other()
                .respond_to(&req)
                .map_into_boxed_body()
        },
        None => {
            if let Err(error) = update_wrong_pwd_timestamp() {
                println!("Cannot set last wrong password timestamp: {error}");
            }

            HttpResponse::Ok().body("Wrong login/password")
        }
    }

}

#[get("/logout")]
async fn logout(user: Identity) -> impl Responder {
    user.logout();
    web::Redirect::to("/")
}

#[get("/captcha")]
async fn captcha() -> impl Responder {
    let mut c = Captcha::new();
    let c = c.add_chars(7);
    let token = c.chars_as_string();
    println!("chars={}", c.chars_as_string());
    let png =
        c
            .apply_filter(Noise::new(0.2))
            .apply_filter(Wave::new(2.0, 20.0))
            .view(340, 120)
            .apply_filter(
                Cow::new()
                    .min_radius(30)
                    .max_radius(50)
                    .circles(2)
                    .area(Geometry::new(30, 250, 50, 70)),
            )
            .as_png()
            .expect("Error: cannot generate PNG!");

    HttpResponse::Ok()
        .insert_header(ContentType::png())
        .insert_header(("X-Captcha-Token", token))
        .body(png)
}

fn captcha_internal(tera: &Tera) -> String {
    let mut c = Captcha::new();
    let c = c.add_chars(7);
    let token = c.chars_as_string();
    println!("[int] chars={}", c.chars_as_string());
    let png =
        c
            .apply_filter(Noise::new(0.2))
            .apply_filter(Wave::new(2.0, 20.0))
            .view(340, 120)
            .apply_filter(
                Cow::new()
                    .min_radius(30)
                    .max_radius(50)
                    .circles(2)
                    .area(Geometry::new(30, 250, 50, 70)),
            )
            .as_png()
            .expect("Error: cannot generate PNG!");

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png);

    /*
    let mut buf = Cursor::new(Vec::new());
    png.write_to(&mut buf, ImageOutputFormat::Png).expect("Cannot write PNG into buf!");
    let b64 = base64::encode(buf.into_inner());
    */

    let mut context = Context::new();
    context.insert("b64", b64.as_str());
    context.insert("token", token.as_str());

    tera
        .render("captcha.html", &context)
        .expect("Cannot render captcha template!")

}