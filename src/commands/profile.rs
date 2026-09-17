
use crate::color::{paint, BOLD, DIM, GREEN};
use crate::config::{Config, DEFAULT_PROFILE};
use crate::env::Env;
use crate::error::Result;

pub fn default_show(env: &mut Env) -> Result<i32> {
    let config = Config::load_from(&env.config_path)?;
    if let Some(p) = config.profiles.get(DEFAULT_PROFILE) {
        let _ = writeln!(env.out, "sl={} tl={}", p.sl, p.tl);
    }
    Ok(0)
}

pub fn default_set(sl: Option<&str>, tl: Option<&str>, env: &mut Env) -> Result<i32> {
    let mut config = Config::load_from(&env.config_path)?;
    config.set_default(sl, tl)?;
    config.save_to(&env.config_path)?;
    if let Some(p) = config.profiles.get(DEFAULT_PROFILE) {
        let _ = writeln!(env.out, "default set: sl={} tl={}", p.sl, p.tl);
    }
    Ok(0)
}

pub fn add(name: &str, sl: &str, tl: &str, env: &mut Env) -> Result<i32> {
    let mut config = Config::load_from(&env.config_path)?;
    config.add(name, sl, tl)?;
    config.save_to(&env.config_path)?;
    let _ = writeln!(env.out, "profile added: {name}");
    Ok(0)
}

pub fn list(env: &mut Env) -> Result<i32> {
    let config = Config::load_from(&env.config_path)?;
    let color = env.out_tty;
    let width = config.profiles.keys().map(String::len).max().unwrap_or(0);
    for (name, p) in &config.profiles {
        let tag = if name == DEFAULT_PROFILE {
            paint(color, DIM, "  (used when no p= is given)")
        } else {
            String::new()
        };
        let _ = writeln!(
            env.out,
            "  {}  {}{}  {}{}{tag}",
            paint(color, BOLD, &format!("{name:<width$}")),
            paint(color, DIM, "sl="),
            paint(color, GREEN, &p.sl),
            paint(color, DIM, "tl="),
            paint(color, GREEN, &p.tl),
        );
    }
    Ok(0)
}

pub fn delete(name: &str, env: &mut Env) -> Result<i32> {
    let mut config = Config::load_from(&env.config_path)?;
    config.delete(name)?;
    config.save_to(&env.config_path)?;
    let _ = writeln!(env.out, "profile deleted: {name}");
    Ok(0)
}

pub fn patch(
    name: &str,
    sl: Option<&str>,
    tl: Option<&str>,
    new_name: Option<&str>,
    env: &mut Env,
) -> Result<i32> {
    let mut config = Config::load_from(&env.config_path)?;
    config.patch(name, sl, tl, new_name)?;
    config.save_to(&env.config_path)?;
    let _ = writeln!(env.out, "profile patched: {name}");
    Ok(0)
}
