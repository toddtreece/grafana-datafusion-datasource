use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use std::sync::Arc;

use anyhow::Result;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use async_trait::async_trait;
use grafana_plugin_sdk::DataSource;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct Release {
  created_at: i64,
  published_at: i64,
  name: Option<String>,
  body: Option<String>,
  url: String,
  tag_name: String,
  draft: bool,
  pre_release: bool,
  owner: String,
  repo: String,
  author: String,
}

#[derive(Debug, Clone)]
pub struct ReleaseTable;

#[async_trait]
impl DataSource for ReleaseTable {
  type Data = Cursor<Vec<u8>>;

  fn schema(&self) -> SchemaRef {
    let mut metadata = BTreeMap::new();
    metadata.insert("filter".to_owned(), "true".to_owned());

    let created_at = Field::new(
      "created_at",
      DataType::Timestamp(TimeUnit::Second, None),
      false,
    );
    let published_at = Field::new(
      "published_at",
      DataType::Timestamp(TimeUnit::Second, None),
      false,
    );
    let name = Field::new("name", DataType::Utf8, true);
    let body = Field::new("body", DataType::Utf8, true);
    let url = Field::new("url", DataType::Utf8, false);
    let tag_name = Field::new("tag_name", DataType::Utf8, false);
    let draft = Field::new("draft", DataType::Boolean, false);
    let pre_release = Field::new("pre_release", DataType::Boolean, false);
    let author = Field::new("author", DataType::Utf8, false);

    // these fields are passed to the github api as filters
    let mut owner = Field::new("owner", DataType::Utf8, false);
    owner.set_metadata(Some(metadata.clone()));
    let mut repo = Field::new("repo", DataType::Utf8, false);
    repo.set_metadata(Some(metadata.clone()));

    let schema = Schema::new(vec![
      created_at,
      published_at,
      name,
      body,
      url,
      tag_name,
      draft,
      pre_release,
      author,
      owner,
      repo,
    ]);

    Arc::new(schema)
  }

  async fn fetch_results(&self, options: HashMap<String, String>) -> Result<Self::Data> {
    let octocrab = octocrab::instance();
    let releases = octocrab
      .repos(
        options.get("owner").unwrap_or(&"".to_owned()),
        options.get("repo").unwrap_or(&"".to_owned()),
      )
      .releases()
      .list()
      .send()
      .await?;

    let results: Vec<u8> = releases
      .items
      .into_iter()
      .map(|r| Release {
        created_at: r.created_at.timestamp(),
        published_at: r.published_at.timestamp(),
        name: r.name,
        body: r.body,
        url: r.url.to_string(),
        tag_name: r.tag_name,
        draft: r.draft,
        pre_release: r.prerelease,
        owner: options.get("owner").unwrap_or(&"".to_owned()).clone(),
        repo: options.get("repo").unwrap_or(&"".to_owned()).clone(),
        author: r.author.login,
      })
      .flat_map(|r| {
        let mut bytes = serde_json::to_vec(&r).unwrap();
        bytes.push(b'\n');
        bytes
      })
      .collect();

    Ok(Cursor::new(results))
  }
}
