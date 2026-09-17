mod common;

use std::time::{Duration, SystemTime};

use common::TempDir;
use tt::cache::{fnv1a, Cache, Key, CAP};
use tt::engine::Translation;

fn key<'a>(text: &'a str) -> Key<'a> {
    Key {
        engine: "google",
        sl: "pt-BR",
        tl: "en",
        text,
    }
}

fn translation(primary: &str) -> Translation {
    Translation {
        primary: primary.to_string(),
        ..Translation::default()
    }
}

#[test]
fn default_cap_is_one_thousand() {
    assert_eq!(CAP, 1000);
}

#[test]
fn hash_is_stable_and_separates_fields() {
    assert_eq!(fnv1a(&key("bom dia")), fnv1a(&key("bom dia")));
    assert_ne!(fnv1a(&key("bom dia")), fnv1a(&key("bom  dia")));
    let a = Key {
        engine: "google",
        sl: "ab",
        tl: "c",
        text: "x",
    };
    let b = Key {
        engine: "google",
        sl: "a",
        tl: "bc",
        text: "x",
    };
    assert_ne!(fnv1a(&a), fnv1a(&b));
}

#[test]
fn empty_cache_misses() {
    let dir = TempDir::new();
    assert_eq!(Cache::new(dir.path("cache")).get(&key("bom dia")), None);
}

#[test]
fn put_then_get_round_trips() {
    let dir = TempDir::new();
    let cache = Cache::new(dir.path("cache"));
    cache.put(&key("bom dia"), &translation("good morning"));
    assert_eq!(cache.get(&key("bom dia")), Some(translation("good morning")));
    assert_eq!(cache.get(&key("boa noite")), None);
}

#[test]
fn different_languages_are_different_entries() {
    let dir = TempDir::new();
    let cache = Cache::new(dir.path("cache"));
    cache.put(&key("bom dia"), &translation("good morning"));
    let spanish = Key {
        engine: "google",
        sl: "pt-BR",
        tl: "es",
        text: "bom dia",
    };
    assert_eq!(cache.get(&spanish), None);
}

#[test]
fn entry_with_matching_hash_but_other_key_is_a_miss() {
    let dir = TempDir::new();
    let cache = Cache::new(dir.path("cache"));
    cache.put(&key("bom dia"), &translation("good morning"));
    let path = dir
        .path("cache")
        .join("entries")
        .join(format!("{:016x}.json", fnv1a(&key("bom dia"))));
    let body = std::fs::read_to_string(&path).unwrap().replace("\"bom dia\"", "\"boa tarde\"");
    std::fs::write(&path, body).unwrap();
    assert_eq!(cache.get(&key("bom dia")), None);
}

#[test]
fn malformed_entry_is_a_miss_and_gets_overwritten() {
    let dir = TempDir::new();
    let cache = Cache::new(dir.path("cache"));
    let entries = dir.path("cache").join("entries");
    std::fs::create_dir_all(&entries).unwrap();
    let path = entries.join(format!("{:016x}.json", fnv1a(&key("bom dia"))));
    std::fs::write(&path, "{not json").unwrap();
    assert_eq!(cache.get(&key("bom dia")), None);
    cache.put(&key("bom dia"), &translation("good morning"));
    assert_eq!(cache.get(&key("bom dia")), Some(translation("good morning")));
}

#[test]
fn eviction_keeps_the_newest_entries_up_to_the_cap() {
    let dir = TempDir::new();
    let cache = Cache::with_cap(dir.path("cache"), 2);
    let base = SystemTime::now() - Duration::from_secs(600);
    for (i, text) in ["one", "two", "three"].iter().enumerate() {
        cache.put(&key(text), &translation(text));
        let path = dir
            .path("cache")
            .join("entries")
            .join(format!("{:016x}.json", fnv1a(&key(text))));
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(base + Duration::from_secs(i as u64 * 60))
            .unwrap();
    }
    cache.put(&key("four"), &translation("four"));
    assert_eq!(cache.stats().unwrap().entries, 2);
    assert_eq!(cache.get(&key("one")), None);
    assert_eq!(cache.get(&key("two")), None);
    assert_eq!(cache.get(&key("three")), Some(translation("three")));
    assert_eq!(cache.get(&key("four")), Some(translation("four")));
}

#[test]
fn stats_count_entries_and_bytes() {
    let dir = TempDir::new();
    let cache = Cache::new(dir.path("cache"));
    let empty = cache.stats().unwrap();
    assert_eq!((empty.entries, empty.bytes), (0, 0));
    cache.put(&key("bom dia"), &translation("good morning"));
    cache.put(&key("boa noite"), &translation("good night"));
    let stats = cache.stats().unwrap();
    assert_eq!(stats.entries, 2);
    assert!(stats.bytes > 100);
}

#[test]
fn clear_removes_everything_and_tolerates_a_missing_dir() {
    let dir = TempDir::new();
    let cache = Cache::new(dir.path("cache"));
    cache.clear().unwrap();
    cache.put(&key("bom dia"), &translation("good morning"));
    cache.clear().unwrap();
    assert_eq!(cache.stats().unwrap().entries, 0);
    assert_eq!(cache.get(&key("bom dia")), None);
}

#[test]
fn unwritable_dir_degrades_to_no_cache() {
    let dir = TempDir::new();
    let file = dir.path("not-a-dir");
    std::fs::write(&file, "x").unwrap();
    let cache = Cache::new(file);
    cache.put(&key("bom dia"), &translation("good morning"));
    assert_eq!(cache.get(&key("bom dia")), None);
}
