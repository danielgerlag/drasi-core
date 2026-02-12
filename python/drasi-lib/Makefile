.PHONY: build lint test integration-test publish clean

build:
	uv run maturin develop

lint:
	cargo clippy --all-targets --all-features
	uv run ruff check drasi_lib/

test:
	cargo test

integration-test:
	uv run pytest ../tests/ -v -k "drasi_lib"

publish:
	uv run maturin build --release

clean:
	cargo clean
	rm -rf target/
