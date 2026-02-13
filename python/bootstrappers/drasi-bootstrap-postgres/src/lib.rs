use std::sync::Mutex;

use _drasi_core::builder::bootstrap_to_capsule;
use _drasi_core::errors::map_err;
use drasi_bootstrap_postgres::{PostgresBootstrapProvider, PostgresBootstrapProviderBuilder};
use pyo3::prelude::*;

/// Builder for creating a PostgresBootstrapProvider.
///
/// Configure connection details and tables before calling build().
#[pyclass(name = "PostgresBootstrapProviderBuilder")]
pub struct PyPostgresBootstrapProviderBuilder {
    inner: Option<PostgresBootstrapProviderBuilder>,
}

#[pymethods]
impl PyPostgresBootstrapProviderBuilder {
    /// Create a new PostgresBootstrapProviderBuilder.
    #[new]
    fn new() -> Self {
        Self {
            inner: Some(PostgresBootstrapProviderBuilder::new()),
        }
    }

    /// Set the PostgreSQL host address.
    fn with_host(&mut self, host: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_host(host));
        Ok(())
    }

    /// Set the PostgreSQL port number.
    fn with_port(&mut self, port: u16) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_port(port));
        Ok(())
    }

    /// Set the PostgreSQL database name.
    fn with_database(&mut self, database: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_database(database));
        Ok(())
    }

    /// Set the PostgreSQL user for authentication.
    fn with_user(&mut self, user: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_user(user));
        Ok(())
    }

    /// Set the PostgreSQL password for authentication.
    fn with_password(&mut self, password: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_password(password));
        Ok(())
    }

    /// Set the list of tables to bootstrap from.
    fn with_tables(&mut self, tables: Vec<String>) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_tables(tables));
        Ok(())
    }

    /// Add a single table to bootstrap from.
    fn with_table(&mut self, table: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_table(table));
        Ok(())
    }

    /// Consume the builder and return a PostgresBootstrapProvider.
    fn build(&mut self) -> PyResult<PyPostgresBootstrapProvider> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let provider = builder.build();
        Ok(PyPostgresBootstrapProvider {
            inner: Mutex::new(Some(provider)),
        })
    }
}

/// Bootstrap provider that loads initial data from a PostgreSQL database.
#[pyclass(name = "PostgresBootstrapProvider")]
pub struct PyPostgresBootstrapProvider {
    inner: Mutex<Option<PostgresBootstrapProvider>>,
}

#[pymethods]
impl PyPostgresBootstrapProvider {
    /// Create a new builder for this provider.
    #[staticmethod]
    fn builder() -> PyPostgresBootstrapProviderBuilder {
        PyPostgresBootstrapProviderBuilder::new()
    }

    /// Return a capsule for use with DrasiLibBuilder.
    fn into_bootstrap_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let provider = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("Provider already consumed")
            })?;
        bootstrap_to_capsule(py, provider)
    }
}

/// Python module for drasi_bootstrap_postgres
#[pymodule]
fn _drasi_bootstrap_postgres(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPostgresBootstrapProviderBuilder>()?;
    m.add_class::<PyPostgresBootstrapProvider>()?;
    Ok(())
}
