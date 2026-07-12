mod cli;
mod commands;

use clap::Parser;
use cli::Cli;
use std::path::PathBuf;
use vagent_core::Spec;

fn default_config() -> PathBuf {
    Spec::default_config_path()
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = cli
        .config
        .or_else(|| std::env::var("VAGENT_CONFIG").ok().map(PathBuf::from))
        .unwrap_or_else(default_config);

    // 无任何子命令:直接进入交互菜单(所有操作在菜单内完成)
    commands::menu::run(&config)?;
    Ok(())
}
