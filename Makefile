.PHONY: test lint lint-fix install

run:
	cargo run --

test: lint
	cargo test

lint:
	cargo fmt --check
	cargo clippy -- -D warnings

lint-fix:
	cargo fmt
	cargo clippy --fix --allow-dirty

release:
	./release.sh

install:
	cargo install --path .
