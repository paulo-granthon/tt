use serde_json::Value;

use crate::engine::{Engine, Query, Synonym, Translation};
use crate::error::{Error, Result};

const ENDPOINT: &str = "https://translate.googleapis.com/translate_a/single";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
(KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

pub struct Google;

impl Google {
    pub fn new() -> Self {
        Google
    }
}

impl Default for Google {
    fn default() -> Self {
        Google
    }
}

impl Engine for Google {
    fn name(&self) -> &'static str {
        "google"
    }

    fn translate(&self, query: Query) -> Result<Translation> {
        let body = ureq::get(ENDPOINT)
            .query("client", "gtx")
            .query("sl", query.sl)
            .query("tl", query.tl)
            .query("dt", "t")
            .query("dt", "bd")
            .query("dt", "at")
            .query("q", query.text)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "*/*")
            .set("Accept-Language", "en-US,en;q=0.9")
            .set("Referer", "https://translate.google.com/")
            .call()
            .map_err(|error| match error {
                ureq::Error::Status(429, _) => Error::Network(format!(
                    "rate limited by Google (HTTP 429): too many requests from this network. \
Wait a while and try again, or open in a browser: {}",
                    crate::browser::translate_url(query.sl, query.tl, query.text)
                )),
                other => Error::Network(other.to_string()),
            })?
            .into_string()
            .map_err(|e| Error::Network(e.to_string()))?;
        parse_response(&body)
    }
}

pub fn parse_response(body: &str) -> Result<Translation> {
    let value: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;

    let mut primary = String::new();
    if let Some(sentences) = value.get(0).and_then(Value::as_array) {
        for sentence in sentences {
            if let Some(chunk) = sentence.get(0).and_then(Value::as_str) {
                primary.push_str(chunk);
            }
        }
    }
    if primary.is_empty() {
        return Err(Error::Parse("no translation in response".to_string()));
    }

    let detected_source = value.get(2).and_then(Value::as_str).map(str::to_string);

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
    })
}
