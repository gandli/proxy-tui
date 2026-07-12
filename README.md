# vagent

Rust 编写的 Xray-core / sing-box 管理工具。spec 驱动、双核抽象、单文件部署。定位为 [v2ray-agent](https://github.com/mack-a/v2ray-agent) 的类型安全替代实现。

> 自托管运维工具。仅用于授权测试环境与自建 VPS。

## 快速开始

一句话安装(对标 v2ray-agent 的 `install.sh` 体验,musl 静态单文件,零依赖):

```bash
wget -P ~ -N --no-check-certificate "https://raw.githubusercontent.com/gandli/proxy-tui/main/install.sh" && bash ~/install.sh
```

安装完成后,**直接运行 `vagent` 进入交互式菜单**(对标 v2ray-agent 的 `vasma`),所有设定都在菜单里点选/输入完成,无需记命令行参数:

```bash
vagent            # 进入管理菜单:用户 / 内核 / 分流 / 证书 / Reality / 应用 ...
```

菜单内一级导航:用户管理 · 内核管理 · 分流规则 · 证书管理 · 服务管理 · Reality · 应用配置 · 查看状态 · 卸载。

> 子命令(如 `vagent user-add alice`)仍保留,供脚本与自动化调用,与菜单等价。

## 普通用户(非 root)用法

vagent 默认走 root-optional:检测到非 root 时,所有路径(配置、证书、二进制、服务单元)自动落在 `$HOME` 下,不碰 `/etc`、`/usr/local`。普通用户用 systemd `--user` 管理常驻进程。

| 资源 | root 路径 | 普通用户路径 |
|---|---|---|
| 配置 spec | `/etc/vagent/spec.toml` | `~/.config/vagent/spec.toml` |
| 证书 / 内核配置 / reality 扫描 | `/etc/vagent/...` | `~/.config/vagent/...` |
| 订阅签名 secret | `/etc/vagent/secret` | `~/.config/vagent/secret` |
| 二进制 | `/usr/local/bin` | `~/.local/bin` |
| 服务单元 | `/etc/systemd/system` | `~/.config/systemd/user` |
| 单元 User 行 | `root` | `%u`(当前用户) |
| 卸载 purge | `/etc/vagent` | `~/.config/vagent` |

```bash
# 普通用户安装(自动选 ~/.local/bin + ~/.config/vagent)
wget -P ~ -N --no-check-certificate "https://raw.githubusercontent.com/gandli/proxy-tui/main/install.sh" && bash ~/install.sh

# 普通用户前台跑(无需 systemd)
vagent apply
vagent core start xray          # 用你自己 PATH 里的 xray 二进制

# 普通用户常驻(systemd --user)
vagent service install --core xray --init systemd
systemctl --user daemon-reload
systemctl --user enable --now vagent-xray
```

> 普通用户前台监听 443/80 等 <1024 端口需要 CAP_NET_BIND_SERVICE;`systemd --user` 模式下由 systemd 处理,手动前台需 `setcap` 或高端口。

二进制来源:CI 自动构建的 musl 静态发行(`vagent` + `vagent-api`),安装脚本从最新 GitHub Release 拉取。普通用户安装到 `~/.local/bin` + `~/.config/vagent`,不强制 root;root 则装到 `/usr/local/bin` + `/etc/vagent` 并注册 systemd。

## 设计

单一真相源:一份 `spec.toml` 描述域名、内核、用户、分流规则。所有内核配置、订阅链接、systemd 单元都从 spec 渲染得出,不反向解析 JSON。

```
spec.toml ──┬─→ render/xray    → <base>/cores/xray/config.json
            ├─→ render/singbox → <base>/cores/singbox/config.json
            ├─→ subscribe      → vless:// vmess:// trojan:// hysteria2:// tuic://
            └─→ routing        → 分流规则段
```
> `<base>` 即 spec.toml 的父目录:`root` 为 `/etc/vagent`,普通用户为 `~/.config/vagent`。

系统副作用(下载、systemctl、acme.sh、写盘)全部经 `Executor` trait 出口,测试注入 `FakeExecutor`,渲染逻辑纯函数可测。

## 架构

| crate | 职责 |
|---|---|
| `core` | 共享核心库:spec、渲染、订阅、路由、TLS、systemd、下载。全部可单测 |
| `cli` | `vagent` 命令行,薄封装:解析参数 + 调 core |
| `bot` | Telegram bot(teloxide,UID 白名单,token 走 `VAGENT_BOT_TOKEN`) |
| `api` | axum loopback API(127.0.0.1:7800)+ 零 JS 面板 |

三前端共享同一份 `core`,互不耦合。

## 协议支持

| 协议 | 承载内核 | 传输 |
|---|---|---|
| VLESS + Reality | Xray | TCP + XTLS-Vision |
| VMess | Xray | WebSocket |
| Trojan | Xray | TLS |
| Hysteria2 | sing-box | QUIC + TLS |
| Tuic | sing-box | QUIC + BBR |

加了 Hysteria2/Tuic 用户时 sing-box 内核自动启用,无需手动切换。

## 命令

```
vagent init --domain example.com          # 生成初始 spec
vagent apply [--dry-run]                   # 渲染并重载启用的内核

# 用户
vagent user-add alice --protocol vless --port 443
vagent user-list
vagent user-link alice                     # 输出分享链接
vagent user-del alice
vagent user-add bob --protocol hysteria2 --port 8443 --transport tcp   # 指定协议/传输

# Reality 密钥与 SNI
vagent reality-gen                          # 用 xray x25519 为所有 Reality 用户生成真实密钥
vagent reality-scan 1.2.3.4                 # 扫描公网 IP 可用 SNI(RealiTLScanner)

# 内核生命周期
vagent core start   --core xray            # start/stop/restart/enable/disable
vagent core-install --core xray --version 1.8.0

# 服务单元(systemd / openrc)
vagent service show   --core xray --init systemd
vagent service install --core xray --init openrc   # Alpine 用 openrc
vagent service install --core api  --init systemd  # 面板 API 单元(同样 root-optional)

# 分流
vagent route direct bank.com               # 强制直连白名单
vagent route warp   netflix.com            # 走 WARP 出站
vagent route block  evil.com               # 黑名单
vagent route ads on                        # geosite 广告拦截
vagent route bt  on                        # 阻断 BT
vagent route list

# 证书(acme.sh)
vagent cert-issue example.com --ca letsencrypt          # standalone
vagent cert-issue example.com --ca zerossl --dns dns_cf # DNS 验证
vagent cert-renew

# 卸载
vagent uninstall [--purge]                 # --purge 一并删配置目录

# 订阅(把多用户节点打包成一个订阅 URL 发给别人)
vagent subscribe                            # 输出 v2rayN 格式订阅 bundle(base64)
vagent subscribe --sign                     # 带服务端 HMAC 签名(用于识别/吊销)
```

配置路径优先级:`--config` > `VAGENT_CONFIG` 环境变量 > 默认(`root` → `/etc/vagent/spec.toml`,普通用户 → `~/.config/vagent/spec.toml`)。所有派生路径(证书、内核配置、secret)都从 spec 的父目录推导。

## 分流优先级

规则按顺序取首个匹配:

1. 直连白名单(`route direct`)
2. 广告拦截(`route ads on`)
3. 域名黑名单(`route block`)
4. BT 阻断(`route bt on`)
5. WARP 分流(`route warp`)

## 测试

```
cargo test --all          # 单测(core 纯函数)+ 集成(assert_cmd 跑真二进制)
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

CLI 集成测试用 `assert_cmd` + `tempfile`,断言 stdout / 退出码 / 生成文件。不使用 Playwright,也不做真机/浏览器 e2e(遵循项目约定,验证仅以 `cargo test` + `FakeExecutor` 单测覆盖)。系统副作用(systemctl、acme.sh、下载)经 `Executor` 出口,测试注入 `FakeExecutor` 捕获命令,不真正执行。

退出码:`0` 成功 / `1` 配置错误 / `2` 系统或权限 / `3` 网络或下载。

## 部署

```
cargo build --release --target x86_64-unknown-linux-musl
```

产出零依赖静态单文件,直接投放 VPS。CI 已含 musl 交叉编译 job。

## 开发流程

1. core 逻辑先行 → 单测
2. CLI 封装 → `assert_cmd` 集成测试
3. 发布前:`cargo test --all` + clippy `-D warnings` + fmt
4. 变更走 PR,不直推 main
