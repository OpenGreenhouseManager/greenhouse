import { HttpClient } from '@angular/common/http';
import { inject, Service } from '@angular/core';
import type { Observable } from 'rxjs';
import type {
  BatchTimeseriesResponseDto,
  DeviceTimeseriesQueryDto,
} from '../../dtos/timeseries';
import { device } from '../../urls/urls';

@Service()
export class GraphService {
  private http = inject(HttpClient);

  // Results come back in the same order as the queries
  getBatchTimeseries(
    queries: DeviceTimeseriesQueryDto[]
  ): Observable<BatchTimeseriesResponseDto> {
    return this.http.post<BatchTimeseriesResponseDto>(
      `${device}/timeseries/batch`,
      { queries },
      { withCredentials: true }
    );
  }
}
