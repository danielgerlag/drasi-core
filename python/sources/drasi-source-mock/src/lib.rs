use std::sync::Mutex;

use _drasi_core::builder::source_to_capsule;
use _drasi_core::errors::map_err;
use drasi_source_mock::{MockSource, MockSourceBuilder};
use pyo3::prelude::*;

/// Builder for configuring and constructing a ``MockSource``.
///
/// Use ``MockSource.builder(id)`` or ``MockSourceBuilder(id)`` to create.
#[pyclass(name = "MockSourceBuilder")]
pub struct PyMockSourceBuilder {
    inner: Option<MockSourceBuilder>,
}

#[pymethods]
impl PyMockSourceBuilder {
    /// Create a new MockSourceBuilder.
    ///
    /// Args:
    ///     id: Unique identifier for this source.
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(MockSourceBuilder::new(id)),
        }
    }

    /// Set the polling interval in milliseconds.
    fn with_interval_ms(&mut self, interval_ms: u64) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_interval_ms(interval_ms));
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

    /// Consume the builder and return a configured ``MockSource``.
    fn build(&mut self) -> PyResult<PyMockSource> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let source = builder.build().map_err(map_err)?;
        Ok(PyMockSource {
            inner: Mutex::new(Some(source)),
        })
    }
}

/// A mock source for testing that generates synthetic graph changes.
#[pyclass(name = "MockSource")]
pub struct PyMockSource {
    inner: Mutex<Option<MockSource>>,
}

#[pymethods]
impl PyMockSource {
    /// Create a new ``MockSourceBuilder`` for the given source id.
    #[staticmethod]
    fn builder(id: &str) -> PyMockSourceBuilder {
        PyMockSourceBuilder::new(id)
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

/// Python module for drasi_source_mock
#[pymodule]
fn _drasi_source_mock(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMockSourceBuilder>()?;
    m.add_class::<PyMockSource>()?;
    Ok(())
}
