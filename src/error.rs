//use serde_json::error::Error as SerdeError;
use axum::{
    body::{self},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    NotFound,
//    BadInsert,
    DeError(bson::de::Error),
    SerError(bson::ser::Error),
    SerdeJson(serde_json::Error),
    MongoDB(mongodb::error::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NotFound => f.write_str("{\"error\": \"Not found\"}"),
//            Error::BadInsert => f.write_str("{\"error\": \"Unable to insert new note\"}"),
            Error::DeError(ref err) => write!(f, "{{\"error\": \"{}\"}}", err),
            Error::SerError(ref err) => write!(f, "{{\"error\": \"{}\"}}", err),
            Error::SerdeJson(ref err) => write!(f, "{{\"error\": \"{}\"}}", err),
            Error::MongoDB(ref err) => write!(f, "{{\"error\": \"{}\"}}", err),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let payload = self.to_string();
        let body = body::boxed(body::Full::from(payload));

        let status_code = match self {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::DeError(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Response::builder().status(status_code).body(body).unwrap()
    }
}

impl From<bson::de::Error> for Error {
    fn from(err: bson::de::Error) -> Error {
        Error::DeError(err)
    }
}

impl From<bson::ser::Error> for Error {
    fn from(err: bson::ser::Error) -> Error {
        Error::SerError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::SerdeJson(err)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(err: mongodb::error::Error) -> Error {
        Error::MongoDB(err)
    }
}
