use std::sync::Mutex;

use drasi_bootstrap_scriptfile::{ScriptFileBootstrapProvider, ScriptFileBootstrapProviderBuilder};
use _drasi_core::builder::bootstrap_to_capsule;
use _drasi_core::errors::map_err;
use pyo3::prelude::*;

// ============================================================================
// PyScriptFileBootstrapProviderBuilder
// ============================================================================

#[pyclass(name = "ScriptFileBootstrapProviderBuilder")]
pub struct PyScriptFileBootstrapProviderBuilder {
    inner: Option<ScriptFileBootstrapProviderBuilder>,
}

#[pymethods]
impl PyScriptFileBootstrapProviderBuilder {
    #[new]
    fn new() -> Self {
        Self {
            inner: Some(ScriptFileBootstrapProviderBuilder::new()),
        }
    }

    fn with_file_paths(&mut self, paths: Vec<String>) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_file_paths(paths));
        Ok(())
    }

    fn with_file(&mut self, path: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_file(path));
        Ok(())
    }

    fn build(&mut self) -> PyResult<PyScriptFileBootstrapProvider> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let provider = inner.build();
        Ok(PyScriptFileBootstrapProvider {
            inner: Mutex::new(Some(provider)),
        })
    }
}

// ============================================================================
// PyScriptFileBootstrapProvider
// ============================================================================

#[pyclass(name = "ScriptFileBootstrapProvider")]
pub struct PyScriptFileBootstrapProvider {
    inner: Mutex<Option<ScriptFileBootstrapProvider>>,
}

#[pymethods]
impl PyScriptFileBootstrapProvider {
    #[staticmethod]
    fn builder() -> PyScriptFileBootstrapProviderBuilder {
        PyScriptFileBootstrapProviderBuilder {
            inner: Some(ScriptFileBootstrapProvider::builder()),
        }
    }

    #[staticmethod]
    fn with_paths(paths: Vec<String>) -> Self {
        Self {
            inner: Mutex::new(Some(ScriptFileBootstrapProvider::with_paths(paths))),
        }
    }

    fn into_bootstrap_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let provider = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "ScriptFileBootstrapProvider already consumed",
                )
            })?;
        bootstrap_to_capsule(py, provider)
    }
}

/// Python module for drasi_bootstrap_scriptfile
#[pymodule]
fn _drasi_bootstrap_scriptfile(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyScriptFileBootstrapProviderBuilder>()?;
    m.add_class::<PyScriptFileBootstrapProvider>()?;
    Ok(())
}
