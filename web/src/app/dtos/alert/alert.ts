export interface AlertDto {
  id: string;
  severity: string;
  identifier: string;
  value: string;
  note?: string;
  created_at: string;
  datasource_id: string;
}
