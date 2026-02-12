use std::sync::Mutex;

use _drasi_core::builder::reaction_to_capsule;
use _drasi_core::errors::map_err;
use drasi_reaction_storedproc_mysql::reaction::MySqlStoredProcReactionBuilder;
use drasi_reaction_storedproc_mysql::MySqlStoredProcReaction;
use pyo3::prelude::*;

#[pyclass]
pub struct PyMySqlStoredProcReactionBuilder {
    inner: Option<MySqlStoredProcReactionBuilder>,
}

#[pymethods]
impl PyMySqlStoredProcReactionBuilder {
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(MySqlStoredProcReactionBuilder::new(id)),
        }
    }

    fn with_hostname(&mut self, hostname: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_hostname(hostname));
        Ok(())
    }

    fn with_port(&mut self, port: u16) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_port(port));
        Ok(())
    }

    fn with_database(&mut self, database: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_database(database));
        Ok(())
    }

    fn with_user(&mut self, user: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_user(user));
        Ok(())
    }

    fn with_password(&mut self, password: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_password(password));
        Ok(())
    }

    fn with_ssl(&mut self, enable: bool) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_ssl(enable));
        Ok(())
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

    fn with_command_timeout_ms(&mut self, timeout_ms: u64) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_command_timeout_ms(timeout_ms));
        Ok(())
    }

    fn with_retry_attempts(&mut self, attempts: u32) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_retry_attempts(attempts));
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

    fn build<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let reaction = inner.build().await.map_err(map_err)?;
            Ok(PyMySqlStoredProcReaction {
                inner: Mutex::new(Some(reaction)),
            })
        })
    }
}

#[pyclass]
pub struct PyMySqlStoredProcReaction {
    inner: Mutex<Option<MySqlStoredProcReaction>>,
}

#[pymethods]
impl PyMySqlStoredProcReaction {
    #[staticmethod]
    fn builder(id: &str) -> PyMySqlStoredProcReactionBuilder {
        PyMySqlStoredProcReactionBuilder {
            inner: Some(MySqlStoredProcReaction::builder(id)),
        }
    }

    fn into_reaction_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let reaction = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "MySqlStoredProcReaction already consumed",
                )
            })?;
        reaction_to_capsule(py, reaction)
    }
}

/// Python module for drasi_reaction_storedproc_mysql
#[pymodule]
fn _drasi_reaction_storedproc_mysql(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMySqlStoredProcReactionBuilder>()?;
    m.add_class::<PyMySqlStoredProcReaction>()?;
    Ok(())
}
