import { DataSourceInstanceSettings, DataQueryRequest, DataQueryResponse, ScopedVars } from '@grafana/data';
import { DataSourceWithBackend, getTemplateSrv } from '@grafana/runtime';
import { Observable } from 'rxjs';
import { DatafusionQuery } from 'types';
export class DatafusionDatasource extends DataSourceWithBackend<DatafusionQuery> {
  constructor(instanceSettings: DataSourceInstanceSettings) {
    super(instanceSettings);
  }

  applyTemplateVariables(query: DatafusionQuery, scoped: ScopedVars): DatafusionQuery {
    if (query.sql) {
      return { ...query, sql: getTemplateSrv().replace(query.sql, scoped) };
    }
    return query;
  }

  query(request: DataQueryRequest<DatafusionQuery>): Observable<DataQueryResponse> {
    return super.query(request);
  }
}
