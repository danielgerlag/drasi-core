use std::sync::Mutex;

use _drasi_core::builder::source_to_capsule;
use _drasi_core::errors::map_err;
use drasi_source_mock::{MockSource, MockSourceBuilder};
use pyo3::prelude::*;

#[pyclass]
pub struct PyMockSourceBuilder {
    inner: Option<MockSourceBuilder>,
}

#[pymethods]
impl PyMockSourceBuilder {
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(MockSourceBuilder::new(id)),
        }
    }

    fn with_interval_ms(&mut self, interval_ms: u64) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_interval_ms(interval_ms));
        Ok(())
    }

    fn with_auto_start(&mut self, auto_start: bool) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_auto_start(auto_start));
        Ok(())
    }

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

#[pyclass]
pub struct PyMockSource {
    inner: Mutex<Option<MockSource>>,
}

#[pymethods]
impl PyMockSource {
    #[staticmethod]
    fn builder(id: &str) -> PyMockSourceBuilder {
        PyMockSourceBuilder::new(id)
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

/// Python module for drasi_source_mock
#[pymodule]
fn _drasi_source_mock(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMockSourceBuilder>()?;
    m.add_class::<PyMockSource>()?;
    Ok(())
}
