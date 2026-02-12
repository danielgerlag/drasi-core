use std::sync::Mutex;

use drasi_bootstrap_noop::NoOpBootstrapProvider;
use _drasi_core::builder::bootstrap_to_capsule;
use _drasi_core::errors::map_err;
use pyo3::prelude::*;

// ============================================================================
// PyNoOpBootstrapProvider
// ============================================================================

#[pyclass]
pub struct PyNoOpBootstrapProvider {
    inner: Mutex<Option<NoOpBootstrapProvider>>,
}

#[pymethods]
impl PyNoOpBootstrapProvider {
    #[new]
    fn new() -> Self {
        Self {
            inner: Mutex::new(Some(NoOpBootstrapProvider::new())),
        }
    }

    fn into_bootstrap_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let provider = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("NoOpBootstrapProvider already consumed")
            })?;
        bootstrap_to_capsule(py, provider)
    }
}

/// Python module for drasi_bootstrap_noop
#[pymodule]
fn _drasi_bootstrap_noop(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNoOpBootstrapProvider>()?;
    Ok(())
}
