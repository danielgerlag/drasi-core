use std::sync::Mutex;

use _drasi_core::builder::reaction_to_capsule;
use _drasi_core::errors::map_err;
use drasi_reaction_log::{LogReaction, LogReactionBuilder, QueryConfig, TemplateSpec};
use pyo3::prelude::*;

// ============================================================================
// PyLogReactionBuilder
// ============================================================================

#[pyclass(name = "LogReactionBuilder")]
pub struct PyLogReactionBuilder {
    inner: Option<LogReactionBuilder>,
}

#[pymethods]
impl PyLogReactionBuilder {
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(LogReactionBuilder::new(id)),
        }
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

    #[pyo3(signature = (added=None, updated=None, deleted=None))]
    fn with_default_template(
        &mut self,
        added: Option<String>,
        updated: Option<String>,
        deleted: Option<String>,
    ) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let config = QueryConfig {
            added: added.map(|t| TemplateSpec {
                template: t,
                ..Default::default()
            }),
            updated: updated.map(|t| TemplateSpec {
                template: t,
                ..Default::default()
            }),
            deleted: deleted.map(|t| TemplateSpec {
                template: t,
                ..Default::default()
            }),
        };
        self.inner = Some(inner.with_default_template(config));
        Ok(())
    }

    #[pyo3(signature = (query_id, added=None, updated=None, deleted=None))]
    fn with_route(
        &mut self,
        query_id: &str,
        added: Option<String>,
        updated: Option<String>,
        deleted: Option<String>,
    ) -> PyResult<()> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let config = QueryConfig {
            added: added.map(|t| TemplateSpec {
                template: t,
                ..Default::default()
            }),
            updated: updated.map(|t| TemplateSpec {
                template: t,
                ..Default::default()
            }),
            deleted: deleted.map(|t| TemplateSpec {
                template: t,
                ..Default::default()
            }),
        };
        self.inner = Some(inner.with_route(query_id, config));
        Ok(())
    }

    fn build(&mut self) -> PyResult<PyLogReaction> {
        let inner = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let reaction = inner.build().map_err(map_err)?;
        Ok(PyLogReaction {
            inner: Mutex::new(Some(reaction)),
        })
    }
}

// ============================================================================
// PyLogReaction
// ============================================================================

#[pyclass(name = "LogReaction")]
pub struct PyLogReaction {
    inner: Mutex<Option<LogReaction>>,
}

#[pymethods]
impl PyLogReaction {
    #[staticmethod]
    fn builder(id: &str) -> PyLogReactionBuilder {
        PyLogReactionBuilder {
            inner: Some(LogReaction::builder(id)),
        }
    }

    fn into_reaction_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let reaction = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("LogReaction already consumed")
            })?;
        reaction_to_capsule(py, reaction)
    }
}

/// Python module for drasi_reaction_log
#[pymodule]
fn _drasi_reaction_log(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLogReactionBuilder>()?;
    m.add_class::<PyLogReaction>()?;
    Ok(())
}
