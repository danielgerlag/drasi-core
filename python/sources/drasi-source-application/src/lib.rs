use std::collections::HashMap;
use std::sync::Mutex;

use _drasi_core::builder::source_to_capsule;
use _drasi_core::errors::map_err;
use drasi_source_application::{
    ApplicationSource, ApplicationSourceConfig, ApplicationSourceHandle, PropertyMapBuilder,
};
use pyo3::prelude::*;

/// Wraps `ElementPropertyMap` as an opaque Python type.
#[pyclass(name = "PropertyMap")]
pub struct PyPropertyMap {
    inner: Mutex<Option<drasi_core::models::ElementPropertyMap>>,
}

impl PyPropertyMap {
    fn take(&self) -> PyResult<drasi_core::models::ElementPropertyMap> {
        self.inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("PropertyMap already consumed")
            })
    }
}

/// Builder for `PyPropertyMap`.
#[pyclass(name = "PropertyMapBuilder")]
pub struct PyPropertyMapBuilder {
    inner: Option<PropertyMapBuilder>,
}

#[pymethods]
impl PyPropertyMapBuilder {
    #[new]
    fn new() -> Self {
        Self {
            inner: Some(PropertyMapBuilder::new()),
        }
    }

    fn with_string(&mut self, key: &str, value: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_string(key, value));
        Ok(())
    }

    fn with_integer(&mut self, key: &str, value: i64) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_integer(key, value));
        Ok(())
    }

    fn with_float(&mut self, key: &str, value: f64) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_float(key, value));
        Ok(())
    }

    fn with_bool(&mut self, key: &str, value: bool) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_bool(key, value));
        Ok(())
    }

    fn with_null(&mut self, key: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_null(key));
        Ok(())
    }

    fn build(&mut self) -> PyResult<PyPropertyMap> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        Ok(PyPropertyMap {
            inner: Mutex::new(Some(builder.build())),
        })
    }
}

/// Wraps `ApplicationSourceHandle` for Python.
#[pyclass(name = "ApplicationSourceHandle")]
#[derive(Clone)]
pub struct PyApplicationSourceHandle {
    inner: ApplicationSourceHandle,
}

#[pymethods]
impl PyApplicationSourceHandle {
    fn send_node_insert<'py>(
        &self,
        py: Python<'py>,
        id: String,
        labels: Vec<String>,
        properties: &PyPropertyMap,
    ) -> PyResult<Bound<'py, PyAny>> {
        let handle = self.inner.clone();
        let props = properties.take()?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            handle
                .send_node_insert(id, labels, props)
                .await
                .map_err(map_err)?;
            Ok(())
        })
    }

    fn send_node_update<'py>(
        &self,
        py: Python<'py>,
        id: String,
        labels: Vec<String>,
        properties: &PyPropertyMap,
    ) -> PyResult<Bound<'py, PyAny>> {
        let handle = self.inner.clone();
        let props = properties.take()?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            handle
                .send_node_update(id, labels, props)
                .await
                .map_err(map_err)?;
            Ok(())
        })
    }

    fn send_delete<'py>(
        &self,
        py: Python<'py>,
        id: String,
        labels: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let handle = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            handle.send_delete(id, labels).await.map_err(map_err)?;
            Ok(())
        })
    }

    fn send_relation_insert<'py>(
        &self,
        py: Python<'py>,
        id: String,
        labels: Vec<String>,
        properties: &PyPropertyMap,
        start_node_id: String,
        end_node_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let handle = self.inner.clone();
        let props = properties.take()?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            handle
                .send_relation_insert(id, labels, props, start_node_id, end_node_id)
                .await
                .map_err(map_err)?;
            Ok(())
        })
    }

    fn source_id(&self) -> String {
        self.inner.source_id().to_string()
    }
}

/// Converts a Python object to a `serde_json::Value`.
fn py_object_to_json(py: Python<'_>, obj: &PyObject) -> PyResult<serde_json::Value> {
    let bound = obj.bind(py);
    pythonize::depythonize(bound).map_err(map_err)
}

/// Wraps `ApplicationSource` for Python.
#[pyclass(name = "ApplicationSource")]
pub struct PyApplicationSource {
    source: Mutex<Option<ApplicationSource>>,
    handle: ApplicationSourceHandle,
}

#[pymethods]
impl PyApplicationSource {
    #[new]
    #[pyo3(signature = (id, properties=None))]
    fn new(py: Python<'_>, id: &str, properties: Option<HashMap<String, PyObject>>) -> PyResult<Self> {
        let props = match properties {
            Some(map) => {
                let mut converted = HashMap::new();
                for (k, v) in map {
                    converted.insert(k, py_object_to_json(py, &v)?);
                }
                converted
            }
            None => HashMap::new(),
        };

        let config = ApplicationSourceConfig { properties: props };
        let (source, handle) = ApplicationSource::new(id, config).map_err(map_err)?;

        Ok(Self {
            source: Mutex::new(Some(source)),
            handle,
        })
    }

    fn get_handle(&self) -> PyApplicationSourceHandle {
        PyApplicationSourceHandle {
            inner: self.handle.clone(),
        }
    }

    fn into_source_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let source = self
            .source
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("Source already consumed")
            })?;
        source_to_capsule(py, source)
    }
}

/// Python module for drasi_source_application
#[pymodule]
fn _drasi_source_application(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPropertyMap>()?;
    m.add_class::<PyPropertyMapBuilder>()?;
    m.add_class::<PyApplicationSourceHandle>()?;
    m.add_class::<PyApplicationSource>()?;
    Ok(())
}
