import type { ArrowResponse, PlotlyTrace } from './types';

type PlotlyElement = HTMLElement & {
  _fullLayout?: unknown;
  data?: Array<Record<string, any>>;
  removeAllListeners?: (event: string) => void;
  on?: (event: string, handler: (...args: any[]) => void) => void;
};

// Cache for chart configurations to avoid recreating them
const CHART_CONFIGS: Map<string, Record<string, unknown>> = new Map();

function getChartConfig(chartId: string, layout: Record<string, unknown>) {
  const cacheKey = `${chartId}-${JSON.stringify(layout)}`;

  if (!CHART_CONFIGS.has(cacheKey)) {
    CHART_CONFIGS.set(cacheKey, {
      displayModeBar: false,
      staticPlot: false,
      responsive: true,
      webGlRenderer: true
    });
  }

  return CHART_CONFIGS.get(cacheKey)!;
}

function showError(chartId: string, message: string): void {
  const errorElement = document.getElementById(chartId + 'Error');
  if (errorElement) {
    errorElement.style.display = 'block';
    errorElement.textContent = message;
  }
}

function hideError(chartId: string): void {
  const errorElement = document.getElementById(chartId + 'Error');
  if (errorElement) {
    errorElement.style.display = 'none';
  }
}

// Use requestAnimationFrame for chart updates to optimize rendering
const chartUpdateFrames: Map<string, number> = new Map();

function createPlot(chartId: string, traces: PlotlyTrace[], layout: Record<string, unknown>, errorMessage = 'No data available'): boolean {
  if (
    !traces ||
    traces.length === 0 ||
    (traces[0].x && traces[0].x.length === 0) ||
    (traces[0].values && traces[0].values.length === 0)
  ) {
    showError(chartId, errorMessage);
    hideChartSkeleton(chartId);
    return false;
  }

  const plotly = window.Plotly;
  if (!plotly) {
    showError(chartId, 'Plotly not loaded');
    hideChartSkeleton(chartId);
    return false;
  }

  hideError(chartId);

  // Cancel any pending chart update for this chart
  if (chartUpdateFrames.has(chartId)) {
    cancelAnimationFrame(chartUpdateFrames.get(chartId)!);
  }

  // Use requestAnimationFrame for smooth rendering
  const frameId = requestAnimationFrame(() => {
    // Use cached config for better performance
    const config = getChartConfig(chartId, layout);

    try {
      // Use Plotly.react for efficient updates (reuses existing DOM)
      plotly.react(chartId, traces, layout, config);
      // Hide skeleton after successful chart render
      hideChartSkeleton(chartId);
      chartUpdateFrames.delete(chartId);
    } catch (error) {
      console.error('Error creating plot:', error);
      showError(chartId, 'Failed to render chart');
      hideChartSkeleton(chartId);
      chartUpdateFrames.delete(chartId);
    }
  });

  chartUpdateFrames.set(chartId, frameId);
  return true;
}

function hideChartSkeleton(chartId: string): void {
  // Find skeleton containers related to this chart
  const skeletons = document.querySelectorAll(`.skeleton-container[data-chart="${chartId}"], #${chartId}-skeleton`);
  const contents = document.querySelectorAll(`.content-container[data-chart="${chartId}"], #${chartId}-content`);

  skeletons.forEach(skeleton => {
    skeleton.classList.add('loaded');
  });

  contents.forEach(content => {
    content.classList.add('loaded');
  });

  // Also hide any generic chart skeletons near the chart element
  const chartElement = document.getElementById(chartId);
  if (chartElement) {
    const nearbySkeletons = chartElement.parentElement?.querySelectorAll('.skeleton-chart, .skeleton-container');
    nearbySkeletons?.forEach(skeleton => {
      skeleton.classList.add('loaded');
      (skeleton as HTMLElement).style.opacity = '0';
      (skeleton as HTMLElement).style.pointerEvents = 'none';
    });
  }
}

// Global state for chart interactions
const selectedDataPoints = new Set<number>();
let brushSelection: unknown = null;
const chartFilters: Map<string, Array<{ x: number; y: number; pointIndex: number; traceIndex: number }>> = new Map();

// Chart control functions for modern UI
function changeBins(element: HTMLElement, chartId: string): void {
  const bins = parseInt(element.getAttribute('data-bins') || '0', 10);
  window.currentBinCount = bins;

  // Update active state
  element.parentElement?.querySelectorAll('.chart-option').forEach(btn => {
    btn.classList.remove('active');
  });
  element.classList.add('active');

  // Re-render chart with new bins
  window.updateCharts();
}

