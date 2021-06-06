mod github;

use std::sync::Arc;

use github::ReleaseTable;
use grafana_plugin_sdk::{start, JSONTableProvider, Plugin};

#[tokio::main]
async fn main() {
  let plugin = Plugin::new("github");
  let table = JSONTableProvider::new(ReleaseTable);
  plugin
    .register_table("releases".to_owned(), Arc::new(table))
    .await
    .expect("failed to register table");
  start(plugin).await.unwrap();
}
