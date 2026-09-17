mod common;

use common::Setup;
use tt::config::Config;

#[test]
fn default_show_prints_builtin_default() {
    let run = Setup::new().piped().run(&["default"]);
    assert_eq!(run.code, 0);
    assert_eq!(run.out, "sl=auto tl=en\n");
}

#[test]
fn default_set_persists_and_reports() {
    let setup = Setup::new().piped();
    let path = setup.config_path();
    let run = setup.run(&["default", "sl=pt", "tl=en"]);
    assert_eq!(run.code, 0);
    assert_eq!(run.out, "default set: sl=pt-BR tl=en\n");
    let saved = Config::load_from(&path).unwrap();
    assert_eq!(saved.profiles["default"].sl, "pt-BR");
}

#[test]
fn default_set_rejects_unknown_language() {
    let run = Setup::new().piped().run(&["default", "tl=klingon"]);
    assert_eq!(run.code, 3);
    assert!(run.err.starts_with("tt: unknown language: klingon"));
}

#[test]
fn profile_add_then_list() {
    let setup = Setup::new().piped();
    let path = setup.config_path();
    let run = setup.run(&["profile", "add", "br", "sl=en", "tl=br"]);
    assert_eq!(run.code, 0);
    assert_eq!(run.out, "profile added: br\n");
    let saved = Config::load_from(&path).unwrap();
    assert_eq!(saved.profiles["br"].tl, "pt-BR");
}

#[test]
fn profile_list_shows_every_profile_and_tags_default() {
    let setup = Setup::new().piped();
    let mut config = Config::with_builtin_default();
    config.add("br", "en", "pt-BR").unwrap();
    config.save_to(&setup.config_path()).unwrap();
    let run = setup.run(&["profile", "list"]);
    assert_eq!(run.code, 0);
    assert_eq!(
        run.out,
        "  br       sl=en  tl=pt-BR\n  default  sl=auto  tl=en  (used when no p= is given)\n"
    );
}

#[test]
fn profile_list_is_colored_on_a_tty() {
    let run = Setup::new().run(&["profile", "list"]);
    assert!(run.out.contains("\x1b[1mdefault\x1b[0m"));
}

#[test]
fn profile_delete_removes_and_reports() {
    let setup = Setup::new().piped();
    let path = setup.config_path();
    let mut config = Config::with_builtin_default();
    config.add("br", "en", "pt-BR").unwrap();
    config.save_to(&path).unwrap();
    let run = setup.run(&["profile", "delete", "br"]);
    assert_eq!(run.code, 0);
    assert_eq!(run.out, "profile deleted: br\n");
    assert!(!Config::load_from(&path).unwrap().profiles.contains_key("br"));
}

#[test]
fn profile_delete_default_is_rejected() {
    let run = Setup::new().piped().run(&["profile", "rm", "default"]);
    assert_eq!(run.code, 2);
    assert_eq!(run.err, "tt: cannot delete the default profile\n");
}

#[test]
fn profile_patch_renames_and_reports() {
    let setup = Setup::new().piped();
    let path = setup.config_path();
    let mut config = Config::with_builtin_default();
    config.add("br", "en", "pt-BR").unwrap();
    config.save_to(&path).unwrap();
    let run = setup.run(&["profile", "patch", "br", "tl=es", "name=spain"]);
    assert_eq!(run.code, 0);
    assert_eq!(run.out, "profile patched: br\n");
    let saved = Config::load_from(&path).unwrap();
    assert_eq!(saved.profiles["spain"].tl, "es");
    assert!(!saved.profiles.contains_key("br"));
}

#[test]
fn profile_unknown_subcommand_is_bad_args() {
    let run = Setup::new().piped().run(&["profile", "explode"]);
    assert_eq!(run.code, 2);
    assert_eq!(run.err, "tt: unknown profile subcommand: explode\n");
}
