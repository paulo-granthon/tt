use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::lang;
use crate::profile::Profile;

pub const DEFAULT_PROFILE: &str = "default";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

pub struct Resolved {
    pub sl: String,
    pub tl: String,
}

impl Config {
    pub fn with_builtin_default() -> Self {
        let mut c = Config::default();
        c.profiles
            .insert(DEFAULT_PROFILE.to_string(), Profile::new("auto", "en"));
        c
    }

    pub fn ensure_default(&mut self) {
        self.profiles
            .entry(DEFAULT_PROFILE.to_string())
            .or_insert_with(|| Profile::new("auto", "en"));
    }

    pub fn path() -> Result<PathBuf> {
        directories::ProjectDirs::from("", "", "tt")
            .map(|d| d.config_dir().join("config.toml"))
            .ok_or_else(|| Error::Config("could not determine config directory".to_string()))
    }

    pub fn load() -> Result<Config> {
        Self::load_from(&Self::path()?)
    }

    pub fn load_from(path: &Path) -> Result<Config> {
        match std::fs::read_to_string(path) {
            Ok(s) => {
                let mut c: Config =
                    toml::from_str(&s).map_err(|e| Error::Config(e.to_string()))?;
                c.ensure_default();
                Ok(c)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::with_builtin_default()),
            Err(e) => Err(Error::Config(e.to_string())),
        }
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::path()?)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::Config(e.to_string()))?;
        }
        let body = toml::to_string_pretty(self).map_err(|e| Error::Config(e.to_string()))?;
        std::fs::write(path, body).map_err(|e| Error::Config(e.to_string()))
    }

    pub fn resolve(
        &self,
        profile: Option<&str>,
        sl: Option<&str>,
        tl: Option<&str>,
    ) -> Result<Resolved> {
        let base_name = profile.unwrap_or(DEFAULT_PROFILE);
        let base = self
            .profiles
            .get(base_name)
            .ok_or_else(|| Error::BadArgs(format!("unknown profile: {base_name}")))?;
        Ok(Resolved {
            sl: lang::resolve_source(sl.unwrap_or(&base.sl))?,
            tl: lang::resolve_target(tl.unwrap_or(&base.tl))?,
        })
    }

    pub fn set_default(&mut self, sl: Option<&str>, tl: Option<&str>) -> Result<()> {
        self.patch(DEFAULT_PROFILE, sl, tl, None)
    }

    pub fn add(&mut self, name: &str, sl: &str, tl: &str) -> Result<()> {
        if name.is_empty() {
            return Err(Error::BadArgs("profile name required".to_string()));
        }
        if self.profiles.contains_key(name) {
            return Err(Error::BadArgs(format!("profile already exists: {name}")));
        }
        let profile = Profile::new(lang::resolve_source(sl)?, lang::resolve_target(tl)?);
        self.profiles.insert(name.to_string(), profile);
        Ok(())
    }

    pub fn delete(&mut self, name: &str) -> Result<()> {
        if name == DEFAULT_PROFILE {
            return Err(Error::BadArgs(
                "cannot delete the default profile".to_string(),
            ));
        }
        self.profiles
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| Error::BadArgs(format!("unknown profile: {name}")))
    }

    pub fn patch(
        &mut self,
        name: &str,
        sl: Option<&str>,
        tl: Option<&str>,
        new_name: Option<&str>,
    ) -> Result<()> {
        let sl = sl.map(lang::resolve_source).transpose()?;
        let tl = tl.map(lang::resolve_target).transpose()?;
        let mut profile = self
            .profiles
            .get(name)
            .cloned()
            .ok_or_else(|| Error::BadArgs(format!("unknown profile: {name}")))?;
        if let Some(sl) = sl {
            profile.sl = sl;
        }
        if let Some(tl) = tl {
            profile.tl = tl;
        }
        match new_name {
            Some(new_name) if new_name != name => {
                if name == DEFAULT_PROFILE {
                    return Err(Error::BadArgs(
                        "cannot rename the default profile".to_string(),
                    ));
                }
                if new_name == DEFAULT_PROFILE {
                    return Err(Error::BadArgs(
                        "cannot name a profile 'default'".to_string(),
                    ));
                }
                if self.profiles.contains_key(new_name) {
                    return Err(Error::BadArgs(format!("profile already exists: {new_name}")));
                }
                self.profiles.remove(name);
                self.profiles.insert(new_name.to_string(), profile);
            }
            _ => {
                self.profiles.insert(name.to_string(), profile);
            }
        }
        Ok(())
    }
}
