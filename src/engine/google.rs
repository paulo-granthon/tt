use std::time::Duration;

use serde_json::Value;

use crate::browser;
use crate::engine::fallback::{race, DEFAULT_POLICY};
use crate::engine::lane::Lane;
use crate::engine::{Engine, Query, Synonym, Translation};
use crate::error::{Error, Result};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
(KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";
const GOOGLEAPIS: &str = "https://translate.googleapis.com";
const CLIENTS5: &str = "https://clients5.google.com";
const GOOGLE_COM: &str = "https://translate.google.com";

pub static LANES: [Lane; 8] = [
    Lane {
        name: "googleapis/dict-chrome-ex",
        build: &|agent, q| single(agent, q, GOOGLEAPIS, "dict-chrome-ex"),
        parse: parse_response,
    },
    Lane {
        name: "googleapis/at",
        build: &|agent, q| single(agent, q, GOOGLEAPIS, "at"),
        parse: parse_response,
    },
    Lane {
        name: "clients5/dict-chrome-ex",
        build: &|agent, q| single(agent, q, CLIENTS5, "dict-chrome-ex"),
        parse: parse_response,
    },
    Lane {
        name: "clients5/at",
        build: &|agent, q| single(agent, q, CLIENTS5, "at"),
        parse: parse_response,
    },
    Lane {
        name: "google.com/dict-chrome-ex",
        build: &|agent, q| single(agent, q, GOOGLE_COM, "dict-chrome-ex"),
        parse: parse_response,
    },
    Lane {
        name: "google.com/at",
        build: &|agent, q| single(agent, q, GOOGLE_COM, "at"),
        parse: parse_response,
    },
    Lane {
        name: "googleapis/t/dict-chrome-ex",
        build: &|agent, q| t(agent, q, GOOGLEAPIS, "dict-chrome-ex"),
        parse: parse_t_response,
    },
    Lane {
        name: "googleapis/t/at",
        build: &|agent, q| t(agent, q, GOOGLEAPIS, "at"),
        parse: parse_t_response,
    },
];

pub fn lane(name: &str) -> Option<&'static Lane> {
    LANES.iter().find(|lane| lane.name == name)
}

fn single(agent: &ureq::Agent, q: &Query, host: &str, client: &str) -> ureq::Request {
    browserish(agent.get(&format!("{host}/translate_a/single")))
        .query("client", client)
        .query("sl", q.sl)
        .query("tl", q.tl)
        .query("dt", "t")
        .query("dt", "bd")
        .query("dt", "at")
        .query("dt", "rm")
        .query("dt", "qc")
        .query("q", q.text)
}

fn t(agent: &ureq::Agent, q: &Query, host: &str, client: &str) -> ureq::Request {
    browserish(agent.get(&format!("{host}/translate_a/t")))
        .query("client", client)
        .query("sl", q.sl)
        .query("tl", q.tl)
        .query("q", q.text)
}

fn browserish(request: ureq::Request) -> ureq::Request {
    request
        .set("User-Agent", USER_AGENT)
        .set("Accept", "*/*")
        .set("Accept-Language", "en-US,en;q=0.9")
        .set("Referer", "https://translate.google.com/")
}

pub struct Google {
    agent: ureq::Agent,
}

impl Google {
    pub fn new() -> Self {
        Google {
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(5))
                .build(),
        }
    }
}

impl Default for Google {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for Google {
    fn name(&self) -> &'static str {
        "google"
    }

    fn translate(&self, query: Query, log: &mut dyn FnMut(&str)) -> Result<Translation> {
        let attempts = LANES
            .iter()
            .map(|lane| {
                let agent = self.agent.clone();
                let (sl, tl, text) = (query.sl.to_string(), query.tl.to_string(), query.text.to_string());
                let attempt = move || {
                    lane.call(
                        &agent,
                        &Query {
                            sl: &sl,
                            tl: &tl,
                            text: &text,
                        },
                    )
                };
                (lane.name.to_string(), attempt)
            })
            .collect();
        race(attempts, &DEFAULT_POLICY, log).map_err(|error| match error {
            Error::Network(why) => Error::Network(format!(
                "{why}; open in a browser: {}",
                browser::translate_url(query.sl, query.tl, query.text)
            )),
            other => other,
        })
    }
}

pub fn parse_response(body: &str) -> Result<Translation> {
    let value: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;

    let mut primary = String::new();
    let (mut source_translit, mut target_translit) = (None, None);
    if let Some(sentences) = value.get(0).and_then(Value::as_array) {
        for sentence in sentences {
            if let Some(chunk) = sentence.get(0).and_then(Value::as_str) {
                primary.push_str(chunk);
            } else {
                target_translit = target_translit.or_else(|| translit(sentence, 2));
                source_translit = source_translit.or_else(|| translit(sentence, 3));
            }
        }
    }
    if primary.is_empty() {
        return Err(Error::Parse("no translation in response".to_string()));
    }

    let detected_source = value.get(2).and_then(Value::as_str).map(str::to_string);

    let correction = value
        .get(7)
        .and_then(Value::as_array)
        .and_then(|c| c.get(1))
        .and_then(Value::as_str)
        .map(str::to_string);

    let mut synonyms = Vec::new();
    if let Some(groups) = value.get(1).and_then(Value::as_array) {
        for group in groups {
            let Some(entries) = group.get(2).and_then(Value::as_array) else {
                continue;
            };
            for entry in entries {
                let Some(word) = entry.get(0).and_then(Value::as_str) else {
                    continue;
                };
                let back = entry
                    .get(1)
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                synonyms.push(Synonym {
                    word: word.to_string(),
                    back,
                });
            }
        }
    }

    Ok(Translation {
        primary,
        detected_source,
        synonyms,
        correction,
        source_translit,
        target_translit,
    })
}

pub fn parse_t_response(body: &str) -> Result<Translation> {
    let value: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;
    let (primary, detected_source) = match value.get(0) {
        Some(Value::String(text)) => (text.clone(), None),
        Some(Value::Array(pair)) => (
            pair.first().and_then(Value::as_str).unwrap_or_default().to_string(),
            pair.get(1).and_then(Value::as_str).map(str::to_string),
        ),
        _ => (String::new(), None),
    };
    if primary.is_empty() {
        return Err(Error::Parse("no translation in response".to_string()));
    }
    Ok(Translation {
        primary,
        detected_source,
        ..Translation::default()
    })
}

fn translit(sentence: &Value, index: usize) -> Option<String> {
    sentence
        .get(index)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}
