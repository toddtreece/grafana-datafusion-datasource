use std::convert::TryFrom;

use anyhow::Result;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use serde::Deserialize;

use crate::proto::pluginv2::{DataQuery, TimeRange};

#[derive(PartialEq, Clone)]
pub struct Query {
  pub ref_id: String,
  pub query_type: String,
  pub interval_ms: i64,
  pub time_range: TimeRange,
  pub max_data_points: i64,
  pub sql: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DataQueryJSON {
  sql: String,
}

impl TryFrom<DataQuery> for Query {
  type Error = serde_json::Error;

  fn try_from(q: DataQuery) -> Result<Query, Self::Error> {
    let json: DataQueryJSON = serde_json::from_slice(&q.json[..])?;
    return Ok(Query {
      ref_id: q.ref_id.clone(),
      query_type: q.query_type.clone(),
      interval_ms: q.interval_ms.clone(),
      time_range: q.time_range.unwrap_or(TimeRange::default()).clone(),
      max_data_points: q.max_data_points.clone(),
      sql: json.sql.clone(),
    });
  }
}

#[async_trait]
pub trait DataProvider {
  async fn handle_query(&self, query: Query) -> Result<Vec<RecordBatch>>;
}
