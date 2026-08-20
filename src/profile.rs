use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub sl: String,
    pub tl: String,
}

impl Profile {
    pub fn new(sl: impl Into<String>, tl: impl Into<String>) -> Self {
        Self {
            sl: sl.into(),
            tl: tl.into(),
        }
    }
}
