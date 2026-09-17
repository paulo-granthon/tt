mod common;

use common::{sample, Fake, Setup};
use tt::cache::Cache;
use tt::config::Config;
use tt::render::{render, Filter, Meta};

const URL: &str = "https://translate.google.com/?sl=auto&tl=en&text=bom%20dia&op=translate";

#[test]
fn translates_with_the_default_profile() {
    let setup = Setup::new().piped();
    let probe = setup.probe.clone();
    let run = setup.run(&["bom dia"]);
    assert_eq!(run.code, 0);
    assert_eq!(probe.calls(), 1);
    assert_eq!(
        probe.last(),
        ("auto".to_string(), "en".to_string(), "bom dia".to_string())
    );
    let meta = Meta {
        sl: "auto",
        tl: "en",
        text: "bom dia",
        engine: "fake",
        cached: false,
    };
    assert_eq!(run.out, format!("{}\n", render(&sample(), Filter::Full, false, &meta)));
    assert!(run.err.is_empty());
}

#[test]
fn joins_multiple_text_tokens() {
    let setup = Setup::new().piped();
    let probe = setup.probe.clone();
    setup.run(&["tl=en", "bom", "dia"]);
    assert_eq!(probe.last().2, "bom dia");
}

#[test]
fn inline_languages_override_the_profile() {
    let setup = Setup::new().piped();
    let probe = setup.probe.clone();
    let mut config = Config::with_builtin_default();
    config.add("br", "en", "pt-BR").unwrap();
    config.save_to(&setup.config_path()).unwrap();
    setup.run(&["p=br", "sl=de", "hallo"]);
    assert_eq!(probe.last(), ("de".to_string(), "pt-BR".to_string(), "hallo".to_string()));
}

#[test]
fn footer_shows_hint_and_url_on_a_tty() {
    let run = Setup::new().run(&["bom dia"]);
    assert_eq!(run.code, 0);
    assert!(run.err.starts_with("\n\x1b[2msl=auto and tl=en come from the default profile"));
    assert!(run.err.contains("tt default sl=<lang> tl=<lang>"));
    assert!(run.err.contains(&format!("\x1b]8;;{URL}\x1b\\{URL}\x1b]8;;\x1b\\\n")));
}

#[test]
fn footer_is_absent_when_piped() {
    let run = Setup::new().piped().run(&["bom dia"]);
    assert!(run.err.is_empty());
}

#[test]
fn footer_is_absent_in_quiet_mode() {
    let run = Setup::new().run(&["-q", "bom dia"]);
    assert_eq!(run.out, "good morning\n");
    assert!(run.err.is_empty());
}

#[test]
fn hint_is_absent_when_any_language_is_explicit() {
    let run = Setup::new().run(&["tl=en", "bom dia"]);
    assert!(!run.err.contains("default profile"));
    assert!(run.err.contains("https://translate.google.com/"));
}

#[test]
fn hint_is_absent_in_verbose_mode() {
    let run = Setup::new().run(&["-v", "bom dia"]);
    assert!(!run.err.contains("default profile"));
    assert!(run.err.contains("https://translate.google.com/"));
}

#[test]
fn identity_languages_skip_the_engine() {
    let setup = Setup::new().piped();
    let probe = setup.probe.clone();
    let run = setup.run(&["sl=en", "tl=en", "hello there"]);
    assert_eq!(run.code, 0);
    assert_eq!(probe.calls(), 0);
    assert_eq!(run.out, "hello there\n");
}

#[test]
fn reads_stdin_when_no_text_is_given() {
    let setup = Setup::new().piped().stdin("  bom dia\n");
    let probe = setup.probe.clone();
    let run = setup.run(&["tl=en"]);
    assert_eq!(run.code, 0);
    assert_eq!(probe.last().2, "bom dia");
}

#[test]
fn dash_forces_stdin() {
    let setup = Setup::new().piped().stdin("oi");
    let probe = setup.probe.clone();
    setup.run(&["tl=en", "-"]);
    assert_eq!(probe.last().2, "oi");
}

#[test]
fn reads_a_file_with_f() {
    let setup = Setup::new().piped();
    let probe = setup.probe.clone();
    let file = setup.dir.path("notes.txt");
    std::fs::write(&file, "bom dia\n").unwrap();
    let run = setup.run(&["tl=en", &format!("f={}", file.display())]);
    assert_eq!(run.code, 0);
    assert_eq!(probe.last().2, "bom dia");
}

#[test]
fn missing_file_is_a_config_error() {
    let run = Setup::new().piped().run(&["tl=en", "f=/nonexistent/tt-file"]);
    assert_eq!(run.code, 6);
    assert!(run.err.starts_with("tt: config error: could not read /nonexistent/tt-file"));
}

#[test]
fn no_text_on_a_tty_is_bad_args() {
    let run = Setup::new().run(&["tl=en"]);
    assert_eq!(run.code, 2);
    assert_eq!(run.err, "tt: no text to translate\n");
}

#[test]
fn whitespace_only_input_is_bad_args() {
    let run = Setup::new().piped().stdin("   \n").run(&["tl=en"]);
    assert_eq!(run.code, 2);
    assert_eq!(run.err, "tt: no text to translate\n");
}

