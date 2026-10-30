export interface DeviceResponseDto {
  id: string;
  name: string;
  address: string;
  description: string;
  canscript: boolean;
  status?: DeviceStatusDto | undefined;
  scraping: boolean;
}

export interface DevicesResponseDto {
  devices: DeviceResponseDto[];
}

export interface PutDeviceDtoRequest {
  name: string;
  description: string;
  address: string;
  can_script: boolean;
}

export interface PostDeviceDtoRequest {
  name: string;
  description: string;
  address: string;
  can_script: boolean;
  scraping: boolean;
}

export enum DeviceStatusDto {
  Offline = 'Offline',
  Online = 'Online',
  Panic = 'Panic',
}

export interface DeviceStatusResponseDto {
  status: DeviceStatusDto;
  datasource_id: string;
}

export enum Mode {
  Input = 'Input',
  Output = 'Output',
  InputOutput = 'InputOutput',
  Unknown = 'Unknown',
}

export interface ConfigResponseDto<T = unknown> {
  mode: Mode;
  input_type?: Type;
  output_type?: Type;
  scripting_api?: ScriptingApi | undefined;
  additional_config: T;
}

export enum Type {
  Number = 'Number',
  String = 'String',
  Stream = 'Stream',
}

export class ScriptingApi {
  url!: string;
  token!: string;
}

export interface DeviceOptionsResponseDto {
  operations: string[];
}
