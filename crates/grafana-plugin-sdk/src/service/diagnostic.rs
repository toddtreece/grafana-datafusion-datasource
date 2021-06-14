use anyhow::Result;
use async_trait::async_trait;

use crate::plugin::DiagnosticsProvider;
use crate::proto::pluginv2::diagnostics_server::Diagnostics;
use crate::proto::pluginv2::{
  CheckHealthRequest, CheckHealthResponse, CollectMetricsRequest, CollectMetricsResponse,
};

pub struct DiagnosticsService<P> {
  provider: P,
}

impl<P: DiagnosticsProvider> DiagnosticsService<P> {
  pub fn new(provider: P) -> Self {
    Self { provider }
  }
}

#[async_trait]
impl<P: DiagnosticsProvider + Sync + Send + 'static> Diagnostics for DiagnosticsService<P> {
  // TODO: pass request to provider
  async fn check_health(
    &self,
    _request: tonic::Request<CheckHealthRequest>,
  ) -> Result<tonic::Response<CheckHealthResponse>, tonic::Status> {
    Ok(tonic::Response::new(self.provider.check_health().await))
  }

  // TODO: implement
  async fn collect_metrics(
    &self,
    _request: tonic::Request<CollectMetricsRequest>,
  ) -> Result<tonic::Response<CollectMetricsResponse>, tonic::Status> {
    Ok(tonic::Response::new(CollectMetricsResponse {
      metrics: None,
    }))
  }
}
