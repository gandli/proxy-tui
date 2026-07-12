# vagent 本地门禁快捷命令
# 用法: make check  (fmt + test + clippy 一次性跑完)
#
# 注意:rustfmt/clippy 版本由仓库根 rust-toolchain.toml 锁定,
# 本地与 CI 一致,避免 fmt 风格差异导致 CI 二次推送。

.PHONY: fmt fmt-check test clippy check build

build:
	cargo build --all

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

test:
	cargo test --all

clippy:
	cargo clippy --all-targets -- -D warnings

# 提交前门禁:格式化 → 测试 → lint
check: fmt test clippy
	@echo "✓ fmt + test + clippy 全绿"
