import { Severity } from './alert';

export interface AggregatedAlert {
  count: number;
  identifier: string;
  severity: Severity;
  source: string;
  first: Date;
  last: Date;
}
