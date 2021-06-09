import { DataQuery } from '@grafana/data';

export interface DatafusionQuery extends DataQuery {
  sql: string;
}
