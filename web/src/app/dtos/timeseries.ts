export interface TimeseriesDto {
  timestamp: number;
  value: Type;
}

export type Type =
  | { Number: number }
  | { Boolean: boolean }
  | { Object: Record<string, Type> }
  | { Array: Type[] }
  | { Measurement: { value: number; unit: string } };

export interface TimeseriesResponseDto {
  timeseries: TimeseriesDto[];
}

export interface DeviceTimeseriesQueryDto {
  device_id: string;
  start: string;
  end: string;
  step: string;
  sub_property?: string | undefined;
}

export interface BatchTimeseriesRequestDto {
  queries: DeviceTimeseriesQueryDto[];
}

export interface BatchTimeseriesResultDto {
  query: DeviceTimeseriesQueryDto;
  timeseries?: TimeseriesDto[];
  error?: string;
}

export interface BatchTimeseriesResponseDto {
  results: BatchTimeseriesResultDto[];
}
