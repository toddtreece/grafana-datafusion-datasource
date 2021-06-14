mod datafusion;
mod plugin;
pub mod proto;
mod service;

pub use crate::datafusion::{DataSource, JSONTableProvider};
pub use crate::plugin::{start, DataProvider, DiagnosticsProvider, HealthStatus, Plugin, Query};
