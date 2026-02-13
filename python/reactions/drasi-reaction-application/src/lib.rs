use std::sync::Mutex;

use _drasi_core::builder::reaction_to_capsule;
use _drasi_core::errors::map_err;
use drasi_reaction_application::{ApplicationReaction, ApplicationReactionBuilder, ApplicationReactionHandle};
use drasi_reaction_application::application::ResultStream;
use pyo3::prelude::*;

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// ============================================================================
// PyApplicationReactionBuilder
// ============================================================================

#[pyclass(name = "ApplicationReactionBuilder")]
pub struct PyApplicationReactionBuilder {
    inner: Option<ApplicationReactionBuilder>,
}

#[pymethods]
impl PyApplicationReactionBuilder {
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(ApplicationReactionBuilder::new(id)),
        }
    }

    fn with_query(&mut self, query_id: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_query(query_id));
        Ok(())
    }

    fn with_queries(&mut self, queries: Vec<String>) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_queries(queries));
        Ok(())
    }

    fn with_priority_queue_capacity(&mut self, capacity: usize) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_priority_queue_capacity(capacity));
        Ok(())
    }

    fn with_auto_start(&mut self, auto_start: bool) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_auto_start(auto_start));
        Ok(())
    }

    fn build(&mut self) -> PyResult<(PyApplicationReaction, PyApplicationReactionHandle)> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let (reaction, handle) = inner.build();
        Ok((
            PyApplicationReaction {
                inner: Mutex::new(Some(reaction)),
            },
            PyApplicationReactionHandle {
                inner: handle,
            },
        ))
    }
}

// ============================================================================
// PyApplicationReaction
// ============================================================================

#[pyclass(name = "ApplicationReaction")]
pub struct PyApplicationReaction {
    inner: Mutex<Option<ApplicationReaction>>,
}

#[pymethods]
impl PyApplicationReaction {
    #[staticmethod]
    fn builder(id: &str) -> PyApplicationReactionBuilder {
        PyApplicationReactionBuilder {
            inner: Some(ApplicationReaction::builder(id)),
        }
    }

    fn into_reaction_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let reaction = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("ApplicationReaction already consumed")
            })?;
        reaction_to_capsule(py, reaction)
    }
}

// ============================================================================
// PyApplicationReactionHandle
// ============================================================================

#[pyclass(name = "ApplicationReactionHandle")]
pub struct PyApplicationReactionHandle {
    inner: ApplicationReactionHandle,
}

#[pymethods]
impl PyApplicationReactionHandle {
    fn reaction_id(&self) -> String {
        self.inner.reaction_id().to_string()
    }

    fn as_stream<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let handle = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            match handle.as_stream().await {
                Some(stream) => Ok(Some(PyResultStream {
                    inner: Arc::new(TokioMutex::new(stream)),
                })),
                None => Ok(None),
            }
        })
    }
}

// ============================================================================
// PyResultStream
// ============================================================================

#[pyclass(name = "ResultStream")]
pub struct PyResultStream {
    inner: Arc<TokioMutex<ResultStream>>,
}

#[pymethods]
impl PyResultStream {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let stream = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = stream.lock().await;
            match guard.next().await {
                Some(result) => Python::with_gil(|py| {
                    let py_obj = pythonize::pythonize(py, &result).map_err(map_err)?;
                    Ok(py_obj.unbind())
                }),
                None => Err(pyo3::exceptions::PyStopAsyncIteration::new_err(())),
            }
        })
    }
}

/// Python module for drasi_reaction_application
#[pymodule]
fn _drasi_reaction_application(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyApplicationReactionBuilder>()?;
    m.add_class::<PyApplicationReaction>()?;
    m.add_class::<PyApplicationReactionHandle>()?;
    m.add_class::<PyResultStream>()?;
    Ok(())
}
