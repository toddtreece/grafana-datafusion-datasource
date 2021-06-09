# Grafana Rust Data Source

This is the result of an experiment that uses [arrow-datafusion](https://github.com/apache/arrow-datafusion) as a datasource.

You will need the following dependencies installed:
* Docker
* node v14
* yarn

To install the rest of the dependencies, you can run:
```
make setup
```

Start grafana:
```
make start
```

## Example 

You can find an example of implementing a datasource [here](crates/datafusion-test-datasource/pkg/github/releases.rs).

![screenshot](screenshot.png)

Example query:
```sql
select
  * 
from 
  datasource.github.releases 
where 
  repo = 'grafana' 
  and owner = 'grafana'
  and tag_name like 'v8%';
```