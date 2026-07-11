//! 卸载:停用并删除所有 vagent 服务单元。
//! 配置目录从 config 推导(root-optional),默认保留,--purge 时删除。

use vagent_core::executor::RealExecutor;
use vagent_core::spec::Spec;
use vagent_core::systemd;

pub fn run(purge: bool) -> anyhow::Result<()> {
    let config = Spec::default_config_path();
    let base = Spec::base_dir(&config);
    systemd::uninstall(&RealExecutor, &base).map_err(|e| anyhow::anyhow!(e))?;
    println!("已停用并移除 vagent 服务单元");
    if purge {
        if base.exists() {
            std::fs::remove_dir_all(&base)?;
            println!("已删除配置目录 {}", base.to_string_lossy());
        }
    } else {
        println!(
            "配置目录 {} 已保留(如需删除请加 --purge)",
            base.to_string_lossy()
        );
    }
    Ok(())
}
