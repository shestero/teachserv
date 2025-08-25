use crate::routes::login::Login;
use actix_identity::Identity;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;
use std::io;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use crate::routes;
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

pub fn students_hash() -> Result<String, io::Error> {
    sha256::try_digest(std::path::Path::new(STUDENT_FILE))
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
