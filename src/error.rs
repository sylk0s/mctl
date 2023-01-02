use warp::{
    Rejection,
    reject::Reject,
    Reply,
    reply,
    http::StatusCode
};
use std::convert::Infallible;
use serde::Serialize;

#[derive(Debug)]
pub struct NotRegistered {
    pub id: String,
}
impl Reject for NotRegistered {}

#[derive(Debug)]
pub struct Dummy;
impl Reject for Dummy {}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

#[derive(Debug)]
pub struct Error {
    reason: String,
}
impl Error {
    pub fn from(reason: &str) -> Error {
        Error {
            reason: reason.to_string()
        }
    }
}
impl Reject for Error {}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else if let Some(NotRegistered {id}) = err.find() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("Server is not registered: {id}");
    } else if let Some(Error {reason}) = err.find() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("Server Error: {reason}");
    } else {
        code = StatusCode::IM_A_TEAPOT;
        message = "Unhandled Rejection???".to_string();
    }

    let json = reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code)) 
}
