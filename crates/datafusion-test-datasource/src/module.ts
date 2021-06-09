import { DataSourcePlugin } from '@grafana/data';
import { QueryEditor } from 'components/QueryEditor';
import { DatafusionDatasource } from './datasource';

export const plugin = new DataSourcePlugin<DatafusionDatasource>(DatafusionDatasource).setQueryEditor(QueryEditor);
