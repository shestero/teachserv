use crate::{attendance::Attendance, teachrec::TeachRec};
use actix_identity::Identity;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use tera::{Context, Tera};

#[get("/table/{name}")]
async fn table(
    name: web::Path<String>,
    request: HttpRequest,
    user: Option<Identity>,
) -> impl Responder {
    if let Some(user) = user {
        let mut tera = Tera::new("templates/**/*").unwrap();
        tera.autoescape_on(vec![]);

        let (id, th_name) = TeachRec::split_id_and_name(user.id().unwrap());
        let id = id.parse().map_or(id, |id: i32| format!("{:04}", id));

        let opens = crate::routes::index::read_attendance_dir(id.as_str())("attendance/open")
            .unwrap_or(Vec::new()); // todo: report errors

        let file_name = format!("attendance/open/{}.tsv", name);
        let file_name = file_name.as_str();
        let table = Attendance::read(file_name).map_or(
            format!("Не удалось прочитать или найти таблицу {file_name}"),
            |attendance| {
                attendance
                    .html()
                    .unwrap_or(format!("Не удалось нарисовать таблицу {file_name}"))
            },
        );

        let mut context = Context::new();
        context.insert("name", name.as_str());
        context.insert("teacher", format!("{th_name} (номер {id})").as_str());
        context.insert("table", table.as_str());

        let body = tera
            .render("table-open.html", &context)
            .expect("Cannot render table-open template!");

        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body)
    } else {
        println!("no auth! redirect to login...");
        // HttpResponse::Ok().content_type("text/html; charset=utf-8").body("Welcome Anonymous!".to_owned())
        web::Redirect::to("/login")
            .temporary()
            .respond_to(&request)
            .map_into_boxed_body()
    }
}
