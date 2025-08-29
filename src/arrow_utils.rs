// arrow_utils.rs - Arrow IPC serialization utilities
use arrow::array::{Array, Float32Array, StringArray, UInt32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use serde::Serialize;
use std::io::Cursor;
use std::sync::Arc;

use crate::models::{HistogramData, ScatterData, VisualizationResponse, StatsData};
use crate::websocket::{WebSocketMessage, BroadcastMessage};

#[derive(Serialize)]
pub struct ArrowVisualizationResponse {
    pub data: Vec<u8>,  // Combined Arrow IPC stream
    pub user_percentile: Option<f32>,
    pub user_dots_percentile: Option<f32>,
    pub processing_time_ms: u64,
    pub total_records: usize,
}

pub fn serialize_histogram_to_arrow(data: &HistogramData) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let schema = Schema::new(vec![
        Field::new("values", DataType::Float32, false),
        Field::new("counts", DataType::UInt32, false),
        Field::new("bins", DataType::Float32, false),
    ]);

    // Create arrays with equal length by padding shorter vectors
    let max_len = data.values.len().max(data.counts.len()).max(data.bins.len());
    
    let mut values = data.values.clone();
    let mut counts = data.counts.clone();
    let mut bins = data.bins.clone();
    
    // Pad vectors to equal length with appropriate default values
    values.resize(max_len, 0.0);
    counts.resize(max_len, 0u32);
    bins.resize(max_len, 0.0);

    let values_array = Float32Array::from(values);
    let counts_array = UInt32Array::from(counts);
    let bins_array = Float32Array::from(bins);

    let record_batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(values_array),
            Arc::new(counts_array),
            Arc::new(bins_array),
        ],
    )?;

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut stream_writer = StreamWriter::try_new(&mut buffer, &record_batch.schema())?;
        stream_writer.write(&record_batch)?;
        stream_writer.finish()?;
    }

    Ok(buffer.into_inner())
}

pub fn serialize_scatter_to_arrow(data: &ScatterData) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let schema = Schema::new(vec![
        Field::new("x", DataType::Float32, false),
        Field::new("y", DataType::Float32, false),
        Field::new("sex", DataType::Utf8, false),
    ]);

    // Ensure all vectors have the same length
    let min_len = data.x.len().min(data.y.len()).min(data.sex.len());
    
    let x_array = Float32Array::from(data.x.iter().take(min_len).cloned().collect::<Vec<f32>>());
    let y_array = Float32Array::from(data.y.iter().take(min_len).cloned().collect::<Vec<f32>>());
    let sex_array = StringArray::from(data.sex.iter().take(min_len).cloned().collect::<Vec<String>>());

    let record_batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(x_array),
            Arc::new(y_array),
            Arc::new(sex_array),
        ],
    )?;

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut stream_writer = StreamWriter::try_new(&mut buffer, &record_batch.schema())?;
        stream_writer.write(&record_batch)?;
        stream_writer.finish()?;
    }

    Ok(buffer.into_inner())
}

