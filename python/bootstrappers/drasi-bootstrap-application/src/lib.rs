use std::sync::Mutex;

use _drasi_core::builder::bootstrap_to_capsule;
use _drasi_core::errors::map_err;
use drasi_bootstrap_application::{
    ApplicationBootstrapProvider, ApplicationBootstrapProviderBuilder,
};
use pyo3::prelude::*;

#[pyclass]
pub struct PyApplicationBootstrapProviderBuilder {
    inner: Option<ApplicationBootstrapProviderBuilder>,
}

#[pymethods]
impl PyApplicationBootstrapProviderBuilder {
    #[new]
    fn new() -> Self {
        Self {
            inner: Some(ApplicationBootstrapProviderBuilder::new()),
        }
    }

    fn build(&mut self) -> PyResult<PyApplicationBootstrapProvider> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let provider = builder.build();
        Ok(PyApplicationBootstrapProvider {
            inner: Mutex::new(Some(provider)),
        })
    }
}

#[pyclass]
pub struct PyApplicationBootstrapProvider {
    inner: Mutex<Option<ApplicationBootstrapProvider>>,
}

#[pymethods]
impl PyApplicationBootstrapProvider {
    #[staticmethod]
    fn builder() -> PyApplicationBootstrapProviderBuilder {
        PyApplicationBootstrapProviderBuilder::new()
    }

    fn into_bootstrap_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let provider = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("Provider already consumed")
            })?;
        bootstrap_to_capsule(py, provider)
    }
}

/// Python module for drasi_bootstrap_application
#[pymodule]
fn _drasi_bootstrap_application(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyApplicationBootstrapProviderBuilder>()?;
    m.add_class::<PyApplicationBootstrapProvider>()?;
    Ok(())
}
