# agents.md — drasi-source-postgres-python

## Overview
This crate wraps `drasi-source-postgres` (at `../../../components/sources/postgres`) as a PyO3 Python module (`drasi_source_postgres`).

## Wrapped Rust Crate
- **Crate**: `drasi-source-postgres`
- **Path**: `../../../components/sources/postgres`
- **Cargo dependency key**: `drasi-source-postgres`

## How to Update When the Underlying Crate Changes

1. **Check what changed**: Review the diff of the wrapped Rust crate.
   Focus on public API changes: new/removed/renamed methods, changed
   signatures, new types, removed types, changed fields on structs/enums.

2. **Update Cargo.toml**: If the upstream crate version changed, update
   the version in this crate's `Cargo.toml` dependency.

3. **Update wrapper code**: For each public API change in the upstream crate:
   - New method → Add corresponding `#[pymethods]` wrapper in the appropriate `src/` file
   - Removed method → Remove the wrapper (check for Python-side usage first)
   - Changed signature → Update the wrapper signature and any type conversions
   - New type → Add PyO3 wrapper type, update `mod.rs` exports
   - Changed enum variant → Update Python enum mapping

4. **Update type stubs**: Update the `.pyi` stub file to match any API changes.

5. **Update tests**: Add/update integration tests in `../tests/` for new functionality.

6. **Verify**: Run `make build && make test && make integration-test && make lint`

## File Mapping
| Upstream file | Wrapper file | What it wraps |
|---|---|---|
| *(to be filled during implementation)* | `src/lib.rs` | Module entry point |

## Key Patterns
- Async methods use `pyo3_asyncio_0_22::tokio::future_into_py()`
- Properties map to `dict[str, Any]` via `pythonize`
- Errors map to `DrasiError` Python exception from `drasi-core-python`
