// arrow_utils.rs - Arrow IPC serialization utilities
use arrow::array::{Float32Array, StringArray, UInt32Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use serde::Serialize;
use std::io::Cursor;
use std::sync::Arc;

use crate::models::{HistogramData, ScatterData};

#[derive(Serialize)]
pub struct ArrowVisualizationResponse {
    pub data: Vec<u8>,  // Combined Arrow IPC stream
    pub user_percentile: Option<f32>,
    pub user_dots_percentile: Option<f32>,
    pub processing_time_ms: u64,
    pub total_records: usize,
}

pub fn serialize_histogram_to_arrow(data: &HistogramData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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

pub fn serialize_scatter_to_arrow(data: &ScatterData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
) -> Result<ArrowVisualizationResponse, Box<dyn std::error::Error>> {
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