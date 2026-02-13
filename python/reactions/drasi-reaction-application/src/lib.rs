use std::sync::Mutex;

use _drasi_core::builder::reaction_to_capsule;
use _drasi_core::errors::map_err;
use drasi_reaction_application::{ApplicationReaction, ApplicationReactionBuilder, ApplicationReactionHandle};
use drasi_reaction_application::application::ResultStream;
use drasi_lib::channels::events::ResultDiff;
use pyo3::prelude::*;

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// ============================================================================
// PyApplicationReactionBuilder
// ============================================================================

/// Builder for configuring an application reaction.
///
/// Use ``ApplicationReaction.builder("my-app")`` to create a new builder.
#[pyclass(name = "ApplicationReactionBuilder")]
pub struct PyApplicationReactionBuilder {
    inner: Option<ApplicationReactionBuilder>,
}

#[pymethods]
impl PyApplicationReactionBuilder {
    /// Create a new ``ApplicationReactionBuilder`` with the given reaction ID.
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(ApplicationReactionBuilder::new(id)),
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

    /// Finalize the builder and create the ApplicationReaction. Consumes this builder.
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

/// An application reaction that allows programmatic consumption of query results.
#[pyclass(name = "ApplicationReaction")]
pub struct PyApplicationReaction {
    inner: Mutex<Option<ApplicationReaction>>,
}

#[pymethods]
impl PyApplicationReaction {
    /// Create a new ``ApplicationReactionBuilder`` with the given reaction ID.
    #[staticmethod]
    fn builder(id: &str) -> PyApplicationReactionBuilder {
        PyApplicationReactionBuilder {
            inner: Some(ApplicationReaction::builder(id)),
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
                pyo3::exceptions::PyRuntimeError::new_err("ApplicationReaction already consumed")
            })?;
        reaction_to_capsule(py, reaction)
    }
}

// ============================================================================
// PyApplicationReactionHandle
// ============================================================================

/// Handle for interacting with a running application reaction.
#[pyclass(name = "ApplicationReactionHandle")]
pub struct PyApplicationReactionHandle {
    inner: ApplicationReactionHandle,
}

#[pymethods]
impl PyApplicationReactionHandle {
    /// Return the reaction ID associated with this handle.
    fn reaction_id(&self) -> String {
        self.inner.reaction_id().to_string()
    }

    /// Get an async ``ResultStream`` for consuming query results. Returns ``None`` if unavailable.
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
// PyResultDiff — typed wrapper for ResultDiff enum
// ============================================================================

/// A single result diff from a continuous query.
///
/// The ``diff_type`` field indicates the kind of change:
///
/// - ``"ADD"`` — a new row matches the query. ``data`` contains the row.
/// - ``"UPDATE"`` — an existing matching row was modified. ``data``, ``before``,
///   and ``after`` are populated. ``grouping_keys`` may be present.
/// - ``"DELETE"`` — a matching row was removed. ``data`` contains the row.
/// - ``"aggregation"`` — an aggregation result changed. ``before`` (possibly
///   ``None``) and ``after`` are populated.
/// - ``"noop"`` — no meaningful change; all fields except ``diff_type`` are ``None``.
#[pyclass(name = "ResultDiff", frozen, get_all)]
#[derive(Debug)]
pub struct PyResultDiff {
    /// The type of diff: ``"ADD"``, ``"UPDATE"``, ``"DELETE"``,
    /// ``"aggregation"``, or ``"noop"``.
    pub diff_type: String,
    /// Row data (present for ADD, UPDATE, DELETE).
    pub data: Option<PyObject>,
    /// Row state before the change (present for UPDATE, aggregation).
    pub before: Option<PyObject>,
    /// Row state after the change (present for UPDATE, aggregation).
    pub after: Option<PyObject>,
    /// Optional grouping keys for grouped UPDATE diffs.
    pub grouping_keys: Option<Vec<String>>,
}

#[pymethods]
impl PyResultDiff {
    /// Return a human-readable string representation.
    fn __repr__(&self) -> String {
        format!("ResultDiff(type={:?})", self.diff_type)
    }
}

impl PyResultDiff {
    fn from_rust(py: Python<'_>, diff: &ResultDiff) -> PyResult<Self> {
        match diff {
            ResultDiff::Add { data } => Ok(Self {
                diff_type: "ADD".to_string(),
                data: Some(pythonize::pythonize(py, data).map_err(map_err)?.unbind()),
                before: None,
                after: None,
                grouping_keys: None,
            }),
            ResultDiff::Delete { data } => Ok(Self {
                diff_type: "DELETE".to_string(),
                data: Some(pythonize::pythonize(py, data).map_err(map_err)?.unbind()),
                before: None,
                after: None,
                grouping_keys: None,
            }),
            ResultDiff::Update {
                data,
                before,
                after,
                grouping_keys,
            } => Ok(Self {
                diff_type: "UPDATE".to_string(),
                data: Some(pythonize::pythonize(py, data).map_err(map_err)?.unbind()),
                before: Some(pythonize::pythonize(py, before).map_err(map_err)?.unbind()),
                after: Some(pythonize::pythonize(py, after).map_err(map_err)?.unbind()),
                grouping_keys: grouping_keys.clone(),
            }),
            ResultDiff::Aggregation { before, after } => Ok(Self {
                diff_type: "aggregation".to_string(),
                data: None,
                before: match before {
                    Some(b) => Some(pythonize::pythonize(py, b).map_err(map_err)?.unbind()),
                    None => None,
                },
                after: Some(pythonize::pythonize(py, after).map_err(map_err)?.unbind()),
                grouping_keys: None,
            }),
            ResultDiff::Noop => Ok(Self {
                diff_type: "noop".to_string(),
                data: None,
                before: None,
                after: None,
                grouping_keys: None,
            }),
        }
    }
}

// ============================================================================
// PyQueryResult — typed wrapper for QueryResult
// ============================================================================

/// A query result containing change diffs produced by a continuous query.
///
/// Yielded by ``ResultStream`` iteration. Contains one or more ``ResultDiff``
/// entries describing what changed.
#[pyclass(name = "QueryResult", frozen)]
pub struct PyQueryResult {
    query_id: String,
    timestamp: String,
    results: PyObject,
    metadata: PyObject,
}

#[pymethods]
impl PyQueryResult {
    /// The ID of the query that produced this result.
    #[getter]
    fn query_id(&self) -> &str {
        &self.query_id
    }

