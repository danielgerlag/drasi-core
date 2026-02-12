use pyo3::prelude::*;

pub mod builder;
pub mod drasi_lib;
pub mod streaming;
pub mod types;

/// Python module for drasi_lib
#[pymodule]
fn _drasi_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    types::register(m)?;
    streaming::register(m)?;
    builder::register(m)?;
    drasi_lib::register(m)?;
    Ok(())
}
