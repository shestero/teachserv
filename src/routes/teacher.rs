use std::collections::HashMap;
use std::fs;
use crate::{attendance::Attendance, teachrec::TeachRec};
use actix_identity::Identity;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use actix_web::web::Redirect;
use serde::Deserialize;
use tera::{Context, Tera};

#[get("/table/{name}")]
async fn table_form(
    name: web::Path<String>,
    request: HttpRequest,
    user: Option<Identity>,
    tera : web::Data<Tera>
) -> impl Responder {
    if let Some(user) = user {
        // TODO: check that name correspond to identity !!
        //   (identity "0" is admin)

        let (id, th_name) = TeachRec::split_id_and_name(user.id().unwrap());
        let id = id.parse().map_or(id, |id: i32| format!("{:04}", id));

        let file_name = format!("attendance/inbox/{}.tsv", name);
        let file_name = file_name.as_str();
        let tbody = Attendance::read(file_name).map_or(
            format!("Не удалось прочитать или найти таблицу {file_name}"),
            |attendance| {
                attendance
                    .html(&tera)
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
        println!("no auth! redirect to login... Request: {:?}", &request);
        // HttpResponse::Ok().content_type("text/html; charset=utf-8").body("Welcome Anonymous!".to_owned())
        web::Redirect::to("/login")
            .temporary()
            .respond_to(&request)
            .map_into_boxed_body()
    }
}


#[derive(Deserialize)]
struct SearchParams {
    seal: Option<String>,
}

impl SearchParams {
    pub fn seal(&self) -> bool {
        self.seal == Some(String::from("yes"))
    }
}

#[post("/table/{name}")]
async fn table(
    name: web::Path<String>,
    request: HttpRequest,
    params: web::Query<SearchParams>,
    body: web::Bytes,
    user: Option<Identity>,
) -> impl Responder {
    if let Some(user) = user {
        // TODO: check that name correspond to identity !!

        let body_str = match String::from_utf8(body.to_vec()) {
            Ok(s) => s,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to parse body: {}", e)),
        };

        let parsed_form: HashMap<String, String> =
            form_urlencoded::parse(body_str.as_bytes())
                .into_owned()
                .collect();

        let seal: bool = params.seal();
        println!("sealed={:?}", seal); // todo

        // Now 'parsed_form' contains a vector of (key, value) tuples
        // You can iterate through it to access individual parameters
        /*
        for (key, value) in parsed_form.iter().filter(|(k, _)| k.starts_with("N")) {
            println!("[1] Key: {}, Value: {}", key, value);
        }
        for (key, value) in parsed_form.iter().filter(|(k, _)| k.starts_with("IN")) {
            println!("[2] Key: {}, Value: {}", key, value);
        }
        */

        let file_name = format!("attendance/inbox/{}.tsv", name);
        let file_name = file_name.as_str();
        let mut attendance = Attendance::read(file_name).unwrap(); // todo
        let dr = attendance.date_range();

        let students =
            attendance
                .students
                .iter()
                .map(|(&id, (st_name, _))| {
                    let id_id = format!("IN{id:05}");
                    let id_parsed: Option<i32> =
                        parsed_form
                            .get(id_id.as_str())
                            .and_then(|s| s.parse().ok());
                    if let Some(id_parsed) = id_parsed {
                        assert!(id == id_parsed);
                    }
                    (id, Some(st_name))
                })
                .chain(Attendance::blank_range().map(|id| (id, None)))
                .filter_map(|(id, st_name)| {
                    let st_id: i32 =
                        parsed_form.get(format!("IN{id:05}").as_str())
                            .and_then(|s| s.parse().ok())?;
                    let st_name =
                        parsed_form.get(format!("N{id:05}").as_str())
                            .filter(|s| !s.is_empty())
                            .or(st_name)?
                            .to_owned();
                    Some((id, st_id, st_name))
                })
                .map(|(id, st_id, st_name)| {
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
                    (st_id, (st_name, marks))
                })
                .collect();

        attendance.students = students;

        let file_name = format!("{}.tsv", name); // todo
        let file_name_open = format!("attendance/inbox/{file_name}");
        let file_name_closed = format!("attendance/outbox/{file_name}");
        attendance.write(file_name_open.as_str());

        let origin = request.clone().uri().path().to_string();
        let redirect = if seal {
            fs::rename(file_name_open.as_str(), file_name_closed.as_str()).expect("Cannot move!!"); // todo
            String::from("/")
        } else { origin };
        Redirect::to(redirect).see_other().respond_to(&request).map_into_boxed_body()

    } else {
        println!("no auth! redirect to login... Request: {:?}", &request);
        // HttpResponse::Ok().content_type("text/html; charset=utf-8").body("Welcome Anonymous!".to_owned())
        Redirect::to("/login").temporary().respond_to(&request).map_into_boxed_body()
    }
}