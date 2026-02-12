use std::ffi::CString;
use std::sync::Mutex;

use pyo3::prelude::*;
use pyo3::types::PyCapsule;

// Capsule names for type safety
const SOURCE_CAPSULE_NAME: &str = "drasi_source_wrapper";
const REACTION_CAPSULE_NAME: &str = "drasi_reaction_wrapper";
const BOOTSTRAP_CAPSULE_NAME: &str = "drasi_bootstrap_wrapper";
const INDEX_CAPSULE_NAME: &str = "drasi_index_wrapper";
const STATE_STORE_CAPSULE_NAME: &str = "drasi_state_store_wrapper";

// ============================================================================
// Cross-cdylib executor
// ============================================================================
// Each cdylib has its own copy of tokio with separate statics.
// A tokio Handle from one cdylib cannot be used from another's code.
// To work around this, we store a function pointer in the capsule that
// runs an async task on the originating cdylib's runtime. The function
// pointer's code lives in the originating cdylib, so it uses the correct
// copy of tokio.

/// Type-erased async executor function.
/// Takes a boxed, pinned, Send future and runs it to completion,
/// returning the result as a type-erased Box.
type AsyncExecutor = fn(
    // The future to execute, boxed and type-erased.
    // The actual future produces a Box<dyn std::any::Any + Send>.
    task: std::pin::Pin<Box<dyn std::future::Future<Output = Box<dyn std::any::Any + Send>> + Send>>,
) -> Box<dyn std::any::Any + Send>;

/// Execute a future on the pyo3-async-runtimes tokio runtime of the current cdylib.
/// This function's code lives in whichever cdylib calls `source_to_capsule()` etc.,
/// so it uses that cdylib's copy of tokio/pyo3-async-runtimes.
fn execute_on_local_runtime(
    task: std::pin::Pin<Box<dyn std::future::Future<Output = Box<dyn std::any::Any + Send>> + Send>>,
) -> Box<dyn std::any::Any + Send> {
    let handle = pyo3_async_runtimes::tokio::get_runtime().handle().clone();
    std::thread::spawn(move || handle.block_on(task))
        .join()
        .expect("Cross-cdylib executor thread panicked")
}

/// Payload stored inside a capsule: the trait object + an executor function.
struct SourcePayload {
    source: Mutex<Option<Box<dyn drasi_lib::Source>>>,
    executor: AsyncExecutor,
}

struct ReactionPayload {
    reaction: Mutex<Option<Box<dyn drasi_lib::Reaction>>>,
    executor: AsyncExecutor,
}

struct BootstrapPayload {
    provider: Mutex<Option<Box<dyn drasi_lib::BootstrapProvider>>>,
}

struct IndexPayload {
    backend: Mutex<Option<std::sync::Arc<dyn drasi_lib::IndexBackendPlugin>>>,
}

struct StateStorePayload {
    provider: Mutex<Option<std::sync::Arc<dyn drasi_lib::StateStoreProvider>>>,
}

// ============================================================================
// Source wrapper that uses the originating cdylib's executor
// ============================================================================

struct SourceWithExecutor {
    inner: std::sync::Arc<Box<dyn drasi_lib::Source>>,
    executor: AsyncExecutor,
}

impl SourceWithExecutor {
    /// Run an async closure on the source's originating runtime.
    fn run<F, T>(&self, f: F) -> T
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let boxed_future: std::pin::Pin<Box<dyn std::future::Future<Output = Box<dyn std::any::Any + Send>> + Send>> =
            Box::pin(async move {
                let result = f.await;
                Box::new(result) as Box<dyn std::any::Any + Send>
            });
        let result = (self.executor)(boxed_future);
        *result.downcast::<T>().expect("executor returned wrong type")
    }
}

