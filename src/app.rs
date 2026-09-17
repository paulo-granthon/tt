
use crate::cli::{parse, Command};
use crate::commands::{help, profile, translate, update};
use crate::env::Env;
use crate::error::Result;

pub fn run(args: &[String], env: &mut Env) -> i32 {
    match dispatch(args, env) {
        Ok(code) => code,
        Err(e) => {
            let _ = writeln!(env.err, "tt: {e}");
            e.exit_code()
        }
    }
}

fn dispatch(args: &[String], env: &mut Env) -> Result<i32> {
    match parse(args)? {
        Command::Help => help::print(help::help(env.out_tty), env),
        Command::Languages => help::print(help::languages(env.out_tty), env),
        Command::Update => update::run(env),
        Command::Translate(request) => translate::run(request, env),
        Command::DefaultShow => profile::default_show(env),
        Command::DefaultSet { sl, tl } => profile::default_set(sl.as_deref(), tl.as_deref(), env),
        Command::ProfileAdd { name, sl, tl } => profile::add(&name, &sl, &tl, env),
        Command::ProfileList => profile::list(env),
        Command::ProfileDelete { name } => profile::delete(&name, env),
        Command::ProfilePatch {
            name,
            sl,
            tl,
            new_name,
        } => profile::patch(&name, sl.as_deref(), tl.as_deref(), new_name.as_deref(), env),
    }
}
