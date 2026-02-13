use std::sync::Mutex;

use _drasi_core::builder::source_to_capsule;
use _drasi_core::errors::map_err;
use drasi_source_postgres::{PostgresReplicationSource, PostgresSourceBuilder, SslMode, TableKeyConfig};
use pyo3::prelude::*;

#[pyclass(name = "PostgresSourceBuilder")]
pub struct PyPostgresSourceBuilder {
    inner: Option<PostgresSourceBuilder>,
}

#[pymethods]
impl PyPostgresSourceBuilder {
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(PostgresSourceBuilder::new(id)),
        }
    }

    fn with_host(&mut self, host: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_host(host));
        Ok(())
    }

    fn with_port(&mut self, port: u16) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_port(port));
        Ok(())
    }

    fn with_database(&mut self, database: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_database(database));
        Ok(())
    }

    fn with_user(&mut self, user: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_user(user));
        Ok(())
    }

    fn with_password(&mut self, password: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_password(password));
        Ok(())
    }

    fn with_tables(&mut self, tables: Vec<String>) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_tables(tables));
        Ok(())
    }

    fn add_table(&mut self, table: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.add_table(table));
        Ok(())
    }

    fn with_slot_name(&mut self, slot_name: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_slot_name(slot_name));
        Ok(())
    }

    fn with_auto_start(&mut self, auto_start: bool) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_auto_start(auto_start));
        Ok(())
    }

    /// Add a table key configuration specifying which columns form the primary key
    /// for a given table. This ensures stable element IDs across insert/update/delete.
    fn add_table_key(&mut self, table: &str, key_columns: Vec<String>) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.add_table_key(TableKeyConfig {
            table: table.to_string(),
            key_columns,
        }));
        Ok(())
    }

    fn with_publication_name(&mut self, name: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_publication_name(name));
        Ok(())
    }

    fn with_ssl_mode(&mut self, mode: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let ssl_mode = match mode {
            "disable" => SslMode::Disable,
            "prefer" => SslMode::Prefer,
            "require" => SslMode::Require,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid ssl_mode: '{mode}'. Expected 'disable', 'prefer', or 'require'"
                )));
            }
        };
        self.inner = Some(builder.with_ssl_mode(ssl_mode));
        Ok(())
    }

    fn build(&mut self) -> PyResult<PyPostgresSource> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let source = builder.build().map_err(map_err)?;
        Ok(PyPostgresSource {
            inner: Mutex::new(Some(source)),
        })
    }
}

#[pyclass(name = "PostgresSource")]
pub struct PyPostgresSource {
    inner: Mutex<Option<PostgresReplicationSource>>,
}

#[pymethods]
impl PyPostgresSource {
    #[staticmethod]
    fn builder(id: &str) -> PyPostgresSourceBuilder {
        PyPostgresSourceBuilder::new(id)
    }

    fn into_source_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let source = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("Source already consumed")
            })?;
        source_to_capsule(py, source)
    }
}

/// Python module for drasi_source_postgres
#[pymodule]
fn _drasi_source_postgres(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPostgresSourceBuilder>()?;
    m.add_class::<PyPostgresSource>()?;
    Ok(())
}
