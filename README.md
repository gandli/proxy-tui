# vagent

Rust 驱动的 Xray-core / sing-box 管理工具(spec 驱动,双核抽象)。

> 自托管运维工具,仅用于授权测试环境与自建 VPS。

## 架构

- `crates/core` — 共享核心库(spec 解析、配置渲染、订阅、TLS 拼装),**全部可单测**
- `crates/cli` — `vagent` 命令行二进制(薄封装,只解析参数 + 调 core)
- `crates/bot` — Telegram bot 前端(Phase 3 接入 teloxide,当前为纯函数骨架)

前端(TUI 已砍)共享同一套 `core`,互不耦合。

## 测试

- 单测:`cargo test`(core 纯函数 + bot handler)
- 集成:`assert_cmd` 跑真二进制,断言 stdout / 退出码 / 生成文件
- 快照:`insta` 锁配置渲染
- CI:fmt + clippy `-D warnings` + `cargo test --all`

## 开发流程

1. core 逻辑 → 单测
2. CLI 封装 → `assert_cmd` 集成
3. 复杂输出 → `insta` / `trycmd`
4. 发布前:全量测试 + clippy + fmt

## MVP

`vagent init` → `vagent render`(VLESS+Reality)→ `vagent status`。