#[async_trait::async_trait]
impl drasi_lib::Source for SourceWithExecutor {
    fn id(&self) -> &str {
        self.inner.id()
    }
    fn type_name(&self) -> &str {
        self.inner.type_name()
    }
    fn properties(&self) -> std::collections::HashMap<String, serde_json::Value> {
        self.inner.properties()
    }
    fn dispatch_mode(&self) -> drasi_lib::DispatchMode {
        self.inner.dispatch_mode()
    }
    fn auto_start(&self) -> bool {
        self.inner.auto_start()
    }
    async fn start(&self) -> anyhow::Result<()> {
        let inner = self.inner.clone();
        self.run(async move { inner.start().await })
    }
    async fn stop(&self) -> anyhow::Result<()> {
        let inner = self.inner.clone();
        self.run(async move { inner.stop().await })
    }
    async fn status(&self) -> drasi_lib::ComponentStatus {
        let inner = self.inner.clone();
        self.run(async move { inner.status().await })
    }
    async fn subscribe(
        &self,
        settings: drasi_lib::SourceSubscriptionSettings,
    ) -> anyhow::Result<drasi_lib::SubscriptionResponse> {
        let inner = self.inner.clone();
        self.run(async move { inner.subscribe(settings).await })
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self.inner.as_any()
    }
    async fn initialize(&self, context: drasi_lib::SourceRuntimeContext) {
        let inner = self.inner.clone();
        self.run(async move { inner.initialize(context).await })
    }
    async fn set_bootstrap_provider(
        &self,
        provider: Box<dyn drasi_lib::bootstrap::BootstrapProvider + 'static>,
    ) {
        let inner = self.inner.clone();
        self.run(async move { inner.set_bootstrap_provider(provider).await })
    }
}

// ============================================================================
// Reaction wrapper
// ============================================================================

struct ReactionWithExecutor {
    inner: std::sync::Arc<Box<dyn drasi_lib::Reaction>>,
    executor: AsyncExecutor,
}

impl ReactionWithExecutor {
    fn run<F, T>(&self, f: F) -> T
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let boxed_future: std::pin::Pin<Box<dyn std::future::Future<Output = Box<dyn std::any::Any + Send>> + Send>> =
            Box::pin(async move {
                let result = f.await;
                Box::new(result) as Box<dyn std::any::Any + Send>
            });
        let result = (self.executor)(boxed_future);
        *result.downcast::<T>().expect("executor returned wrong type")
    }
}

#[async_trait::async_trait]
impl drasi_lib::Reaction for ReactionWithExecutor {
    fn id(&self) -> &str {
        self.inner.id()
    }
    fn type_name(&self) -> &str {
        self.inner.type_name()
    }
    fn properties(&self) -> std::collections::HashMap<String, serde_json::Value> {
        self.inner.properties()
    }
    fn auto_start(&self) -> bool {
        self.inner.auto_start()
    }
    fn query_ids(&self) -> Vec<String> {
        self.inner.query_ids()
    }
    async fn start(&self) -> anyhow::Result<()> {
        let inner = self.inner.clone();
        self.run(async move { inner.start().await })
    }
    async fn stop(&self) -> anyhow::Result<()> {
        let inner = self.inner.clone();
        self.run(async move { inner.stop().await })
    }
    async fn status(&self) -> drasi_lib::ComponentStatus {
        let inner = self.inner.clone();
        self.run(async move { inner.status().await })
    }
    async fn initialize(&self, context: drasi_lib::ReactionRuntimeContext) {
        let inner = self.inner.clone();
        self.run(async move { inner.initialize(context).await })
    }
}

/// Create a PyCapsule containing a `Box<dyn Source>` + executor function.
/// The capsule owns the inner value and can be extracted exactly once.
pub fn source_to_capsule(
    py: Python<'_>,
    source: impl drasi_lib::Source + 'static,
) -> PyResult<PyObject> {
    let payload = SourcePayload {
        source: Mutex::new(Some(Box::new(source))),
        executor: execute_on_local_runtime,
    };
    let name = CString::new(SOURCE_CAPSULE_NAME).unwrap();
    let capsule = PyCapsule::new_bound(py, payload, Some(name))?;
    Ok(capsule.into())
}

/// Extract a `Box<dyn Source>` from a PyCapsule, wrapped with the executor.
pub fn source_from_capsule(obj: &Bound<'_, PyAny>) -> PyResult<Box<dyn drasi_lib::Source>> {
    let capsule = obj.downcast::<PyCapsule>().map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err(
            "Expected a source wrapper capsule (returned by into_source_wrapper())",
        )
    })?;
    let payload = unsafe { &*(capsule.pointer() as *const SourcePayload) };
    let source = payload
        .source
        .lock()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
        .take()
        .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Source already consumed"))?;
    Ok(Box::new(SourceWithExecutor {
        inner: std::sync::Arc::new(source),
        executor: payload.executor,
    }))
}

/// Create a PyCapsule containing a `Box<dyn Reaction>` + executor function.
pub fn reaction_to_capsule(
    py: Python<'_>,
    reaction: impl drasi_lib::Reaction + 'static,
) -> PyResult<PyObject> {
    let payload = ReactionPayload {
        reaction: Mutex::new(Some(Box::new(reaction))),
        executor: execute_on_local_runtime,
    };
    let name = CString::new(REACTION_CAPSULE_NAME).unwrap();
    let capsule = PyCapsule::new_bound(py, payload, Some(name))?;
    Ok(capsule.into())
}

