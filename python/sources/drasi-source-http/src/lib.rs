use std::sync::Mutex;

use _drasi_core::builder::source_to_capsule;
use _drasi_core::errors::map_err;
use drasi_source_http::{HttpSource, HttpSourceBuilder, HttpSourceConfig};
use pyo3::prelude::*;

#[pyclass]
pub struct PyHttpSourceBuilder {
    inner: Option<HttpSourceBuilder>,
}

#[pymethods]
impl PyHttpSourceBuilder {
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(HttpSourceBuilder::new(id)),
        }
    }

    fn with_host(&mut self, host: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_host(host));
        Ok(())
    }

    fn with_port(&mut self, port: u16) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_port(port));
        Ok(())
    }

    fn with_endpoint(&mut self, endpoint: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_endpoint(endpoint));
        Ok(())
    }

    fn with_timeout_ms(&mut self, timeout_ms: u64) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_timeout_ms(timeout_ms));
        Ok(())
    }

    fn with_auto_start(&mut self, auto_start: bool) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_auto_start(auto_start));
        Ok(())
    }

    /// Configure the source from a JSON string matching HttpSourceConfig.
    /// This supports the full configuration including webhook mode.
    fn with_config_json(&mut self, json_str: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let config: HttpSourceConfig =
            serde_json::from_str(json_str).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid config JSON: {e}"))
            })?;
        self.inner = Some(builder.with_config(config));
        Ok(())
    }

    fn build(&mut self) -> PyResult<PyHttpSource> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let source = builder.build().map_err(map_err)?;
        Ok(PyHttpSource {
            inner: Mutex::new(Some(source)),
        })
    }
}

#[pyclass]
pub struct PyHttpSource {
    inner: Mutex<Option<HttpSource>>,
}

#[pymethods]
impl PyHttpSource {
    #[staticmethod]
    fn builder(id: &str) -> PyHttpSourceBuilder {
        PyHttpSourceBuilder::new(id)
    }

    fn into_source_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let source = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("Source already consumed")
            })?;
        source_to_capsule(py, source)
    }
}

/// Python module for drasi_source_http
#[pymodule]
fn _drasi_source_http(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyHttpSourceBuilder>()?;
    m.add_class::<PyHttpSource>()?;
    Ok(())
}
