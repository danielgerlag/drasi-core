use std::sync::Mutex;

use _drasi_core::builder::reaction_to_capsule;
use _drasi_core::errors::map_err;
use drasi_reaction_grpc_adaptive::{AdaptiveGrpcReaction, GrpcAdaptiveReactionBuilder};
use pyo3::prelude::*;

/// Builder for configuring a gRPC reaction.
///
/// Use ``GrpcReaction.builder("my-grpc")`` to create a new builder.
#[pyclass(name = "GrpcReactionBuilder")]
pub struct PyGrpcReactionBuilder {
    inner: Option<GrpcAdaptiveReactionBuilder>,
}

#[pymethods]
impl PyGrpcReactionBuilder {
    /// Create a new ``GrpcReactionBuilder`` with the given reaction ID.
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(GrpcAdaptiveReactionBuilder::new(id)),
        }
    }

    /// Add a single query by ID to this reaction.
    fn with_query(&mut self, query_id: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_query(query_id));
        Ok(())
    }

    /// Add multiple queries by ID to this reaction.
    fn with_queries(&mut self, queries: Vec<String>) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_queries(queries));
        Ok(())
    }

    /// Set the gRPC server endpoint URL.
    fn with_endpoint(&mut self, endpoint: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_endpoint(endpoint));
        Ok(())
    }

    /// Set the gRPC request timeout in milliseconds.
    fn with_timeout_ms(&mut self, timeout_ms: u64) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_timeout_ms(timeout_ms));
        Ok(())
    }

    /// Set the maximum number of retry attempts for failed requests.
    fn with_max_retries(&mut self, retries: u32) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_max_retries(retries));
        Ok(())
    }

    /// Set the capacity of the priority queue for ordering results.
    fn with_priority_queue_capacity(&mut self, capacity: usize) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_priority_queue_capacity(capacity));
        Ok(())
    }

    /// Set whether the reaction starts automatically when added to the Drasi instance.
    fn with_auto_start(&mut self, auto_start: bool) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_auto_start(auto_start));
        Ok(())
    }

    /// Set the batch size (alias for ``with_max_batch_size``).
    fn with_batch_size(&mut self, batch_size: usize) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_max_batch_size(batch_size));
        Ok(())
    }

    /// Set the minimum batch size for batched gRPC requests.
    fn with_min_batch_size(&mut self, size: usize) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_min_batch_size(size));
        Ok(())
    }

    /// Set the maximum batch size for batched gRPC requests.
    fn with_max_batch_size(&mut self, size: usize) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_max_batch_size(size));
        Ok(())
    }

    /// Add a metadata key-value pair to gRPC requests.
    fn with_metadata(&mut self, key: &str, value: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_metadata(key, value));
        Ok(())
    }

    /// Finalize the builder and create the GrpcReaction. Consumes this builder.
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

/// A reaction that sends query result changes via gRPC.
#[pyclass(name = "GrpcReaction")]
pub struct PyGrpcReaction {
    inner: Mutex<Option<AdaptiveGrpcReaction>>,
}

#[pymethods]
impl PyGrpcReaction {
    /// Create a new ``GrpcReactionBuilder`` with the given reaction ID.
    #[staticmethod]
    fn builder(id: &str) -> PyGrpcReactionBuilder {
        PyGrpcReactionBuilder {
            inner: Some(AdaptiveGrpcReaction::builder(id)),
        }
    }

    /// Return a capsule wrapping this reaction for use with ``DrasiLibBuilder``.
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
