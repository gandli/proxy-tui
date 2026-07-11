use crate::spec::Spec;
use crate::Error;
use serde_json::json;

/// 渲染 sing-box 配置(MVP 占位:仅 direct 出站,hander 在 Phase 2 补全)。
pub fn render(_spec: &Spec) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "outbounds": [{ "type": "direct", "tag": "direct" }]
    }))
}

pub fn render_string(spec: &Spec) -> Result<String, Error> {
    let v = render(spec)?;
    serde_json::to_string_pretty(&v).map_err(|e| Error::Render(e.to_string()))
}