/// Extract a `Box<dyn Reaction>` from a PyCapsule, wrapped with the executor.
pub fn reaction_from_capsule(obj: &Bound<'_, PyAny>) -> PyResult<Box<dyn drasi_lib::Reaction>> {
    let capsule = obj.downcast::<PyCapsule>().map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err(
            "Expected a reaction wrapper capsule (returned by into_reaction_wrapper())",
        )
    })?;
    let payload = unsafe { &*(capsule.pointer() as *const ReactionPayload) };
    let reaction = payload
        .reaction
        .lock()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
        .take()
        .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Reaction already consumed"))?;
    Ok(Box::new(ReactionWithExecutor {
        inner: std::sync::Arc::new(reaction),
        executor: payload.executor,
    }))
}

/// Create a PyCapsule containing a `Box<dyn BootstrapProvider>`.
pub fn bootstrap_to_capsule(
    py: Python<'_>,
    provider: impl drasi_lib::BootstrapProvider + 'static,
) -> PyResult<PyObject> {
    let payload = BootstrapPayload {
        provider: Mutex::new(Some(Box::new(provider))),
    };
    let name = CString::new(BOOTSTRAP_CAPSULE_NAME).unwrap();
    let capsule = PyCapsule::new_bound(py, payload, Some(name))?;
    Ok(capsule.into())
}

/// Extract a `Box<dyn BootstrapProvider>` from a PyCapsule.
pub fn bootstrap_from_capsule(
    obj: &Bound<'_, PyAny>,
) -> PyResult<Box<dyn drasi_lib::BootstrapProvider>> {
    let capsule = obj.downcast::<PyCapsule>().map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err(
            "Expected a bootstrap provider wrapper capsule (returned by into_bootstrap_wrapper())",
        )
    })?;
    let payload = unsafe { &*(capsule.pointer() as *const BootstrapPayload) };
    payload
        .provider
        .lock()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
        .take()
        .ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("BootstrapProvider already consumed")
        })
}

/// Create a PyCapsule containing an `Arc<dyn IndexBackendPlugin>`.
pub fn index_to_capsule(
    py: Python<'_>,
    backend: std::sync::Arc<dyn drasi_lib::IndexBackendPlugin>,
) -> PyResult<PyObject> {
    let payload = IndexPayload {
        backend: Mutex::new(Some(backend)),
    };
    let name = CString::new(INDEX_CAPSULE_NAME).unwrap();
    let capsule = PyCapsule::new_bound(py, payload, Some(name))?;
    Ok(capsule.into())
}

/// Extract an `Arc<dyn IndexBackendPlugin>` from a PyCapsule.
pub fn index_from_capsule(
    obj: &Bound<'_, PyAny>,
) -> PyResult<std::sync::Arc<dyn drasi_lib::IndexBackendPlugin>> {
    let capsule = obj.downcast::<PyCapsule>().map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err(
            "Expected an index backend wrapper capsule (returned by into_index_wrapper())",
        )
    })?;
    let payload = unsafe { &*(capsule.pointer() as *const IndexPayload) };
    payload
        .backend
        .lock()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
        .take()
        .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("IndexBackend already consumed"))
}

/// Create a PyCapsule containing an `Arc<dyn StateStoreProvider>`.
pub fn state_store_to_capsule(
    py: Python<'_>,
    provider: std::sync::Arc<dyn drasi_lib::StateStoreProvider>,
) -> PyResult<PyObject> {
    let payload = StateStorePayload {
        provider: Mutex::new(Some(provider)),
    };
    let name = CString::new(STATE_STORE_CAPSULE_NAME).unwrap();
    let capsule = PyCapsule::new_bound(py, payload, Some(name))?;
    Ok(capsule.into())
}

/// Extract an `Arc<dyn StateStoreProvider>` from a PyCapsule.
pub fn state_store_from_capsule(
    obj: &Bound<'_, PyAny>,
) -> PyResult<std::sync::Arc<dyn drasi_lib::StateStoreProvider>> {
    let capsule = obj.downcast::<PyCapsule>().map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err(
            "Expected a state store wrapper capsule (returned by into_state_store_wrapper())",
        )
    })?;
    let payload = unsafe { &*(capsule.pointer() as *const StateStorePayload) };
    payload
        .provider
        .lock()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
        .take()
        .ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("StateStoreProvider already consumed")
        })
}

/// Register builder types on the module (no #[pyclass] wrappers needed)
pub fn register(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
