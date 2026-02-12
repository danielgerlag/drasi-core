use pyo3::prelude::*;
use tokio::sync::broadcast;

use crate::types::{ComponentEvent, LogMessage};

use std::sync::Arc;
use tokio::sync::Mutex;

/// An event subscription that provides history + live async iteration.
#[pyclass]
pub struct EventSubscription {
    /// Historical events at time of subscription
    #[pyo3(get)]
    history: Vec<ComponentEvent>,
    /// Live stream for new events
    stream: Option<Py<EventStream>>,
}

impl EventSubscription {
    pub fn new(
        py: Python<'_>,
        history: Vec<drasi_lib::ComponentEvent>,
        receiver: broadcast::Receiver<drasi_lib::ComponentEvent>,
    ) -> PyResult<Self> {
        let stream = Py::new(py, EventStream::new(receiver))?;
        Ok(Self {
            history: history.into_iter().map(ComponentEvent::from).collect(),
            stream: Some(stream),
        })
    }
}

#[pymethods]
impl EventSubscription {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyResult<Py<EventStream>> {
        slf.stream
            .as_ref()
            .map(|s| s.clone_ref(slf.py()))
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("EventSubscription already consumed")
            })
    }
}

/// Async iterator over component events
#[pyclass]
pub struct EventStream {
    receiver: Arc<Mutex<broadcast::Receiver<drasi_lib::ComponentEvent>>>,
}

impl EventStream {
    pub fn new(receiver: broadcast::Receiver<drasi_lib::ComponentEvent>) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

#[pymethods]
impl EventStream {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let rx = self.receiver.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = rx.lock().await;
            loop {
                match guard.recv().await {
                    Ok(event) => return Ok(ComponentEvent::from(event)),
                    Err(broadcast::error::RecvError::Closed) => {
                        return Err(pyo3::exceptions::PyStopAsyncIteration::new_err(()));
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                }
            }
        })
    }
}

/// Async iterator over log messages
#[pyclass]
pub struct LogStream {
    receiver: Arc<Mutex<broadcast::Receiver<drasi_lib::LogMessage>>>,
}

impl LogStream {
    pub fn new(receiver: broadcast::Receiver<drasi_lib::LogMessage>) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

#[pymethods]
impl LogStream {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let rx = self.receiver.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = rx.lock().await;
            loop {
                match guard.recv().await {
                    Ok(msg) => return Ok(LogMessage::from(msg)),
                    Err(broadcast::error::RecvError::Closed) => {
                        return Err(pyo3::exceptions::PyStopAsyncIteration::new_err(()));
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                }
            }
        })
    }
}

/// Log subscription with history + live streaming
#[pyclass]
pub struct LogSubscription {
    #[pyo3(get)]
    history: Vec<LogMessage>,
    stream: Option<Py<LogStream>>,
}

impl LogSubscription {
    pub fn new(
        py: Python<'_>,
        history: Vec<drasi_lib::LogMessage>,
        receiver: broadcast::Receiver<drasi_lib::LogMessage>,
    ) -> PyResult<Self> {
        let stream = Py::new(py, LogStream::new(receiver))?;
        Ok(Self {
            history: history.into_iter().map(LogMessage::from).collect(),
            stream: Some(stream),
        })
    }
}

#[pymethods]
impl LogSubscription {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyResult<Py<LogStream>> {
        slf.stream
            .as_ref()
            .map(|s| s.clone_ref(slf.py()))
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("LogSubscription already consumed")
            })
    }
}

/// Register streaming types on the module
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<EventStream>()?;
    m.add_class::<LogStream>()?;
    m.add_class::<EventSubscription>()?;
    m.add_class::<LogSubscription>()?;
    Ok(())
}
