.PHONY: test
test:
	cargo fmt -- --check
	cargo-sort --check --workspace
	cargo clippy --all-features --workspace -- -D warnings
	cargo test --all-features --workspace

.PHONY: format
format:
	cargo fmt
	cargo-sort --workspace

.PHONY: installer
installer:
	makensis installer/windows.nsi
