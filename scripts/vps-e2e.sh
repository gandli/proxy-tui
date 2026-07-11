#!/usr/bin/env bash
# vagent 真机验收脚本(Docker 模拟 VPS)。
# 验证 vagent 渲染的配置能被真实 xray/sing-box 内核加载。
# 合规边界:仅用于授权测试环境/自建 VPS。
set -euo pipefail

echo "== 构建 vagent linux 二进制 =="
cargo build --release --bins

echo "== 起 Ubuntu 验收容器 =="
CID=$(docker run -d --privileged -v "$PWD/target/release/vagent:/usr/local/bin/vagent:ro" ubuntu:22.04 sleep infinity)
cleanup() { docker rm -f "$CID" >/dev/null 2>&1 || true; }
trap cleanup EXIT

docker exec "$CID" bash -c '
set -e
apt-get update -qq && apt-get install -y -qq curl unzip jq ca-certificates openssl >/dev/null 2>&1

echo "== 下载内核二进制 =="
curl -sL -o /tmp/xray.zip "https://github.com/XTLS/Xray-core/releases/download/v1.8.23/Xray-linux-64.zip"
unzip -oq /tmp/xray.zip -d /tmp/xray-bin
cp /tmp/xray-bin/xray /usr/local/bin/xray && chmod +x /usr/local/bin/xray
curl -sL -o /tmp/sb.tar.gz "https://github.com/SagerNet/sing-box/releases/download/v1.10.0/sing-box-1.10.0-linux-amd64.tar.gz"
tar xzf /tmp/sb.tar.gz -C /tmp
cp /tmp/sing-box-*/sing-box /usr/local/bin/sing-box && chmod +x /usr/local/bin/sing-box

export VAGENT_CONFIG=/etc/vagent/spec.toml
mkdir -p /etc/vagent

echo "== vagent 全流程 =="
vagent init --domain example.com
vagent user-add alice
vagent user-add bob --protocol tuic --port 9443
vagent reality-gen || true
vagent render --core xray --out /tmp/xray.json
vagent render --core singbox --out /tmp/sb.json

echo "== 自签证书(验收用,真实部署用 cert-issue) =="
mkdir -p /etc/vagent/certs
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout /etc/vagent/certs/example.com.key \
  -out /etc/vagent/certs/example.com.cer -days 365 -subj "/CN=example.com" >/dev/null 2>&1

echo "== 内核校验 =="
xray run -test -config /tmp/xray.json && echo "XRAY CONFIG OK"
sing-box check -c /tmp/sb.json && echo "SING-BOX CONFIG OK"
'
echo "== 验收通过 =="
