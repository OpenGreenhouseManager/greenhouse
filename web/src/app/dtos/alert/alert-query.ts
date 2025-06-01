export enum Severity {
  Info = 0,
  Warning = 1,
  Error = 2,
  Fatal = 3,
}
export interface AlertQuery {
  severity?: Severity;
  identifier?: string;
  created_at?: string;
  datasource_id?: string;
}
