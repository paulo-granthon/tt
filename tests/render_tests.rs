use tt::engine::{Synonym, Translation};
use tt::render::{render, Filter, Meta};

fn sample() -> Translation {
    Translation {
        primary: "olá".to_string(),
        detected_source: Some("en".to_string()),
        synonyms: vec![
            Synonym {
                word: "olá".to_string(),
                back: vec!["hello".to_string(), "hi".to_string()],
            },
            Synonym {
                word: "oi".to_string(),
                back: vec!["hi".to_string()],
            },
        ],
    }
}

fn no_synonyms() -> Translation {
    Translation {
        primary: "casa".to_string(),
        detected_source: Some("en".to_string()),
        synonyms: vec![],
    }
}

fn meta<'a>() -> Meta<'a> {
    Meta {
        sl: "en",
        tl: "pt-BR",
        text: "hello",
    }
}

#[test]
fn quiet_prints_only_primary() {
    assert_eq!(render(&sample(), Filter::Quiet, false, &meta()), "olá");
}

#[test]
fn quiet_never_has_color() {
    assert_eq!(render(&sample(), Filter::Quiet, true, &meta()), "olá");
}

#[test]
fn synonyms_prints_only_block() {
    let out = render(&sample(), Filter::Synonyms, false, &meta());
    assert_eq!(out, "olá  hello, hi\noi  hi");
    assert!(!out.contains("casa"));
}

#[test]
fn full_has_primary_and_synonyms() {
    let out = render(&sample(), Filter::Full, false, &meta());
    assert!(out.starts_with("olá"));
    assert!(out.contains("oi  hi"));
}

#[test]
fn full_without_synonyms_is_just_primary() {
    assert_eq!(render(&no_synonyms(), Filter::Full, false, &meta()), "casa");
}

#[test]
fn plain_output_has_no_ansi() {
    let out = render(&sample(), Filter::Full, false, &meta());
    assert!(!out.contains('\x1b'));
}

#[test]
fn color_output_has_ansi() {
    let out = render(&sample(), Filter::Full, true, &meta());
    assert!(out.contains('\x1b'));
}

#[test]
fn synonyms_filter_states_when_empty() {
    let out = render(&no_synonyms(), Filter::Synonyms, false, &meta());
    assert_eq!(out, "no synonyms for this translation");
    assert!(!out.is_empty());
}

#[test]
fn synonyms_filter_empty_message_colored_on_tty() {
    let out = render(&no_synonyms(), Filter::Synonyms, true, &meta());
    assert!(out.contains("no synonyms"));
    assert!(out.contains('\x1b'));
}

#[test]
fn synonyms_block_without_back_has_no_double_space() {
    let t = Translation {
        primary: "x".to_string(),
        detected_source: None,
        synonyms: vec![Synonym {
            word: "solo".to_string(),
            back: vec![],
        }],
    };
    assert_eq!(render(&t, Filter::Synonyms, false, &meta()), "solo");
}

#[test]
fn verbose_has_all_sections() {
    let out = render(&sample(), Filter::Full, false, &meta());
    let _ = out;
    let v = render(&sample(), Filter::Verbose, false, &meta());
    assert!(v.contains("translating"));
    assert!(v.contains("ORIGINAL"));
    assert!(v.contains("TRANSLATION"));
    assert!(v.contains("SYNONYMS"));
    assert!(v.contains("hello"));
    assert!(v.contains("olá"));
}

#[test]
fn verbose_resolves_language_names() {
    let v = render(&sample(), Filter::Verbose, false, &meta());
    assert!(v.contains("English"));
    assert!(v.contains("Portuguese (Brazil)"));
}

#[test]
fn verbose_auto_uses_detected_source_name() {
    let m = Meta {
        sl: "auto",
        tl: "es",
        text: "hello",
    };
    let v = render(&sample(), Filter::Verbose, false, &m);
    assert!(v.contains("English"));
    assert!(v.contains("Spanish"));
}

#[test]
fn verbose_states_when_no_synonyms() {
    let v = render(&no_synonyms(), Filter::Verbose, false, &meta());
    assert!(v.contains("no synonyms for this translation"));
}
