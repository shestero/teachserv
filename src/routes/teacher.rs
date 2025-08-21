use std::collections::HashMap;
use crate::{attendance::Attendance, teachrec::TeachRec};
use actix_identity::Identity;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use tera::{Context, Tera};

#[get("/table/{name}")]
async fn table_form(
    name: web::Path<String>,
    request: HttpRequest,
    user: Option<Identity>,
) -> impl Responder {
    if let Some(user) = user {
        // TODO: check that name correspond to identity !!

        let mut tera = Tera::new("templates/**/*").unwrap();
        tera.autoescape_on(vec![]);

        let (id, th_name) = TeachRec::split_id_and_name(user.id().unwrap());
        let id = id.parse().map_or(id, |id: i32| format!("{:04}", id));

        /*
        let opens = crate::routes::index::read_attendance_dir(id.as_str())("attendance/open")
            .unwrap_or(Vec::new()); // todo: report errors
        */

        let file_name = format!("attendance/open/{}.tsv", name);
        let file_name = file_name.as_str();
        let tbody = Attendance::read(file_name).map_or(
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
        context.insert("table", tbody.as_str());

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

#[post("/table/{name}")]
async fn table(
    name: web::Path<String>,
    request: HttpRequest,
    body: web::Bytes,
    user: Option<Identity>,
) -> impl Responder {
    if let Some(user) = user {
        // TODO: check that name correspond to identity !!

        println!("name={name}"); // todo
        let body_str = match String::from_utf8(body.to_vec()) {
            Ok(s) => s,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to parse body: {}", e)),
        };

        let parsed_form: HashMap<String, String> =
            form_urlencoded::parse(body_str.as_bytes())
                .into_owned()
                .collect();

        let seal: bool = parsed_form.get("seal").map_or(false, |v| v == "on");
        println!("sealed={seal}");

        // Now 'parsed_form' contains a vector of (key, value) tuples
        // You can iterate through it to access individual parameters
        for (key, value) in parsed_form.iter().filter(|(k, _)| k.starts_with("N")) {
            println!("Key: {}, Value: {}", key, value);
        }

        let file_name = format!("attendance/open/{}.tsv", name);
        let file_name = file_name.as_str();
        let mut attendance = Attendance::read(file_name).unwrap(); // todo
        let dr = attendance.date_range();

        let students =
            attendance
                .students
                .iter()
                .map(|(&id, (name, _))| {
                    let marks: Vec<String> =
                        dr
                            .iter()
                            .map(|d| {
                                let field: String = format!("S{id:05}D{d}");
                                parsed_form
                                    .get(&field)
                                    .map(|v| v.parse::<i16>().map_or(String::new(), |_| v.clone()))
                                    .unwrap_or(String::new())

                            })
                            .collect();
                    if id > 0 { (id, (name.clone(), marks)) } else {
                        let name: String = parsed_form.get(format!("N{id:05}").as_str()).map_or(String::new(), |s| s.clone());
                        (id, (name, marks))
                    }
                })
                .collect();

        attendance.students = students;

        let file_name = format!("attendance/{}.tsv", name); // todo
        attendance.write(file_name.as_str());

        HttpResponse::Ok().body("ok")

    } else {
        println!("no auth! redirect to login...");
        // HttpResponse::Ok().content_type("text/html; charset=utf-8").body("Welcome Anonymous!".to_owned())
        web::Redirect::to("/login")
            .temporary()
            .respond_to(&request)
            .map_into_boxed_body()
    }
}