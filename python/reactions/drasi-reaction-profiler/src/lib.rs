use std::sync::Mutex;

use _drasi_core::builder::reaction_to_capsule;
use _drasi_core::errors::map_err;
use drasi_reaction_profiler::{ProfilerReaction, ProfilerReactionBuilder};
use pyo3::prelude::*;

#[pyclass]
pub struct PyProfilerReactionBuilder {
    inner: Option<ProfilerReactionBuilder>,
}

#[pymethods]
impl PyProfilerReactionBuilder {
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(ProfilerReactionBuilder::new(id)),
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

    fn with_window_size(&mut self, size: usize) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_window_size(size));
        Ok(())
    }

    fn with_report_interval_secs(&mut self, secs: u64) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_report_interval_secs(secs));
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

    fn build(&mut self) -> PyResult<PyProfilerReaction> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let reaction = inner.build().map_err(map_err)?;
        Ok(PyProfilerReaction {
            inner: Mutex::new(Some(reaction)),
        })
    }
}

#[pyclass]
pub struct PyProfilerReaction {
    inner: Mutex<Option<ProfilerReaction>>,
}

#[pymethods]
impl PyProfilerReaction {
    #[staticmethod]
    fn builder(id: &str) -> PyProfilerReactionBuilder {
        PyProfilerReactionBuilder {
            inner: Some(ProfilerReaction::builder(id)),
        }
    }

    fn into_reaction_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let reaction = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("ProfilerReaction already consumed")
            })?;
        reaction_to_capsule(py, reaction)
    }
}

/// Python module for drasi_reaction_profiler
#[pymodule]
fn _drasi_reaction_profiler(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyProfilerReactionBuilder>()?;
    m.add_class::<PyProfilerReaction>()?;
    Ok(())
}
