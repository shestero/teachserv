use std::arch::x86_64;
use std::fs::DirEntry;
use std::{fs, io};
use std::ffi::OsStr;
use std::path::Path;
use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest, HttpMessage};
use actix_identity::Identity;
use actix_session::storage::RedisSessionStore;

use ::captcha::Captcha;
use ::captcha::filters::Noise;
use actix_web::http::header::ContentType;
use tera::{Context, Tera};
use crate::attendance::Attendance;
use crate::routes::login::Login;
use crate::routes::user_agent_info;
use crate::teachrec::TeachRec;


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
async fn index(req: HttpRequest, user: Option<Identity>) -> impl Responder {
    user_agent_info(&req, "index");
    if let Some(user) = user {
        let mut tera = Tera::new("templates/**/*").unwrap();
        tera.autoescape_on(vec![]);

        let (id, name) = TeachRec::split_id_and_name(user.id().unwrap());
        let id = id.parse().map_or(id, |id: i32| format!("{:04}", id));
        let opens =
            read_attendance_dir(id.as_str())("attendance/inbox")
                .unwrap_or(Vec::new()); // todo: report errors

        let mut context = Context::new();
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
async fn login_form() -> impl Responder {
    let mut tera = Tera::new("templates/**/*").unwrap();
    tera.autoescape_on(vec![]);
    let mut context = Context::new();

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
        None =>
            HttpResponse::Ok().body("Wrong login/password")
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
    let c = c.add_chars(5);
    println!("chars={}", c.chars_as_string());
    let png =
        c
            .apply_filter(Noise::new(0.1))
            .view(220, 120)
            .as_png()
            .expect("Error: cannot generate PNG!");

    HttpResponse::Ok()
        .insert_header(ContentType::png())
        .body(png)
}