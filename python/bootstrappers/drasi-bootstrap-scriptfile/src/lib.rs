use std::sync::Mutex;

use drasi_bootstrap_scriptfile::{ScriptFileBootstrapProvider, ScriptFileBootstrapProviderBuilder};
use _drasi_core::builder::bootstrap_to_capsule;
use _drasi_core::errors::map_err;
use pyo3::prelude::*;

// ============================================================================
// PyScriptFileBootstrapProviderBuilder
// ============================================================================

/// Builder for creating a ScriptFileBootstrapProvider.
#[pyclass(name = "ScriptFileBootstrapProviderBuilder")]
pub struct PyScriptFileBootstrapProviderBuilder {
    inner: Option<ScriptFileBootstrapProviderBuilder>,
}

#[pymethods]
impl PyScriptFileBootstrapProviderBuilder {
    /// Create a new ScriptFileBootstrapProviderBuilder.
    #[new]
    fn new() -> Self {
        Self {
            inner: Some(ScriptFileBootstrapProviderBuilder::new()),
        }
    }

    /// Set the list of script file paths to execute during bootstrap.
    fn with_file_paths(&mut self, paths: Vec<String>) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_file_paths(paths));
        Ok(())
    }

    /// Add a single script file path to execute during bootstrap.
    fn with_file(&mut self, path: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_file(path));
        Ok(())
    }

    /// Consume the builder and return a ScriptFileBootstrapProvider.
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

/// Bootstrap provider that executes script files to supply initial data.
#[pyclass(name = "ScriptFileBootstrapProvider")]
pub struct PyScriptFileBootstrapProvider {
    inner: Mutex<Option<ScriptFileBootstrapProvider>>,
}

#[pymethods]
impl PyScriptFileBootstrapProvider {
    /// Create a new builder for this provider.
    #[staticmethod]
    fn builder() -> PyScriptFileBootstrapProviderBuilder {
        PyScriptFileBootstrapProviderBuilder {
            inner: Some(ScriptFileBootstrapProvider::builder()),
        }
    }

    /// Create a provider directly from a list of script file paths.
    #[staticmethod]
    fn with_paths(paths: Vec<String>) -> Self {
        Self {
            inner: Mutex::new(Some(ScriptFileBootstrapProvider::with_paths(paths))),
        }
    }

    /// Return a capsule for use with DrasiLibBuilder.
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
