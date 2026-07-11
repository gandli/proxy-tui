#!/usr/bin/env bash
# vagent 一键安装脚本(对标 v2ray-agent install.sh 的体验)。
# 用法(普通用户也行):
#   wget -P ~ -N --no-check-certificate "https://raw.githubusercontent.com/gandli/proxy-tui/main/install.sh" && bash ~/install.sh
#
# 尽量不要求 root:
#   - root 用户:装到 /usr/local/bin + /etc/vagent + systemd 单元
#   - 普通用户:装到 ~/.local/bin + ~/.config/vagent,不碰 systemd(手动前台跑)
#
# 合规边界:仅用于授权测试环境 / 自建 VPS。
set -euo pipefail

REPO="gandli/proxy-tui"
VERSION="${1:-latest}"

echo "== vagent 安装器 =="

# 按权限选安装根:root 走系统目录,普通用户走 HOME
if [ "$(id -u)" = "0" ]; then
  BIN_DIR="/usr/local/bin"
  SPEC_DIR="/etc/vagent"
  ROOT_INSTALL=1
else
  BIN_DIR="$HOME/.local/bin"
  SPEC_DIR="$HOME/.config/vagent"
  ROOT_INSTALL=0
fi

# 确保 bin 目录存在并加入 PATH(普通用户)
mkdir -p "$BIN_DIR"
if [ "$ROOT_INSTALL" = "0" ]; then
  case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) export PATH="$BIN_DIR:$PATH" ;;
  esac
fi

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
echo "二进制已安装: $("$BIN_DIR/vagent" --version 2>&1 | head -1)"

echo "== 初始化 spec =="
mkdir -p "$SPEC_DIR"
[ -f "$SPEC_DIR/spec.toml" ] || "$BIN_DIR/vagent" init --domain "$(hostname -f 2>/dev/null || echo example.com)" --config "$SPEC_DIR/spec.toml"

if [ "$ROOT_INSTALL" = "1" ]; then
  echo "== 安装 systemd 单元 =="
  "$BIN_DIR/vagent" service install --core xray --init systemd || true
  "$BIN_DIR/vagent" service install --core api --init systemd || true

  echo "== 启动 =="
  systemctl daemon-reload 2>/dev/null || true
  systemctl enable vagent-xray 2>/dev/null || true
  systemctl start vagent-xray 2>/dev/null || true
else
  echo "== 普通用户模式:跳过 systemd =="
  echo "可手动前台运行(无 root 也能起 xray,superuser 仅 443/80 等 <1024 端口需要):"
  echo "  vagent apply && vagent core start xray   # 用你自己的 xray 二进制"
fi

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
