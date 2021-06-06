use std::any::Any;
use std::sync::Arc;

use arrow::datatypes::SchemaRef;
use datafusion::datasource::datasource::Statistics;
use datafusion::datasource::datasource::TableProviderFilterPushDown;
use datafusion::datasource::TableProvider;
use datafusion::error::Result;
use datafusion::logical_plan::{Expr, Operator};
use datafusion::physical_plan::ExecutionPlan;

use crate::datafusion::{DataSource, JSONExec};

#[derive(Debug)]
pub struct JSONTableProvider<D>
where
  D: DataSource,
{
  datasource: D,
  statistics: Statistics,
}

impl<D> JSONTableProvider<D>
where
  D: DataSource,
{
  pub fn new(datasource: D) -> Self {
    Self {
      datasource,
      statistics: Statistics::default(),
    }
  }

  fn supports_filtering(&self, name: String) -> bool {
    let schema = self.datasource.schema();

    let field = schema.field_with_name(name.as_str());
    if field.is_err() {
      return false;
    }

    let metadata = field.unwrap().metadata().clone();
    if metadata.is_none() {
      return false;
    }

    metadata.unwrap().contains_key("filter")
  }
}

impl<D> TableProvider for JSONTableProvider<D>
where
  D: DataSource,
{
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn schema(&self) -> SchemaRef {
    self.datasource.schema().clone()
  }

  // TODO: clean this up
  fn supports_filter_pushdown(&self, filter: &Expr) -> Result<TableProviderFilterPushDown> {
    if let Expr::BinaryExpr { left, right, op } = filter {
      if let Expr::Column(name) = &**left {
        if self.supports_filtering(name.clone()) {
          if matches!(op, Operator::Eq) && matches!(**right, Expr::Literal(..)) {
            Ok(TableProviderFilterPushDown::Exact)
          } else {
            Ok(TableProviderFilterPushDown::Unsupported)
          }
        } else {
          Ok(TableProviderFilterPushDown::Unsupported)
        }
      } else {
        Ok(TableProviderFilterPushDown::Unsupported)
      }
    } else {
      Ok(TableProviderFilterPushDown::Unsupported)
    }
  }

  fn scan(
    &self,
    projection: &Option<Vec<usize>>,
    _batch_size: usize,
    filters: &[Expr],
    _limit: Option<usize>,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    Ok(Arc::new(JSONExec::new(
      self.datasource.clone(),
      filters.to_vec(),
      projection,
    )))
  }

  fn statistics(&self) -> Statistics {
    self.statistics.clone()
  }
}
