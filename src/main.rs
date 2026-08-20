use std::io::{IsTerminal, Write};
use std::process::exit;

use tt::cli::{parse, Command};
use tt::color::{paint, BOLD, CYAN, DIM, GREEN, ITALIC, MAGENTA, YELLOW};
use tt::config::Config;
use tt::engine::{default_engine, Query};
use tt::error::Result;
use tt::lang::LANGUAGES;
use tt::render::{render, Filter, Meta};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => exit(code),
        Err(e) => {
            eprintln!("tt: {e}");
            exit(e.exit_code());
        }
    }
}

fn run(args: &[String]) -> Result<i32> {
    match parse(args)? {
        Command::Help => {
            print!("{}", help(std::io::stdout().is_terminal()));
            Ok(0)
        }
        Command::Languages => {
            print!("{}", languages(std::io::stdout().is_terminal()));
            Ok(0)
        }
        Command::Translate {
            profile,
            sl,
            tl,
            text,
            filter,
        } => {
            let all_default = profile.is_none() && sl.is_none() && tl.is_none();
            let config = Config::load()?;
            let resolved = config.resolve(profile.as_deref(), sl.as_deref(), tl.as_deref())?;
            let translation = default_engine().translate(Query {
                sl: &resolved.sl,
                tl: &resolved.tl,
                text: &text,
            })?;
            let color = std::io::stdout().is_terminal();
            let meta = Meta {
                sl: &resolved.sl,
                tl: &resolved.tl,
                text: &text,
            };
            let mut out = std::io::stdout();
            let _ = writeln!(out, "{}", render(&translation, filter, color, &meta));
            if color && all_default && filter == Filter::Full {
                eprintln!("{}", default_hint(&resolved.sl, &resolved.tl));
            }
            Ok(0)
        }
        Command::DefaultShow => {
            let config = Config::load()?;
            if let Some(p) = config.profiles.get("default") {
                println!("sl={} tl={}", p.sl, p.tl);
            }
            Ok(0)
        }
        Command::DefaultSet { sl, tl } => {
            let mut config = Config::load()?;
            config.set_default(sl.as_deref(), tl.as_deref())?;
            config.save()?;
            if let Some(p) = config.profiles.get("default") {
                println!("default set: sl={} tl={}", p.sl, p.tl);
            }
            Ok(0)
        }
        Command::ProfileAdd { name, sl, tl } => {
            let mut config = Config::load()?;
            config.add(&name, &sl, &tl)?;
            config.save()?;
            println!("profile added: {name}");
            Ok(0)
        }
        Command::ProfileList => {
            let config = Config::load()?;
            let color = std::io::stdout().is_terminal();
            let width = config.profiles.keys().map(String::len).max().unwrap_or(0);
            for (name, p) in &config.profiles {
                let tag = if name == "default" {
                    paint(color, DIM, "  (used when no p= is given)")
                } else {
                    String::new()
                };
                println!(
                    "  {}  {}{}  {}{}{tag}",
                    paint(color, BOLD, &format!("{name:<width$}")),
                    paint(color, DIM, "sl="),
                    paint(color, GREEN, &p.sl),
                    paint(color, DIM, "tl="),
                    paint(color, GREEN, &p.tl),
                );
            }
            Ok(0)
        }
        Command::ProfileDelete { name } => {
            let mut config = Config::load()?;
            config.delete(&name)?;
            config.save()?;
            println!("profile deleted: {name}");
            Ok(0)
        }
        Command::ProfilePatch {
            name,
            sl,
            tl,
            new_name,
        } => {
            let mut config = Config::load()?;
            config.patch(&name, sl.as_deref(), tl.as_deref(), new_name.as_deref())?;
            config.save()?;
            println!("profile patched: {name}");
            Ok(0)
        }
    }
}

fn default_hint(sl: &str, tl: &str) -> String {
    let color = std::io::stderr().is_terminal();
    paint(
        color,
        DIM,
        &format!(
            "sl={sl} and tl={tl} come from the default profile; \
change them with `tt default sl=<lang> tl=<lang>`"
        ),
    )
}

fn row(color: bool, cmd: &str, desc: &str, width: usize) -> String {
    format!(
        "  {}{}\n",
        paint(color, CYAN, &format!("{cmd:<width$}")),
        paint(color, DIM, desc)
    )
}

fn languages(color: bool) -> String {
    let mut o = paint(color, BOLD, "SUPPORTED LANGUAGES");
    o.push('\n');
    for (code, name) in LANGUAGES {
        o.push_str(&format!("  {}  {name}\n", paint(color, CYAN, &format!("{code:<7}"))));
    }
    o.push('\n');
    o.push_str(&paint(
        color,
        DIM,
        "aliases and any capitalization work: pt, br, ptbr, pt-BR all mean Portuguese (Brazil)",
    ));
    o.push('\n');
    o
}

fn help(color: bool) -> String {
    let sec = |t: &str| paint(color, YELLOW, t);
    let mut o = format!(
        "{}  {}\n\n",
        paint(color, BOLD, "tt"),
        paint(color, DIM, "command-line text translator")
    );

    o.push_str(&format!("{}\n", sec("TRANSLATE")));
    o.push_str(&format!(
        "  {}\n",
        paint(
            color,
            &format!("{ITALIC}{MAGENTA}"),
            "languages are named params: order is free and the source language is optional"
        )
    ));
    for (c, d) in [
        ("tt <text>", "translate with your default profile"),
        ("tt tl=<lang> <text>", "set the target, auto-detect the source language"),
        ("tt sl=<lang> tl=<lang> <text>", "set both languages explicitly"),
        ("tt p=<profile> <text>", "translate using a saved profile"),
    ] {
        o.push_str(&row(color, c, d, 32));
    }
    o.push('\n');

    o.push_str(&format!("{}\n", sec("FLAGS")));
    for (c, d) in [
        ("-q, --quiet", "only the primary translation"),
        ("-s, --synonyms", "only the synonyms and back translations"),
        ("-v, --verbose", "a labeled breakdown: languages, original, result"),
        ("-h, --help", "show this help"),
    ] {
        o.push_str(&row(color, c, d, 32));
    }
    o.push('\n');

    o.push_str(&format!("{}\n", sec("DEFAULTS")));
    for (c, d) in [
        ("tt default", "show the default languages"),
        ("tt default sl=<lang> tl=<lang>", "set the defaults"),
    ] {
        o.push_str(&row(color, c, d, 32));
    }
    o.push('\n');

    o.push_str(&format!("{}\n", sec("PROFILES")));
    for (c, d) in [
        ("tt profile add <name> sl=<l> tl=<l>", "create a profile"),
        ("tt profile list", "list profiles"),
        ("tt profile patch <name> [sl=] [tl=] [name=]", "change a profile"),
        ("tt profile delete <name>", "remove a profile"),
    ] {
        o.push_str(&row(color, c, d, 44));
    }
    o.push('\n');

    o.push_str(&format!("{}\n", sec("LANGUAGES")));
    o.push_str(&row(color, "tt languages", "list supported language codes", 32));
    o.push('\n');

    o.push_str(&format!("{}\n", sec("EXAMPLES")));
    let arrow = paint(color, GREEN, "->");
    for (c, r) in [
        ("tt \"bom dia\"", "good morning"),
        ("tt tl=es \"good morning\"", "buenos días"),
        ("tt sl=de tl=en \"danke\"", "thank you"),
    ] {
        o.push_str(&format!(
            "  {}{} {}\n",
            paint(color, CYAN, &format!("{c:<32}")),
            arrow,
            paint(color, DIM, r)
        ));
    }
    o
}
