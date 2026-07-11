//! 分流规则:MVP 仅「黑名单域名」+「BT 阻断」,spec 驱动 → 渲染成内核 routing 段。
//! 与 render 同构:纯函数,不碰系统,可单测。

use crate::spec::Spec;
use crate::Error;
use serde_json::json;

impl Spec {
    /// 渲染 routing 规则段(Xray routing 结构)。
    pub fn routing_json(&self) -> Result<serde_json::Value, Error> {
        let mut rules: Vec<serde_json::Value> = vec![];

        if !self.rules.domain_blocklist.is_empty() {
            rules.push(json!({
                "type": "field",
                "domain": self.rules.domain_blocklist,
                "outboundTag": "block"
            }));
        }
        if self.rules.block_bt {
            rules.push(json!({
                "type": "field",
                "protocol": ["bittorrent"],
                "outboundTag": "block"
            }));
        }

        Ok(json!({
            "domainStrategy": "IPIfNonMatch",
            "rules": rules
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::Spec;

    #[test]
    fn empty_rules_no_block_entries() {
        let spec = Spec::default_for("x.com");
        let r = spec.routing_json().unwrap();
        assert_eq!(r["rules"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn domain_blocklist_renders_field() {
        let mut spec = Spec::default_for("x.com");
        spec.rules.domain_blocklist.push("evil.com".into());
        let r = spec.routing_json().unwrap();
        let rules = r["rules"].as_array().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0]["outboundTag"], "block");
        assert!(rules[0]["domain"]
            .as_array()
            .unwrap()
            .contains(&json!("evil.com")));
    }

    #[test]
    fn block_bt_renders_protocol_rule() {
        let mut spec = Spec::default_for("x.com");
        spec.rules.block_bt = true;
        let r = spec.routing_json().unwrap();
        let rules = r["rules"].as_array().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0]["protocol"][0], "bittorrent");
    }

    #[test]
    fn both_rules_combine() {
        let mut spec = Spec::default_for("x.com");
        spec.rules.domain_blocklist.push("a.com".into());
        spec.rules.block_bt = true;
        let r = spec.routing_json().unwrap();
        assert_eq!(r["rules"].as_array().unwrap().len(), 2);
    }
}
