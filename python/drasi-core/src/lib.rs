use pyo3::prelude::*;

pub mod builder;
pub mod errors;
pub mod types;

/// Python module for drasi_core
#[pymodule]
fn _drasi_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    errors::register(m)?;
    types::register(m)?;
    builder::register(m)?;
    Ok(())
}
