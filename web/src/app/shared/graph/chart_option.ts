export const baseChartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  resizeDelay: 0, // Immediate resize
  devicePixelRatio: 1, // Prevent high DPI scaling issues
  layout: {
    padding: 0,
  },
  scales: {
    x: {
      display: true,
      ticks: {
        padding: 2,
        font: {
          size: 10,
          family: "'Nunito Variable', sans-serif",
        },
        color: '#66705f',
        maxTicksLimit: 4,
        maxRotation: 0,
        autoSkipPadding: 2,
      },
      grid: {
        color: '#e3e6dc',
        drawBorder: false,
        lineWidth: 0.5,
      },
      border: {
        display: false,
      },
    },
    y: {
      display: true,
      ticks: {
        padding: 2,
        font: {
          size: 10,
          family: "'Nunito Variable', sans-serif",
        },
        color: '#66705f',
        maxTicksLimit: 4,
      },
      grid: {
        color: '#e3e6dc',
        drawBorder: false,
        lineWidth: 0.5,
      },
      border: {
        display: false,
      },
    },
  },
};

// Line chart specific options
export const chartOptions = {
  ...baseChartOptions,
  plugins: {
    legend: {
      display: false,
      position: 'top' as const,
      labels: {
        boxWidth: 8,
        padding: 4,
        font: {
          size: 10,
          family: "'Nunito Variable', sans-serif",
        },
        color: '#24301f', // Use app text color
      },
    },
    tooltip: {
      mode: 'index' as const,
      intersect: false,
      backgroundColor: '#ffffff',
      titleColor: '#24301f',
      bodyColor: '#24301f',
      borderColor: '#e3e6dc',
      borderWidth: 1,
      cornerRadius: 6,
      padding: 6,
      titleMarginBottom: 4,
      displayColors: false,
    },
  },

  elements: {
    point: {
      radius: 1, // Smaller points for tight spaces
      hoverRadius: 3,
      backgroundColor: '#4f7a4a',
      borderColor: '#4f7a4a',
    },
    line: {
      borderWidth: 1.5, // Thin lines for small charts
      borderColor: '#4f7a4a',
      backgroundColor: 'rgba(79, 122, 74, 0.12)',
    },
  },
};

// Bar chart specific options
export const barChartOptions = {
  ...baseChartOptions,
  plugins: {
    legend: {
      display: false,
      position: 'top' as const,
      labels: {
        boxWidth: 12,
        padding: 10,
        font: {
          size: 10,
          family: "'Nunito Variable', sans-serif",
        },
        color: '#24301f', // Use app text color
      },
    },
    tooltip: {
      mode: 'index' as const,
      intersect: false,
      backgroundColor: '#ffffff',
      titleColor: '#24301f',
      bodyColor: '#24301f',
      borderColor: '#e3e6dc',
      borderWidth: 1,
      cornerRadius: 6,
    },
  },

  aspectRatio: 0.8,
  scales: {
    ...baseChartOptions.scales,
    x: {
      ...baseChartOptions.scales.x,
      stacked: true,
    },
    y: {
      ...baseChartOptions.scales.y,
      stacked: true,
    },
  },
};

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
