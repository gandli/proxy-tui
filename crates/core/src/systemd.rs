//! systemd 单元生成(纯函数,可单测)。
//! 真实 enable 在 VPS 上执行;这里只产出单元文本。

use crate::Error;

/// 生成 vagent-xray.service 单元文本。
/// binary_path 默认 /usr/local/bin/vagent,config 默认 /etc/vagent/spec.toml。
pub fn xray_unit(binary_path: &str, config: &str) -> String {
    format!(
        "[Unit]\n\
Description=vagent Xray-core managed by vagent\n\
After=network.target\n\
\n\
[Service]\n\
Type=simple\n\
ExecStart={bin} apply --config {cfg}\n\
Restart=on-failure\n\
RestartSec=3\n\
User=root\n\
\n\
[Install]\n\
WantedBy=multi-user.target\n",
        bin = binary_path,
        cfg = config
    )
}

/// 生成 vagent-api.service(loopback 面板 API)。
pub fn api_unit(binary_path: &str) -> String {
    format!(
        "[Unit]\n\
Description=vagent local API (loopback panel)\n\
After=network.target\n\
\n\
[Service]\n\
Type=simple\n\
ExecStart={bin}\n\
Restart=on-failure\n\
RestartSec=3\n\
User=root\n\
\n\
[Install]\n\
WantedBy=multi-user.target\n",
        bin = binary_path
    )
}

/// 写单元到 /etc/systemd/system/(需 root,不在单测范围)。
pub fn install_unit(name: &str, content: &str) -> Result<(), Error> {
    let path = format!("/etc/systemd/system/{name}");
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xray_unit_contains_execstart() {
        let u = xray_unit("/usr/local/bin/vagent", "/etc/vagent/spec.toml");
        assert!(u.contains("Description=vagent Xray-core"));
        assert!(u.contains("ExecStart=/usr/local/bin/vagent apply --config /etc/vagent/spec.toml"));
        assert!(u.contains("WantedBy=multi-user.target"));
        assert!(u.contains("[Install]"));
    }

    #[test]
    fn api_unit_looback_service() {
        let u = api_unit("/usr/local/bin/vagent-api");
        assert!(u.contains("vagent local API"));
        assert!(u.contains("ExecStart=/usr/local/bin/vagent-api"));
    }
}
