use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    App(String),
    Rusqlite(String),
    StdIo(String),
    Ron(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::App(msg) => write!(f, "App error: {msg}"),
            Self::Rusqlite(msg) => write!(f, "Rusqlite error: {msg}"),
            Self::StdIo(msg) => write!(f, "StdIo error: {msg}"),
            Self::Ron(msg) => write!(f, "RON error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

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