PACKAGE_NAME := drasi-reaction-storedproc-postgres-python

.PHONY: build lint test integration-test publish clean help

help:
	@echo "Available targets:"
	@echo "  build            - Build the Python package (maturin develop)"
	@echo "  lint             - Run cargo clippy + Python linting"
	@echo "  test             - Run Rust unit tests"
	@echo "  integration-test - Run Python integration tests (pytest)"
	@echo "  publish          - Build and publish wheel (maturin build --release)"
	@echo "  clean            - Clean build artifacts"

build:
	uv run maturin develop -m Cargo.toml

lint:
	cargo clippy -p $(PACKAGE_NAME) --all-targets --all-features
	uv run ruff check .

test:
	cargo test -p $(PACKAGE_NAME)

integration-test:
	uv run pytest ../tests/ -v -k "$(PACKAGE_NAME)" --timeout=60

publish:
	uv run maturin build -m Cargo.toml --release

clean:
	cargo clean -p $(PACKAGE_NAME)
