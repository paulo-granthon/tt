use tt::lang::{canonical, normalize_key, resolve_source, resolve_target};

#[test]
fn normalize_strips_separators_and_case() {
    assert_eq!(normalize_key("pt-BR"), "ptbr");
    assert_eq!(normalize_key("PT_br"), "ptbr");
    assert_eq!(normalize_key("  Pt Br "), "ptbr");
    assert_eq!(normalize_key("EN"), "en");
}

#[test]
fn ptbr_reachable_through_every_alias() {
    for alias in ["pt", "br", "ptbr", "pt-br", "PT-BR", "Pt_Br", "portuguese", "brazilian"] {
        assert_eq!(canonical(alias), Some("pt-BR"), "alias {alias} failed");
    }
}

#[test]
fn common_languages_canonicalize() {
    assert_eq!(canonical("english"), Some("en"));
    assert_eq!(canonical("EN"), Some("en"));
    assert_eq!(canonical("es"), Some("es"));
    assert_eq!(canonical("spanish"), Some("es"));
    assert_eq!(canonical("fr"), Some("fr"));
    assert_eq!(canonical("de"), Some("de"));
    assert_eq!(canonical("japanese"), Some("ja"));
    assert_eq!(canonical("zh"), Some("zh-CN"));
    assert_eq!(canonical("zhtw"), Some("zh-TW"));
}

#[test]
fn auto_and_detect_map_to_auto() {
    assert_eq!(canonical("auto"), Some("auto"));
    assert_eq!(canonical("detect"), Some("auto"));
    assert_eq!(canonical("AUTO"), Some("auto"));
}

#[test]
fn unknown_language_is_none() {
    assert_eq!(canonical("klingon"), None);
    assert_eq!(canonical("xx"), None);
    assert_eq!(canonical(""), None);
}

#[test]
fn resolve_source_accepts_auto() {
    assert_eq!(resolve_source("auto").unwrap(), "auto");
    assert_eq!(resolve_source("PT").unwrap(), "pt-BR");
}

#[test]
fn resolve_source_rejects_unknown() {
    assert!(resolve_source("nope").is_err());
}

#[test]
fn resolve_target_rejects_auto() {
    assert!(resolve_target("auto").is_err());
    assert!(resolve_target("detect").is_err());
}

#[test]
fn resolve_target_accepts_real_language() {
    assert_eq!(resolve_target("en").unwrap(), "en");
    assert_eq!(resolve_target("br").unwrap(), "pt-BR");
}

#[test]
fn resolve_target_rejects_unknown() {
    assert!(resolve_target("zzz").is_err());
}
