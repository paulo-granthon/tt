use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};

const INSTALL_URL: &str = "https://raw.githubusercontent.com/paulo-granthon/tt/main/install.sh";

pub fn run() -> Result<()> {
    let script = ureq::get(INSTALL_URL)
        .call()
        .map_err(|e| Error::Network(e.to_string()))?
        .into_string()
        .map_err(|e| Error::Network(e.to_string()))?;

    let mut command = Command::new("sh");
    command.arg("-s").stdin(Stdio::piped());
    if let Some(prefix) = install_prefix() {
        command.env("PREFIX", prefix);
    }

    let mut child = command
        .spawn()
        .map_err(|e| Error::Config(format!("could not run installer: {e}")))?;
    child
        .stdin
        .take()
        .ok_or_else(|| Error::Config("could not pipe to installer".to_string()))?
        .write_all(script.as_bytes())
        .map_err(|e| Error::Config(e.to_string()))?;

    if child.wait().map_err(|e| Error::Config(e.to_string()))?.success() {
        Ok(())
    } else {
        Err(Error::Config("update failed".to_string()))
    }
}

fn install_prefix() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    exe.parent()?.parent().map(|p| p.to_path_buf())
}
