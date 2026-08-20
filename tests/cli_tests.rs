use tt::cli::{parse, Command};
use tt::render::Filter;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn empty_args_is_help() {
    assert_eq!(parse(&args(&[])).unwrap(), Command::Help);
}

#[test]
fn help_flags_are_help() {
    for flag in ["help", "-h", "--help"] {
        assert_eq!(parse(&args(&[flag])).unwrap(), Command::Help);
    }
}

#[test]
fn bare_text_uses_default_profile() {
    let cmd = parse(&args(&["bom dia"])).unwrap();
    assert_eq!(
        cmd,
        Command::Translate {
            profile: None,
            sl: None,
            tl: None,
            text: "bom dia".to_string(),
            filter: Filter::Full,
        }
    );
}

#[test]
fn unquoted_words_join_into_text() {
    let cmd = parse(&args(&["bom", "dia"])).unwrap();
    match cmd {
        Command::Translate { text, .. } => assert_eq!(text, "bom dia"),
        other => panic!("expected translate, got {other:?}"),
    }
}

#[test]
fn params_are_order_independent() {
    let a = parse(&args(&["sl=pt", "tl=en", "oi"])).unwrap();
    let b = parse(&args(&["oi", "tl=en", "sl=pt"])).unwrap();
    assert_eq!(a, b);
}

#[test]
fn parses_sl_tl_profile() {
    let cmd = parse(&args(&["p=br", "sl=de", "tl=en", "hallo"])).unwrap();
    assert_eq!(
        cmd,
        Command::Translate {
            profile: Some("br".to_string()),
            sl: Some("de".to_string()),
            tl: Some("en".to_string()),
            text: "hallo".to_string(),
            filter: Filter::Full,
        }
    );
}

#[test]
fn quiet_flag_sets_filter() {
    match parse(&args(&["-q", "hi"])).unwrap() {
        Command::Translate { filter, .. } => assert_eq!(filter, Filter::Quiet),
        other => panic!("got {other:?}"),
    }
    match parse(&args(&["--quiet", "hi"])).unwrap() {
        Command::Translate { filter, .. } => assert_eq!(filter, Filter::Quiet),
        other => panic!("got {other:?}"),
    }
}

#[test]
fn synonyms_flag_sets_filter() {
    match parse(&args(&["--synonyms", "hi"])).unwrap() {
        Command::Translate { filter, .. } => assert_eq!(filter, Filter::Synonyms),
        other => panic!("got {other:?}"),
    }
    match parse(&args(&["-s", "hi"])).unwrap() {
        Command::Translate { filter, .. } => assert_eq!(filter, Filter::Synonyms),
        other => panic!("got {other:?}"),
    }
}

#[test]
fn verbose_flag_sets_filter() {
    for flag in ["-v", "--verbose"] {
        match parse(&args(&[flag, "hi"])).unwrap() {
            Command::Translate { filter, .. } => assert_eq!(filter, Filter::Verbose),
            other => panic!("got {other:?}"),
        }
    }
}

#[test]
fn output_filters_are_mutually_exclusive() {
    assert!(parse(&args(&["-q", "-s", "hi"])).is_err());
    assert!(parse(&args(&["-q", "-v", "hi"])).is_err());
    assert!(parse(&args(&["-s", "-v", "hi"])).is_err());
    assert!(parse(&args(&["-q", "-s", "-v", "hi"])).is_err());
}

#[test]
fn empty_text_is_error() {
    assert!(parse(&args(&["sl=pt", "tl=en"])).is_err());
}

#[test]
fn text_with_equals_is_not_a_param() {
    match parse(&args(&["E=mc2"])).unwrap() {
        Command::Translate { text, .. } => assert_eq!(text, "E=mc2"),
        other => panic!("got {other:?}"),
    }
}

#[test]
fn languages_command() {
    assert_eq!(parse(&args(&["languages"])).unwrap(), Command::Languages);
    assert_eq!(parse(&args(&["langs"])).unwrap(), Command::Languages);
}

#[test]
fn languages_rejects_extra_args() {
    assert!(parse(&args(&["languages", "en"])).is_err());
}

#[test]
fn update_command() {
    assert_eq!(parse(&args(&["update"])).unwrap(), Command::Update);
}

#[test]
fn update_rejects_extra_args() {
    assert!(parse(&args(&["update", "now"])).is_err());
}

#[test]
fn default_no_args_shows() {
    assert_eq!(parse(&args(&["default"])).unwrap(), Command::DefaultShow);
}

#[test]
fn default_with_values_sets() {
    assert_eq!(
        parse(&args(&["default", "sl=pt", "tl=en"])).unwrap(),
        Command::DefaultSet {
            sl: Some("pt".to_string()),
            tl: Some("en".to_string()),
        }
    );
}

#[test]
fn default_rejects_unknown_key() {
    assert!(parse(&args(&["default", "p=br"])).is_err());
}

#[test]
fn profile_add_full() {
    assert_eq!(
        parse(&args(&["profile", "add", "br", "sl=en", "tl=pt"])).unwrap(),
        Command::ProfileAdd {
            name: "br".to_string(),
            sl: "en".to_string(),
            tl: "pt".to_string(),
        }
    );
}

#[test]
fn profile_add_requires_all_fields() {
    assert!(parse(&args(&["profile", "add", "br", "sl=en"])).is_err());
    assert!(parse(&args(&["profile", "add", "br", "tl=pt"])).is_err());
    assert!(parse(&args(&["profile", "add", "sl=en", "tl=pt"])).is_err());
}

#[test]
fn profile_list_aliases() {
    assert_eq!(parse(&args(&["profile", "list"])).unwrap(), Command::ProfileList);
    assert_eq!(parse(&args(&["profile", "ls"])).unwrap(), Command::ProfileList);
}

#[test]
fn profile_list_rejects_extra_args() {
    assert!(parse(&args(&["profile", "list", "br"])).is_err());
}

#[test]
fn profile_delete_aliases() {
    assert_eq!(
        parse(&args(&["profile", "delete", "br"])).unwrap(),
        Command::ProfileDelete { name: "br".to_string() }
    );
    assert_eq!(
        parse(&args(&["profile", "rm", "br"])).unwrap(),
        Command::ProfileDelete { name: "br".to_string() }
    );
}

#[test]
fn profile_delete_requires_name() {
    assert!(parse(&args(&["profile", "delete"])).is_err());
}

#[test]
fn profile_patch_partial() {
    assert_eq!(
        parse(&args(&["profile", "patch", "br", "tl=es"])).unwrap(),
        Command::ProfilePatch {
            name: "br".to_string(),
            sl: None,
            tl: Some("es".to_string()),
            new_name: None,
        }
    );
}

#[test]
fn profile_patch_rename() {
    assert_eq!(
        parse(&args(&["profile", "patch", "br", "name=brazil", "sl=en"])).unwrap(),
        Command::ProfilePatch {
            name: "br".to_string(),
            sl: Some("en".to_string()),
            tl: None,
            new_name: Some("brazil".to_string()),
        }
    );
}

#[test]
fn profile_patch_requires_a_change() {
    assert!(parse(&args(&["profile", "patch", "br"])).is_err());
}

#[test]
fn profile_requires_subcommand() {
    assert!(parse(&args(&["profile"])).is_err());
}

#[test]
fn profile_unknown_subcommand() {
    assert!(parse(&args(&["profile", "frobnicate", "br"])).is_err());
}
