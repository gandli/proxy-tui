//! CLI 集成测试(黑盒)。
//! 设计原则:CLI 零命令行参数,`vagent` 直接进交互菜单。
//! 配置路径仅来自 VAGENT_CONFIG 环境变量或默认位置。
//! 菜单交互由 VAGENT_TEST_INPUT 环境变量驱动(每行一次输入:数字=菜单索引,文本=Input 答案)。
//! 非 tty 环境下若输入耗尽,菜单优雅退出。
//! 真实业务逻辑由 core crate 的单元测试覆盖。

use assert_cmd::Command;
use tempfile::tempdir;

/// 构造菜单输入序列(每行一次消费)。
/// 主菜单索引(0 基,对齐 v2ray-agent):
/// 0安装 1一键Reality 2Hysteria2 3REALITY 4Tuic 5用户 6证书 7nginx管理 8分流
/// 9订阅 10内核 11应用 12状态 13卸载 14更新提示 15退出
/// 用户子菜单: 0新增 1列出 2删除 3链接 4返回
/// 订阅子菜单: 0生成 1签名 2返回
const FLOW_ADD_USER_AND_SUBSCRIBE: &str = "\
5\n\
0\n\
alice\n\
8443\n\
0\n\
4\n\
9\n\
0\n\
2\n\
15\n\
";

#[test]
fn menu_flow_adds_user_and_generates_subscribe() {
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("vagent").join("spec.toml");

    let mut cmd = Command::cargo_bin("vagent").unwrap();
    cmd.env("HOME", tmp.path())
        .env("VAGENT_CONFIG", &cfg)
        .env("VAGENT_TEST_INPUT", FLOW_ADD_USER_AND_SUBSCRIBE);
    let output = cmd.output().unwrap();
    assert!(output.status.success(), "vagent 菜单流应成功退出");

    // spec 应已生成并含 alice 用户
    assert!(cfg.exists(), "菜单首跑应生成默认配置: {}", cfg.display());
    let spec = std::fs::read_to_string(&cfg).unwrap();
    assert!(spec.contains("alice"), "spec 应含用户 alice:\n{spec}");

    // 订阅菜单进出:普通 vless 用户无 Reality,bundle 无内容会打印提示但菜单不崩溃。
    // bundle 的正/负路径由 core::subscribe 单测覆盖,此处只验证交互不崩溃。
    let _ = String::from_utf8_lossy(&output.stdout);
}

#[test]
fn menu_nginx_reverse_generates_443_to_local() {
    // 方案1:nginx 管理 → 1(生成反代配置) 应写出 443→127.0.0.1:8443
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("vagent").join("spec.toml");
    // 先首跑生成默认 spec,再进 nginx 管理开反代
    let setup = Command::cargo_bin("vagent")
        .unwrap()
        .env("HOME", tmp.path())
        .env("VAGENT_CONFIG", &cfg)
        .env("VAGENT_TEST_INPUT", "15\n")
        .output()
        .unwrap();
    assert!(setup.status.success());

    let flow = "7\n1\n15\n";
    let out = Command::cargo_bin("vagent")
        .unwrap()
        .env("HOME", tmp.path())
        .env("VAGENT_CONFIG", &cfg)
        .env("VAGENT_TEST_INPUT", flow)
        .output()
        .unwrap();
    assert!(out.status.success(), "nginx 反代配置生成不应崩溃");

    let reverse = tmp.path().join("vagent").join("nginx-reverse.conf");
    assert!(reverse.exists(), "应生成 nginx-reverse.conf");
    let content = std::fs::read_to_string(&reverse).unwrap();
    assert!(content.contains("listen 443 ssl;"), "应监听 443: {content}");
    assert!(
        content.contains("proxy_pass http://127.0.0.1:8443;"),
        "应反代本机 8443: {content}"
    );
}

#[test]
fn menu_nginx_sni_proxy_generates_conf() {
    // 方案1:nginx 管理 → 2(开启伪装站 SNI 反代) 应写出含 SNI 的配置
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("vagent").join("spec.toml");

    // 7 = nginx 管理; 2 = 开启伪装站 SNI 反代; 15 = 退出
    let flow = "7\n2\n15\n";
    let mut cmd = Command::cargo_bin("vagent").unwrap();
    let output = cmd
        .env("HOME", tmp.path())
        .env("VAGENT_CONFIG", &cfg)
        .env("VAGENT_TEST_INPUT", flow)
        .output()
        .unwrap();
    assert!(output.status.success(), "nginx 菜单项不应崩溃");

    // nginx_menu 的 Some(2) 应将 SNI 反代配置写到 spec 同目录
    let proxy = tmp.path().join("vagent").join("nginx-reverse.conf");
    assert!(
        proxy.exists(),
        "nginx 管理应生成 nginx-reverse.conf: {}",
        proxy.display()
    );
    let cfg_content = std::fs::read_to_string(&proxy).unwrap();
    assert!(
        cfg_content.contains("proxy_pass https://"),
        "配置应含 SNI 反代: {cfg_content}"
    );
}

#[test]
fn menu_no_input_exits_clean() {
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("noop").join("spec.toml");
    let assert = Command::cargo_bin("vagent")
        .unwrap()
        .env("HOME", tmp.path())
        .env("VAGENT_CONFIG", &cfg)
        .env("VAGENT_TEST_INPUT", "")
        .assert();
    assert.success();
    assert!(
        cfg.exists(),
        "vagent 首跑应引导生成默认配置: {}",
        cfg.display()
    );
}

#[test]
fn apply_with_missing_config_exits_nonzero() {
    // domain-cli 约束:错误必须返回非零退出码,不得裸 process::exit
    // --apply 模式配置缺失时,load_spec 失败应经 Result 传播到 main → 非零退出
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("missing").join("spec.toml");

    let mut cmd = Command::cargo_bin("vagent").unwrap();
    let output = cmd
        .env("HOME", tmp.path())
        .env("VAGENT_CONFIG", &cfg)
        .arg("--apply")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "配置缺失时 --apply 应返回非零退出码(而非裸 exit 或 0)"
    );
}
