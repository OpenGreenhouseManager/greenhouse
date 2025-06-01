import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { AggregatedAlert } from '../models/aggregated-alert';
import { map, Observable } from 'rxjs';
import { AlertAggrigatedDto } from '../../dtos/alert/aggrigated-alert';
import { IntervalQuery } from '../../dtos/alert/interval-query';
import { alert } from '../../urls/urls';
import { AlertQuery } from '../../dtos/alert/alert-query';
import { AlertDto } from '../../dtos/alert/alert';
import { Alert, Severity } from '../models/alert';

@Injectable({
  providedIn: 'root',
})
export class AlertService {
  constructor(private http: HttpClient) {}

  getAlerts(start: Date, end: Date): Observable<AggregatedAlert[]> {
    const query: IntervalQuery = {
      start: start.toISOString(),
      end: end.toISOString(),
    };

    return this.http
      .get<AlertAggrigatedDto[]>(alert, {
        withCredentials: true,
        params: query as HttpParams,
      })
      .pipe(
        map(x => {
          const a: AggregatedAlert[] = [];
          for (let i = 0; i < x.length; i++) {
            const alert = x[i];
            a.push({
              count: alert.count,
              identifier: alert.identifier,
              severity: this.convertSeverity(alert.severity),
              source: alert.source,
              first: new Date(alert.first),
              last: new Date(alert.last),
            });
          }
          return a;
        })
      );
  }

  queryAlerts(query: AlertQuery): Observable<Alert[]> {
    return this.http
      .get<AlertDto[]>(alert + '/filter', {
        withCredentials: true,
        params: query as HttpParams,
      })
      .pipe(
        map(x => {
          const a: Alert[] = [];
          for (let i = 0; i < x.length; i++) {
            const alert = x[i];
            a.push({
              id: alert.id,
              severity: this.convertSeverity(alert.severity),
              identifier: alert.identifier,
              value: alert.value,
              note: alert.note,
              created_at: new Date(alert.created_at),
              datasource_id: alert.datasource_id,
            });
          }
          return a;
        })
      );
  }

  convertSeverity(severity: string): Severity {
    switch (severity) {
      case 'Info':
        return Severity.Info;
      case 'Warning':
        return Severity.Warning;
      case 'Error':
        return Severity.Error;
      case 'Fatal':
        return Severity.Fatal;
      default:
        throw new Error(`Unknown severity: ${severity}`);
    }
  }
}
