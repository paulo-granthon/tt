use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::engine::Translation;
use crate::error::{Error, Result};

pub const CAP: usize = 1000;

pub struct Key<'a> {
    pub engine: &'a str,
    pub sl: &'a str,
    pub tl: &'a str,
    pub text: &'a str,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    engine: String,
    sl: String,
    tl: String,
    text: String,
    translation: Translation,
}

pub struct Stats {
    pub entries: usize,
    pub bytes: u64,
}

pub struct Cache {
    dir: PathBuf,
    cap: usize,
}

pub fn fnv1a(key: &Key) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for field in [key.engine, key.sl, key.tl, key.text] {
        for byte in field.bytes().chain(std::iter::once(0)) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
    }
    hash
}

impl Cache {
    pub fn new(dir: PathBuf) -> Self {
        Cache { dir, cap: CAP }
    }

    pub fn with_cap(dir: PathBuf, cap: usize) -> Self {
        Cache { dir, cap }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn get(&self, key: &Key) -> Option<Translation> {
        let entry: Entry = serde_json::from_str(&std::fs::read_to_string(self.entry(key)).ok()?).ok()?;
        (entry.engine == key.engine && entry.sl == key.sl && entry.tl == key.tl && entry.text == key.text)
            .then_some(entry.translation)
    }

    pub fn put(&self, key: &Key, translation: &Translation) {
        let path = self.entry(key);
        let Some(parent) = path.parent() else { return };
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
        let body = match serde_json::to_string(&Entry {
            engine: key.engine.to_string(),
            sl: key.sl.to_string(),
            tl: key.tl.to_string(),
            text: key.text.to_string(),
            translation: translation.clone(),
        }) {
            Ok(body) => body,
            Err(_) => return,
        };
        let staged = path.with_extension("tmp");
        if std::fs::write(&staged, body).is_err() || std::fs::rename(&staged, &path).is_err() {
            let _ = std::fs::remove_file(&staged);
            return;
        }
        self.evict();
    }

    pub fn clear(&self) -> Result<()> {
        match std::fs::remove_dir_all(self.entries()) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(Error::Config(e.to_string())),
            _ => Ok(()),
        }
    }

    pub fn stats(&self) -> Result<Stats> {
        let mut stats = Stats { entries: 0, bytes: 0 };
        for file in self.files() {
            stats.entries += 1;
            stats.bytes += file.metadata().map(|m| m.len()).unwrap_or(0);
        }
        Ok(stats)
    }

    fn entries(&self) -> PathBuf {
        self.dir.join("entries")
    }

    fn entry(&self, key: &Key) -> PathBuf {
        self.entries().join(format!("{:016x}.json", fnv1a(key)))
    }

    fn files(&self) -> Vec<std::fs::DirEntry> {
        let Ok(read) = std::fs::read_dir(self.entries()) else {
            return Vec::new();
        };
        read.flatten()
            .filter(|f| f.path().extension().is_some_and(|e| e == "json"))
            .collect()
    }

    fn evict(&self) {
        let mut files = self.files();
        if files.len() <= self.cap {
            return;
        }
        files.sort_by_key(|f| {
            f.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });
        for file in files.iter().take(files.len() - self.cap) {
            let _ = std::fs::remove_file(file.path());
        }
    }
}