pub fn serialize_all_visualization_data(
    histogram_data: &HistogramData,
    scatter_data: &ScatterData,
    dots_histogram_data: &HistogramData,
    dots_scatter_data: &ScatterData,
    user_percentile: Option<f32>,
    user_dots_percentile: Option<f32>,
    processing_time_ms: u64,
    total_records: usize,
) -> Result<ArrowVisualizationResponse, Box<dyn std::error::Error + Send + Sync>> {
    // Create a comprehensive schema that includes all data types
    let schema = Schema::new(vec![
        // Data type indicator
        Field::new("data_type", DataType::Utf8, false),
        // Histogram fields
        Field::new("hist_values", DataType::Float32, true),
        Field::new("hist_counts", DataType::UInt32, true),
        Field::new("hist_bins", DataType::Float32, true),
        // Scatter fields
        Field::new("scatter_x", DataType::Float32, true),
        Field::new("scatter_y", DataType::Float32, true),
        Field::new("scatter_sex", DataType::Utf8, true),
    ]);

    let mut buffer = Cursor::new(Vec::new());
    let mut stream_writer = StreamWriter::try_new(&mut buffer, &Arc::new(schema.clone()))?;

    // Helper function to pad vectors to the same length
    fn pad_to_length<T: Clone>(vec: &[T], target_len: usize, default: T) -> Vec<T> {
        let mut result = vec.to_vec();
        result.resize(target_len, default);
        result
    }

    // Determine max length across all data
    let max_hist_len = histogram_data.values.len()
        .max(histogram_data.counts.len())
        .max(histogram_data.bins.len());
    let max_scatter_len = scatter_data.x.len()
        .max(scatter_data.y.len())
        .max(scatter_data.sex.len());
    let max_dots_hist_len = dots_histogram_data.values.len()
        .max(dots_histogram_data.counts.len())
        .max(dots_histogram_data.bins.len());
    let max_dots_scatter_len = dots_scatter_data.x.len()
        .max(dots_scatter_data.y.len())
        .max(dots_scatter_data.sex.len());

    // Raw histogram batch
    let raw_hist_len = max_hist_len.max(1);
    let raw_hist_batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            Arc::new(StringArray::from(vec!["raw_histogram"; raw_hist_len])),
            Arc::new(Float32Array::from(pad_to_length(&histogram_data.values, raw_hist_len, 0.0))),
            Arc::new(UInt32Array::from(pad_to_length(&histogram_data.counts, raw_hist_len, 0u32))),
            Arc::new(Float32Array::from(pad_to_length(&histogram_data.bins, raw_hist_len, 0.0))),
            Arc::new(Float32Array::from(vec![0.0; raw_hist_len])), // scatter_x (null)
            Arc::new(Float32Array::from(vec![0.0; raw_hist_len])), // scatter_y (null)  
            Arc::new(StringArray::from(vec![""; raw_hist_len])), // scatter_sex (null)
        ],
    )?;

    // Raw scatter batch
    let raw_scatter_len = max_scatter_len.max(1);
    let raw_scatter_batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            Arc::new(StringArray::from(vec!["raw_scatter"; raw_scatter_len])),
            Arc::new(Float32Array::from(vec![0.0; raw_scatter_len])), // hist_values (null)
            Arc::new(UInt32Array::from(vec![0u32; raw_scatter_len])), // hist_counts (null)
            Arc::new(Float32Array::from(vec![0.0; raw_scatter_len])), // hist_bins (null)
            Arc::new(Float32Array::from(pad_to_length(&scatter_data.x, raw_scatter_len, 0.0))),
            Arc::new(Float32Array::from(pad_to_length(&scatter_data.y, raw_scatter_len, 0.0))),
            Arc::new(StringArray::from(pad_to_length(&scatter_data.sex, raw_scatter_len, String::new()))),
        ],
    )?;

    // DOTS histogram batch
    let dots_hist_len = max_dots_hist_len.max(1);
    let dots_hist_batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            Arc::new(StringArray::from(vec!["dots_histogram"; dots_hist_len])),
            Arc::new(Float32Array::from(pad_to_length(&dots_histogram_data.values, dots_hist_len, 0.0))),
            Arc::new(UInt32Array::from(pad_to_length(&dots_histogram_data.counts, dots_hist_len, 0u32))),
            Arc::new(Float32Array::from(pad_to_length(&dots_histogram_data.bins, dots_hist_len, 0.0))),
            Arc::new(Float32Array::from(vec![0.0; dots_hist_len])), // scatter_x (null)
            Arc::new(Float32Array::from(vec![0.0; dots_hist_len])), // scatter_y (null)
            Arc::new(StringArray::from(vec![""; dots_hist_len])), // scatter_sex (null)
        ],
    )?;

    // DOTS scatter batch
    let dots_scatter_len = max_dots_scatter_len.max(1);
    let dots_scatter_batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            Arc::new(StringArray::from(vec!["dots_scatter"; dots_scatter_len])),
            Arc::new(Float32Array::from(vec![0.0; dots_scatter_len])), // hist_values (null)
            Arc::new(UInt32Array::from(vec![0u32; dots_scatter_len])), // hist_counts (null)
            Arc::new(Float32Array::from(vec![0.0; dots_scatter_len])), // hist_bins (null)
            Arc::new(Float32Array::from(pad_to_length(&dots_scatter_data.x, dots_scatter_len, 0.0))),
            Arc::new(Float32Array::from(pad_to_length(&dots_scatter_data.y, dots_scatter_len, 0.0))),
            Arc::new(StringArray::from(pad_to_length(&dots_scatter_data.sex, dots_scatter_len, String::new()))),
        ],
    )?;

    // Write all batches
    stream_writer.write(&raw_hist_batch)?;
    stream_writer.write(&raw_scatter_batch)?;
    stream_writer.write(&dots_hist_batch)?;
    stream_writer.write(&dots_scatter_batch)?;
    stream_writer.finish()?;

    Ok(ArrowVisualizationResponse {
        data: buffer.into_inner(),
        user_percentile,
        user_dots_percentile,
        processing_time_ms,
        total_records,
    })
}

/// Serialize any VisualizationResponse to Arrow IPC binary format
pub fn serialize_visualization_response_to_arrow(response: &VisualizationResponse) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    serialize_all_visualization_data(
        &response.histogram_data,
        &response.scatter_data,
        &response.dots_histogram_data,
        &response.dots_scatter_data,
        response.user_percentile,
        response.user_dots_percentile,
        response.processing_time_ms,
        response.total_records,
    ).map(|arrow_response| arrow_response.data)
}

