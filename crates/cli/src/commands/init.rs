use std::path::Path;
use vagent_core::{save_spec, Spec};

pub fn run(domain: &str, config: &Path, dry_run: bool) -> anyhow::Result<()> {
    let spec = Spec::default_for(domain);
    if dry_run {
        let s = toml::to_string_pretty(&spec)?;
        println!("{s}");
        return Ok(());
    }
    save_spec(&spec, config)?;
    println!("spec written to {}", config.display());
    Ok(())
}
