import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import {
  ConfigResponseDto,
  DeviceOptionsResponseDto,
  DeviceResponseDto,
  DevicesResponseDto,
  DeviceStatusResponseDto,
  PostDeviceDtoRequest,
  PutDeviceDtoRequest,
} from '../../dtos/device';
import { device } from '../../urls/urls';

@Injectable({
  providedIn: 'root',
})
export class DeviceService {
  private http = inject(HttpClient);

  getAllDevices(): Observable<DevicesResponseDto> {
    return this.http.get<DevicesResponseDto>(device, {
      withCredentials: true,
    });
  }

  getDeviceById(deviceId: string): Observable<DeviceResponseDto> {
    return this.http.get<DeviceResponseDto>(`${device}/${deviceId}`, {
      withCredentials: true,
    });
  }

  addDevice(deviceData: PostDeviceDtoRequest): Observable<DeviceResponseDto> {
    return this.http.post<DeviceResponseDto>(device, deviceData, {
      withCredentials: true,
    });
  }

  updateDevice(
    deviceId: string,
    deviceData: PutDeviceDtoRequest
  ): Observable<DeviceResponseDto> {
    return this.http.put<DeviceResponseDto>(
      `${device}/${deviceId}`,
      deviceData,
      {
        withCredentials: true,
      }
    );
  }

  getDeviceStatus(deviceId: string): Observable<DeviceStatusResponseDto> {
    return this.http.get<DeviceStatusResponseDto>(
      `${device}/${deviceId}/status`,
      {
        withCredentials: true,
      }
    );
  }

  getDeviceConfig(deviceId: string): Observable<ConfigResponseDto> {
    return this.http.get<ConfigResponseDto>(`${device}/${deviceId}/config`, {
      withCredentials: true,
    });
  }

  registerDevice(deviceId: string): void {
    this.http
      .put<DeviceResponseDto>(
        `${device}/${deviceId}/activate`,
        {},
        { withCredentials: true }
      )
      .subscribe({
        next: () => {},
        error: error => {
          console.error(error);
        },
      });
  }

  updateDeviceAdditionalConfig(
    deviceId: string,
    additionalConfig: unknown
  ): Observable<string> {
    return this.http.put<string>(
      `${device}/${deviceId}/config`,
      { additional_config: additionalConfig },
      {
        withCredentials: true,
        responseType: 'text' as unknown as 'json',
      }
    );
  }

  getDeviceOptions(deviceId: string): Observable<DeviceOptionsResponseDto> {
    return this.http.get<DeviceOptionsResponseDto>(
      `${device}/${deviceId}/options`,
      {
        withCredentials: true,
      }
    );
  }
}
