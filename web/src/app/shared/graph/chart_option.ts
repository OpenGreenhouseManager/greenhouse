import type { BarSeriesOption, LineSeriesOption } from 'echarts/charts';
import type {
  GridComponentOption,
  LegendComponentOption,
  TooltipComponentOption,
} from 'echarts/components';
import type { ComposeOption } from 'echarts/core';

export type LineChartOption = ComposeOption<
  | LineSeriesOption
  | GridComponentOption
  | LegendComponentOption
  | TooltipComponentOption
>;

export type BarChartOption = ComposeOption<
  | BarSeriesOption
  | GridComponentOption
  | LegendComponentOption
  | TooltipComponentOption
>;

// Default color palette for charts
export const chartColors = {
  primary: '#4f7a4a',
  secondary: '#c75b7a',
  info: '#4f8a9a',
  warning: '#d99a2b',
  error: '#c2483d',
  fatal: '#8f2d26',
  success: '#5a9a4f',
};

export const chartTheme = {
  font: "'Nunito Variable', sans-serif",
  text: '#24301f',
  muted: '#66705f',
  grid: '#e3e6dc',
};

export interface LineSeries {
  name: string;
  unit: string;
  color: string;
  // [unix ms, value]; null leaves a gap in the line
  data: [number, number | null][];
}

export interface BarSeries {
  name: string;
  color: string;
  data: number[];
}

// Round numbers to at most two decimal places
export function formatNumber(val: unknown): string {
  if (val === null || val === undefined) return '';
  const num =
    typeof val === 'number' ? val : typeof val === 'string' ? Number(val) : NaN;
  if (!Number.isFinite(num)) return String(val);
  return new Intl.NumberFormat(undefined, {
    maximumFractionDigits: 2,
  }).format(num);
}

function withAlpha(hex: string, alpha: number): string {
  const a = Math.round(alpha * 255)
    .toString(16)
    .padStart(2, '0');
  return `${hex}${a}`;
}

function valueAxis(unit: string, position: 'left' | 'right') {
  return {
    type: 'value' as const,
    position,
    scale: true,
    name: unit,
    nameGap: 8,
    splitNumber: 3,
    // the right axis shares the left axis' grid lines
    ...(position === 'right' && { splitLine: { show: false } }),
    axisLabel: { formatter: (v: number) => formatNumber(v) },
  };
}

export function lineChartOptions(
  series: LineSeries[],
  separateAxes: boolean
): LineChartOption {
  const units = new Set(series.map(s => s.unit).filter(u => !!u));
  const yAxis = separateAxes
    ? [
        valueAxis(series[0]?.unit ?? '', 'left'),
        valueAxis(series[1]?.unit ?? '', 'right'),
      ]
    : // merged axis: only show unit if all defined units are identical
      [valueAxis(units.size === 1 ? ([...units][0] ?? '') : '', 'left')];
  const showLegend = series.length > 1;

  return {
    animationDuration: 600,
    grid: {
      left: 4,
      right: 4,
      top: showLegend ? 32 : 20,
      bottom: 4,
    },
    legend: { show: showLegend, top: 0, left: 'center' },
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'line',
        lineStyle: { color: chartTheme.muted, type: 'dashed', width: 1 },
      },
    },
    xAxis: {
      type: 'time',
      splitNumber: 3,
      axisLabel: { hideOverlap: true },
    },
    yAxis,
    series: series.map((s, idx) => ({
      type: 'line',
      name: s.name,
      data: s.data,
      yAxisIndex: separateAxes && idx > 0 ? 1 : 0,
      smooth: 0.3,
      showSymbol: false,
      symbolSize: 6,
      color: s.color,
      lineStyle: { width: 2 },
      areaStyle: {
        color: {
          type: 'linear',
          x: 0,
          y: 0,
          x2: 0,
          y2: 1,
          colorStops: [
            { offset: 0, color: withAlpha(s.color, 0.28) },
            { offset: 1, color: withAlpha(s.color, 0) },
          ],
        },
      },
      emphasis: { focus: 'series' },
      tooltip: {
        valueFormatter: v => `${formatNumber(v)}${s.unit ? ` ${s.unit}` : ''}`,
      },
    })),
  };
}

export function barChartOptions(
  categories: string[],
  series: BarSeries[]
): BarChartOption {
  return {
    animationDuration: 600,
    grid: { left: 4, right: 4, top: 28, bottom: 4 },
    legend: { top: 0, left: 'center' },
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    xAxis: { type: 'category', data: categories },
    yAxis: {
      type: 'value',
      minInterval: 1,
      splitNumber: 3,
    },
    series: series.map(s => ({
      type: 'bar',
      name: s.name,
      data: s.data,
      stack: 'alerts',
      color: s.color,
      barMaxWidth: 18,
      itemStyle: { borderRadius: 3, borderColor: '#ffffff', borderWidth: 1 },
      emphasis: { focus: 'series' },
    })),
  };
}
