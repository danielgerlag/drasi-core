use std::sync::Mutex;

use _drasi_core::builder::source_to_capsule;
use _drasi_core::errors::map_err;
use drasi_source_grpc::{GrpcSource, GrpcSourceBuilder};
use pyo3::prelude::*;

#[pyclass(name = "GrpcSourceBuilder")]
pub struct PyGrpcSourceBuilder {
    inner: Option<GrpcSourceBuilder>,
}

#[pymethods]
impl PyGrpcSourceBuilder {
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(GrpcSourceBuilder::new(id)),
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

    fn build(&mut self) -> PyResult<PyGrpcSource> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let source = builder.build().map_err(map_err)?;
        Ok(PyGrpcSource {
            inner: Mutex::new(Some(source)),
        })
    }
}

#[pyclass(name = "GrpcSource")]
pub struct PyGrpcSource {
    inner: Mutex<Option<GrpcSource>>,
}

#[pymethods]
impl PyGrpcSource {
    #[staticmethod]
    fn builder(id: &str) -> PyGrpcSourceBuilder {
        PyGrpcSourceBuilder::new(id)
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

/// Python module for drasi_source_grpc
#[pymodule]
fn _drasi_source_grpc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGrpcSourceBuilder>()?;
    m.add_class::<PyGrpcSource>()?;
    Ok(())
}
