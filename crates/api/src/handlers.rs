//! API 视图层:纯函数,将 Spec 转成 JSON 视图,可单测、不碰网络。

use serde_json::json;
use vagent_core::Spec;

/// 状态视图(供 /api/status 与面板渲染)。
pub fn status_view(spec: &Spec) -> serde_json::Value {
    json!({
        "domain": spec.domain,
        "cores": {
            "xray": spec.cores.xray,
            "singbox": spec.cores.singbox
        },
        "rules": {
            "domain_blocklist": spec.rules.domain_blocklist,
            "block_bt": spec.rules.block_bt
        },
        "users": spec.users.iter().map(|u| json!({
            "name": u.name,
            "protocol": format!("{:?}", u.protocol).to_lowercase(),
            "port": u.port,
            "reality": u.reality
        })).collect::<Vec<_>>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vagent_core::Protocol;

    #[test]
    fn status_view_includes_domain_and_users() {
        let mut spec = Spec::default_for("v.example.com");
        spec.add_user("alice", Protocol::Vless, 443, true);
        let v = status_view(&spec);
        assert_eq!(v["domain"], serde_json::json!("v.example.com"));
        assert_eq!(v["users"].as_array().unwrap().len(), 1);
        assert_eq!(v["users"][0]["name"], serde_json::json!("alice"));
        assert_eq!(v["users"][0]["protocol"], serde_json::json!("vless"));
        assert_eq!(v["users"][0]["reality"], serde_json::json!(true));
    }

    #[test]
    fn status_view_rules_present() {
        let mut spec = Spec::default_for("x.com");
        spec.rules.block_bt = true;
        let v = status_view(&spec);
        assert_eq!(v["rules"]["block_bt"], serde_json::json!(true));
    }
}
