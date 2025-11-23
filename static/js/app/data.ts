import type { ArrowApi, ArrowResponse, FetchParams, SexValue } from './types';

// Request deduplication map to avoid redundant network calls
const pendingRequests: Map<string, Promise<ArrowResponse>> = new Map();

function ensureArrow(): ArrowApi {
  if (!window.Arrow || typeof window.Arrow.tableFromIPC !== 'function') {
    throw new Error('Apache Arrow library not loaded or tableFromIPC not available');
  }
  return window.Arrow;
}

// Function to fetch and parse comprehensive Arrow data
async function fetchArrowData(params: FetchParams): Promise<ArrowResponse> {
  const key = JSON.stringify(params);

  // Return existing promise if request is in flight
  if (pendingRequests.has(key)) {
    console.log('ÐY"" Deduplicating request');
    return pendingRequests.get(key)!;
  }

  const promise: Promise<ArrowResponse> = (async () => {
    try {
      const response = await fetch('/api/visualize-arrow', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(params)
      });

      if (!response.ok) {
        throw new Error('HTTP error! status: ' + response.status);
      }

      const arrayBuffer = await response.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);

      // Parse Arrow IPC stream
      const table = ensureArrow().tableFromIPC(uint8Array);

      if (!table || table.length === 0) {
        throw new Error('No Arrow data received');
      }

      // Parse data by data_type
      const result: ArrowResponse = {
        histogram_data: { values: [], counts: [], bins: [], min_val: 0, max_val: 0 },
        scatter_data: { x: [], y: [], sex: [] },
        dots_histogram_data: { values: [], counts: [], bins: [], min_val: 0, max_val: 0 },
        dots_scatter_data: { x: [], y: [], sex: [] },
        user_percentile: null,
        user_dots_percentile: null,
        processing_time_ms: 0,
        total_records: 0
      };

      // Get column arrays from the table using getChildAt
      const dataTypes = table.getChildAt<string>(0)?.toArray() ?? []; // data_type column
      const histValues = table.getChildAt<number>(1)?.toArray() ?? []; // hist_values column
      const histCounts = table.getChildAt<number>(2)?.toArray() ?? []; // hist_counts column
      const histBins = table.getChildAt<number>(3)?.toArray() ?? []; // hist_bins column
      const scatterX = table.getChildAt<number>(4)?.toArray() ?? []; // scatter_x column
      const scatterY = table.getChildAt<number>(5)?.toArray() ?? []; // scatter_y column
      const scatterSex = table.getChildAt<string>(6)?.toArray() ?? []; // scatter_sex column

      for (let i = 0; i < dataTypes.length; i++) {
        const dataType = dataTypes[i];

        if (dataType === 'raw_histogram') {
          if (histValues[i] > 0) result.histogram_data.values.push(histValues[i]);
          if (histCounts[i] > 0) result.histogram_data.counts.push(histCounts[i]);
          if (histBins[i] > 0) result.histogram_data.bins.push(histBins[i]);
        } else if (dataType === 'raw_scatter') {
          if (scatterX[i] > 0 && scatterY[i] > 0) {
            result.scatter_data.x.push(scatterX[i]);
            result.scatter_data.y.push(scatterY[i]);
            result.scatter_data.sex.push(scatterSex[i] as SexValue);
          }
        } else if (dataType === 'dots_histogram') {
          if (histValues[i] > 0) result.dots_histogram_data.values.push(histValues[i]);
          if (histCounts[i] > 0) result.dots_histogram_data.counts.push(histCounts[i]);
          if (histBins[i] > 0) result.dots_histogram_data.bins.push(histBins[i]);
        } else if (dataType === 'dots_scatter') {
          if (scatterX[i] > 0 && scatterY[i] > 0) {
            result.dots_scatter_data.x.push(scatterX[i]);
            result.dots_scatter_data.y.push(scatterY[i]);
            result.dots_scatter_data.sex.push(scatterSex[i] as SexValue);
          }
        }
      }

      // Calculate min/max for histograms
      if (result.histogram_data.values.length > 0) {
        result.histogram_data.min_val = Math.min(...result.histogram_data.values);
        result.histogram_data.max_val = Math.max(...result.histogram_data.values);
      }
      if (result.dots_histogram_data.values.length > 0) {
        result.dots_histogram_data.min_val = Math.min(...result.dots_histogram_data.values);
        result.dots_histogram_data.max_val = Math.max(...result.dots_histogram_data.values);
      }

      const userPercentileHeader = response.headers.get('X-User-Percentile');
      result.user_percentile = userPercentileHeader ? parseFloat(userPercentileHeader) : null;

      const userDotsPercentileHeader = response.headers.get('X-User-Dots-Percentile');
      result.user_dots_percentile = userDotsPercentileHeader ? parseFloat(userDotsPercentileHeader) : null;

      const processingTimeHeader = response.headers.get('X-Processing-Time-Ms');
      result.processing_time_ms = processingTimeHeader ? parseInt(processingTimeHeader, 10) : 0;

      const totalRecordsHeader = response.headers.get('X-Total-Records');
      result.total_records = totalRecordsHeader ? parseInt(totalRecordsHeader, 10) : 0;

      console.log('ƒo. Arrow IPC data parsed successfully:', result);
      return result;
    } catch (error) {
      console.error('ƒ?O Arrow data fetch error:', error);
      throw error;
    }
  })();

  // Store the promise
  pendingRequests.set(key, promise);

  // Clean up after request completes
  promise.finally(() => {
    pendingRequests.delete(key);
  });

  return promise;
}

// Expose functions to global scope
window.fetchArrowData = fetchArrowData;
