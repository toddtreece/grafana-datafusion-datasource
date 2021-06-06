import { DataSourceInstanceSettings } from "@grafana/data";
import { DataSourceWithBackend } from "@grafana/runtime";
import { DataQueryRequest, DataQueryResponse } from "@grafana/data";
import { Observable } from "rxjs";
import { DatafusionQuery } from "types";

export class DatafusionDatasource extends DataSourceWithBackend<DatafusionQuery> {
  constructor(instanceSettings: DataSourceInstanceSettings) {
    super(instanceSettings);
  }

  query(
    request: DataQueryRequest<DatafusionQuery>
  ): Observable<DataQueryResponse> {
    return super.query(request);
  }
}
