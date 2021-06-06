use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::SchemaRef;
use arrow::json::ReaderBuilder;
use async_trait::async_trait;
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_plan::Expr;
use datafusion::physical_plan::{ExecutionPlan, Partitioning, SendableRecordBatchStream};

use crate::datafusion::{DataSource, MemoryStream};

#[derive(Debug)]
pub struct JSONExec<D>
where
  D: DataSource,
{
  projection: Option<Vec<usize>>,
  filters: Vec<Expr>,
  datasource: D,
}

impl<D> JSONExec<D>
where
  D: DataSource,
{
  pub fn new(datasource: D, filters: Vec<Expr>, projection: &Option<Vec<usize>>) -> Self {
    Self {
      filters,
      projection: projection.clone(),
      datasource,
    }
  }
}

#[async_trait]
impl<D> ExecutionPlan for JSONExec<D>
where
  D: DataSource,
{
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn schema(&self) -> SchemaRef {
    self.datasource.schema().clone()
  }

  fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
    vec![]
  }

  fn output_partitioning(&self) -> Partitioning {
    Partitioning::UnknownPartitioning(1)
  }

  fn with_new_children(&self, _: Vec<Arc<dyn ExecutionPlan>>) -> Result<Arc<dyn ExecutionPlan>> {
    Err(DataFusionError::Internal(format!(
      "Children cannot be replaced in {:?}",
      self
    )))
  }

  async fn execute(&self, _partition: usize) -> Result<SendableRecordBatchStream> {
    let options =
      self
        .filters
        .clone()
        .into_iter()
        .fold(HashMap::<String, String>::new(), |mut acc, exp| {
          if let Expr::BinaryExpr { left, right, op: _ } = exp {
            match (*left, *right) {
              (Expr::Column(key), Expr::Literal(val)) => {
                acc.insert(key, val.to_string());
              }
              _ => {}
            }
          };
          acc
        });

    let results = match self.datasource.fetch_results(options).await {
      Ok(r) => Ok(r),
      Err(e) => Err(DataFusionError::Execution(e.to_string())),
    }?;

    let schema = self.schema();
    let mut builder = ReaderBuilder::new().with_schema(schema.clone());

    if let Some(projection) = self.projection.clone() {
      builder = builder.with_projection(
        projection
          .clone()
          .into_iter()
          .filter(|i| i < &schema.fields().len())
          .map(|i| schema.fields().get(i))
          .filter(|f| f.is_some())
          .map(|f| f.unwrap().name().clone())
          .collect(),
      );
    }

    let mut reader = builder.build(results)?;
    let mut results = Vec::new();

    while let Ok(Some(r)) = reader.next() {
      results.push(r);
    }

    Ok(Box::pin(MemoryStream::try_new(
      results.clone(),
      reader.schema().clone(),
      None,
    )?))
  }
}
