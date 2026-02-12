use pyo3::prelude::*;

/// Dispatch mode enum exposed to Python
#[pyclass(eq, frozen)]
#[derive(Debug, Clone, PartialEq)]
pub enum DispatchMode {
    Channel,
    Broadcast,
}

impl From<drasi_lib::DispatchMode> for DispatchMode {
    fn from(m: drasi_lib::DispatchMode) -> Self {
        match m {
            drasi_lib::DispatchMode::Channel => Self::Channel,
            drasi_lib::DispatchMode::Broadcast => Self::Broadcast,
        }
    }
}

impl From<DispatchMode> for drasi_lib::DispatchMode {
    fn from(m: DispatchMode) -> Self {
        match m {
            DispatchMode::Channel => Self::Channel,
            DispatchMode::Broadcast => Self::Broadcast,
        }
    }
}

/// Register types on the module
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DispatchMode>()?;
    Ok(())
}
