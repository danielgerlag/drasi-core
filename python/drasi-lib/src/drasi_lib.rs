use pyo3::prelude::*;

use _drasi_core::builder::{reaction_from_capsule, source_from_capsule};
use _drasi_core::errors::map_err;
use crate::builder::PyQueryConfig;
use crate::streaming::{EventSubscription, LogSubscription};
use crate::types::ComponentStatus;

/// The main Drasi runtime instance.
///
/// Manages sources, queries, and reactions. Created via DrasiLibBuilder.build().
#[pyclass(name = "DrasiLib")]
pub struct PyDrasiLib {
    inner: drasi_lib::DrasiLib,
}

impl PyDrasiLib {
    pub fn new(inner: drasi_lib::DrasiLib) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyDrasiLib {
    // ========================================================================
    // Lifecycle
    // ========================================================================

    /// Start the server and all auto-start components
    fn start<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.start().await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Stop the server and all running components
    fn stop<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.stop().await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Check if the server is currently running
    fn is_running<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            Ok(lib.is_running().await)
        })
    }

    // ========================================================================
    // Source Operations
    // ========================================================================

    /// Add a source instance (via capsule)
    fn add_source<'py>(
        &self,
        py: Python<'py>,
        source: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let src = source_from_capsule(source)?;
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.add_source(src).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Remove a source by ID
    fn remove_source<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.remove_source(&id).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Start a source by ID
    fn start_source<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.start_source(&id).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Stop a source by ID
    fn stop_source<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.stop_source(&id).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// List all sources with their statuses
    fn list_sources<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let sources = lib.list_sources().await.map_err(map_err)?;
            let result: Vec<(String, ComponentStatus)> = sources
                .into_iter()
                .map(|(id, status)| (id, status.into()))
                .collect();
            Ok(result)
        })
    }

    /// Get the status of a source
    fn get_source_status<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let status = lib.get_source_status(&id).await.map_err(map_err)?;
            Ok(ComponentStatus::from(status))
        })
    }

    // ========================================================================
    // Query Operations
    // ========================================================================

    /// Add a query configuration
    fn add_query<'py>(
        &self,
        py: Python<'py>,
        query: &mut PyQueryConfig,
    ) -> PyResult<Bound<'py, PyAny>> {
        let config = query.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("QueryConfig already consumed")
        })?;
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.add_query(config).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Remove a query by ID
    fn remove_query<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.remove_query(&id).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Start a query by ID
    fn start_query<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.start_query(&id).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Stop a query by ID
    fn stop_query<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.stop_query(&id).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// List all queries with their statuses
    fn list_queries<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let queries = lib.list_queries().await.map_err(map_err)?;
            let result: Vec<(String, ComponentStatus)> = queries
                .into_iter()
                .map(|(id, status)| (id, status.into()))
                .collect();
            Ok(result)
        })
    }

    /// Get the status of a query
    fn get_query_status<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let status = lib.get_query_status(&id).await.map_err(map_err)?;
            Ok(ComponentStatus::from(status))
        })
    }

    // ========================================================================
    // Reaction Operations
    // ========================================================================

    /// Add a reaction instance (via capsule)
    fn add_reaction<'py>(
        &self,
        py: Python<'py>,
        reaction: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rxn = reaction_from_capsule(reaction)?;
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.add_reaction(rxn).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Remove a reaction by ID
    fn remove_reaction<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.remove_reaction(&id).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Start a reaction by ID
    fn start_reaction<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.start_reaction(&id).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Stop a reaction by ID
    fn stop_reaction<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            lib.stop_reaction(&id).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// List all reactions with their statuses
    fn list_reactions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let reactions = lib.list_reactions().await.map_err(map_err)?;
            let result: Vec<(String, ComponentStatus)> = reactions
                .into_iter()
                .map(|(id, status)| (id, status.into()))
                .collect();
            Ok(result)
        })
    }

    /// Get the status of a reaction
    fn get_reaction_status<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let status = lib
                .get_reaction_status(&id)
                .await
                .map_err(map_err)?;
            Ok(ComponentStatus::from(status))
        })
    }

    // ========================================================================
    // Event and Log Subscriptions
    // ========================================================================

    /// Subscribe to lifecycle events for a source
    fn subscribe_source_events<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (history, receiver) = lib
                .subscribe_source_events(&id)
                .await
                .map_err(map_err)?;
            Python::with_gil(|py| EventSubscription::new(py, history, receiver))
        })
    }

    /// Subscribe to lifecycle events for a query
    fn subscribe_query_events<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (history, receiver) = lib
                .subscribe_query_events(&id)
                .await
                .map_err(map_err)?;
            Python::with_gil(|py| EventSubscription::new(py, history, receiver))
        })
    }

    /// Subscribe to lifecycle events for a reaction
    fn subscribe_reaction_events<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (history, receiver) = lib
                .subscribe_reaction_events(&id)
                .await
                .map_err(map_err)?;
            Python::with_gil(|py| EventSubscription::new(py, history, receiver))
        })
    }

    /// Subscribe to logs for a source
    fn subscribe_source_logs<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (history, receiver) = lib
                .subscribe_source_logs(&id)
                .await
                .map_err(map_err)?;
            Python::with_gil(|py| LogSubscription::new(py, history, receiver))
        })
    }

    /// Subscribe to logs for a query
    fn subscribe_query_logs<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (history, receiver) = lib
                .subscribe_query_logs(&id)
                .await
                .map_err(map_err)?;
            Python::with_gil(|py| LogSubscription::new(py, history, receiver))
        })
    }

    /// Subscribe to logs for a reaction
    fn subscribe_reaction_logs<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lib = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (history, receiver) = lib
                .subscribe_reaction_logs(&id)
                .await
                .map_err(map_err)?;
            Python::with_gil(|py| LogSubscription::new(py, history, receiver))
        })
    }

    /// Return a string representation of this DrasiLib instance.
    fn __repr__(&self) -> String {
        format!("DrasiLib(id={:?})", self.inner.get_config().id)
    }
}

/// Register DrasiLib type on the module
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDrasiLib>()?;
    Ok(())
}
