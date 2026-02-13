use std::sync::Mutex;

use _drasi_core::builder::source_to_capsule;
use _drasi_core::errors::map_err;
use drasi_source_grpc::{GrpcSource, GrpcSourceBuilder};
use pyo3::prelude::*;

/// Builder for configuring and constructing a ``GrpcSource``.
///
/// Use ``GrpcSource.builder(id)`` or ``GrpcSourceBuilder(id)`` to create.
#[pyclass(name = "GrpcSourceBuilder")]
pub struct PyGrpcSourceBuilder {
    inner: Option<GrpcSourceBuilder>,
}

#[pymethods]
impl PyGrpcSourceBuilder {
    /// Create a new GrpcSourceBuilder.
    ///
    /// Args:
    ///     id: Unique identifier for this source.
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(GrpcSourceBuilder::new(id)),
        }
    }

    /// Set the gRPC server hostname.
    fn with_host(&mut self, host: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_host(host));
        Ok(())
    }

    /// Set the gRPC server port.
    fn with_port(&mut self, port: u16) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_port(port));
        Ok(())
    }

    /// Set the gRPC endpoint path.
    fn with_endpoint(&mut self, endpoint: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_endpoint(endpoint));
        Ok(())
    }

    /// Set the connection timeout in milliseconds.
    fn with_timeout_ms(&mut self, timeout_ms: u64) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_timeout_ms(timeout_ms));
        Ok(())
    }

    /// Set whether the source starts automatically when added to a ``DrasiLib``.
    fn with_auto_start(&mut self, auto_start: bool) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_auto_start(auto_start));
        Ok(())
    }

    /// Consume the builder and return a configured ``GrpcSource``.
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

/// A source that receives graph changes over gRPC.
#[pyclass(name = "GrpcSource")]
pub struct PyGrpcSource {
    inner: Mutex<Option<GrpcSource>>,
}

#[pymethods]
impl PyGrpcSource {
    /// Create a new ``GrpcSourceBuilder`` for the given source id.
    #[staticmethod]
    fn builder(id: &str) -> PyGrpcSourceBuilder {
        PyGrpcSourceBuilder::new(id)
    }

    /// Consume the source and return a PyCapsule for use with ``DrasiLibBuilder``.
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
