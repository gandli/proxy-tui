//! Xray-core 配置渲染。
//! MVP 产出**完整合法**配置:inbounds(VLESS+Reality) + outbounds(direct/block) + routing。

use crate::spec::Spec;
use crate::Error;

/// 渲染 Xray-core 配置(JSON)—— MVP 仅处理 VLESS+Reality 入站。
/// 纯函数:输入 Spec,输出 serde_json::Value,不触网不落盘。
pub fn render(spec: &Spec) -> Result<serde_json::Value, Error> {
    let inbounds: Vec<serde_json::Value> = spec
        .users
        .iter()
        .filter(|u| matches!(u.protocol, crate::spec::Protocol::Vless) && u.reality)
        .map(|u| {
            serde_json::json!({
                "listen": "0.0.0.0",
                "port": u.port,
                "protocol": "vless",
                "settings": {
                    "clients": [{ "id": u.uuid, "level": 0 }]
                },
                "streamSettings": {
                    "network": "tcp",
                    "security": "reality",
                    "realitySettings": {
                        "dest": format!("{}:443", spec.domain),
                        "serverNames": [spec.domain.clone()],
                        "privateKey": "<generated-by-xray>",
                        "shortIds": [""]
                    }
                },
                "sniffing": { "enabled": true, "destOverride": ["http", "tls"] }
            })
        })
        .collect();

    let routing = spec.routing_json()?;

    Ok(serde_json::json!({
        "log": { "loglevel": "warning" },
        "inbounds": inbounds,
        "outbounds": [
            { "protocol": "freedom", "tag": "direct" },
            { "protocol": "blackhole", "tag": "block" }
        ],
        "routing": routing
    }))
}

pub fn render_string(spec: &Spec) -> Result<String, Error> {
    let v = render(spec)?;
    serde_json::to_string_pretty(&v).map_err(|e| Error::Render(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{Spec, User};

    #[test]
    fn render_filters_non_reality_vless() {
        let mut spec = Spec::default_for("x.com");
        spec.users
            .push(User::new("a", crate::spec::Protocol::Vless, 443, true));
        spec.users
            .push(User::new("b", crate::spec::Protocol::Vmess, 443, false));
        spec.users
            .push(User::new("c", crate::spec::Protocol::Vless, 8443, false));
        let v = render(&spec).unwrap();
        let inbounds = v["inbounds"].as_array().unwrap();
        assert_eq!(inbounds.len(), 1, "仅 VLESS+Reality 应入站");
        assert_eq!(inbounds[0]["port"], 443);
    }

    #[test]
    fn render_has_both_outbounds() {
        let spec = Spec::default_for("x.com");
        let v = render(&spec).unwrap();
        let tags: Vec<&str> = v["outbounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["tag"].as_str().unwrap())
            .collect();
        assert!(tags.contains(&"direct"));
        assert!(tags.contains(&"block"));
    }

    #[test]
    fn render_includes_routing_with_rules() {
        let mut spec = Spec::default_for("x.com");
        spec.rules.block_bt = true;
        let v = render(&spec).unwrap();
        assert!(!v["routing"]["rules"].as_array().unwrap().is_empty());
        assert_eq!(v["routing"]["domainStrategy"], "IPIfNonMatch");
    }

    #[test]
    fn render_reality_fields_present() {
        let mut spec = Spec::default_for("x.com");
        spec.users
            .push(User::new("a", crate::spec::Protocol::Vless, 443, true));
        let v = render(&spec).unwrap();
        let ib = &v["inbounds"][0];
        assert_eq!(ib["streamSettings"]["security"], "reality");
        assert_eq!(ib["streamSettings"]["realitySettings"]["dest"], "x.com:443");
    }
}
