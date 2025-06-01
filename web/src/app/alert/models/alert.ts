export enum Severity {
  Info = 0,
  Warning = 1,
  Error = 2,
  Fatal = 3,
}

export interface Alert {
  id: string;
  severity: Severity;
  identifier: string;
  value: string;
  note?: string;
  created_at: Date;
  datasource_id: string;
}
