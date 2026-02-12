# Drasi Python Bindings

Python bindings for [drasi-core](https://github.com/drasi-project/drasi-core), providing PyO3-based wrappers for the Drasi continuous query engine and all its component plugins.

## Requirements

- Python 3.11+
- Rust toolchain (for building native extensions)
- [uv](https://docs.astral.sh/uv/) (Python package/environment manager)
- [maturin](https://github.com/PyO3/maturin) (installed as dev dependency)

## Quick Start

```bash
# Set up environment
cd python
uv sync

# Build all packages
make build-all

# Run tests
make test-all

# Run an example
uv run python examples/basic_usage.py
```

## Packages

### Core

| Package | Description |
|---------|-------------|
| `drasi-core` | Shared types: `DispatchMode`, `DrasiError`, `PySourceWrapper`, `PyReactionWrapper`, etc. |
| `drasi-lib` | High-level API: `DrasiLibBuilder`, `Query`, `PyDrasiLib`, `ComponentStatus`, streaming |

### Sources

| Package | Description |
|---------|-------------|
| `drasi-source-application` | In-process application source with `PropertyMapBuilder` |
| `drasi-source-http` | HTTP endpoint source |
| `drasi-source-grpc` | gRPC source |
| `drasi-source-postgres` | PostgreSQL WAL replication source |
| `drasi-source-mock` | Mock source for testing |

### Reactions

| Package | Description |
|---------|-------------|
| `drasi-reaction-application` | In-process application reaction with result streaming |
| `drasi-reaction-log` | Handlebars template-based log reaction |
| `drasi-reaction-http` | HTTP adaptive reaction |
| `drasi-reaction-grpc` | gRPC adaptive reaction |
| `drasi-reaction-sse` | Server-Sent Events reaction |
| `drasi-reaction-profiler` | Performance profiler reaction |
| `drasi-reaction-storedproc-mssql` | MSSQL stored procedure reaction |
| `drasi-reaction-storedproc-mysql` | MySQL stored procedure reaction |
| `drasi-reaction-storedproc-postgres` | PostgreSQL stored procedure reaction |

### Bootstrappers

| Package | Description |
|---------|-------------|
| `drasi-bootstrap-noop` | No-op bootstrap provider |
| `drasi-bootstrap-scriptfile` | JSONL script file bootstrap |
| `drasi-bootstrap-postgres` | PostgreSQL bootstrap provider |
| `drasi-bootstrap-application` | Application bootstrap provider |

### Storage

| Package | Description |
|---------|-------------|
| `drasi-index-rocksdb` | RocksDB index backend |
| `drasi-state-store-redb` | Redb state store |

## Usage Example

```python
import asyncio
from drasi_lib import DrasiLibBuilder, Query
from drasi_source_application import PyApplicationSource, PyPropertyMapBuilder
from drasi_reaction_application import PyApplicationReaction

async def main():
    # Create source and get handle for pushing data
    source = PyApplicationSource("my-source")
    handle = source.get_handle()

    # Create reaction to receive query results
    builder = PyApplicationReaction.builder("my-reaction")
    builder.with_query("my-query")
    reaction, reaction_handle = builder.build()

    # Build DrasiLib
    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("example")
    lib_builder.with_source(source.into_source_wrapper())

    query = Query.cypher("my-query")
    query.query("MATCH (p:Person) RETURN p.name AS name, p.age AS age")
    query.from_source("my-source")
    query.auto_start(True)
    lib_builder.with_query(query.build())
    lib_builder.with_reaction(reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()

    # Push data
    props = PyPropertyMapBuilder()
    props.with_string("name", "Alice")
    props.with_integer("age", 30)
    await handle.send_node_insert("person-1", ["Person"], props.build())

    # Read results
    stream = await reaction_handle.as_stream()
    if stream:
        async for result in stream:
            print(result)

    await lib.stop()

asyncio.run(main())
```

## Development

### Building a single package

```bash
cd drasi-core   # or any package directory
make build      # builds with maturin develop
make lint       # cargo clippy + ruff
make test       # cargo test
```

### Running integration tests

```bash
# Build all packages first
make build-all

# Run all integration tests
make integration-test-all

# Or run specific tests
uv run pytest tests/test_lifecycle.py -v
```

### Project structure

```
python/
├── Makefile                    # Root: build-all, test-all, lint-all
├── pyproject.toml              # uv workspace
├── drasi-core/                 # Core package
├── sources/                    # Source packages
├── reactions/                  # Reaction packages
├── bootstrappers/              # Bootstrap provider packages
├── indexes/                    # Index backend packages
├── state-stores/               # State store packages
├── tests/                      # pytest integration tests
└── examples/                   # Usage examples
```

Each package contains:
- `Cargo.toml` — Rust crate config
- `pyproject.toml` — Python package config (maturin)
- `Makefile` — Build/test/lint targets
- `agents.md` — Instructions for updating when upstream crate changes
- `src/lib.rs` — PyO3 module implementation
- `{module}/` — Python package with `__init__.py`, `__init__.pyi`, `py.typed`

## License

Apache-2.0
