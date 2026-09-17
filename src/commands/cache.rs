use crate::cache::Cache;
use crate::env::Env;
use crate::error::Result;

pub fn stats(env: &mut Env) -> Result<i32> {
    let cache = Cache::new(env.cache_dir.clone());
    let stats = cache.stats()?;
    let _ = writeln!(
        env.out,
        "entries: {}\nsize: {}\npath: {}",
        stats.entries,
        size(stats.bytes),
        cache.dir().display()
    );
    Ok(0)
}

pub fn clear(env: &mut Env) -> Result<i32> {
    Cache::new(env.cache_dir.clone()).clear()?;
    let _ = writeln!(env.out, "cache cleared");
    Ok(0)
}

fn size(bytes: u64) -> String {
    match bytes {
        0..=1023 => format!("{bytes} B"),
        1024..=1_048_575 => format!("{:.1} KB", bytes as f64 / 1024.0),
        _ => format!("{:.1} MB", bytes as f64 / 1_048_576.0),
    }
}
