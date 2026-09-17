
use crate::color::{paint, BOLD, CYAN, DIM, GREEN, ITALIC, MAGENTA, YELLOW};
use crate::env::Env;
use crate::error::Result;
use crate::lang::LANGUAGES;

pub fn print(text: String, env: &mut Env) -> Result<i32> {
    let _ = write!(env.out, "{text}");
    Ok(0)
}

fn row(color: bool, cmd: &str, desc: &str, width: usize) -> String {
    format!(
        "  {}{}\n",
        paint(color, CYAN, &format!("{cmd:<width$}")),
        paint(color, DIM, desc)
    )
}

pub fn languages(color: bool) -> String {
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

pub fn help(color: bool) -> String {
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
        ("tt f=<file> ...", "translate the contents of a file"),
        ("... | tt ...", "translate piped stdin when given no text"),
    ] {
        o.push_str(&row(color, c, d, 32));
    }
    o.push('\n');

    o.push_str(&format!("{}\n", sec("FLAGS")));
    for (c, d) in [
        ("-q, --quiet", "only the primary translation"),
        ("-s, --synonyms", "only the synonyms and back translations"),
        ("-v, --verbose", "a labeled breakdown: languages, original, result"),
        ("-j, --json", "the full result as one JSON line, for scripts"),
        ("--no-cache", "skip the on-disk cache for this call"),
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

    o.push_str(&format!("{}\n", sec("CACHE")));
    for (c, d) in [
        ("tt cache", "show cached entries, size and path"),
        ("tt cache clear", "delete every cached translation"),
    ] {
        o.push_str(&row(color, c, d, 32));
    }
    o.push('\n');

    o.push_str(&format!("{}\n", sec("UPDATE")));
    o.push_str(&row(color, "tt update", "install the latest release", 32));
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
