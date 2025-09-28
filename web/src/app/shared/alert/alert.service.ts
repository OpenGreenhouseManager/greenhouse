import { HttpClient, HttpParams } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { AlertDto } from '../../dtos/alert/alert';
import { alert } from '../../urls/urls';

export interface AlertQuery {
  severity?: string;
  identifier?: string;
  created_at?: Date;
  datasource_id?: string;
}

@Injectable({
  providedIn: 'root',
})
export class AlertService {
  private http = inject(HttpClient);

  getAlerts(query: AlertQuery): Observable<AlertDto[]> {
    let params = new HttpParams();
    if (query.severity) {
      params = params.set('severity', query.severity);
    }
    if (query.identifier) {
      params = params.set('identifier', query.identifier);
    }
    if (query.created_at) {
      params = params.set('created_at', query.created_at.toISOString());
    }
    if (query.datasource_id) {
      params = params.set('datasource_id', query.datasource_id);
    }
    return this.http.get<AlertDto[]>(`${alert}/filter`, {
      withCredentials: true,
      params,
    });
  }
}
