use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{Read, Seek};

use anyhow::Result;
use arrow::datatypes::SchemaRef;
use async_trait::async_trait;

#[async_trait]
pub trait DataSource: Send + Sync + Clone + Debug + 'static {
  type Data: Read + Seek + Send + Sync + Debug + Clone;

  async fn fetch_results(&self, options: HashMap<String, String>) -> Result<Self::Data>;

  fn schema(&self) -> SchemaRef;
}
