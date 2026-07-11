//! Reality SNI 扫描:下载 RealiTLScanner 并对公网 IP 扫可用 SNI。
//! 经 RealExecutor 执行(真实副作用,需联网)。

use vagent_core::executor::RealExecutor;
use vagent_core::reality_scan;

pub fn run(public_ip: &str) -> anyhow::Result<()> {
    reality_scan::download(&RealExecutor).map_err(|e| anyhow::anyhow!(e))?;
    println!("扫描器已就绪,开始扫描 {public_ip} ...");
    let domains = reality_scan::scan(public_ip, &RealExecutor).map_err(|e| anyhow::anyhow!(e))?;
    if domains.is_empty() {
        println!("未找到可用 SNI(目标网络可能无前置站点)");
    } else {
        println!("可用 SNI:");
        for d in &domains {
            println!("  {d}");
        }
    }
    Ok(())
}