function switchRankings(element: HTMLElement): void {
  const type = element.getAttribute('data-type');

  // Update active state
  element.parentElement?.querySelectorAll('.chart-option').forEach(btn => {
    btn.classList.remove('active');
  });
  element.classList.add('active');

  // Update rankings table based on type
  updateRankingsTable(type || '');
}

function toggleTrendline(chartId: string): void {
  // Implement trendline toggle
  console.log('Toggle trendline for', chartId);
}

function togglePoints(chartId: string): void {
  // Implement points toggle
  console.log('Toggle points for', chartId);
}

function updateRankingsTable(type: string): void {
  // Implementation for updating rankings table
  console.log('Update rankings table with type:', type);
}

// Crossfilter-style chart linking functionality
let crossfilteringSetupAttempts = 0;
const MAX_CROSSFILTERING_ATTEMPTS = 10; // Maximum 10 seconds of retries

function setupChartCrossfiltering(): void {
  const chartIds = ['weightDistribution', 'dotsDistribution', 'bodyweightScatter', 'dotsScatter'];
  let allChartsReady = true;
  let readyCount = 0;

  chartIds.forEach(chartId => {
    const chartElement = document.getElementById(chartId) as PlotlyElement | null;
    if (!chartElement || !chartElement._fullLayout) {
      allChartsReady = false;
      return;
    }
    readyCount++;

    // Add selection event handlers using Plotly's event system
    try {
      // Remove existing listeners to avoid duplicates
      chartElement.removeAllListeners && chartElement.removeAllListeners('plotly_selected');
      chartElement.removeAllListeners && chartElement.removeAllListeners('plotly_deselect');
      chartElement.removeAllListeners && chartElement.removeAllListeners('plotly_relayout');

      chartElement.on?.('plotly_selected', function (eventData: any) {
        handleChartSelection(chartId, eventData);
      });

      chartElement.on?.('plotly_deselect', function () {
        clearChartSelection(chartId);
      });

      // Add brush selection for histograms
      if (chartId.includes('Distribution')) {
        chartElement.on?.('plotly_relayout', function (eventData: any) {
          handleHistogramBrush(chartId, eventData);
        });
      }

      console.log(`Crossfiltering setup complete for ${chartId}`);
    } catch (error) {
      console.error(`Failed to setup crossfiltering for ${chartId}:`, error);
    }
  });

  // Retry logic with maximum attempts
  if (!allChartsReady) {
    crossfilteringSetupAttempts++;

    if (crossfilteringSetupAttempts < MAX_CROSSFILTERING_ATTEMPTS) {
      console.log(
        `Charts not ready for crossfiltering (${readyCount}/${chartIds.length} ready), retrying... (attempt ${crossfilteringSetupAttempts}/${MAX_CROSSFILTERING_ATTEMPTS})`
      );
      setTimeout(() => setupChartCrossfiltering(), 1000);
    } else {
      console.warn(
        `ƒsÿ‹÷? Crossfiltering setup timed out after ${MAX_CROSSFILTERING_ATTEMPTS} attempts. ${readyCount}/${chartIds.length} charts ready.`
      );
    }
  } else {
    console.log(`ƒo. All ${chartIds.length} charts ready for crossfiltering`);
    crossfilteringSetupAttempts = 0; // Reset for future calls
  }
}

function handleChartSelection(sourceChartId: string, eventData: any): void {
  if (!eventData || !eventData.points) return;

  // Extract selected data points
  const selectedPoints = eventData.points.map((point: any) => ({
    x: point.x as number,
    y: point.y as number,
    pointIndex: point.pointIndex as number,
    traceIndex: point.traceIndex as number
  }));

  // Store selection for this chart
  chartFilters.set(sourceChartId, selectedPoints);

  // Update other charts to highlight filtered data
  updateLinkedCharts(sourceChartId, selectedPoints);
}

function clearChartSelection(sourceChartId: string): void {
  chartFilters.delete(sourceChartId);

  // Reset all charts to show full data
  const chartIds = ['weightDistribution', 'dotsDistribution', 'bodyweightScatter', 'dotsScatter'];
  chartIds.forEach(chartId => {
    if (chartId !== sourceChartId) {
      resetChartHighlight(chartId);
    }
  });
}

function handleHistogramBrush(chartId: string, eventData: Record<string, number>): void {
  // Handle brushing on histograms for range selection
  if (eventData['xaxis.range[0]'] !== undefined && eventData['xaxis.range[1]'] !== undefined) {
    const range = {
      min: eventData['xaxis.range[0]'],
      max: eventData['xaxis.range[1]']
    };

    // Filter other charts based on this range
    applyRangeFilter(chartId, range);
  }
}

