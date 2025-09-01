use actix_web::{get, put, delete, error, HttpResponse, Responder};
use actix_web::web::Path;

fn is_alphanumeric_underscore_dot(s: &str) -> bool {
    let dot_count = s.chars().filter(|c| *c == '.').count();
    !s.is_empty() &&
        s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') &&
        (dot_count == 1 || dot_count == 2)
}

fn check_file_name(file_name: &str) -> actix_web::Result<()> {
    if is_alphanumeric_underscore_dot(file_name) {
        Ok(())
    } else {
        Err(error::ErrorBadRequest("Invalid data provided"))
    }
}

#[put("/attendances")] // /api
pub async fn attendances() -> actix_web::Result<impl Responder> {
    Ok(HttpResponse::Ok().body("Under constructions"))
}

#[put("/attendance/{file}/{hash}")] // /api
pub async fn put_attendance(
    path: Path<(String, Option<String>)>,
    body: String
) -> actix_web::Result<impl Responder> {
    let (file, hash) = path.into_inner();
    put_attendance_with_hash(file, hash, body).await
}

#[put("/attendance/{file}")] // /api
pub async fn put_attendance_no_hash(
    file: Path<String>,
    body: String
) -> actix_web::Result<impl Responder> {
    put_attendance_with_hash(file.into_inner(), None, body).await
}

async fn put_attendance_with_hash(
    file: String,
    hash: Option<String>,
    body: String
) -> actix_web::Result<impl Responder> {
    println!("hash={:?}", hash);
    check_file_name(file.as_str())?;

    if let Some(hash_given) = hash {
        let hash_calculated = sha256::digest(body);
        println!("given: {hash_given} =?= calculated: {hash_calculated}");
        if hash_given != hash_calculated {
            return Err(error::ErrorForbidden("Wrong hash!"))
        }
    }

    Ok(HttpResponse::Ok().body("Under constructions"))
}

#[get("/attendance1/{file}")] // /api
pub async fn get_attendance(file: Path<String>) -> actix_web::Result<impl Responder> {
    check_file_name(file.as_str())?;
    Ok(HttpResponse::Ok().body("Under constructions"))
}

#[delete("/attendance")] // /api
pub async fn delete_attendance(file: Path<String>) -> actix_web::Result<impl Responder> {
    check_file_name(file.as_str())?;
    Ok(HttpResponse::Ok().body("Under constructions"))
}