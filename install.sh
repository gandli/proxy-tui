#!/usr/bin/env bash
# vagent 一键安装脚本(对标 v2ray-agent install.sh 的体验)。
# 用法:
#   wget -P /root -N --no-check-certificate "https://raw.githubusercontent.com/gandli/proxy-tui/main/install.sh" && bash /root/install.sh
#
# 合规边界:仅用于授权测试环境 / 自建 VPS。
set -euo pipefail

REPO="gandli/proxy-tui"
BIN_DIR="/usr/local/bin"
SPEC_DIR="/etc/vagent"
SERVICE_DIR="/etc/systemd/system"
VERSION="${1:-latest}"

echo "== vagent 安装器 =="

# 解析最新 release 版本(若未指定)
if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep -oE '"tag_name": *"v?[0-9.]+"' | head -1 | grep -oE 'v?[0-9.]+') || true
  [ -z "$VERSION" ] && VERSION="latest"
fi
echo "目标版本: $VERSION"

BASE="https://github.com/${REPO}/releases/download/${VERSION}"
# latest 无 tag,改用 main 分支的 dist 资产兜底(若存在)
if [ "$VERSION" = "latest" ]; then
  BASE="https://raw.githubusercontent.com/${REPO}/main/dist"
fi

echo "== 下载 musl 单文件二进制 =="
if [ "${SKIP_DOWNLOAD:-0}" = "1" ]; then
  echo "(SKIP_DOWNLOAD=1,使用已存在的二进制)"
else
  curl -sL -o "$BIN_DIR/vagent" "${BASE}/vagent" && chmod +x "$BIN_DIR/vagent"
  curl -sL -o "$BIN_DIR/vagent-api" "${BASE}/vagent-api" && chmod +x "$BIN_DIR/vagent-api"
fi
echo "二进制已安装: $(vagent --version 2>&1 | head -1)"

echo "== 初始化 spec =="
mkdir -p "$SPEC_DIR"
[ -f "$SPEC_DIR/spec.toml" ] || vagent init --domain "$(hostname -f 2>/dev/null || echo example.com)"

echo "== 安装 systemd 单元 =="
vagent service install --core xray --init systemd || true
vagent service install --core api --init systemd || true

echo "== 启动 =="
systemctl daemon-reload 2>/dev/null || true
systemctl enable vagent-xray 2>/dev/null || true
systemctl start vagent-xray 2>/dev/null || true

echo ""
echo "== 安装完成 =="
echo "常用命令:"
echo "  vagent user-add alice              # 新增 Reality 用户"
echo "  vagent user-link alice             # 生成分享链接"
echo "  vagent reality-gen                 # 生成 Reality 密钥"
echo "  vagent apply                       # 渲染并应用配置"
echo "  vagent --help                      # 全部子命令"
echo ""
echo "再次配置:直接运行 vagent <子命令>"
