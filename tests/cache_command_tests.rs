mod common;

use common::Setup;
use tt::cache::{Cache, Key};
use tt::engine::Translation;

#[test]
fn cache_stats_on_an_empty_cache() {
    let setup = Setup::new().piped();
    let dir = setup.cache_dir();
    let run = setup.run(&["cache"]);
    assert_eq!(run.code, 0);
    assert_eq!(run.out, format!("entries: 0\nsize: 0 B\npath: {}\n", dir.display()));
}

#[test]
fn cache_stats_after_a_translation() {
    let first = Setup::new().piped().run(&["bom dia"]);
    let run = Setup::reuse(first.dir).piped().run(&["cache"]);
    assert!(run.out.starts_with("entries: 1\nsize: "));
    assert!(run.out.contains(" B\npath: "));
}

#[test]
fn cache_clear_empties_the_cache() {
    let setup = Setup::new().piped();
    let dir = setup.cache_dir();
    Cache::new(dir.clone()).put(
        &Key {
            engine: "fake",
            sl: "auto",
            tl: "en",
            text: "bom dia",
        },
        &Translation::default(),
    );
    let run = setup.run(&["cache", "clear"]);
    assert_eq!(run.code, 0);
    assert_eq!(run.out, "cache cleared\n");
    assert_eq!(Cache::new(dir).stats().unwrap().entries, 0);
}

#[test]
fn cache_rejects_unknown_subcommands() {
    let run = Setup::new().piped().run(&["cache", "nope"]);
    assert_eq!(run.code, 2);
    assert_eq!(run.err, "tt: unknown cache subcommand: nope\n");
}

#[test]
fn cache_clear_rejects_extra_arguments() {
    let run = Setup::new().piped().run(&["cache", "clear", "now"]);
    assert_eq!(run.code, 2);
    assert_eq!(run.err, "tt: 'cache clear' takes no arguments\n");
}
