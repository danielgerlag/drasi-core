use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

// DrasiError Python exception
pyo3::create_exception!(drasi_core, DrasiError, pyo3::exceptions::PyException);

/// Map a drasi_lib::DrasiError into a Python DrasiError exception
pub fn to_py_err(err: drasi_lib::DrasiError) -> PyErr {
    DrasiError::new_err(err.to_string())
}

/// Map an anyhow::Error into a Python RuntimeError
pub fn anyhow_to_py_err(err: anyhow::Error) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

/// Map any error that implements Display into a Python RuntimeError
pub fn map_err<E: std::fmt::Display>(err: E) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

/// Register error types on the module
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("DrasiError", m.py().get_type_bound::<DrasiError>())?;
    Ok(())
}
