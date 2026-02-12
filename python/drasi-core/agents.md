# agents.md — drasi-core-python

## Overview
This crate provides shared base types for the Drasi Python bindings. It wraps
the trait-object wrapper types (`PySourceWrapper`, `PyReactionWrapper`, etc.),
error mapping, and shared enums (`DispatchMode`). All component crates depend
on this crate for these shared types.

## Wrapped Rust Crate
- **Crate**: `drasi-lib` (for trait definitions only)
- **Path**: `../../lib`

## How to Update When the Underlying Crate Changes

1. **Check what changed**: Review public trait changes in `drasi-lib`.
   Focus on: `Source`, `Reaction`, `BootstrapProvider`, `IndexBackendPlugin`,
   `StateStoreProvider` traits. Also check `DispatchMode` enum.

2. **Update wrapper code**: If traits changed, update the corresponding
   Py*Wrapper types in `src/builder.rs`.

3. **Update type stubs**: Update `drasi_core/__init__.pyi`.

4. **Verify**: Run `make build && make lint`

## File Mapping
| Wrapper file | What it contains |
|---|---|
| `src/lib.rs` | Module entry point |
| `src/errors.rs` | DrasiError exception, `map_err()`, `to_py_err()` |
| `src/types.rs` | `DispatchMode` enum |
| `src/builder.rs` | `PySourceWrapper`, `PyReactionWrapper`, `PyBootstrapProviderWrapper`, `PyIndexBackendWrapper`, `PyStateStoreProviderWrapper`, `PyQueryConfig` |
