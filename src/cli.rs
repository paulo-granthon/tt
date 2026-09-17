use crate::error::{Error, Result};
use crate::render::Filter;

#[derive(Debug, PartialEq, Eq)]
pub struct Translate {
    pub profile: Option<String>,
    pub sl: Option<String>,
    pub tl: Option<String>,
    pub text: Option<String>,
    pub file: Option<String>,
    pub filter: Filter,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Translate(Translate),
    DefaultShow,
    DefaultSet {
        sl: Option<String>,
        tl: Option<String>,
    },
    ProfileAdd {
        name: String,
        sl: String,
        tl: String,
    },
    ProfileList,
    Languages,
    Update,
    ProfileDelete {
        name: String,
    },
    ProfilePatch {
        name: String,
        sl: Option<String>,
        tl: Option<String>,
        new_name: Option<String>,
    },
    Help,
}

fn split_kv(token: &str) -> Option<(&str, &str)> {
    token.split_once('=')
}

pub fn parse(args: &[String]) -> Result<Command> {
    match args.first().map(String::as_str) {
        None => Ok(Command::Help),
        Some("help" | "-h" | "--help") => Ok(Command::Help),
        Some("languages" | "langs") => {
            if args.len() > 1 {
                return Err(Error::BadArgs("'languages' takes no arguments".to_string()));
            }
            Ok(Command::Languages)
        }
        Some("update") => {
            if args.len() > 1 {
                return Err(Error::BadArgs("'update' takes no arguments".to_string()));
            }
            Ok(Command::Update)
        }
        Some("default") => parse_default(&args[1..]),
        Some("profile") => parse_profile(&args[1..]),
        _ => parse_translate(args),
    }
}

fn parse_default(rest: &[String]) -> Result<Command> {
    let (mut sl, mut tl) = (None, None);
    for token in rest {
        match split_kv(token) {
            Some(("sl", v)) => sl = Some(v.to_string()),
            Some(("tl", v)) => tl = Some(v.to_string()),
            _ => return Err(Error::BadArgs(format!("unexpected argument to 'default': {token}"))),
        }
    }
    if sl.is_none() && tl.is_none() {
        Ok(Command::DefaultShow)
    } else {
        Ok(Command::DefaultSet { sl, tl })
    }
}

fn parse_profile(rest: &[String]) -> Result<Command> {
    let action = rest.first().map(String::as_str).ok_or_else(|| {
        Error::BadArgs("profile requires a subcommand: add|list|delete|patch".to_string())
    })?;
    let tail = &rest[1..];
    match action {
        "list" | "ls" => {
            if !tail.is_empty() {
                return Err(Error::BadArgs("'profile list' takes no arguments".to_string()));
            }
            Ok(Command::ProfileList)
        }
        "add" => parse_add(tail),
        "delete" | "rm" => {
            let name = tail
                .first()
                .ok_or_else(|| Error::BadArgs("'profile delete' requires a name".to_string()))?;
            if tail.len() > 1 {
                return Err(Error::BadArgs("too many arguments to 'profile delete'".to_string()));
            }
            Ok(Command::ProfileDelete { name: name.clone() })
        }
        "patch" => parse_patch(tail),
        other => Err(Error::BadArgs(format!("unknown profile subcommand: {other}"))),
    }
}

fn parse_add(tail: &[String]) -> Result<Command> {
    let (mut name, mut sl, mut tl) = (None, None, None);
    for token in tail {
        match split_kv(token) {
            Some(("sl", v)) => sl = Some(v.to_string()),
            Some(("tl", v)) => tl = Some(v.to_string()),
            Some((key, _)) => return Err(Error::BadArgs(format!("unexpected key: {key}="))),
            None => {
                if name.is_some() {
                    return Err(Error::BadArgs("multiple profile names given".to_string()));
                }
                name = Some(token.clone());
            }
        }
    }
    Ok(Command::ProfileAdd {
        name: name.ok_or_else(|| Error::BadArgs("'profile add' requires a name".to_string()))?,
        sl: sl.ok_or_else(|| Error::BadArgs("'profile add' requires sl=".to_string()))?,
        tl: tl.ok_or_else(|| Error::BadArgs("'profile add' requires tl=".to_string()))?,
    })
}

fn parse_patch(tail: &[String]) -> Result<Command> {
    let (mut name, mut sl, mut tl, mut new_name) = (None, None, None, None);
    for token in tail {
        match split_kv(token) {
            Some(("sl", v)) => sl = Some(v.to_string()),
            Some(("tl", v)) => tl = Some(v.to_string()),
            Some(("name", v)) => new_name = Some(v.to_string()),
            Some((key, _)) => return Err(Error::BadArgs(format!("unexpected key: {key}="))),
            None => {
                if name.is_some() {
                    return Err(Error::BadArgs("multiple profile names given".to_string()));
                }
                name = Some(token.clone());
            }
        }
    }
    let name = name.ok_or_else(|| Error::BadArgs("'profile patch' requires a name".to_string()))?;
    if sl.is_none() && tl.is_none() && new_name.is_none() {
        return Err(Error::BadArgs(
            "'profile patch' requires at least one of sl=, tl=, name=".to_string(),
        ));
    }
    Ok(Command::ProfilePatch {
        name,
        sl,
        tl,
        new_name,
    })
}

fn parse_translate(args: &[String]) -> Result<Command> {
    let (mut profile, mut sl, mut tl, mut file) = (None, None, None, None);
    let (mut quiet, mut synonyms, mut verbose) = (false, false, false);
    let mut text_parts = Vec::new();
    for token in args {
        match token.as_str() {
            "--quiet" | "-q" => quiet = true,
            "--synonyms" | "-s" => synonyms = true,
            "--verbose" | "-v" => verbose = true,
            _ => match split_kv(token) {
                Some(("sl", v)) => sl = Some(v.to_string()),
                Some(("tl", v)) => tl = Some(v.to_string()),
                Some(("p", v)) => profile = Some(v.to_string()),
                Some(("f", v)) => file = Some(v.to_string()),
                _ => text_parts.push(token.clone()),
            },
        }
    }
    if u8::from(quiet) + u8::from(synonyms) + u8::from(verbose) > 1 {
        return Err(Error::BadArgs(
            "--quiet, --synonyms and --verbose are mutually exclusive".to_string(),
        ));
    }
    let text = if text_parts.is_empty() {
        None
    } else {
        Some(text_parts.join(" "))
    };
    if file.is_some() && text.is_some() {
        return Err(Error::BadArgs(
            "cannot combine f= with inline text; use one".to_string(),
        ));
    }
    Ok(Command::Translate(Translate {
        profile,
        sl,
        tl,
        text,
        file,
        filter: if quiet {
            Filter::Quiet
        } else if synonyms {
            Filter::Synonyms
        } else if verbose {
            Filter::Verbose
        } else {
            Filter::Full
        },
    }))
}
