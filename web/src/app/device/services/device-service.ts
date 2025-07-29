import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import {
  DeviceResponseDto,
  DeviceStatusResponseDto,
  ConfigResponseDto,
  PostDeviceDtoRequest,
  PutDeviceDtoRequest,
} from '../../dtos/device';
import { device } from '../../urls/urls';

@Injectable({
  providedIn: 'root',
})
export class DeviceService {
  constructor(private http: HttpClient) {}

  getAllDevices(): Observable<DeviceResponseDto[]> {
    return this.http.get<DeviceResponseDto[]>(device, {
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
}
