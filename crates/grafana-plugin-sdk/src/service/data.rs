use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::DerefMut;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::plugin::{DataProvider, Query};
use crate::proto::pluginv2::data_server::Data;
use crate::proto::pluginv2::{DataQuery, DataResponse, QueryDataRequest, QueryDataResponse};

pub struct DataService<P> {
  provider: P,
}

impl<P: DataProvider> DataService<P> {
  pub fn new(provider: P) -> Self {
    Self { provider }
  }

  async fn to_vec(&self, batch: RecordBatch) -> Result<Vec<u8>> {
    let buffer: RwLock<Vec<u8>> = RwLock::new(Vec::new());
    let mut write_buf = buffer.write().await;

    {
      let mut writer = FileWriter::try_new(write_buf.deref_mut(), &batch.schema())?;
      writer.write(&batch)?;
      writer
        .finish()
        .or(Err(anyhow!("unable to finish write buffer")))?;
    }

    drop(write_buf);

    let v = buffer.read().await.to_owned();
    Ok(v)
  }

  async fn handle_query(&self, q: &DataQuery) -> Result<Arc<Vec<Vec<u8>>>> {
    let batches = self
      .provider
      .handle_query(Query::try_from(q.clone())?)
      .await?;

    let mut frames = vec![];

    for batch in batches.iter() {
      frames.push(self.to_vec(batch.clone()).await?);
    }

    Ok(Arc::new(frames))
  }
}

#[async_trait]
impl<P: DataProvider + Sync + Send + 'static> Data for DataService<P> {
  async fn query_data(
    &self,
    request: tonic::Request<QueryDataRequest>,
  ) -> Result<tonic::Response<QueryDataResponse>, tonic::Status> {
    let mut responses: HashMap<String, DataResponse> = HashMap::new();
    for query in request.into_inner().queries.iter() {
      let mut frames = Arc::new(vec![]);
      let mut error = "".to_string();
      let results = self.handle_query(query).await;

      match results {
        Ok(f) => frames = Arc::clone(&f),
        Err(e) => error = e.to_string(),
      }

      responses.insert(
        query.clone().ref_id,
        DataResponse {
          frames: frames.to_vec(),
          error,
          json_meta: vec![],
        },
      );
    }

    Ok(tonic::Response::new(QueryDataResponse { responses }))
  }
}
