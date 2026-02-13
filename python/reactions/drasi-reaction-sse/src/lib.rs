use std::sync::Mutex;

use _drasi_core::builder::reaction_to_capsule;
use _drasi_core::errors::map_err;
use drasi_reaction_sse::{SseReaction, SseReactionBuilder};
use pyo3::prelude::*;

/// Builder for configuring an SSE reaction.
///
/// Use ``SseReaction.builder("my-sse")`` to create a new builder.
#[pyclass(name = "SseReactionBuilder")]
pub struct PySseReactionBuilder {
    inner: Option<SseReactionBuilder>,
}

#[pymethods]
impl PySseReactionBuilder {
    /// Create a new ``SseReactionBuilder`` with the given reaction ID.
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(SseReactionBuilder::new(id)),
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

    /// Set the hostname or IP address the SSE server binds to.
    fn with_host(&mut self, host: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_host(host));
        Ok(())
    }

    /// Set the port the SSE server listens on.
    fn with_port(&mut self, port: u16) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_port(port));
        Ok(())
    }

    /// Set the SSE endpoint path (e.g. ``"/events"``).
    fn with_sse_path(&mut self, path: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_sse_path(path));
        Ok(())
    }

    /// Set the heartbeat interval in milliseconds for keep-alive messages.
    fn with_heartbeat_interval_ms(&mut self, interval_ms: u64) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_heartbeat_interval_ms(interval_ms));
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

    /// Finalize the builder and create the SseReaction. Consumes this builder.
    fn build(&mut self) -> PyResult<PySseReaction> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let reaction = inner.build().map_err(map_err)?;
        Ok(PySseReaction {
            inner: Mutex::new(Some(reaction)),
        })
    }
}

/// A reaction that streams query result changes over Server-Sent Events.
#[pyclass(name = "SseReaction")]
pub struct PySseReaction {
    inner: Mutex<Option<SseReaction>>,
}

#[pymethods]
impl PySseReaction {
    /// Create a new ``SseReactionBuilder`` with the given reaction ID.
    #[staticmethod]
    fn builder(id: &str) -> PySseReactionBuilder {
        PySseReactionBuilder {
            inner: Some(SseReaction::builder(id)),
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
                pyo3::exceptions::PyRuntimeError::new_err("SseReaction already consumed")
            })?;
        reaction_to_capsule(py, reaction)
    }
}

/// Python module for drasi_reaction_sse
#[pymodule]
fn _drasi_reaction_sse(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySseReactionBuilder>()?;
    m.add_class::<PySseReaction>()?;
    Ok(())
}
