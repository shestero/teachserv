use actix_web::{get, put, delete, error, HttpResponse, Responder, web};
use actix_web::web::{JsonBody, Path};
use std::fs;
use log::*;

use crate::filerec::FileRec;
use crate::files_with_age;

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

#[get("/attendances/{direction}")] // /api
pub async fn attendances(direction: Path<String>) -> actix_web::Result<impl Responder> {
    if direction.as_str() != "inbox" && direction.as_str() != "outbox" {
        let msg = format!("Wrong direction: {}!", direction.as_str());
        log::error!("{msg}");
        return Err(error::ErrorMethodNotAllowed(msg))
    }

    let folder = format!("attendance/{direction}");
    println!("mask={folder}");
    let files =
        files_with_age(folder.as_str())?
            .into_iter()
            .filter_map(|r @ FileRec { file: _, age } | {
                let _ext = r.file.extension().filter(|&ext| ext == "tsv")?;
                let file = r.file.file_name()?.to_str()?;
                Some(file.to_owned())
            })
            .collect::<Vec<_>>();

    Ok(web::Json::<Vec<String>>(files))
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
        let hash_calculated = sha256::digest(&body);
        println!("given: {hash_given} =?= calculated: {hash_calculated}");
        if hash_given != hash_calculated {
            return Err(error::ErrorForbidden("Wrong hash!"))
        }
    }

    let file_path = format!("attendance/inbox/{file}");
    if fs::exists(&file_path)? {
        // return Err(error::ErrorNotFound("File already exists"))
        warn!("Warning: file {} already exists", &file_path);
    }
    println!("file_path={file_path}");
    fs::write(&file_path, body)?;

    Ok(HttpResponse::Ok().body("OK"))
}

#[get("/attendance/outbox/{file}")] // /api
pub async fn get_attendance(file: Path<String>) -> actix_web::Result<impl Responder> {
    check_file_name(file.as_str())?;

    let file_path = format!("attendance/outbox/{file}");
    let contents = fs::read_to_string(file_path)?;
    Ok(HttpResponse::Ok().body(contents))
}

#[delete("/attendance/{direction}/{file}")] // /api
pub async fn delete_attendance(params: Path<(String, String)>) -> actix_web::Result<impl Responder> {
    let (direction, file) = params.into_inner();
    if direction.as_str() != "inbox" && direction.as_str() != "outbox" {
        let msg = format!("Wrong direction: {}!", direction.as_str());
        log::error!("{msg}");
        return Err(error::ErrorMethodNotAllowed(msg))
    }

    check_file_name(file.as_str())?;

    let file_path = format!("attendance/{direction}/{file}");
    if !fs::exists(&file_path)? {
        // return Err(error::ErrorNotFound("File not exists"))
        warn!("Warning: file {} not exists", &file_path);
    }

    let _ = fs::remove_file(&file_path)?;

    Ok(HttpResponse::Ok().body("OK"))
}