function updateLinkedCharts(sourceChartId: string, selectedPoints: Array<{ x: number; y: number }>): void {
  const chartIds = ['weightDistribution', 'dotsDistribution', 'bodyweightScatter', 'dotsScatter'];

  chartIds.forEach(chartId => {
    if (chartId !== sourceChartId) {
      highlightDataInChart(chartId, selectedPoints);
    }
  });
}

function highlightDataInChart(chartId: string, selectedPoints: Array<{ x: number; y: number }>): void {
  const chartElement = document.getElementById(chartId) as PlotlyElement | null;
  const plotly = window.Plotly;
  if (!chartElement || !chartElement.data || !plotly) return;

  // Create highlight overlay
  const updates = {
    'marker.opacity': chartElement.data.map(trace => {
      if (!trace.x || !trace.y) return trace.marker?.opacity || 1;

      return (trace.x as number[]).map((x: number, i: number) => {
        const y = (trace.y as number[])[i];
        // Check if this point matches any selected points
        const isSelected = selectedPoints.some(sp => Math.abs(sp.x - x) < 0.01 && Math.abs(sp.y - y) < 0.01);
        return isSelected ? 1 : 0.3;
      });
    })
  };

  plotly.restyle(chartId, updates);
}

function resetChartHighlight(chartId: string): void {
  const chartElement = document.getElementById(chartId) as PlotlyElement | null;
  const plotly = window.Plotly;
  if (!chartElement || !chartElement.data || !plotly) return;

  const updates = {
    'marker.opacity': chartElement.data.map(trace => {
      if (trace.type === 'histogram') return 1;
      return Array.isArray(trace.x) ? new Array(trace.x.length).fill(0.6) : 1;
    })
  };

  plotly.restyle(chartId, updates);
}

function applyRangeFilter(sourceChartId: string, range: { min: number; max: number }): void {
  // Apply range filter to scatter plots based on histogram selection
  const scatterCharts = ['bodyweightScatter', 'dotsScatter'];

  scatterCharts.forEach(chartId => {
    if (chartId !== sourceChartId) {
      filterScatterByRange(chartId, range, sourceChartId);
    }
  });
}

function filterScatterByRange(chartId: string, range: { min: number; max: number }, sourceChartId: string): void {
  const chartElement = document.getElementById(chartId) as PlotlyElement | null;
  const plotly = window.Plotly;
  if (!chartElement || !chartElement.data || !plotly) return;

  // Determine which axis to filter based on source chart
  let filterAxis: 'x' | 'y' = 'y'; // default to y-axis (weight/DOTS values)
  if (sourceChartId === 'bodyweightScatter' || sourceChartId === 'dotsScatter') {
    filterAxis = 'x'; // filter by bodyweight
  }

  const updates = {
    'marker.opacity': chartElement.data.map(trace => {
      if (!trace.x || !trace.y) return trace.marker?.opacity || 1;

      return (trace.x as number[]).map((x: number, i: number) => {
        const value = filterAxis === 'x' ? x : (trace.y as number[])[i];
        const inRange = value >= range.min && value <= range.max;
        return inRange ? 0.8 : 0.2;
      });
    })
  };

  plotly.restyle(chartId, updates);
}

// High-quality export functionality
async function exportChart(chartId: string, format: 'png' | 'svg' | 'jpeg' = 'png', filename: string | null = null): Promise<void> {
  const chartElement = document.getElementById(chartId);
  const plotly = window.Plotly;
  if (!chartElement || !plotly) {
    console.error(`Chart ${chartId} not found`);
    return;
  }

  const exportFilename = filename || `${chartId}_export_${new Date().toISOString().slice(0, 10)}`;

  try {
    const config = {
      format: format,
      width: 1200,
      height: 800,
      scale: 2 // High DPI for crisp exports
    };

    // Use Plotly's built-in export with high quality settings
    const imageData = await plotly.toImage(chartElement, config);

    // Create download link
    const link = document.createElement('a');
    link.download = `${exportFilename}.${format}`;
    link.href = imageData;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);

    console.log(`Exported ${chartId} as ${format}`);
  } catch (error) {
    console.error('Export failed:', error);
    alert(`Failed to export chart: ${(error as Error).message}`);
  }
}