    /// ISO 8601 timestamp of when the result was produced.
    #[getter]
    fn timestamp(&self) -> &str {
        &self.timestamp
    }

    /// List of ``ResultDiff`` objects describing the changes.
    #[getter]
    fn results(&self, py: Python<'_>) -> PyObject {
        self.results.clone_ref(py)
    }

    /// Metadata dict associated with this result (usually empty).
    #[getter]
    fn metadata(&self, py: Python<'_>) -> PyObject {
        self.metadata.clone_ref(py)
    }

    /// Return a human-readable string representation.
    fn __repr__(&self) -> String {
        format!(
            "QueryResult(query_id={:?}, timestamp={:?})",
            self.query_id,
            self.timestamp,
        )
    }
}

impl PyQueryResult {
    fn from_rust(
        py: Python<'_>,
        result: &drasi_lib::channels::events::QueryResult,
    ) -> PyResult<Self> {
        let diffs: Vec<Py<PyResultDiff>> = result
            .results
            .iter()
            .map(|d| Py::new(py, PyResultDiff::from_rust(py, d)?))
            .collect::<PyResult<Vec<_>>>()?;
        let results_list = pyo3::types::PyList::new_bound(py, diffs);
        let metadata = pythonize::pythonize(py, &result.metadata)
            .map_err(map_err)?
            .unbind();
        Ok(Self {
            query_id: result.query_id.clone(),
            timestamp: result.timestamp.to_rfc3339(),
            results: results_list.unbind().into(),
            metadata,
        })
    }
}

// ============================================================================
// PyResultStream
// ============================================================================

/// Async iterator over query result changes from an application reaction.
///
/// Yields ``QueryResult`` objects. Use ``async for result in stream:`` to consume.
#[pyclass(name = "ResultStream")]
pub struct PyResultStream {
    inner: Arc<TokioMutex<ResultStream>>,
}

#[pymethods]
impl PyResultStream {
    /// Return self to support ``async for`` iteration.
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Await the next ``QueryResult``. Raises ``StopAsyncIteration`` when the stream ends.
    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let stream = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = stream.lock().await;
            match guard.next().await {
                Some(result) => Python::with_gil(|py| {
                    PyQueryResult::from_rust(py, &result)
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
    m.add_class::<PyResultDiff>()?;
    m.add_class::<PyQueryResult>()?;
    Ok(())
}
