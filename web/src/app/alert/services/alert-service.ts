import type { HttpParams } from '@angular/common/http';
import { HttpClient } from '@angular/common/http';
import { inject, Service } from '@angular/core';
import type { Observable } from 'rxjs';
import { map } from 'rxjs';
import type { AggrigatedAlertsDto } from '../../dtos/generated/data_storage_service/alert/AggrigatedAlertsDto';
import type { AlertQuery } from '../../dtos/generated/data_storage_service/alert/AlertQuery';
import type { AlertsDto } from '../../dtos/generated/data_storage_service/alert/AlertsDto';
import type { IntervalQuery } from '../../dtos/generated/data_storage_service/alert/IntervalQuery';
import type { Severity as SeverityDto } from '../../dtos/generated/data_storage_service/alert/Severity';
import { alert } from '../../urls/urls';
import type { AggregatedAlert } from '../models/aggregated-alert';
import type { Alert } from '../models/alert';
import { Severity } from '../models/alert';

@Service()
export class AlertService {
  private http = inject(HttpClient);

  getAlerts(start: Date, end: Date): Observable<AggregatedAlert[]> {
    const query: IntervalQuery = {
      start: start.toISOString(),
      end: end.toISOString(),
    };

    return this.http
      .get<AggrigatedAlertsDto>(alert, {
        withCredentials: true,
        params: query as HttpParams,
      })
      .pipe(
        map(x => {
          const a: AggregatedAlert[] = [];
          for (let i = 0; i < x.alerts.length; i++) {
            const alert = x.alerts[i];
            if (!alert) {
              continue;
            }
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
      .get<AlertsDto>(`${alert}/filter`, {
        withCredentials: true,
        params: query as HttpParams,
      })
      .pipe(
        map(x => {
          const a: Alert[] = [];
          for (let i = 0; i < x.alerts.length; i++) {
            const alert = x.alerts[i];
            if (!alert) {
              continue;
            }
            a.push({
              id: alert.id,
              severity: this.convertSeverity(alert.severity),
              identifier: alert.identifier,
              value: alert.value,
              note: alert.note ?? undefined,
              created_at: new Date(alert.created_at),
              datasource_id: alert.datasource_id,
            });
          }
          return a;
        })
      );
  }

  convertSeverity(severity: SeverityDto): Severity {
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
