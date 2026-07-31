use anyhow::Result;
use iron_insights_core::{
    BINARY_FORMAT_VERSION, COMBINED_MAGIC, HEATMAP_MAGIC, HISTOGRAM_MAGIC, encode_counts,
};

#[derive(Debug)]
pub(super) struct HistogramData {
    pub(super) min: f32,
    pub(super) max: f32,
    pub(super) counts: Vec<u32>,
    pub(super) total: u64,
}

#[derive(Debug)]
pub(super) struct HeatmapData {
    pub(super) min_x: f32,
    pub(super) max_x: f32,
    pub(super) min_y: f32,
    pub(super) max_y: f32,
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) grid: Vec<u32>,
    pub(super) total: u64,
}

pub(super) fn build_histogram(values: &[f32], base: f32) -> Result<HistogramData> {
    let min_val = values
        .iter()
        .copied()
        .reduce(f32::min)
        .ok_or_else(|| anyhow::anyhow!("cannot build histogram for empty input"))?;
    let max_val = values
        .iter()
        .copied()
        .reduce(f32::max)
        .ok_or_else(|| anyhow::anyhow!("cannot build histogram for empty input"))?;

    let min_edge = (min_val / base).floor() * base;
    let max_edge = ((max_val / base).floor() + 1.0f32) * base;
    let bins = (((max_edge - min_edge) / base).round() as usize).max(1);

    let mut counts = vec![0u32; bins];
    for value in values {
        let raw = ((value - min_edge) / base).floor();
        let idx = raw.clamp(0.0f32, (bins - 1) as f32) as usize;
        counts[idx] = counts[idx].saturating_add(1);
    }

    Ok(HistogramData {
        min: min_edge,
        max: max_edge,
        total: values.len() as u64,
        counts,
    })
}

pub(super) fn build_heatmap(
    points: &[(f32, f32)],
    x_base: f32,
    y_base: f32,
) -> Result<HeatmapData> {
    if points.is_empty() {
        return Ok(HeatmapData {
            min_x: 0.0f32,
            max_x: 0.0f32,
            min_y: 0.0f32,
            max_y: 0.0f32,
            width: 0,
            height: 0,
            grid: Vec::new(),
            total: 0,
        });
    }

    let (mut min_x, mut max_x) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);

    for (x, y) in points {
        min_x = min_x.min(*x);
        max_x = max_x.max(*x);
        min_y = min_y.min(*y);
        max_y = max_y.max(*y);
    }

    let min_x_edge = (min_x / x_base).floor() * x_base;
    let max_x_edge = ((max_x / x_base).floor() + 1.0f32) * x_base;
    let min_y_edge = (min_y / y_base).floor() * y_base;
    let max_y_edge = ((max_y / y_base).floor() + 1.0f32) * y_base;

    let width = (((max_x_edge - min_x_edge) / x_base).round() as usize).max(1);
    let height = (((max_y_edge - min_y_edge) / y_base).round() as usize).max(1);

    let mut grid = vec![0u32; width * height];
    for (x, y) in points {
        let ix = (((x - min_x_edge) / x_base).floor()).clamp(0.0f32, (width - 1) as f32) as usize;
        let iy = (((y - min_y_edge) / y_base).floor()).clamp(0.0f32, (height - 1) as f32) as usize;
        let idx = iy * width + ix;
        grid[idx] = grid[idx].saturating_add(1);
    }

    Ok(HeatmapData {
        min_x: min_x_edge,
        max_x: max_x_edge,
        min_y: min_y_edge,
        max_y: max_y_edge,
        width,
        height,
        total: points.len() as u64,
        grid,
    })
}

fn hist_bytes(hist: &HistogramData, x_base: f32) -> Vec<u8> {
    let payload = encode_counts(&hist.counts);
    let mut bytes = Vec::with_capacity(4 + 2 + (3 * 4) + 4 + payload.len());
    bytes.extend_from_slice(&HISTOGRAM_MAGIC);
    bytes.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&x_base.to_le_bytes());
    bytes.extend_from_slice(&hist.min.to_le_bytes());
    bytes.extend_from_slice(&hist.max.to_le_bytes());
    bytes.extend_from_slice(&(hist.counts.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&payload);
    bytes
}

fn heat_bytes(heat: &HeatmapData, x_base: f32, y_base: f32) -> Vec<u8> {
    let payload = encode_counts(&heat.grid);
    let mut bytes = Vec::with_capacity(4 + 2 + (6 * 4) + (2 * 4) + payload.len());
    bytes.extend_from_slice(&HEATMAP_MAGIC);
    bytes.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&x_base.to_le_bytes());
    bytes.extend_from_slice(&y_base.to_le_bytes());
    bytes.extend_from_slice(&heat.min_x.to_le_bytes());
    bytes.extend_from_slice(&heat.max_x.to_le_bytes());
    bytes.extend_from_slice(&heat.min_y.to_le_bytes());
    bytes.extend_from_slice(&heat.max_y.to_le_bytes());
    bytes.extend_from_slice(&(heat.width as u32).to_le_bytes());
    bytes.extend_from_slice(&(heat.height as u32).to_le_bytes());
    bytes.extend_from_slice(&payload);
    bytes
}

pub(super) const INLINE_THRESHOLD: usize = 400;

pub(super) fn build_combined_bytes(
    hist: &HistogramData,
    heat: &HeatmapData,
    x_base: f32,
    y_base: f32,
) -> Vec<u8> {
    let h = hist_bytes(hist, x_base);
    let m = heat_bytes(heat, x_base, y_base);
    let hist_len = h.len() as u32;
    let mut combined = Vec::with_capacity(10 + h.len() + m.len());
    combined.extend_from_slice(&COMBINED_MAGIC);
    combined.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    combined.extend_from_slice(&hist_len.to_le_bytes());
    combined.extend_from_slice(&h);
    combined.extend_from_slice(&m);
    combined
}
