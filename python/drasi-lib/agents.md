# agents.md — drasi-lib-python

## Overview

This crate wraps the Rust `drasi-lib` crate as a Python extension module using PyO3.
It provides the high-level DrasiLib API: `DrasiLibBuilder`, `Query`, `PyDrasiLib`,
lifecycle types (`ComponentStatus`, `ComponentEvent`, `LogMessage`, `QueryResult`),
and streaming types (`EventSubscription`, `LogSubscription`).

## Wrapped Crate

- **Rust crate**: `drasi-lib` (at `../../lib/`)
- **Cargo.toml dep**: `drasi-lib = { path = "../../lib" }`
- **Also depends on**: `drasi-core-python` (at `../drasi-core/`) for shared wrapper types

## Updating to a New Version

1. Check what changed in the upstream `drasi-lib` crate:
   ```bash
   cd ../../lib && git log --oneline -10
   ```
2. Review the public API in `lib/src/lib.rs` for new/changed/removed exports.
3. Update `src/types.rs` if any enums or data types changed.
4. Update `src/builder.rs` if `DrasiLibBuilder` or `Query` methods changed.
5. Update `src/drasi_lib.rs` if `DrasiLib` methods changed.
6. Update `src/streaming.rs` if event/log subscription types changed.
7. Run `cargo check` to verify.
8. Update the `.pyi` type stubs in `drasi_lib/__init__.pyi`.

## File Mapping

| Python module file | Wraps |
|---|---|
| `src/types.rs` | `drasi_lib::channels::*`, `drasi_lib::ComponentStatus`, `drasi_lib::ComponentType`, etc. |
| `src/builder.rs` | `drasi_lib::DrasiLibBuilder`, `drasi_lib::builder::Query` |
| `src/drasi_lib.rs` | `drasi_lib::DrasiLib` |
| `src/streaming.rs` | Event/log broadcast subscription wrappers |
