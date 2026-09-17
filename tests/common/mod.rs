#![allow(dead_code)]

use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use tt::app;
use tt::engine::{Engine, Query, Synonym, Translation};
use tt::env::Env;
use tt::error::{Error, Result};

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct TempDir(pub PathBuf);

impl TempDir {
    pub fn new() -> Self {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("tt-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone, Default)]
pub struct Probe {
    pub calls: Arc<AtomicUsize>,
    pub seen: Arc<Mutex<Vec<(String, String, String)>>>,
}

impl Probe {
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    pub fn last(&self) -> (String, String, String) {
        self.seen.lock().unwrap().last().cloned().unwrap()
    }
}

pub struct Fake {
    translation: Option<Translation>,
    failure: Option<String>,
    probe: Probe,
}

impl Fake {
    pub fn ok(translation: Translation) -> (Box<dyn Engine>, Probe) {
        let probe = Probe::default();
        let engine = Fake {
            translation: Some(translation),
            failure: None,
            probe: probe.clone(),
        };
        (Box::new(engine), probe)
    }

    pub fn failing(message: &str) -> (Box<dyn Engine>, Probe) {
        let probe = Probe::default();
        let engine = Fake {
            translation: None,
            failure: Some(message.to_string()),
            probe: probe.clone(),
        };
        (Box::new(engine), probe)
    }
}

impl Engine for Fake {
    fn name(&self) -> &'static str {
        "fake"
    }

    fn translate(&self, query: Query, _log: &mut dyn FnMut(&str)) -> Result<Translation> {
        self.probe.calls.fetch_add(1, Ordering::SeqCst);
        self.probe.seen.lock().unwrap().push((
            query.sl.to_string(),
            query.tl.to_string(),
            query.text.to_string(),
        ));
        match &self.failure {
            Some(message) => Err(Error::Network(message.clone())),
            None => Ok(self.translation.clone().unwrap()),
        }
    }
}

pub fn sample() -> Translation {
    Translation {
        primary: "good morning".to_string(),
        detected_source: Some("pt".to_string()),
        synonyms: vec![Synonym {
            word: "good morning".to_string(),
            back: vec!["bom dia".to_string()],
        }],
        ..Translation::default()
    }
}

pub struct Outcome {
    pub code: i32,
    pub out: String,
    pub err: String,
    pub dir: TempDir,
}

pub struct Setup {
    pub dir: TempDir,
    pub tty: bool,
    pub stdin: String,
    pub engine: Box<dyn Engine>,
    pub probe: Probe,
}

impl Setup {
    pub fn new() -> Self {
        let (engine, probe) = Fake::ok(sample());
        Setup {
            dir: TempDir::new(),
            tty: true,
            stdin: String::new(),
            engine,
            probe,
        }
    }

    pub fn piped(mut self) -> Self {
        self.tty = false;
        self
    }

    pub fn stdin(mut self, text: &str) -> Self {
        self.stdin = text.to_string();
        self
    }

    pub fn engine(mut self, engine: Box<dyn Engine>, probe: Probe) -> Self {
        self.engine = engine;
        self.probe = probe;
        self
    }

    pub fn config_path(&self) -> PathBuf {
        self.dir.path("config.toml")
    }

    pub fn run(self, args: &[&str]) -> Outcome {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let mut stdin = Cursor::new(self.stdin.into_bytes());
        let stdin_tty = stdin.get_ref().is_empty();
        let code = {
            let mut env = Env {
                out: &mut out,
                err: &mut err,
                stdin: &mut stdin,
                out_tty: self.tty,
                err_tty: self.tty,
                stdin_tty,
                engine: self.engine,
                config_path: self.dir.path("config.toml"),
            };
            app::run(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>(), &mut env)
        };
        Outcome {
            code,
            out: String::from_utf8(out).unwrap(),
            err: String::from_utf8(err).unwrap(),
            dir: self.dir,
        }
    }
}
