use actix_web::HttpRequest;
use actix_web::http::header::HeaderValue;

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


