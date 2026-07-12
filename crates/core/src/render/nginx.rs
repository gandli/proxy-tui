use crate::spec::Spec;
use crate::Error;

/// 渲染「伪装站 SNI 反代」server block:
/// 把流量透传到外部真实站点(domain:443),用于 Reality 流量特征伪装。
/// 这是 nginx 只占 443 往外转的"出站伪装"用途。
pub fn render_sni(spec: &Spec) -> Result<String, Error> {
    let domain = &spec.domain;
    let block = format!(
        "server {{
    listen 443;
    server_name {domain};
    location / {{
        proxy_pass https://{domain}:443;
        proxy_ssl_server_name on;
        proxy_ssl_name {domain};
    }}
}}"
    );
    Ok(block)
}

/// 渲染「入站反代」server block:
/// nginx 监听 443,把外部流量转发到本机 xray/sing-box(reverse_port,通常 8443)。
/// root VPS 标准路径:nginx 以 root 持有 443,xray 绑高位端口,由 nginx 反代进来。
pub fn render_reverse(spec: &Spec) -> Result<String, Error> {
    let domain = &spec.domain;
    // reverse_port 缺省(0)时兜底 8443(标准高位端口,非 root 可绑)
    let port = if spec.nginx.reverse_port == 0 {
        8443
    } else {
        spec.nginx.reverse_port
    };
    let block = format!(
        "server {{
    listen 443 ssl;
    server_name {domain};
    ssl_certificate     /etc/vagent/certs/{domain}.cer;
    ssl_certificate_key /etc/vagent/certs/{domain}.key;
    location / {{
        proxy_pass http://127.0.0.1:{port};
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }}
}}"
    );
    Ok(block)
}

/// 按 spec.nginx 字段组合渲染(反代本机 + 可选伪装站)。
/// 无任何 nginx 配置时返回空串(调用方据此跳过写盘/包含)。
pub fn render_all(spec: &Spec) -> Result<String, Error> {
    let mut blocks = vec![];
    if spec.nginx.reverse_proxy {
        blocks.push(render_reverse(spec)?);
    }
    if spec.nginx.sni_proxy {
        blocks.push(render_sni(spec)?);
    }
    Ok(blocks.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::Spec;

    #[test]
    fn render_sni_contains_domain() {
        let spec = Spec::default_for("v.example.com");
        let cfg = render_sni(&spec).unwrap();
        assert!(cfg.contains("v.example.com"));
        assert!(cfg.contains("proxy_pass https://v.example.com:443"));
    }

    #[test]
    fn render_reverse_forwards_to_local_port() {
        let mut spec = Spec::default_for("v.example.com");
        spec.nginx.reverse_proxy = true;
        spec.nginx.reverse_port = 8443;
        let cfg = render_reverse(&spec).unwrap();
        assert!(cfg.contains("listen 443 ssl;"));
        assert!(cfg.contains("proxy_pass http://127.0.0.1:8443;"));
        assert!(cfg.contains("/etc/vagent/certs/v.example.com.cer"));
    }

    #[test]
    fn render_all_empty_when_no_nginx() {
        let spec = Spec::default_for("v.example.com");
        assert!(!spec.nginx.active());
        assert_eq!(render_all(&spec).unwrap(), "");
    }

    #[test]
    fn render_all_combines_reverse_and_sni() {
        let mut spec = Spec::default_for("v.example.com");
        spec.nginx.reverse_proxy = true;
        spec.nginx.sni_proxy = true;
        let cfg = render_all(&spec).unwrap();
        assert!(cfg.contains("proxy_pass http://127.0.0.1:8443;"));
        assert!(cfg.contains("proxy_pass https://v.example.com:443"));
    }
}