// Export all charts as individual files
async function exportAllCharts(format: 'png' | 'svg' | 'jpeg' = 'png'): Promise<void> {
  const chartIds = ['weightDistribution', 'dotsDistribution', 'bodyweightScatter', 'dotsScatter'];
  const timestamp = new Date().toISOString().slice(0, 10);

  console.log(`Starting bulk export of all charts as ${format}...`);

  for (let i = 0; i < chartIds.length; i++) {
    const chartId = chartIds[i];
    const chartElement = document.getElementById(chartId) as PlotlyElement | null;
    if (chartElement && chartElement._fullLayout) {
      try {
        console.log(`Exporting chart ${i + 1}/4: ${chartId}`);
        await exportChart(chartId, format, `iron_insights_${chartId}_${timestamp}`);
        // Small delay between exports to prevent browser issues
        await new Promise(resolve => setTimeout(resolve, 800));
      } catch (error) {
        console.error(`Failed to export ${chartId}:`, error);
        alert(`Failed to export ${chartId}: ${(error as Error).message}`);
      }
    } else {
      console.warn(`Chart ${chartId} not ready for export`);
    }
  }

  console.log(`Bulk export complete! Downloaded ${chartIds.length} ${format} files.`);
}

// Export current data as CSV
function exportDataAsCSV(): void {
  if (!window.lastResponse || !window.lastResponse.scatter_data) {
    alert('No data available to export');
    return;
  }

  const data = window.lastResponse.scatter_data;
  const headers = ['Bodyweight', 'Weight', 'Sex', 'DOTS_Score'];
  const rows: string[] = [];

  // Add header row
  rows.push(headers.join(','));

  // Add data rows
  for (let i = 0; i < data.x.length; i++) {
    const dotsData = (window.lastResponse as ArrowResponse).dots_scatter_data;
    const dotsScore = dotsData && dotsData.y[i] ? dotsData.y[i] : '';

    rows.push([data.x[i] || '', data.y[i] || '', data.sex[i] || '', dotsScore].join(','));
  }

  const csvContent = rows.join('\n');
  const blob = new Blob([csvContent], { type: 'text/csv' });
  const url = window.URL.createObjectURL(blob);

  const link = document.createElement('a');
  link.href = url;
  link.download = `iron_insights_data_${new Date().toISOString().slice(0, 10)}.csv`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  window.URL.revokeObjectURL(url);
}

// Enhanced chart creation with export and interaction setup
function createPlotEnhanced(chartId: string, traces: PlotlyTrace[], layout: Record<string, unknown>, errorMessage = 'No data available'): boolean {
  const success = createPlot(chartId, traces, layout, errorMessage);

  if (success) {
    // Setup interactions after chart is created
    setTimeout(() => {
      setupChartInteractions(chartId);
    }, 100);
  }

  return success;
}

function setupChartInteractions(chartId: string): void {
  const chartElement = document.getElementById(chartId) as PlotlyElement | null;
  const plotly = window.Plotly;
  if (!chartElement || !chartElement._fullLayout || !plotly) return;

  // Enable selection and brush tools
  const isHistogram = chartId.includes('Distribution');

  try {
    if (isHistogram) {
      // Enable brush selection for histograms
      plotly.relayout(chartId, {
        dragmode: 'select',
        selectdirection: 'horizontal'
      });
    } else {
      // Enable lasso/box select for scatter plots
      plotly.relayout(chartId, {
        dragmode: 'select'
      });
    }

    // Add double-click to reset selection using Plotly's event system
    chartElement.on?.('plotly_doubleclick', function () {
      clearChartSelection(chartId);
    });

    console.log(`Chart interactions setup for ${chartId}`);
  } catch (error) {
    console.error(`Failed to setup interactions for ${chartId}:`, error);
  }
}

// Export dropdown functionality
function toggleExportDropdown(button: HTMLElement): void {
  const dropdown = button.parentElement as HTMLElement | null;
  if (!dropdown) return;
  const menu = dropdown.querySelector('.export-menu') as HTMLElement | null;
  if (!menu) return;
  const isVisible = menu.style.display === 'block';

  // Close all other export dropdowns
  document.querySelectorAll<HTMLElement>('.export-menu').forEach(m => {
    m.style.display = 'none';
  });

  // Toggle this dropdown
  menu.style.display = isVisible ? 'none' : 'block';

  // Close dropdown when clicking outside
  if (!isVisible) {
    setTimeout(() => {
      document.addEventListener('click', function closeDropdown(e) {
        if (!dropdown.contains(e.target as Node)) {
          menu.style.display = 'none';
          document.removeEventListener('click', closeDropdown);
        }
      });
    }, 0);
  }
}

// Expose functions to global scope
window.changeBins = changeBins;
window.switchRankings = switchRankings;
window.toggleTrendline = toggleTrendline;
window.togglePoints = togglePoints;
window.exportAllCharts = exportAllCharts;
window.exportDataAsCSV = exportDataAsCSV;
window.toggleExportDropdown = toggleExportDropdown;
window.setupChartCrossfiltering = setupChartCrossfiltering;
window.createPlotEnhanced = createPlotEnhanced;
window.showError = showError;
window.hideError = hideError;
window.hideChartSkeleton = hideChartSkeleton;
