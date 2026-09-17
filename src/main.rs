use std::io::IsTerminal;
use std::process::exit;

use tt::config::Config;
use tt::engine::default_engine;
use tt::env::Env;

fn main() {
    let config_path = match Config::path() {
        Ok(path) => path,
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
    };
    exit(tt::app::run(&args, &mut env));
}
