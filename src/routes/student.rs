use crate::routes::login::Login;
use actix_identity::Identity;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;
use std::io;
use actix_web::http::StatusCode;
use actix_web_httpauth::extractors::basic::BasicAuth;
use serde::{Deserialize, Serialize};
use crate::{api_login, api_password, routes};
use crate::teachrec::TeachRec;
//use std::iter::Map;

#[derive(Debug, Deserialize, Serialize)]
pub struct Student {
    id: i16,
    #[serde(rename(deserialize = "ФИО"))]
    name: String,
}

const STUDENT_FILE: &str = "students.tsv";

pub fn read_students() -> csv::Result<HashMap<i16, String>> {
    csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(STUDENT_FILE)?
        .deserialize()
        .map(|res| res.map(|s: Student| (s.id, s.name)))
        .collect()
}

#[get("/students/hash")]
pub async fn students_hash(auth: BasicAuth, req: HttpRequest) -> actix_web::Result<impl Responder> {
    routes::user_agent_info(&req, "students/hash");
    // Access the username and password
    let username = auth.user_id();
    let password = auth.password().unwrap_or_default(); // password() returns Option<&str>

    // Implement your authentication logic here
    // For example, compare against hardcoded values or a database
    if *api_login == username && *api_password == password {
        let hash = sha256::try_digest(std::path::Path::new(STUDENT_FILE))?;
        Ok(HttpResponse::Ok().body(hash))
    } else {
        println!("no auth!");
        Ok(HttpResponse::Unauthorized().body("Unauthorized"))
    }
}

#[get("/students")]
pub async fn students(
    req: HttpRequest,
    query: web::Query<HashMap<String, String>>,
    user: Option<Identity>,
) -> impl Responder {
    routes::user_agent_info(&req, "students");
    if user.is_none() {
        println!("no auth!");
        return HttpResponse::Forbidden().finish();
    }
    let filter =
        query.get("filter")
            .filter(|s| s.chars().count() >= 2)
            .map(|s| s.to_uppercase());
    if filter.is_none() {
        println!("No filter!");
        return HttpResponse::BadRequest().finish();
    }
    let filter = filter.unwrap();

    match read_students() {
        Ok(students) => {
            let output: Vec<Student> =
                students
                    .iter()
                    .filter(|(_, name)| name.to_uppercase().starts_with(filter.to_uppercase().as_str()))
                    .map(|(&id, name)| Student { id, name: name.clone() })
                    .collect();

            HttpResponse::Ok().json(output)

        },
        Err(e) => {
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Get student's list failed: {:?}", e))
        }
    }
}
