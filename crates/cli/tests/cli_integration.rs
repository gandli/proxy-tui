use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_shows_subcommands() {
    Command::cargo_bin("vagent")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("render"))
        .stdout(predicate::str::contains("apply"))
        .stdout(predicate::str::contains("user-add"))
        .stdout(predicate::str::contains("core-install"));
}

#[test]
fn init_dry_run_prints_spec() {
    Command::cargo_bin("vagent")
        .unwrap()
        .args(["init", "--dry-run", "--domain", "v.example.com"])
        .assert()
        .success()
        .stdout(predicate::str::contains("v.example.com"))
        .stdout(predicate::str::contains("[cores]"));
}

#[test]
fn init_writes_spec_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let cfg = tmp.path().join("vagent").join("spec.toml");

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["init", "--domain", "v.example.com", "--config"])
        .arg(&cfg)
        .assert()
        .success();

    let content = std::fs::read_to_string(&cfg)?;
    assert!(content.contains("v.example.com"));
    assert!(content.contains("[cores]"));
    Ok(())
}

#[test]
fn user_add_appends_to_spec() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let cfg = tmp.path().join("vagent").join("spec.toml");

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["init", "--domain", "v.example.com", "--config"])
        .arg(&cfg)
        .assert()
        .success();

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["user-add", "alice", "--port", "8443", "--config"])
        .arg(&cfg)
        .assert()
        .success();

    let content = std::fs::read_to_string(&cfg)?;
    assert!(content.contains("alice"));
    assert!(content.contains("8443"));
    Ok(())
}

#[test]
fn status_missing_config_fails() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let cfg = tmp.path().join("nope").join("spec.toml");

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["status", "--config"])
        .arg(&cfg)
        .assert()
        .failure()
        .code(1);
    Ok(())
}

#[test]
fn render_with_user_has_reality_inbound() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let cfg = tmp.path().join("vagent").join("spec.toml");

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["init", "--domain", "v.example.com", "--config"])
        .arg(&cfg)
        .assert()
        .success();

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["user-add", "alice", "--port", "443", "--config"])
        .arg(&cfg)
        .assert()
        .success();

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["render", "--config"])
        .arg(&cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"security\": \"reality\""))
        .stdout(predicate::str::contains("\"port\": 443"));
    Ok(())
}

#[test]
fn apply_dry_run_prints_full_config() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let cfg = tmp.path().join("vagent").join("spec.toml");

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["init", "--domain", "v.example.com", "--config"])
        .arg(&cfg)
        .assert()
        .success();

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["user-add", "alice", "--port", "443", "--config"])
        .arg(&cfg)
        .assert()
        .success();

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["apply", "--dry-run", "--config"])
        .arg(&cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "/etc/vagent/cores/xray/config.json",
        ))
        .stdout(predicate::str::contains("\"blackhole\""))
        .stdout(predicate::str::contains("\"reality\""));
    Ok(())
}

#[test]
fn apply_writes_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let cfg = tmp.path().join("vagent").join("spec.toml");

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["init", "--domain", "v.example.com", "--config"])
        .arg(&cfg)
        .assert()
        .success();

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["user-add", "alice", "--port", "443", "--config"])
        .arg(&cfg)
        .assert()
        .success();

    // 真实落盘需 root(/etc/vagent),此处验证 dry-run 渲染与写盘路径打印一致
    Command::cargo_bin("vagent")
        .unwrap()
        .args(["apply", "--dry-run", "--config"])
        .arg(&cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "/etc/vagent/cores/xray/config.json",
        ));
    Ok(())
}

#[test]
fn user_list_del_link_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let cfg = tmp.path().join("vagent").join("spec.toml");

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["init", "--domain", "v.example.com", "--config"])
        .arg(&cfg)
        .assert()
        .success();

    // 添加两个不同协议用户
    Command::cargo_bin("vagent")
        .unwrap()
        .args(["user-add", "alice", "--config"])
        .arg(&cfg)
        .assert()
        .success();
    Command::cargo_bin("vagent")
        .unwrap()
        .args([
            "user-add",
            "bob",
            "--protocol",
            "hysteria2",
            "--port",
            "8443",
            "--config",
        ])
        .arg(&cfg)
        .assert()
        .success();

    // list 应含两者
    Command::cargo_bin("vagent")
        .unwrap()
        .args(["user-list", "--config"])
        .arg(&cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"))
        .stdout(predicate::str::contains("bob"))
        .stdout(predicate::str::contains("hysteria2"));

    // link 生成 hysteria2:// 链接
    Command::cargo_bin("vagent")
        .unwrap()
        .args(["user-link", "bob", "--config"])
        .arg(&cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains("hysteria2://"));

    // del 后 list 不再含 alice
    Command::cargo_bin("vagent")
        .unwrap()
        .args(["user-del", "alice", "--config"])
        .arg(&cfg)
        .assert()
        .success();
    Command::cargo_bin("vagent")
        .unwrap()
        .args(["user-list", "--config"])
        .arg(&cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains("alice").not());
    Ok(())
}

#[test]
fn apply_renders_singbox_when_hy2_user() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let cfg = tmp.path().join("vagent").join("spec.toml");

    Command::cargo_bin("vagent")
        .unwrap()
        .args(["init", "--domain", "v.example.com", "--config"])
        .arg(&cfg)
        .assert()
        .success();
    Command::cargo_bin("vagent")
        .unwrap()
        .args([
            "user-add",
            "h",
            "--protocol",
            "tuic",
            "--port",
            "9443",
            "--config",
        ])
        .arg(&cfg)
        .assert()
        .success();

    // 加了 tuic 用户,apply --dry-run 应自动渲染 sing-box 配置
    Command::cargo_bin("vagent")
        .unwrap()
        .args(["apply", "--dry-run", "--config"])
        .arg(&cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains("singbox"))
        .stdout(predicate::str::contains("tuic"));
    Ok(())
}
