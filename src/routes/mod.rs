use actix_web::HttpRequest;
use actix_web::http::header::HeaderValue;
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web_httpauth::extractors::basic::BasicAuth;
use crate::{api_login, api_password};

pub mod index;
pub mod login;
pub mod teacher;
pub mod student;

// Write User-Agent information
pub fn user_agent_info(req: &HttpRequest, prefix: &str) {
    let user_agent_header: Option<&HeaderValue> = req.headers().get("User-Agent");

    if let Some(header_value) = user_agent_header {
        if let Ok(user_agent_str) = header_value.to_str() {
            println!("{prefix}: User agent: {user_agent_str}");
        }
    }
}

pub async fn basic_auth_validator(
    req: ServiceRequest,
    auth: BasicAuth,
) -> Result<ServiceRequest, (actix_web::error::Error, ServiceRequest)> {
    // Access the username and password
    let username = auth.user_id();
    let password = auth.password().unwrap_or_default(); // password() returns Option<&str>

    // Implement your authentication logic here
    // For example, compare against hardcoded values or a database
    if *api_login == username && *api_password == password {
        Ok(req)
    } else {
        Err((ErrorUnauthorized("Invalid credentials"), req))
    }
}
