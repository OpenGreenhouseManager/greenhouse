export interface TimeseriesDto {
  timestamp: number;
  value: Type;
}

export type Type =
  | { Number: number }
  | { Boolean: boolean }
  | { Object: Record<string, Type> }
  | { Array: Type[] };

export interface TimeseriesResponseDto {
  timeseries: TimeseriesDto[];
}
