mod cli;
mod commands;

use clap::Parser;
use cli::Cli;
use std::path::PathBuf;
use vagent_core::Spec;

fn resolve_config() -> PathBuf {
    // 零命令行参数:配置路径仅来自 VAGENT_CONFIG 环境变量或默认位置
    std::env::var("VAGENT_CONFIG")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(Spec::default_config_path)
}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse(); // 零参数,仅解析 --version/--help
    let config = resolve_config();

    // 直接进入交互菜单(所有操作在菜单内完成)
    commands::menu::run(&config)?;
    Ok(())
}
