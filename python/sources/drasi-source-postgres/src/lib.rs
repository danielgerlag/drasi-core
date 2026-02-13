use std::sync::Mutex;

use _drasi_core::builder::source_to_capsule;
use _drasi_core::errors::map_err;
use drasi_source_postgres::{PostgresReplicationSource, PostgresSourceBuilder, SslMode, TableKeyConfig};
use pyo3::prelude::*;

/// Builder for configuring and constructing a ``PostgresSource``.
///
/// Use ``PostgresSource.builder(id)`` or ``PostgresSourceBuilder(id)`` to create.
#[pyclass(name = "PostgresSourceBuilder")]
pub struct PyPostgresSourceBuilder {
    inner: Option<PostgresSourceBuilder>,
}

#[pymethods]
impl PyPostgresSourceBuilder {
    /// Create a new PostgresSourceBuilder.
    ///
    /// Args:
    ///     id: Unique identifier for this source.
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(PostgresSourceBuilder::new(id)),
        }
    }

    /// Set the PostgreSQL server hostname.
    fn with_host(&mut self, host: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_host(host));
        Ok(())
    }

    /// Set the PostgreSQL server port.
    fn with_port(&mut self, port: u16) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_port(port));
        Ok(())
    }

    /// Set the database name to connect to.
    fn with_database(&mut self, database: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_database(database));
        Ok(())
    }

    /// Set the database user for authentication.
    fn with_user(&mut self, user: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_user(user));
        Ok(())
    }

    /// Set the database password for authentication.
    fn with_password(&mut self, password: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_password(password));
        Ok(())
    }

    /// Set the list of tables to replicate from.
    fn with_tables(&mut self, tables: Vec<String>) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_tables(tables));
        Ok(())
    }

    /// Add a single table to the replication set.
    fn add_table(&mut self, table: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.add_table(table));
        Ok(())
    }

    /// Set the PostgreSQL replication slot name.
    fn with_slot_name(&mut self, slot_name: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_slot_name(slot_name));
        Ok(())
    }

    /// Set whether the source starts automatically when added to a ``DrasiLib``.
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

    /// Set the PostgreSQL publication name for logical replication.
    fn with_publication_name(&mut self, name: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_publication_name(name));
        Ok(())
    }

    /// Set the SSL mode for the connection ('disable', 'prefer', or 'require').
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

    /// Consume the builder and return a configured ``PostgresSource``.
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

/// A source that streams changes from a PostgreSQL database via logical replication.
#[pyclass(name = "PostgresSource")]
pub struct PyPostgresSource {
    inner: Mutex<Option<PostgresReplicationSource>>,
}

#[pymethods]
impl PyPostgresSource {
    /// Create a new ``PostgresSourceBuilder`` for the given source id.
    #[staticmethod]
    fn builder(id: &str) -> PyPostgresSourceBuilder {
        PyPostgresSourceBuilder::new(id)
    }

    /// Consume the source and return a PyCapsule for use with ``DrasiLibBuilder``.
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
