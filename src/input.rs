use crate::error::{Error, Result};

#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    Text(String),
    File(String),
    Stdin,
}

pub fn resolve(text: Option<String>, file: Option<String>, stdin_is_tty: bool) -> Result<Source> {
    if let Some(path) = file {
        return Ok(Source::File(path));
    }
    match text {
        Some(t) if t == "-" => Ok(Source::Stdin),
        Some(t) => Ok(Source::Text(t)),
        None if stdin_is_tty => Err(Error::BadArgs("no text to translate".to_string())),
        None => Ok(Source::Stdin),
    }
}
