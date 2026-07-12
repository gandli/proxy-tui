use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "vagent", version, about = "Xray/sing-box 管理驱动 (spec 驱动)")]
pub struct Cli {
    /// 配置路径(默认 root: /etc/vagent/spec.toml,普通用户: ~/.config/vagent/spec.toml)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}
