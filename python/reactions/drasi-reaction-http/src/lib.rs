use std::sync::Mutex;

use _drasi_core::builder::reaction_to_capsule;
use _drasi_core::errors::map_err;
use drasi_reaction_http_adaptive::{AdaptiveHttpReaction, HttpAdaptiveReactionBuilder};
use pyo3::prelude::*;

/// Builder for configuring an HTTP reaction.
///
/// Use ``HttpReaction.builder("my-http")`` to create a new builder.
#[pyclass(name = "HttpReactionBuilder")]
pub struct PyHttpReactionBuilder {
    inner: Option<HttpAdaptiveReactionBuilder>,
}

#[pymethods]
impl PyHttpReactionBuilder {
    /// Create a new ``HttpReactionBuilder`` with the given reaction ID.
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(HttpAdaptiveReactionBuilder::new(id)),
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

    /// Set the base URL for HTTP requests.
    fn with_base_url(&mut self, base_url: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_base_url(base_url));
        Ok(())
    }

    /// Set the bearer token for authenticating HTTP requests.
    fn with_token(&mut self, token: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_token(token));
        Ok(())
    }

    /// Set the HTTP request timeout in milliseconds.
    fn with_timeout_ms(&mut self, timeout_ms: u64) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_timeout_ms(timeout_ms));
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

    /// Add a query-specific route configuration from a JSON string.
    /// The JSON should match QueryConfig: {"added": {...}, "updated": {...}, "deleted": {...}}
    /// where each value is a CallSpec: {"url": "...", "method": "...", "body": "...", "headers": {...}}
    fn with_route_json(&mut self, query_id: &str, config_json: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let config: drasi_reaction_http::config::QueryConfig =
            serde_json::from_str(config_json).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid route config JSON: {e}"))
            })?;
        self.inner = Some(inner.with_route(query_id, config));
        Ok(())
    }

    /// Finalize the builder and create the HttpReaction. Consumes this builder.
    fn build(&mut self) -> PyResult<PyHttpReaction> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let reaction = inner.build().map_err(map_err)?;
        Ok(PyHttpReaction {
            inner: Mutex::new(Some(reaction)),
        })
    }
}

/// A reaction that sends query result changes as HTTP requests.
#[pyclass(name = "HttpReaction")]
pub struct PyHttpReaction {
    inner: Mutex<Option<AdaptiveHttpReaction>>,
}

#[pymethods]
impl PyHttpReaction {
    /// Create a new ``HttpReactionBuilder`` with the given reaction ID.
    #[staticmethod]
    fn builder(id: &str) -> PyHttpReactionBuilder {
        PyHttpReactionBuilder {
            inner: Some(AdaptiveHttpReaction::builder(id)),
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
                pyo3::exceptions::PyRuntimeError::new_err("HttpReaction already consumed")
            })?;
        reaction_to_capsule(py, reaction)
    }
}

/// Python module for drasi_reaction_http
#[pymodule]
fn _drasi_reaction_http(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyHttpReactionBuilder>()?;
    m.add_class::<PyHttpReaction>()?;
    Ok(())
}
