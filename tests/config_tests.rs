use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use tt::config::Config;

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn temp_path() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = format!("tt-test-{}-{}-{}.toml", std::process::id(), n, "cfg");
    std::env::temp_dir().join(name)
}

struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn missing_file_yields_builtin_default() {
    let cfg = Config::load_from(&temp_path()).unwrap();
    let d = cfg.profiles.get("default").unwrap();
    assert_eq!(d.sl, "auto");
    assert_eq!(d.tl, "en");
}

#[test]
fn save_then_load_round_trips() {
    let path = temp_path();
    let _guard = TempFile(path.clone());
    let mut cfg = Config::with_builtin_default();
    cfg.add("br", "en", "pt").unwrap();
    cfg.save_to(&path).unwrap();

    let loaded = Config::load_from(&path).unwrap();
    let br = loaded.profiles.get("br").unwrap();
    assert_eq!(br.sl, "en");
    assert_eq!(br.tl, "pt-BR");
    assert!(loaded.profiles.contains_key("default"));
}

#[test]
fn load_injects_default_when_absent_from_file() {
    let path = temp_path();
    let _guard = TempFile(path.clone());
    std::fs::write(&path, "[profiles.br]\nsl = \"en\"\ntl = \"pt-BR\"\n").unwrap();
    let loaded = Config::load_from(&path).unwrap();
    assert!(loaded.profiles.contains_key("default"));
    assert!(loaded.profiles.contains_key("br"));
}

#[test]
fn set_default_canonicalizes() {
    let mut cfg = Config::with_builtin_default();
    cfg.set_default(Some("pt"), Some("english")).unwrap();
    let d = cfg.profiles.get("default").unwrap();
    assert_eq!(d.sl, "pt-BR");
    assert_eq!(d.tl, "en");
}

#[test]
fn add_canonicalizes_and_rejects_duplicate() {
    let mut cfg = Config::with_builtin_default();
    cfg.add("br", "english", "brazilian").unwrap();
    let br = cfg.profiles.get("br").unwrap();
    assert_eq!(br.sl, "en");
    assert_eq!(br.tl, "pt-BR");
    assert!(cfg.add("br", "en", "pt").is_err());
}

#[test]
fn add_rejects_unknown_language() {
    let mut cfg = Config::with_builtin_default();
    assert!(cfg.add("x", "klingon", "en").is_err());
    assert!(cfg.add("x", "en", "klingon").is_err());
}

#[test]
fn add_rejects_auto_target() {
    let mut cfg = Config::with_builtin_default();
    assert!(cfg.add("x", "auto", "auto").is_err());
}

#[test]
fn delete_removes_profile() {
    let mut cfg = Config::with_builtin_default();
    cfg.add("br", "en", "pt").unwrap();
    cfg.delete("br").unwrap();
    assert!(!cfg.profiles.contains_key("br"));
}

#[test]
fn delete_default_is_rejected() {
    let mut cfg = Config::with_builtin_default();
    assert!(cfg.delete("default").is_err());
}

#[test]
fn delete_unknown_is_rejected() {
    let mut cfg = Config::with_builtin_default();
    assert!(cfg.delete("ghost").is_err());
}

#[test]
fn patch_updates_fields() {
    let mut cfg = Config::with_builtin_default();
    cfg.add("br", "en", "pt").unwrap();
    cfg.patch("br", Some("de"), None, None).unwrap();
    let br = cfg.profiles.get("br").unwrap();
    assert_eq!(br.sl, "de");
    assert_eq!(br.tl, "pt-BR");
}

#[test]
fn patch_renames() {
    let mut cfg = Config::with_builtin_default();
    cfg.add("br", "en", "pt").unwrap();
    cfg.patch("br", None, None, Some("brazil")).unwrap();
    assert!(!cfg.profiles.contains_key("br"));
    assert!(cfg.profiles.contains_key("brazil"));
}

#[test]
fn patch_rename_onto_existing_is_rejected() {
    let mut cfg = Config::with_builtin_default();
    cfg.add("br", "en", "pt").unwrap();
    cfg.add("es", "en", "es").unwrap();
    assert!(cfg.patch("br", None, None, Some("es")).is_err());
}

#[test]
fn cannot_rename_default() {
    let mut cfg = Config::with_builtin_default();
    assert!(cfg.patch("default", None, None, Some("main")).is_err());
}

#[test]
fn cannot_rename_a_profile_to_default() {
    let mut cfg = Config::with_builtin_default();
    cfg.add("br", "en", "pt").unwrap();
    assert!(cfg.patch("br", None, None, Some("default")).is_err());
}

#[test]
fn patch_unknown_profile_is_rejected() {
    let mut cfg = Config::with_builtin_default();
    assert!(cfg.patch("ghost", Some("en"), None, None).is_err());
}

#[test]
fn patch_rejects_bad_language() {
    let mut cfg = Config::with_builtin_default();
    cfg.add("br", "en", "pt").unwrap();
    assert!(cfg.patch("br", Some("klingon"), None, None).is_err());
}

#[test]
fn resolve_uses_default_when_nothing_given() {
    let cfg = Config::with_builtin_default();
    let r = cfg.resolve(None, None, None).unwrap();
    assert_eq!(r.sl, "auto");
    assert_eq!(r.tl, "en");
}

#[test]
fn resolve_inline_overrides_profile() {
    let mut cfg = Config::with_builtin_default();
    cfg.add("br", "en", "pt").unwrap();
    let r = cfg.resolve(Some("br"), Some("de"), None).unwrap();
    assert_eq!(r.sl, "de");
    assert_eq!(r.tl, "pt-BR");
}

#[test]
fn resolve_inline_tl_overrides_profile() {
    let mut cfg = Config::with_builtin_default();
    cfg.add("br", "en", "pt").unwrap();
    let r = cfg.resolve(Some("br"), None, Some("es")).unwrap();
    assert_eq!(r.sl, "en");
    assert_eq!(r.tl, "es");
}

#[test]
fn resolve_unknown_profile_is_rejected() {
    let cfg = Config::with_builtin_default();
    assert!(cfg.resolve(Some("ghost"), None, None).is_err());
}

#[test]
fn resolve_rejects_bad_inline_language() {
    let cfg = Config::with_builtin_default();
    assert!(cfg.resolve(None, Some("klingon"), None).is_err());
}
