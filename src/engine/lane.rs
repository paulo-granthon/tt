use crate::engine::{Query, Translation};
use crate::error::{Error, Result};

pub type Build = &'static (dyn Fn(&ureq::Agent, &Query) -> ureq::Request + Sync);

pub struct Lane {
    pub name: &'static str,
    pub build: Build,
    pub parse: fn(&str) -> Result<Translation>,
}

impl Lane {
    pub fn call(&self, agent: &ureq::Agent, query: &Query) -> Result<Translation> {
        let body = (self.build)(agent, query)
            .call()
            .map_err(|error| match error {
                ureq::Error::Status(429, _) => Error::Network("HTTP 429 rate limited".to_string()),
                ureq::Error::Status(code, _) => Error::Network(format!("HTTP {code}")),
                ureq::Error::Transport(transport) => Error::Network(transport.to_string()),
            })?
            .into_string()
            .map_err(|e| Error::Network(e.to_string()))?;
        (self.parse)(&body)
    }
}
