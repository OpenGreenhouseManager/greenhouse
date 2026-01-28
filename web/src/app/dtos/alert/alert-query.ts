export enum Severity {
  Info = 0,
  Warning = 1,
  Error = 2,
  Fatal = 3,
}
export interface AlertQuery {
  severity?: Severity | undefined;
  identifier?: string | undefined;
  created_at?: string | undefined;
  datasource_id?: string | undefined;
}
