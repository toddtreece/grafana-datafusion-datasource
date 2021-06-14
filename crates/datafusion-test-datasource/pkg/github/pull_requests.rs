use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use std::sync::Arc;

use anyhow::Result;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use async_trait::async_trait;
use cached::proc_macro::cached;
use grafana_plugin_sdk::DataSource;
use octocrab::models::pulls::PullRequest as GitHubPull;
use octocrab::models::IssueState;
use octocrab::params::State;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct PullRequest {
  id: u64,
  created_at: i64,
  updated_at: Option<i64>,
  closed_at: Option<i64>,
  merged_at: Option<i64>,
  state: String,
  title: String,
  body: Option<String>,
  url: String,
  author: String,
  owner: String,
  repo: String,
}

// TODO: this should probably implement From/Into
fn to_ts(d: chrono::DateTime<chrono::Utc>) -> i64 {
  d.timestamp() * 1000 * 1000 * 1000
}

// TODO: this should probably implement From/Into
fn to_optional_ts(o: Option<chrono::DateTime<chrono::Utc>>) -> Option<i64> {
  match o {
    Some(d) => Some(to_ts(d)),
    None => None,
  }
}

#[cached(time = 600, result = true)]
async fn cached_fetch(owner: String, repo: String) -> Result<Vec<GitHubPull>> {
  let octocrab = octocrab::instance();

  let mut current_page = octocrab
    .pulls(owner, repo)
    .list()
    .state(State::All)
    .per_page(100)
    .send()
    .await?;

  let mut pulls = current_page.take_items();

  while let Some(mut page) = octocrab.get_page::<GitHubPull>(&current_page.next).await? {
    pulls.extend(page.take_items());
    current_page = page;
  }

  Ok(pulls)
}

#[derive(Debug, Clone)]
pub struct PullRequestTable;

#[async_trait]
impl DataSource for PullRequestTable {
  type Data = Cursor<Vec<u8>>;

  fn schema(&self) -> SchemaRef {
    let mut metadata = BTreeMap::new();
    metadata.insert("filter".to_owned(), "true".to_owned());

    let id = Field::new("id", DataType::UInt64, false);
    let created_at = Field::new(
      "created_at",
      DataType::Timestamp(TimeUnit::Nanosecond, None),
      false,
    );
    let updated_at = Field::new(
      "updated_at",
      DataType::Timestamp(TimeUnit::Nanosecond, None),
      true,
    );
    let closed_at = Field::new(
      "closed_at",
      DataType::Timestamp(TimeUnit::Nanosecond, None),
      true,
    );
    let merged_at = Field::new(
      "merged_at",
      DataType::Timestamp(TimeUnit::Nanosecond, None),
      true,
    );
    let title = Field::new("title", DataType::Utf8, false);
    let body = Field::new("body", DataType::Utf8, true);
    let url = Field::new("url", DataType::Utf8, false);
    let author = Field::new("author", DataType::Utf8, false);
    let state = Field::new("state", DataType::Utf8, false);

    // these fields are passed to the github api as filters
    let mut owner = Field::new("owner", DataType::Utf8, false);
    owner.set_metadata(Some(metadata.clone()));
    let mut repo = Field::new("repo", DataType::Utf8, false);
    repo.set_metadata(Some(metadata.clone()));

    let schema = Schema::new(vec![
      id, created_at, updated_at, merged_at, closed_at, title, body, url, state, author, owner,
      repo,
    ]);

    Arc::new(schema)
  }

  async fn fetch_results(&self, options: HashMap<String, String>) -> Result<Self::Data> {
    let owner = options.get("owner").unwrap_or(&"".to_owned()).clone();
    let repo = options.get("repo").unwrap_or(&"".to_owned()).clone();
    let pulls = cached_fetch(owner, repo).await?;

    let pulls: Vec<u8> = pulls
      .into_iter()
      .map(|r| PullRequest {
        id: *r.id.as_ref(),
        created_at: to_ts(r.created_at),
        updated_at: to_optional_ts(r.updated_at),
        closed_at: to_optional_ts(r.closed_at),
        merged_at: to_optional_ts(r.merged_at),
        title: r.title,
        body: r.body,
        url: r.url.to_string(),
        author: r.user.login,
        state: match r.state {
          IssueState::Open => "open".to_owned(),
          IssueState::Closed => "closed".to_owned(),
          _ => "unknown".to_owned(),
        },
        owner: options.get("owner").unwrap_or(&"".to_owned()).clone(),
        repo: options.get("repo").unwrap_or(&"".to_owned()).clone(),
      })
      .flat_map(|r| {
        let mut bytes = serde_json::to_vec(&r).unwrap();
        bytes.push(b'\n');
        bytes
      })
      .collect();

    Ok(Cursor::new(pulls))
  }
}
