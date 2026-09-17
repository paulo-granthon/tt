use std::io::{Read, Write};
use std::path::PathBuf;

use crate::engine::Engine;

pub struct Env<'a> {
    pub out: &'a mut dyn Write,
    pub err: &'a mut dyn Write,
    pub stdin: &'a mut dyn Read,
    pub out_tty: bool,
    pub err_tty: bool,
    pub stdin_tty: bool,
    pub engine: Box<dyn Engine>,
    pub config_path: PathBuf,
}
