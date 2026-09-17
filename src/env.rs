use std::io::{Read, Write};
use std::path::PathBuf;

use crate::engine::Engine;
use crate::error::{Error, Result};

pub fn cache_dir() -> Result<PathBuf> {
    directories::ProjectDirs::from("", "", "tt")
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .ok_or_else(|| Error::Config("could not determine cache directory".to_string()))
}

pub struct Env<'a> {
    pub out: &'a mut dyn Write,
    pub err: &'a mut dyn Write,
    pub stdin: &'a mut dyn Read,
    pub out_tty: bool,
    pub err_tty: bool,
    pub stdin_tty: bool,
    pub engine: Box<dyn Engine>,
    pub config_path: PathBuf,
    pub cache_dir: PathBuf,
}
