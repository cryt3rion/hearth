use anyhow::Result;
use serde::Serialize;

pub fn print<T: Serialize>(value: &T) -> Result<()> {
    let mut out = std::io::stdout().lock();
    serde_json::to_writer_pretty(&mut out, value)?;
    use std::io::Write;
    writeln!(out)?;
    Ok(())
}
