import React from 'react';
import { QueryEditorProps } from '@grafana/data';
import { CodeEditor, InfoBox } from '@grafana/ui';
import { DatafusionDatasource } from 'datasource';
import { DatafusionQuery } from 'types';

export type Props = QueryEditorProps<DatafusionDatasource, DatafusionQuery>;

export const QueryEditor = (props: Props) => {
  const { onChange, onRunQuery, query } = props;

  const handleChange = (q: string) => {
    if (q.trim()) {
      onChange({ ...query, sql: q });
      onRunQuery();
    }
  };

  return (
    <>
      <CodeEditor
        width="100%"
        height={200}
        language="sql"
        value={query.sql || ''}
        showMiniMap={false}
        showLineNumbers={true}
        onBlur={handleChange}
        onSave={handleChange}
      />
      <InfoBox>
        To save and re-run the query, press <code>ctrl + s</code> / <code>cmd + s</code>.
      </InfoBox>
    </>
  );
};
