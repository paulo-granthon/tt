use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::exit;

use tt::config::Config;
use tt::engine::default_engine;
use tt::env::{cache_dir, Env};
use tt::error::Result;

fn paths() -> Result<(PathBuf, PathBuf)> {
    Ok((Config::path()?, cache_dir()?))
}

fn main() {
    let (config_path, cache_dir) = match paths() {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("tt: {e}");
            exit(e.exit_code());
        }
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut out, mut err, mut stdin) = (std::io::stdout(), std::io::stderr(), std::io::stdin());
    let (out_tty, err_tty, stdin_tty) = (out.is_terminal(), err.is_terminal(), stdin.is_terminal());
    let mut env = Env {
        out: &mut out,
        err: &mut err,
        stdin: &mut stdin,
        out_tty,
        err_tty,
        stdin_tty,
        engine: default_engine(),
        config_path,
        cache_dir,
    };
    exit(tt::app::run(&args, &mut env));
}
