use crate::browser;
use crate::cli::Translate;
use crate::config::Config;
use crate::engine::{Query, Translation};
use crate::env::Env;
use crate::error::{Error, Result};
use crate::input::{self, Source};
use crate::lang;
use crate::render::{default_hint, footer, render, Filter, Meta};

pub fn run(request: Translate, env: &mut Env) -> Result<i32> {
    let Translate {
        profile,
        sl,
        tl,
        text,
        file,
        filter,
    } = request;
    let all_default = profile.is_none() && sl.is_none() && tl.is_none() && text.is_some() && file.is_none();
    let config = Config::load_from(&env.config_path)?;
    let resolved = config.resolve(profile.as_deref(), sl.as_deref(), tl.as_deref())?;
    let input = read_input(text, file, env)?;
    let translation = if lang::is_identity(&resolved.sl, &resolved.tl) {
        Translation {
            primary: input.clone(),
            detected_source: Some(resolved.sl.clone()),
            ..Translation::default()
        }
    } else {
        let err = &mut *env.err;
        env.engine.translate(
            Query {
                sl: &resolved.sl,
                tl: &resolved.tl,
                text: &input,
            },
            &mut |line| {
                let _ = writeln!(err, "{line}");
            },
        )?
    };
    let meta = Meta {
        sl: &resolved.sl,
        tl: &resolved.tl,
        text: &input,
        engine: env.engine.name(),
    };
    let _ = writeln!(env.out, "{}", render(&translation, filter, env.out_tty, &meta));
    if env.out_tty && !matches!(filter, Filter::Quiet | Filter::Json) {
        let hint = (all_default && filter == Filter::Full).then(|| default_hint(&resolved.sl, &resolved.tl));
        let url = browser::translate_url(&resolved.sl, &resolved.tl, &input);
        let _ = writeln!(env.err, "{}", footer(env.err_tty, hint.as_deref(), &url));
    }
    Ok(0)
}

fn read_input(text: Option<String>, file: Option<String>, env: &mut Env) -> Result<String> {
    let raw = match input::resolve(text, file, env.stdin_tty)? {
        Source::Text(t) => t,
        Source::File(path) => std::fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("could not read {path}: {e}")))?,
        Source::Stdin => {
            let mut buffer = String::new();
            env.stdin
                .read_to_string(&mut buffer)
                .map_err(|e| Error::Config(e.to_string()))?;
            buffer
        }
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(Error::BadArgs("no text to translate".to_string()));
    }
    Ok(trimmed.to_string())
}
