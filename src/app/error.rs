use std::fmt::{Display, Formatter};

#[cfg(not(target_arch = "wasm32"))]
use axum::{
    http::StatusCode as AxumStatusCode,
    response::{IntoResponse as IntoAxumResponse, Response as AxumResponse},
};

#[derive(Debug)]
pub enum Error {
    App(String),
    StdIo(String),
    Ron(String),

    #[cfg(not(target_arch = "wasm32"))]
    Rusqlite(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::App(msg) => write!(f, "App error: {msg}"),
            Self::StdIo(msg) => write!(f, "StdIo error: {msg}"),
            Self::Ron(msg) => write!(f, "RON error: {msg}"),
            
            #[cfg(not(target_arch = "wasm32"))]
            Self::Rusqlite(msg) => write!(f, "Rusqlite error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(not(target_arch = "wasm32"))]
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Rusqlite(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::StdIo(e.to_string())
    }
}

impl From<ron::Error> for Error {
    fn from(e: ron::Error) -> Self {
        Error::Ron(e.to_string())
    }
}

impl From<ron::error::SpannedError> for Error {
    fn from(e: ron::error::SpannedError) -> Self {
        Error::Ron(e.to_string())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::App(e)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl IntoAxumResponse for Error {
    fn into_response(self) -> AxumResponse {
        let status = match self {
            Error::App(_) => AxumStatusCode::BAD_REQUEST,
            Error::StdIo(_) => AxumStatusCode::INTERNAL_SERVER_ERROR,
            Error::Ron(_) => AxumStatusCode::INTERNAL_SERVER_ERROR,

            #[cfg(not(target_arch = "wasm32"))]
            Error::Rusqlite(_) => AxumStatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        Error::App(e.to_string())
    }
}