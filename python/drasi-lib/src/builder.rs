use pyo3::prelude::*;

use _drasi_core::builder::{
    index_from_capsule, reaction_from_capsule, source_from_capsule,
    state_store_from_capsule,
};
use _drasi_core::errors::map_err;
use _drasi_middleware::PySourceMiddlewareConfig;

/// Fluent builder for query configurations.
///
/// Use Query.cypher(id) or Query.gql(id) to create a builder,
/// then chain setter methods and call build() to produce a QueryConfig.
#[pyclass]
pub struct Query {
    inner: Option<drasi_lib::builder::Query>,
}

#[pymethods]
impl Query {
    /// Create a Cypher query builder
    #[staticmethod]
    fn cypher(id: &str) -> Self {
        Self {
            inner: Some(drasi_lib::builder::Query::cypher(id)),
        }
    }

    /// Create a GQL query builder
    #[staticmethod]
    fn gql(id: &str) -> Self {
        Self {
            inner: Some(drasi_lib::builder::Query::gql(id)),
        }
    }

    /// Set the query string
    fn query(&mut self, query: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Query builder already consumed")
        })?;
        self.inner = Some(inner.query(query));
        Ok(())
    }

    /// Set the source ID this query reads from
    fn from_source(&mut self, source_id: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Query builder already consumed")
        })?;
        self.inner = Some(inner.from_source(source_id));
        Ok(())
    }

    /// Set whether the query auto-starts
    fn auto_start(&mut self, auto_start: bool) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Query builder already consumed")
        })?;
        self.inner = Some(inner.auto_start(auto_start));
        Ok(())
    }

    /// Enable or disable bootstrap
    fn enable_bootstrap(&mut self, enable: bool) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Query builder already consumed")
        })?;
        self.inner = Some(inner.enable_bootstrap(enable));
        Ok(())
    }

    /// Add middleware to the query
    fn with_middleware(&mut self, middleware: PySourceMiddlewareConfig) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Query builder already consumed")
        })?;
        self.inner = Some(inner.with_middleware(middleware.inner));
        Ok(())
    }

    /// Subscribe to a source with a middleware pipeline
    fn from_source_with_pipeline(
        &mut self,
        source_id: &str,
        pipeline: Vec<String>,
    ) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Query builder already consumed")
        })?;
        self.inner = Some(inner.from_source_with_pipeline(source_id, pipeline));
        Ok(())
    }

    /// Set priority queue capacity
    fn with_priority_queue_capacity(&mut self, capacity: usize) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Query builder already consumed")
        })?;
        self.inner = Some(inner.with_priority_queue_capacity(capacity));
        Ok(())
    }

    /// Set dispatch buffer capacity
    fn with_dispatch_buffer_capacity(&mut self, capacity: usize) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Query builder already consumed")
        })?;
        self.inner = Some(inner.with_dispatch_buffer_capacity(capacity));
        Ok(())
    }

    /// Set dispatch mode
    fn with_dispatch_mode(&mut self, mode: _drasi_core::types::DispatchMode) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Query builder already consumed")
        })?;
        self.inner = Some(inner.with_dispatch_mode(mode.into()));
        Ok(())
    }

    /// Consume the builder and return a QueryConfig.
    fn build(&mut self) -> PyResult<PyQueryConfig> {
        let inner = self
            .inner
            .take()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Query already built"))?;
        Ok(PyQueryConfig {
            inner: Some(inner.build()),
        })
    }
}

/// Opaque query configuration produced by Query.build().
///
/// Pass this to DrasiLibBuilder.with_query() or DrasiLib.add_query().
#[pyclass(name = "QueryConfig")]
pub struct PyQueryConfig {
    pub(crate) inner: Option<drasi_lib::QueryConfig>,
}

/// Fluent builder for constructing and initializing a DrasiLib instance.
///
/// Add sources, queries, reactions, and optional providers before calling build().
#[pyclass]
pub struct DrasiLibBuilder {
    inner: Option<drasi_lib::DrasiLibBuilder>,
}

#[pymethods]
impl DrasiLibBuilder {
    /// Create a new DrasiLibBuilder.
    #[new]
    fn new() -> Self {
        Self {
            inner: Some(drasi_lib::DrasiLib::builder()),
        }
    }

    /// Set the server ID
    fn with_id(&mut self, id: &str) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_id(id));
        Ok(())
    }

    /// Set priority queue capacity
    fn with_priority_queue_capacity(&mut self, capacity: usize) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_priority_queue_capacity(capacity));
        Ok(())
    }

    /// Set dispatch buffer capacity
    fn with_dispatch_buffer_capacity(&mut self, capacity: usize) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(inner.with_dispatch_buffer_capacity(capacity));
        Ok(())
    }

    /// Add a pre-built source instance (takes ownership via capsule)
    fn with_source(&mut self, source: &Bound<'_, PyAny>) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let src = source_from_capsule(source)?;
        self.inner = Some(inner.with_source(src));
        Ok(())
    }

    /// Add a query configuration
    fn with_query(&mut self, query: &mut PyQueryConfig) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let config = query.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("QueryConfig already consumed")
        })?;
        self.inner = Some(inner.with_query(config));
        Ok(())
    }

    /// Add a pre-built reaction instance (takes ownership via capsule)
    fn with_reaction(&mut self, reaction: &Bound<'_, PyAny>) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let rxn = reaction_from_capsule(reaction)?;
        self.inner = Some(inner.with_reaction(rxn));
        Ok(())
    }

    /// Add an index backend provider (via capsule)
    fn with_index_provider(&mut self, provider: &Bound<'_, PyAny>) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let prov = index_from_capsule(provider)?;
        self.inner = Some(inner.with_index_provider(prov));
        Ok(())
    }

    /// Add a state store provider (via capsule)
    fn with_state_store_provider(
        &mut self,
        provider: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let prov = state_store_from_capsule(provider)?;
        self.inner = Some(inner.with_state_store_provider(prov));
        Ok(())
    }

    /// Consume the builder and asynchronously initialize a DrasiLib instance.
    fn build<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self
            .inner
            .take()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Builder already used"))?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let lib = inner.build().await.map_err(map_err)?;
            Ok(super::drasi_lib::PyDrasiLib::new(lib))
        })
    }
}

/// Register builder types on the module
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Query>()?;
    m.add_class::<PyQueryConfig>()?;
    m.add_class::<DrasiLibBuilder>()?;
    Ok(())
}
