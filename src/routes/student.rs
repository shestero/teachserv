use actix_identity::Identity;
use actix_web::{get, put, web, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;
use std::fs;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use crate::routes;

#[derive(Debug, Deserialize, Serialize)]
pub struct Student {
    id: i16,
    #[serde(rename(deserialize = "ФИО"))]
    name: String,
}

pub const STUDENTS_FILE: &str = "students.tsv";
pub const TEACHERS_FILE: &str = "teachers.tsv";

pub fn read_students() -> csv::Result<HashMap<i16, String>> {
    csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(STUDENTS_FILE)?
        .deserialize()
        .map(|res| res.map(|s: Student| (s.id, s.name)))
        .collect()
}

#[get("/students/hash")] // /api
pub async fn students_hash() -> actix_web::Result<impl Responder> {
    let hash = sha256::try_digest(std::path::Path::new(STUDENTS_FILE))?;
    Ok(HttpResponse::Ok().body(hash))
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

#[put("/students")]
pub async fn put_students(body: String) -> impl Responder {
    fs::write(STUDENTS_FILE, body).map(|_| "OK")
}

// copy-paste /teachers/hash
#[get("/teachers/hash")] // /api
pub async fn teachers_hash() -> actix_web::Result<impl Responder> {
    let hash = sha256::try_digest(std::path::Path::new(TEACHERS_FILE))?;
    Ok(HttpResponse::Ok().body(hash))
}

// copy-paste put /teachers
#[put("/teachers")]
pub async fn put_teachers(body: String) -> impl Responder {
    fs::write(TEACHERS_FILE, body).map(|_| "OK")
}