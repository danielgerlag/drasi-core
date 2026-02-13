use std::sync::{Arc, Mutex};

use drasi_index_rocksdb::RocksDbIndexProvider;
use _drasi_core::builder::index_to_capsule;
use _drasi_core::errors::map_err;
use pyo3::prelude::*;

/// Persistent index provider backed by RocksDB.
///
/// Args:
///     path: Directory path for the RocksDB database files.
///     enable_archive: Enable archive mode for historical queries.
///     direct_io: Use direct I/O for disk access.
#[pyclass(name = "RocksDbIndexProvider")]
pub struct PyRocksDbIndexProvider {
    inner: Mutex<Option<RocksDbIndexProvider>>,
}

#[pymethods]
impl PyRocksDbIndexProvider {
    /// Create a new RocksDbIndexProvider.
    #[new]
    #[pyo3(signature = (path, enable_archive=false, direct_io=false))]
    fn new(path: &str, enable_archive: bool, direct_io: bool) -> Self {
        Self {
            inner: Mutex::new(Some(RocksDbIndexProvider::new(
                path,
                enable_archive,
                direct_io,
            ))),
        }
    }

    /// Return a capsule for use with DrasiLibBuilder.with_index_provider().
    fn into_index_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let provider = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("RocksDbIndexProvider already consumed")
            })?;
        index_to_capsule(py, Arc::new(provider))
    }
}

/// Python module for drasi_index_rocksdb
#[pymodule]
fn _drasi_index_rocksdb(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRocksDbIndexProvider>()?;
    Ok(())
}
