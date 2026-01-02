.PHONY: build release fmt clippy test clean

build:
	cargo build

release:
	cargo build --release

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace

clean:
	cargo clean
