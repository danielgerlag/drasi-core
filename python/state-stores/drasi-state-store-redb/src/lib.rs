use std::sync::{Arc, Mutex};

use drasi_state_store_redb::RedbStateStoreProvider;
use _drasi_core::builder::state_store_to_capsule;
use _drasi_core::errors::map_err;
use pyo3::prelude::*;

#[pyclass]
pub struct PyRedbStateStoreProvider {
    inner: Mutex<Option<RedbStateStoreProvider>>,
}

#[pymethods]
impl PyRedbStateStoreProvider {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let provider = RedbStateStoreProvider::new(path).map_err(map_err)?;
        Ok(Self {
            inner: Mutex::new(Some(provider)),
        })
    }

    fn into_state_store_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let provider = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("RedbStateStoreProvider already consumed")
            })?;
        state_store_to_capsule(py, Arc::new(provider))
    }
}

/// Python module for drasi_state_store_redb
#[pymodule]
fn _drasi_state_store_redb(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRedbStateStoreProvider>()?;
    Ok(())
}
