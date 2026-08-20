use std::fmt;

#[derive(Debug)]
pub enum Error {
    BadArgs(String),
    UnknownLanguage(String),
    Network(String),
    Parse(String),
    Config(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::BadArgs(_) => 2,
            Error::UnknownLanguage(_) => 3,
            Error::Network(_) => 4,
            Error::Parse(_) => 5,
            Error::Config(_) => 6,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadArgs(m) => write!(f, "{m}"),
            Error::UnknownLanguage(m) => write!(f, "unknown language: {m}"),
            Error::Network(m) => write!(f, "network error: {m}"),
            Error::Parse(m) => write!(f, "could not read translation response: {m}"),
            Error::Config(m) => write!(f, "config error: {m}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
