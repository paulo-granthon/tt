pub mod google;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Synonym {
    pub word: String,
    pub back: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Translation {
    pub primary: String,
    pub detected_source: Option<String>,
    pub synonyms: Vec<Synonym>,
}

pub struct Query<'a> {
    pub sl: &'a str,
    pub tl: &'a str,
    pub text: &'a str,
}

pub trait Engine {
    fn name(&self) -> &'static str;
    fn translate(&self, query: Query) -> Result<Translation>;
}

pub fn default_engine() -> Box<dyn Engine> {
    Box::new(google::Google::new())
}
