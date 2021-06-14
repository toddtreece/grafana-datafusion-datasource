use async_trait::async_trait;

pub use crate::proto::pluginv2::check_health_response::HealthStatus;
pub use crate::proto::pluginv2::CheckHealthResponse;

#[async_trait]
pub trait DiagnosticsProvider {
  async fn check_health(&self) -> CheckHealthResponse;
}
