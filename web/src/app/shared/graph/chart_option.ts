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
          size: 8,
          family: 'sans-serif',
        },
        color: '#080b05',
        maxTicksLimit: 4,
        maxRotation: 0,
        autoSkipPadding: 2,
      },
      grid: {
        color: '#71774456',
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
          size: 8,
          family: 'sans-serif',
        },
        color: '#080b05',
        maxTicksLimit: 4,
      },
      grid: {
        color: '#71774456',
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
          size: 8,
          family: 'sans-serif',
        },
        color: '#080b05', // Use app text color
      },
    },
    tooltip: {
      mode: 'index' as const,
      intersect: false,
      backgroundColor: '#eff1ed',
      titleColor: '#080b05',
      bodyColor: '#080b05',
      borderColor: '#71774456',
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
      backgroundColor: '#717744',
      borderColor: '#717744',
    },
    line: {
      borderWidth: 1, // Thinner lines for small charts
      borderColor: '#717744',
      backgroundColor: 'rgba(113, 119, 68, 0.1)',
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
          family: 'sans-serif',
        },
        color: '#080b05', // Use app text color
      },
    },
    tooltip: {
      mode: 'index' as const,
      intersect: false,
      backgroundColor: '#eff1ed',
      titleColor: '#080b05',
      bodyColor: '#080b05',
      borderColor: '#71774456',
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
  primary: '#717744',
  secondary: '#766153',
  info: '#66A9A9',
  warning: '#f59e0b',
  error: '#f44336',
  fatal: '#b71c1c',
  success: '#22c55e',
};
