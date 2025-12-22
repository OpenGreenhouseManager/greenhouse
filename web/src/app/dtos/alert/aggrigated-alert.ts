export interface AlertAggrigatedDto {
  count: number;
  identifier: string;
  severity: string;
  source: string;
  first: string;
  last: string;
}

export interface AlertsAggrigatedDto {
  alerts: AlertAggrigatedDto[];
}
