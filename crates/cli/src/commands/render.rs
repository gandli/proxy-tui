use std::path::Path;
use vagent_core::{load_spec, render::xray};

pub fn run(config: &Path) -> anyhow::Result<()> {
    let spec = load_spec(config)?;
    let out = xray::render_string(&spec)?;
    println!("{out}");
    Ok(())
}
