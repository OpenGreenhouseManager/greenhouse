import { BarChart, LineChart } from 'echarts/charts';
import {
  GridComponent,
  LegendComponent,
  TooltipComponent,
} from 'echarts/components';
import * as echarts from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { chartColors, chartTheme } from './chart_option';

// Only register what the app uses so the bundle stays small.
echarts.use([
  LineChart,
  BarChart,
  GridComponent,
  LegendComponent,
  TooltipComponent,
  CanvasRenderer,
]);

const axis = {
  axisLine: { show: false },
  axisTick: { show: false },
  axisLabel: { color: chartTheme.muted, fontSize: 10 },
  nameTextStyle: { color: chartTheme.muted, fontSize: 10 },
  splitLine: {
    lineStyle: { color: chartTheme.grid, type: 'dashed' },
  },
};

echarts.registerTheme('greenhouse', {
  color: [
    chartColors.primary,
    chartColors.secondary,
    chartColors.info,
    chartColors.warning,
    chartColors.success,
    chartColors.error,
    chartColors.fatal,
  ],
  backgroundColor: 'transparent',
  textStyle: { fontFamily: chartTheme.font, color: chartTheme.text },
  categoryAxis: { ...axis, splitLine: { show: false } },
  valueAxis: axis,
  timeAxis: { ...axis, splitLine: { show: false } },
  legend: {
    textStyle: { color: chartTheme.text, fontSize: 10 },
    itemWidth: 10,
    itemHeight: 6,
    icon: 'roundRect',
  },
  tooltip: {
    backgroundColor: '#ffffff',
    borderColor: chartTheme.grid,
    borderWidth: 1,
    padding: [6, 8],
    textStyle: { color: chartTheme.text, fontSize: 11 },
    extraCssText:
      'border-radius: 6px; box-shadow: 0 4px 12px rgba(36, 48, 31, 0.08);',
  },
});

export { echarts };
