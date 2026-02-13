use drasi_core::models::SourceMiddlewareConfig;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Python wrapper for SourceMiddlewareConfig
#[pyclass(name = "SourceMiddlewareConfig")]
#[derive(Clone)]
pub struct PySourceMiddlewareConfig {
    pub inner: SourceMiddlewareConfig,
}

#[pymethods]
impl PySourceMiddlewareConfig {
    /// Create a new middleware configuration.
    ///
    /// Args:
    ///     kind: The middleware type (e.g. "map", "decoder", "relabel")
    ///     name: A unique name for this middleware instance
    ///     config: A dict of middleware-specific configuration
    #[new]
    fn new(kind: &str, name: &str, config: &Bound<'_, PyDict>) -> PyResult<Self> {
        let config_value: serde_json::Value = pythonize::depythonize(config)?;
        let config_map = match config_value {
            serde_json::Value::Object(map) => map,
            _ => {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "config must be a dict",
                ))
            }
        };
        Ok(Self {
            inner: SourceMiddlewareConfig::new(kind, name, config_map),
        })
    }
}

/// Python module for drasi_middleware
#[pymodule]
fn _drasi_middleware(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySourceMiddlewareConfig>()?;
    Ok(())
}
