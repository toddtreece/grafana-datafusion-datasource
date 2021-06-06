use std::task::{Context, Poll};

use arrow::datatypes::SchemaRef;
use arrow::error::Result as ArrowResult;
use arrow::record_batch::RecordBatch;
use datafusion::physical_plan::RecordBatchStream;
use futures::Stream;

pub(crate) struct MemoryStream {
  data: Vec<RecordBatch>,
  schema: SchemaRef,
  projection: Option<Vec<usize>>,
  index: usize,
}

impl MemoryStream {
  pub fn try_new(
    data: Vec<RecordBatch>,
    schema: SchemaRef,
    projection: Option<Vec<usize>>,
  ) -> ArrowResult<Self> {
    Ok(Self {
      data,
      schema,
      projection,
      index: 0,
    })
  }
}

impl Stream for MemoryStream {
  type Item = ArrowResult<RecordBatch>;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    _: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    Poll::Ready(if self.index < self.data.len() {
      self.index += 1;
      let batch = &self.data[self.index - 1];
      match &self.projection {
        Some(columns) => Some(RecordBatch::try_new(
          self.schema.clone(),
          columns.iter().map(|i| batch.column(*i).clone()).collect(),
        )),
        None => Some(Ok(batch.clone())),
      }
    } else {
      None
    })
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.data.len(), Some(self.data.len()))
  }
}

impl RecordBatchStream for MemoryStream {
  fn schema(&self) -> SchemaRef {
    self.schema.clone()
  }
}
