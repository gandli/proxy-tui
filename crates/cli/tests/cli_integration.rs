//! CLI 集成测试(黑盒)。
//! 设计原则:CLI 不接受子命令参数,`vagent` 直接进交互菜单。
//! 非 tty 环境下 dialoguer 直接返回默认值/None,菜单优雅退出(exit 0)。
//! 真实业务逻辑由 core crate 的单元测试 + cli 内联测试覆盖。

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn vagent_no_args_enters_menu_and_exits_clean() {
    let tmp = tempdir().unwrap();
    // 用一个不存在的 config,引导初始化会写默认 spec 后再进菜单
    let cfg = tmp.path().join("nope").join("spec.toml");
    let assert = Command::cargo_bin("vagent")
        .unwrap()
        .env("HOME", tmp.path())
        .arg("--config")
        .arg(&cfg)
        .assert();
    // 非 tty 下菜单优雅退出,退出码 0(即使 config 原本不存在)
    assert.success();
    // 引导初始化应已生成默认 spec
    assert!(
        cfg.exists(),
        "vagent 首跑应引导生成默认配置: {}",
        cfg.display()
    );
}
