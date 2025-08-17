use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest, HttpMessage};
use actix_identity::Identity;
use actix_session::storage::RedisSessionStore;

use ::captcha::Captcha;
use ::captcha::filters::Noise;
use actix_web::http::header::ContentType;
use tera::{Context, Tera};
use crate::routes::login::Login;
use crate::teachrec;
use crate::teachrec::TeachRec;

#[get("/")]
async fn index(request: HttpRequest, user: Option<Identity>) -> impl Responder {
    if let Some(user) = user {
        let body: String =
            format!("Добро пожаловать, {}!", TeachRec::name_only(user.id().unwrap())) +
                "<br><a href=\"logout\">Выйти</a>";
        let row: String =
            "<tr>".to_string() + &(1..70).map(|i| format!("<td>{i}</td>")).collect::<Vec<_>>().join("") + "</tr>\n";
        let table: String =
            format!(
                "\
                <div style=\"margin: 20px\">\
                <div style=\"overflow-x: auto; width: 100%;\">\
                <table border=\"1\" cellpadding=\"10\" \
                style=\"white-space: nowrap; border: medium solid; border-collapse: collapse;\">\
                {row}\
                {row}\
                </table>\
                </div>\
                </div>"
            );
        HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body + table.as_str())
    } else {
        // HttpResponse::Ok().content_type("text/html; charset=utf-8").body("Welcome Anonymous!".to_owned())
        web::Redirect::to("/login").temporary().respond_to(&request).map_into_boxed_body()
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
async fn login(request: HttpRequest, form: web::Form<Login>) -> impl Responder {
    //let teachers = rdr.deserialize().collect::<Vec<_>>();

    // Some kind of authentication should happen here
    // e.g. password-based, biometric, etc.
    // [...]

    match teachrec::TeachRec::find(form.into_inner()) {
        Some(rec) => {
            // attach a verified user identity to the active session
            Identity::login(&request.extensions(), rec.id_and_name()).unwrap();
            web::Redirect::to("/").see_other().respond_to(&request).map_into_boxed_body()
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