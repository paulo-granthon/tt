mod common;

use common::Setup;

#[test]
fn no_args_prints_help() {
    let run = Setup::new().piped().run(&[]);
    assert_eq!(run.code, 0);
    assert!(run.out.starts_with("tt  command-line text translator\n"));
    for section in [
        "TRANSLATE", "FLAGS", "DEFAULTS", "PROFILES", "LANGUAGES", "UPDATE", "EXAMPLES",
    ] {
        assert!(run.out.contains(&format!("\n{section}\n")), "missing {section}");
    }
    assert!(run.err.is_empty());
}

#[test]
fn help_flag_matches_bare_help() {
    let bare = Setup::new().piped().run(&[]).out;
    for flag in ["help", "-h", "--help"] {
        assert_eq!(Setup::new().piped().run(&[flag]).out, bare);
    }
}

#[test]
fn help_is_colored_on_a_tty_only() {
    assert!(Setup::new().run(&["--help"]).out.contains("\x1b["));
    assert!(!Setup::new().piped().run(&["--help"]).out.contains("\x1b["));
}

#[test]
fn help_mentions_every_command() {
    let out = Setup::new().piped().run(&["--help"]).out;
    for cmd in [
        "tt <text>", "tt tl=<lang> <text>", "tt p=<profile> <text>", "tt f=<file> ...",
        "-q, --quiet", "-s, --synonyms", "-v, --verbose", "tt default", "tt profile add",
        "tt profile list", "tt profile patch", "tt profile delete", "tt languages", "tt update",
    ] {
        assert!(out.contains(cmd), "missing {cmd}");
    }
}

#[test]
fn languages_lists_codes_and_names() {
    let run = Setup::new().piped().run(&["languages"]);
    assert_eq!(run.code, 0);
    assert!(run.out.starts_with("SUPPORTED LANGUAGES\n"));
    assert!(run.out.contains("  pt-BR    Portuguese (Brazil)\n"));
    assert!(run.out.contains("  en       English\n"));
    assert!(run.out.contains("aliases and any capitalization work"));
}

#[test]
fn langs_alias_matches_languages() {
    assert_eq!(
        Setup::new().piped().run(&["langs"]).out,
        Setup::new().piped().run(&["languages"]).out
    );
}

#[test]
fn languages_rejects_arguments() {
    let run = Setup::new().piped().run(&["languages", "extra"]);
    assert_eq!(run.code, 2);
    assert_eq!(run.err, "tt: 'languages' takes no arguments\n");
    assert!(run.out.is_empty());
}
