
use crate::env::Env;
use crate::error::Result;

pub fn run(env: &mut Env) -> Result<i32> {
    let _ = writeln!(env.out, "updating tt to the latest release");
    crate::update::run()?;
    let _ = writeln!(env.out, "tt is up to date");
    Ok(0)
}
