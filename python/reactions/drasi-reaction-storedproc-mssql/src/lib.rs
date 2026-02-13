use std::sync::Mutex;

use _drasi_core::builder::reaction_to_capsule;
use _drasi_core::errors::map_err;
use drasi_reaction_storedproc_mssql::reaction::MsSqlStoredProcReactionBuilder;
use drasi_reaction_storedproc_mssql::MsSqlStoredProcReaction;
use pyo3::prelude::*;

/// Builder for configuring a Microsoft SQL Server stored procedure reaction.
///
/// Use ``MsSqlStoredProcReaction.builder("my-mssql")`` to create a new builder.
#[pyclass(name = "MsSqlStoredProcReactionBuilder")]
pub struct PyMsSqlStoredProcReactionBuilder {
    inner: Option<MsSqlStoredProcReactionBuilder>,
}

#[pymethods]
impl PyMsSqlStoredProcReactionBuilder {
    /// Create a new ``MsSqlStoredProcReactionBuilder`` with the given reaction ID.
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(MsSqlStoredProcReactionBuilder::new(id)),
        }
    }

    /// Set the MSSQL server hostname.
    fn with_hostname(&mut self, hostname: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_hostname(hostname));
        Ok(())
    }

    /// Set the MSSQL server port.
    fn with_port(&mut self, port: u16) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_port(port));
        Ok(())
    }

    /// Set the database name to connect to.
    fn with_database(&mut self, database: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_database(database));
        Ok(())
    }

    /// Set the database user for authentication.
    fn with_user(&mut self, user: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_user(user));
        Ok(())
    }

    /// Set the database password for authentication.
    fn with_password(&mut self, password: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_password(password));
        Ok(())
    }

    /// Enable or disable SSL for the database connection.
    fn with_ssl(&mut self, enable: bool) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_ssl(enable));
        Ok(())
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

    /// Set the command execution timeout in milliseconds.
    fn with_command_timeout_ms(&mut self, timeout_ms: u64) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_command_timeout_ms(timeout_ms));
        Ok(())
    }

    /// Set the number of retry attempts for failed commands.
    fn with_retry_attempts(&mut self, attempts: u32) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_retry_attempts(attempts));
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

    /// Finalize the builder and create the MsSqlStoredProcReaction. Consumes this builder.
    fn build<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let reaction = inner.build().await.map_err(map_err)?;
            Ok(PyMsSqlStoredProcReaction {
                inner: Mutex::new(Some(reaction)),
            })
        })
    }
}

/// A reaction that calls MSSQL stored procedures on query result changes.
#[pyclass(name = "MsSqlStoredProcReaction")]
pub struct PyMsSqlStoredProcReaction {
    inner: Mutex<Option<MsSqlStoredProcReaction>>,
}

#[pymethods]
impl PyMsSqlStoredProcReaction {
    /// Create a new ``MsSqlStoredProcReactionBuilder`` with the given reaction ID.
    #[staticmethod]
    fn builder(id: &str) -> PyMsSqlStoredProcReactionBuilder {
        PyMsSqlStoredProcReactionBuilder {
            inner: Some(MsSqlStoredProcReaction::builder(id)),
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
                pyo3::exceptions::PyRuntimeError::new_err(
                    "MsSqlStoredProcReaction already consumed",
                )
            })?;
        reaction_to_capsule(py, reaction)
    }
}

/// Python module for drasi_reaction_storedproc_mssql
#[pymodule]
fn _drasi_reaction_storedproc_mssql(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMsSqlStoredProcReactionBuilder>()?;
    m.add_class::<PyMsSqlStoredProcReaction>()?;
    Ok(())
}
