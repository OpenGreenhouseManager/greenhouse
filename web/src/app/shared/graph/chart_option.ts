export const baseChartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  resizeDelay: 0, // Immediate resize
  devicePixelRatio: 1, // Prevent high DPI scaling issues
  scales: {
    x: {
      display: true,
      ticks: {
        font: {
          size: 8,
          family: 'sans-serif',
        },
        color: '#080b05',
        maxTicksLimit: 5,
      },
      grid: {
        color: '#71774456',
        drawBorder: false,
      },
    },
    y: {
      display: true,
      ticks: {
        font: {
          size: 8,
          family: 'sans-serif',
        },
        color: '#080b05',
        maxTicksLimit: 5,
      },
      grid: {
        color: '#71774456',
        drawBorder: false,
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

  elements: {
    point: {
      radius: 2, // Smaller points for tight spaces
      hoverRadius: 4,
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
  info: '#0ea5e9',
  warning: '#f97316',
  error: '#f44336',
  fatal: '#b71c1c',
  success: '#22c55e',
};
