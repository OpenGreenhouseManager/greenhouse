import { HttpClient, HttpParams } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { TimeseriesResponseDto } from '../../dtos/timeseries';
import { device } from '../../urls/urls';

export interface TimeseriesQuery {
  start: Date;
  end: Date;
  sub_property?: string;
  step?: string;
}

@Injectable({
  providedIn: 'root',
})
export class GraphService {
  private http = inject(HttpClient);

  getTimeseries(
    deviceId: string,
    query: TimeseriesQuery
  ): Observable<TimeseriesResponseDto> {
    let params = new HttpParams()
      .set('start', query.start.toISOString())
      .set('end', query.end.toISOString());
    if (query.sub_property) {
      params = params.set('sub_property', query.sub_property);
    }
    if (query.step) {
      params = params.set('step', query.step);
    }

    return this.http.get<TimeseriesResponseDto>(
      `${device}/${deviceId}/timeseries`,
      {
        withCredentials: true,
        params,
      }
    );
  }
}