#[test]
fn unknown_profile_is_bad_args() {
    let run = Setup::new().piped().run(&["p=nope", "oi"]);
    assert_eq!(run.code, 2);
    assert_eq!(run.err, "tt: unknown profile: nope\n");
}

#[test]
fn unknown_language_exits_three() {
    let run = Setup::new().piped().run(&["tl=klingon", "oi"]);
    assert_eq!(run.code, 3);
    assert!(run.err.starts_with("tt: unknown language: klingon"));
}

#[test]
fn engine_failure_exits_four_with_message() {
    let (engine, probe) = Fake::failing("boom");
    let run = Setup::new().piped().engine(engine, probe).run(&["oi"]);
    assert_eq!(run.code, 4);
    assert_eq!(run.err, "tt: network error: boom\n");
    assert!(run.out.is_empty());
}

#[test]
fn exclusive_flags_are_rejected() {
    let run = Setup::new().piped().run(&["-q", "-v", "oi"]);
    assert_eq!(run.code, 2);
    assert_eq!(run.err, "tt: --quiet, --synonyms, --verbose and --json are mutually exclusive\n");
}

#[test]
fn json_prints_one_structured_line_and_no_footer() {
    let run = Setup::new().run(&["-j", "bom dia"]);
    assert_eq!(run.code, 0);
    assert!(run.err.is_empty());
    assert_eq!(run.out.matches('\n').count(), 1);
    assert!(!run.out.contains("\x1b["));
    let value: serde_json::Value = serde_json::from_str(&run.out).unwrap();
    assert_eq!(value["sl"], "auto");
    assert_eq!(value["tl"], "en");
    assert_eq!(value["detected"], "pt");
    assert_eq!(value["text"], "bom dia");
    assert_eq!(value["primary"], "good morning");
    assert_eq!(value["correction"], serde_json::Value::Null);
    assert_eq!(value["synonyms"][0]["word"], "good morning");
    assert_eq!(value["synonyms"][0]["back"][0], "bom dia");
    assert_eq!(value["engine"], "fake");
    assert_eq!(value["cached"], false);
}

#[test]
fn second_identical_query_is_served_from_the_cache() {
    let first = Setup::new().piped();
    let probe = first.probe.clone();
    let first = first.run(&["bom dia"]);
    assert_eq!(probe.calls(), 1);
    let second = Setup::reuse(first.dir).piped();
    let probe = second.probe.clone();
    let second = second.run(&["bom dia"]);
    assert_eq!(probe.calls(), 0);
    assert_eq!(second.out, first.out);
}

#[test]
fn no_cache_skips_lookup_and_store() {
    let first = Setup::new().piped().run(&["bom dia"]);
    let second = Setup::reuse(first.dir).piped();
    let probe = second.probe.clone();
    let second = second.run(&["--no-cache", "bom dia"]);
    assert_eq!(probe.calls(), 1);
    let third = Setup::reuse(second.dir).piped();
    let cache_dir = third.cache_dir();
    let stats = Cache::new(cache_dir).stats().unwrap();
    assert_eq!(stats.entries, 1);
    let probe = third.probe.clone();
    third.run(&["--no-cache", "boa noite"]);
    assert_eq!(probe.calls(), 1);
}

#[test]
fn no_cache_never_stores() {
    let setup = Setup::new().piped();
    let cache_dir = setup.cache_dir();
    setup.run(&["--no-cache", "bom dia"]);
    assert_eq!(Cache::new(cache_dir).stats().unwrap().entries, 0);
}

#[test]
fn identity_results_are_never_cached() {
    let setup = Setup::new().piped();
    let cache_dir = setup.cache_dir();
    setup.run(&["sl=en", "tl=en", "hello"]);
    assert_eq!(Cache::new(cache_dir).stats().unwrap().entries, 0);
}

#[test]
fn engine_failures_are_not_cached() {
    let (engine, probe) = Fake::failing("boom");
    let setup = Setup::new().piped().engine(engine, probe);
    let cache_dir = setup.cache_dir();
    setup.run(&["oi"]);
    assert_eq!(Cache::new(cache_dir).stats().unwrap().entries, 0);
}

#[test]
fn json_reports_a_cache_hit() {
    let first = Setup::new().piped().run(&["-j", "bom dia"]);
    assert!(first.out.contains("\"cached\":false"));
    let second = Setup::reuse(first.dir).piped().run(&["-j", "bom dia"]);
    assert!(second.out.contains("\"cached\":true"));
}

#[test]
fn verbose_marks_a_cache_hit() {
    let first = Setup::new().piped().run(&["-v", "bom dia"]);
    assert!(!first.out.contains("(cached)"));
    let second = Setup::reuse(first.dir).piped().run(&["-v", "bom dia"]);
    assert!(second.out.starts_with("translating Portuguese (Brazil) (detected) -> English (cached)\n"));
}

#[test]
fn full_output_is_identical_on_a_hit() {
    let first = Setup::new().piped().run(&["bom dia"]);
    let second = Setup::reuse(first.dir).piped().run(&["bom dia"]);
    assert_eq!(first.out, second.out);
}
