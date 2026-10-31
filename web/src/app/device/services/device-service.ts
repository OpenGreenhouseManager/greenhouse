import { HttpClient } from '@angular/common/http';
import { inject, Service } from '@angular/core';
import type { Observable } from 'rxjs';
import type { DeviceResponseDto } from '../../dtos/generated/device_service/DeviceResponseDto';
import type { DevicesResponseDto } from '../../dtos/generated/device_service/DevicesResponseDto';
import type { OperationsDto } from '../../dtos/generated/device_service/OperationsDto';
import type { PostDeviceDtoRequest } from '../../dtos/generated/device_service/PostDeviceDtoRequest';
import type { PutDeviceDtoRequest } from '../../dtos/generated/device_service/PutDeviceDtoRequest';
import type { ConfigResponseDto } from '../../dtos/generated/smart_device/ConfigResponseDto';
import type { DeviceStatusResponseDto } from '../../dtos/generated/smart_device/DeviceStatusResponseDto';
import { device } from '../../urls/urls';

@Service()
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

  getDeviceConfig(deviceId: string): Observable<ConfigResponseDto<unknown>> {
    return this.http.get<ConfigResponseDto<unknown>>(
      `${device}/${deviceId}/config`,
      {
        withCredentials: true,
      }
    );
  }

  registerDevice(deviceId: string): Observable<DeviceResponseDto> {
    return this.http.put<DeviceResponseDto>(
      `${device}/${deviceId}/activate`,
      {},
      { withCredentials: true }
    );
  }

  updateDeviceAdditionalConfig(
    deviceId: string,
    additionalConfig: unknown
  ): Observable<string> {
    return this.http.put(
      `${device}/${deviceId}/config`,
      { additional_config: additionalConfig },
      {
        withCredentials: true,
        responseType: 'text',
      }
    );
  }

  getDeviceOptions(deviceId: string): Observable<OperationsDto> {
    return this.http.get<OperationsDto>(`${device}/${deviceId}/options`, {
      withCredentials: true,
    });
  }
}
