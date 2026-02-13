use pyo3::prelude::*;

/// Lifecycle status of a Drasi component (source, query, or reaction).
#[pyclass(eq, frozen)]
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
}

impl From<drasi_lib::ComponentStatus> for ComponentStatus {
    fn from(s: drasi_lib::ComponentStatus) -> Self {
        match s {
            drasi_lib::ComponentStatus::Starting => Self::Starting,
            drasi_lib::ComponentStatus::Running => Self::Running,
            drasi_lib::ComponentStatus::Stopping => Self::Stopping,
            drasi_lib::ComponentStatus::Stopped => Self::Stopped,
            drasi_lib::ComponentStatus::Error => Self::Error,
        }
    }
}

impl From<ComponentStatus> for drasi_lib::ComponentStatus {
    fn from(s: ComponentStatus) -> Self {
        match s {
            ComponentStatus::Starting => Self::Starting,
            ComponentStatus::Running => Self::Running,
            ComponentStatus::Stopping => Self::Stopping,
            ComponentStatus::Stopped => Self::Stopped,
            ComponentStatus::Error => Self::Error,
        }
    }
}

/// The kind of Drasi component: Source, Query, or Reaction.
#[pyclass(eq, frozen)]
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Source,
    Query,
    Reaction,
}

impl From<drasi_lib::ComponentType> for ComponentType {
    fn from(t: drasi_lib::ComponentType) -> Self {
        match t {
            drasi_lib::ComponentType::Source => Self::Source,
            drasi_lib::ComponentType::Query => Self::Query,
            drasi_lib::ComponentType::Reaction => Self::Reaction,
        }
    }
}

/// Severity level for log messages: Trace, Debug, Info, Warn, or Error.
#[pyclass(eq, eq_int, frozen)]
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<drasi_lib::LogLevel> for LogLevel {
    fn from(l: drasi_lib::LogLevel) -> Self {
        match l {
            drasi_lib::LogLevel::Trace => Self::Trace,
            drasi_lib::LogLevel::Debug => Self::Debug,
            drasi_lib::LogLevel::Info => Self::Info,
            drasi_lib::LogLevel::Warn => Self::Warn,
            drasi_lib::LogLevel::Error => Self::Error,
        }
    }
}

/// A lifecycle event emitted when a component changes status.
#[pyclass(frozen, get_all)]
#[derive(Debug, Clone)]
pub struct ComponentEvent {
    pub component_id: String,
    pub component_type: ComponentType,
    pub status: ComponentStatus,
    pub timestamp: String,
    pub message: Option<String>,
}

impl From<drasi_lib::ComponentEvent> for ComponentEvent {
    fn from(e: drasi_lib::ComponentEvent) -> Self {
        Self {
            component_id: e.component_id,
            component_type: e.component_type.into(),
            status: e.status.into(),
            timestamp: e.timestamp.to_rfc3339(),
            message: e.message,
        }
    }
}

#[pymethods]
impl ComponentEvent {
    /// Return a human-readable string representation.
    fn __repr__(&self) -> String {
        format!(
            "ComponentEvent(id={:?}, type={:?}, status={:?}, message={:?})",
            self.component_id, self.component_type, self.status, self.message,
        )
    }
}

/// A log message emitted by a Drasi component.
#[pyclass(frozen, get_all)]
#[derive(Debug, Clone)]
pub struct LogMessage {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub instance_id: String,
    pub component_id: String,
    pub component_type: ComponentType,
}

impl From<drasi_lib::LogMessage> for LogMessage {
    fn from(l: drasi_lib::LogMessage) -> Self {
        Self {
            timestamp: l.timestamp.to_rfc3339(),
            level: l.level.into(),
            message: l.message,
            instance_id: l.instance_id,
            component_id: l.component_id,
            component_type: l.component_type.into(),
        }
    }
}

#[pymethods]
impl LogMessage {
    /// Return a human-readable string representation.
    fn __repr__(&self) -> String {
        format!(
            "LogMessage(level={:?}, component={:?}, message={:?})",
            self.level, self.component_id, self.message,
        )
    }
}

/// A query result containing change diffs produced by a continuous query.
#[pyclass(frozen, get_all)]
#[derive(Debug)]
pub struct QueryResult {
    pub query_id: String,
    pub timestamp: String,
    pub results: Py<PyAny>,
    pub metadata: Py<PyAny>,
}

impl QueryResult {
    pub fn from_rust(py: Python<'_>, r: &drasi_lib::channels::QueryResult) -> PyResult<Self> {
        let results = pythonize::pythonize(py, &r.results)?;
        let metadata = pythonize::pythonize(py, &r.metadata)?;
        Ok(Self {
            query_id: r.query_id.clone(),
            timestamp: r.timestamp.to_rfc3339(),
            results: results.into(),
            metadata: metadata.into(),
        })
    }
}

#[pymethods]
impl QueryResult {
    /// Return a human-readable string representation.
    fn __repr__(&self) -> String {
        format!(
            "QueryResult(query_id={:?}, timestamp={:?})",
            self.query_id, self.timestamp,
        )
    }
}

/// Register all types on the module
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ComponentStatus>()?;
    m.add_class::<ComponentType>()?;
    m.add_class::<LogLevel>()?;
    m.add_class::<ComponentEvent>()?;
    m.add_class::<LogMessage>()?;
    m.add_class::<QueryResult>()?;
    Ok(())
}
