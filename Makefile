ci:
	cargo fmt --all -- --check
	cargo clippy --features "unit-test" -- -D warnings
	cargo test --features "unit-test" --
