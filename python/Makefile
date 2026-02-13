# Makefile for Drasi Python bindings

CRATES := \
	drasi-core \
	drasi-lib \
	sources/drasi-source-application \
	sources/drasi-source-postgres \
	sources/drasi-source-http \
	sources/drasi-source-grpc \
	sources/drasi-source-mock \
	reactions/drasi-reaction-application \
	reactions/drasi-reaction-http \
	reactions/drasi-reaction-grpc \
	reactions/drasi-reaction-sse \
	reactions/drasi-reaction-log \
	reactions/drasi-reaction-profiler \
	reactions/drasi-reaction-storedproc-mssql \
	reactions/drasi-reaction-storedproc-mysql \
	reactions/drasi-reaction-storedproc-postgres \
	bootstrappers/drasi-bootstrap-scriptfile \
	bootstrappers/drasi-bootstrap-postgres \
	bootstrappers/drasi-bootstrap-application \
	bootstrappers/drasi-bootstrap-noop \
	indexes/drasi-index-rocksdb \
	state-stores/drasi-state-store-redb \
	middleware/drasi-middleware

.PHONY: help build-all lint-all test-all integration-test-all publish-all clean-all

help:
	@echo "Available targets:"
	@echo "  build-all            - Build all Python packages (maturin develop)"
	@echo "  lint-all             - Lint all packages (clippy + ruff)"
	@echo "  test-all             - Run all Rust unit tests"
	@echo "  integration-test-all - Run all Python integration tests (pytest)"
	@echo "  publish-all          - Build release wheels for all packages"
	@echo "  clean-all            - Clean all build artifacts"

build-all:
	@for dir in $(CRATES); do \
		echo "=== Building $$dir ===" && \
		$(MAKE) -C $$dir build || exit 1; \
	done

lint-all:
	@for dir in $(CRATES); do \
		echo "=== Linting $$dir ===" && \
		$(MAKE) -C $$dir lint || exit 1; \
	done

test-all:
	@for dir in $(CRATES); do \
		echo "=== Testing $$dir ===" && \
		$(MAKE) -C $$dir test || exit 1; \
	done

integration-test-all:
	uv run pytest tests/ -v --timeout=60

publish-all:
	@for dir in $(CRATES); do \
		echo "=== Publishing $$dir ===" && \
		$(MAKE) -C $$dir publish || exit 1; \
	done

clean-all:
	@for dir in $(CRATES); do \
		echo "=== Cleaning $$dir ===" && \
		$(MAKE) -C $$dir clean || exit 1; \
	done