/// Deserialize Arrow IPC binary format back to VisualizationResponse
pub fn deserialize_visualization_response_from_arrow(data: &[u8]) -> Result<VisualizationResponse, Box<dyn std::error::Error + Send + Sync>> {
    use arrow_ipc::reader::StreamReader;
    use std::io::Cursor;
    
    let cursor = Cursor::new(data);
    let mut reader = StreamReader::try_new(cursor, None)?;
    
    let mut histogram_data = HistogramData {
        values: Vec::new(),
        counts: Vec::new(),
        bins: Vec::new(),
        min_val: 0.0,
        max_val: 0.0,
    };
    let mut scatter_data = ScatterData {
        x: Vec::new(),
        y: Vec::new(),
        sex: Vec::new(),
    };
    let mut dots_histogram_data = histogram_data.clone();
    let mut dots_scatter_data = scatter_data.clone();
    
    let user_percentile = None;
    let user_dots_percentile = None;
    let processing_time_ms = 0;
    let total_records = 0;
    
    // Read all record batches and reconstruct data
    while let Some(batch) = reader.next() {
        let batch = batch?;
        let data_type_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or("Invalid data_type column")?;
        
        let data_type = data_type_array.value(0);
        match data_type {
                "raw_histogram" => {
                    if let (Some(values), Some(counts), Some(bins)) = (
                        batch.column(1).as_any().downcast_ref::<Float32Array>(),
                        batch.column(2).as_any().downcast_ref::<UInt32Array>(),
                        batch.column(3).as_any().downcast_ref::<Float32Array>(),
                    ) {
                        histogram_data.values = values.values().to_vec();
                        histogram_data.counts = counts.values().to_vec();
                        histogram_data.bins = bins.values().to_vec();
                        if !histogram_data.values.is_empty() {
                            histogram_data.min_val = histogram_data.values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
                            histogram_data.max_val = histogram_data.values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                        }
                    }
                }
                "raw_scatter" => {
                    if let (Some(x), Some(y), Some(sex)) = (
                        batch.column(4).as_any().downcast_ref::<Float32Array>(),
                        batch.column(5).as_any().downcast_ref::<Float32Array>(),
                        batch.column(6).as_any().downcast_ref::<StringArray>(),
                    ) {
                        scatter_data.x = x.values().to_vec();
                        scatter_data.y = y.values().to_vec();
                        scatter_data.sex = (0..sex.len()).map(|i| sex.value(i).to_string()).collect();
                    }
                }
                "dots_histogram" => {
                    if let (Some(values), Some(counts), Some(bins)) = (
                        batch.column(1).as_any().downcast_ref::<Float32Array>(),
                        batch.column(2).as_any().downcast_ref::<UInt32Array>(),
                        batch.column(3).as_any().downcast_ref::<Float32Array>(),
                    ) {
                        dots_histogram_data.values = values.values().to_vec();
                        dots_histogram_data.counts = counts.values().to_vec();
                        dots_histogram_data.bins = bins.values().to_vec();
                        if !dots_histogram_data.values.is_empty() {
                            dots_histogram_data.min_val = dots_histogram_data.values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
                            dots_histogram_data.max_val = dots_histogram_data.values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                        }
                    }
                }
                "dots_scatter" => {
                    if let (Some(x), Some(y), Some(sex)) = (
                        batch.column(4).as_any().downcast_ref::<Float32Array>(),
                        batch.column(5).as_any().downcast_ref::<Float32Array>(),
                        batch.column(6).as_any().downcast_ref::<StringArray>(),
                    ) {
                        dots_scatter_data.x = x.values().to_vec();
                        dots_scatter_data.y = y.values().to_vec();
                        dots_scatter_data.sex = (0..sex.len()).map(|i| sex.value(i).to_string()).collect();
                    }
                }
                _ => {}
        }
    }
    
    Ok(VisualizationResponse {
        histogram_data,
        scatter_data,
        dots_histogram_data,
        dots_scatter_data,
        user_percentile,
        user_dots_percentile,
        processing_time_ms,
        total_records,
    })
}

/// Serialize stats data to Arrow IPC format
pub fn serialize_stats_to_arrow(stats: &StatsData) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let schema = Schema::new(vec![
        Field::new("total_records", DataType::UInt32, false),
        Field::new("cache_entries", DataType::UInt32, false),
        Field::new("cache_size", DataType::UInt64, false),
        Field::new("scoring_system", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
    ]);

    let total_records_array = UInt32Array::from(vec![stats.total_records]);
    let cache_entries_array = UInt32Array::from(vec![stats.cache_entries]);
    let cache_size_array = UInt64Array::from(vec![stats.cache_size]);
    let scoring_system_array = StringArray::from(vec![stats.scoring_system.clone()]);
    let status_array = StringArray::from(vec![stats.status.clone()]);

    let record_batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(total_records_array),
            Arc::new(cache_entries_array),
            Arc::new(cache_size_array),
            Arc::new(scoring_system_array),
            Arc::new(status_array),
        ],
    )?;

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut stream_writer = StreamWriter::try_new(&mut buffer, &record_batch.schema())?;
        stream_writer.write(&record_batch)?;
        stream_writer.finish()?;
    }

    Ok(buffer.into_inner())
}