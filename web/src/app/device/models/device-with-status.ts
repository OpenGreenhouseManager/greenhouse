import type { DeviceResponseDto } from '../../dtos/generated/device_service/DeviceResponseDto';
import type { DeviceStatusDto } from '../../dtos/generated/smart_device/DeviceStatusDto';

/** `Offline` is client-side only: the status request failed. */
export type DeviceStatus = DeviceStatusDto | 'Offline';

/** A device enriched with its status, which is fetched separately. */
export type DeviceWithStatus = DeviceResponseDto & {
  status?: DeviceStatus | undefined;
};
