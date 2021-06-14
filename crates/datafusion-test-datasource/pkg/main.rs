mod github;

use std::sync::Arc;

use github::{PullRequestTable, ReleaseTable};
use grafana_plugin_sdk::{start, JSONTableProvider, Plugin};

#[tokio::main]
async fn main() {
  let plugin = Plugin::new("github");

  let releases = JSONTableProvider::new(ReleaseTable);
  plugin
    .register_table("releases".to_owned(), Arc::new(releases))
    .await
    .expect("failed to register releases");

  let prs = JSONTableProvider::new(PullRequestTable);
  plugin
    .register_table("pull_requests".to_owned(), Arc::new(prs))
    .await
    .expect("failed to register pull_requests");

  start(plugin).await.unwrap();
}
