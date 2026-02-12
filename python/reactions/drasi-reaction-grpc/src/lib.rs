use std::sync::Mutex;

use _drasi_core::builder::reaction_to_capsule;
use _drasi_core::errors::map_err;
use drasi_reaction_grpc_adaptive::{AdaptiveGrpcReaction, GrpcAdaptiveReactionBuilder};
use pyo3::prelude::*;

#[pyclass]
pub struct PyGrpcReactionBuilder {
    inner: Option<GrpcAdaptiveReactionBuilder>,
}

#[pymethods]
impl PyGrpcReactionBuilder {
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(GrpcAdaptiveReactionBuilder::new(id)),
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

    fn with_endpoint(&mut self, endpoint: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_endpoint(endpoint));
        Ok(())
    }

    fn with_timeout_ms(&mut self, timeout_ms: u64) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_timeout_ms(timeout_ms));
        Ok(())
    }

    fn with_max_retries(&mut self, retries: u32) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_max_retries(retries));
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

    fn build(&mut self) -> PyResult<PyGrpcReaction> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let reaction = inner.build().map_err(map_err)?;
        Ok(PyGrpcReaction {
            inner: Mutex::new(Some(reaction)),
        })
    }
}

#[pyclass]
pub struct PyGrpcReaction {
    inner: Mutex<Option<AdaptiveGrpcReaction>>,
}

#[pymethods]
impl PyGrpcReaction {
    #[staticmethod]
    fn builder(id: &str) -> PyGrpcReactionBuilder {
        PyGrpcReactionBuilder {
            inner: Some(AdaptiveGrpcReaction::builder(id)),
        }
    }

    fn into_reaction_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let reaction = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("GrpcReaction already consumed")
            })?;
        reaction_to_capsule(py, reaction)
    }
}

/// Python module for drasi_reaction_grpc
#[pymodule]
fn _drasi_reaction_grpc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGrpcReactionBuilder>()?;
    m.add_class::<PyGrpcReaction>()?;
    Ok(())
}
