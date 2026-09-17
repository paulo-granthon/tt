pub mod fallback;
pub mod google;
pub mod lane;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Synonym {
    pub word: String,
    pub back: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Translation {
    pub primary: String,
    pub detected_source: Option<String>,
    pub synonyms: Vec<Synonym>,
    pub correction: Option<String>,
    pub source_translit: Option<String>,
    pub target_translit: Option<String>,
}

pub struct Query<'a> {
    pub sl: &'a str,
    pub tl: &'a str,
    pub text: &'a str,
}

pub trait Engine {
    fn name(&self) -> &'static str;
    fn translate(&self, query: Query, log: &mut dyn FnMut(&str)) -> Result<Translation>;
}

pub fn default_engine() -> Box<dyn Engine> {
    Box::new(google::Google::new())
}
