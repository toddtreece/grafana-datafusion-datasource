mod data;
mod diagnostic;

use std::net::SocketAddr;
use std::net::TcpListener;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use datafusion::catalog::catalog::MemoryCatalogProvider;
use datafusion::catalog::schema::MemorySchemaProvider;
use datafusion::datasource::TableProvider;
use datafusion::prelude::*;
use tokio::sync::Mutex;
use tonic::transport::Server;

use crate::proto::pluginv2::data_server::DataServer;
use crate::proto::pluginv2::diagnostics_server::DiagnosticsServer;
use crate::service::DataService;
use crate::service::DiagnosticsService;

pub use data::{DataProvider, Query};
pub use diagnostic::{CheckHealthResponse, DiagnosticsProvider, HealthStatus};

fn get_addr() -> Result<SocketAddr> {
  Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?)
}

#[derive(Clone)]
pub struct Plugin {
  name: String,
  ctx: Arc<Mutex<ExecutionContext>>,
}

#[async_trait]
impl DataProvider for Plugin {
  async fn handle_query(&self, query: Query) -> Result<Vec<RecordBatch>> {
    let ctx = Arc::clone(&self.ctx);
    let mut lock = ctx.lock().await;
    let df = lock.sql(query.sql.as_str())?;
    let result: Vec<RecordBatch> = df.collect().await?;
    Ok(result)
  }
}

// TODO: allow plugin to implement
#[async_trait]
impl DiagnosticsProvider for Plugin {
  async fn check_health(&self) -> CheckHealthResponse {
    CheckHealthResponse {
      status: HealthStatus::Ok.into(),
      message: "Ok".to_string(),
      json_details: vec![],
    }
  }
}

impl Plugin {
  pub fn new(name: &str) -> Self {
    let ctx = ExecutionContext::with_config(ExecutionConfig::new().with_information_schema(true));

    let catalog_provider = MemoryCatalogProvider::new();
    let schema_provider = MemorySchemaProvider::new();

    catalog_provider.register_schema(name.to_owned().clone(), Arc::new(schema_provider));
    ctx.register_catalog("datasource", Arc::new(catalog_provider));

    Self {
      name: name.to_owned(),
      ctx: Arc::new(Mutex::new(ctx)),
    }
  }

  pub async fn register_table(
    &self,
    table_name: String,
    table: Arc<dyn TableProvider>,
  ) -> Result<()> {
    let ctx = Arc::clone(&self.ctx);
    let lock = ctx.lock().await;

    lock
      .catalog("datasource")
      .ok_or(anyhow!("catalog not found"))?
      .schema(self.name.as_str())
      .ok_or(anyhow!("schema not found"))?
      .register_table(table_name.to_owned(), Arc::clone(&table))?;
    Ok(())
  }
}

pub async fn start(plugin: Plugin) -> Result<()> {
  let addr = get_addr()?;

  // this is required for go-plugin handshakes
  println!("1|2|tcp|{}:{}|grpc", "localhost", addr.port());

  Server::builder()
    .add_service(DataServer::new(DataService::new(plugin.clone())))
    .add_service(DiagnosticsServer::new(DiagnosticsService::new(plugin)))
    .serve(addr)
    .await?;

  Ok(())
}
