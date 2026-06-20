/// Current version of the binary payload format for histograms and heatmaps.
pub const BINARY_FORMAT_VERSION: u16 = 1;
/// Magic byte sequence identifying a standalone Iron Insights Histogram (`IIH1`).
pub const HISTOGRAM_MAGIC: [u8; 4] = *b"IIH1";
/// Magic byte sequence identifying a standalone Iron Insights Heatmap (`IIM1`).
pub const HEATMAP_MAGIC: [u8; 4] = *b"IIM1";
/// Magic for the combined histogram+heatmap binary (IIC1 = Iron Insights Combined v1).
/// Layout: `[IIC1][version u16 LE][hist_len u32 LE][IIH1 blob][IIM1 blob]`
pub const COMBINED_MAGIC: [u8; 4] = *b"IIC1";

/// A parsed histogram binary payload for a single lifter cohort slice.
///
/// `counts[i]` is the number of lifters whose best lift falls in the bin
/// `[min + i * base_bin, min + (i + 1) * base_bin)`.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramBin {
    /// Lower bound of the first bin (kg).
    pub min: f32,
    /// Upper bound of the last bin (kg).
    pub max: f32,
    /// Width of each bin (kg).
    pub base_bin: f32,
    /// Per-bin lifter counts.
    pub counts: Vec<u32>,
    pub(crate) total: u32,
}

/// A parsed heatmap binary payload mapping bodyweight (x) vs lift (y) cell counts.
///
/// `grid[y * width + x]` is the number of lifters in that (bodyweight, lift) cell.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatmapBin {
    /// Lower bodyweight bound (kg).
    pub min_x: f32,
    /// Upper bodyweight bound (kg).
    pub max_x: f32,
    /// Lower lift bound (kg).
    pub min_y: f32,
    /// Upper lift bound (kg).
    pub max_y: f32,
    /// Bodyweight bin width (kg).
    pub base_x: f32,
    /// Lift bin width (kg).
    pub base_y: f32,
    /// Number of bodyweight columns.
    pub width: usize,
    /// Number of lift rows.
    pub height: usize,
    /// Row-major grid of lifter counts (`grid[y * width + x]`).
    pub grid: Vec<u32>,
}

impl HistogramBin {
    /// Constructs a new [`HistogramBin`] and calculates the total lifter count.
    pub fn new(min: f32, max: f32, base_bin: f32, counts: Vec<u32>) -> Self {
        let total = counts.iter().copied().sum();
        Self {
            min,
            max,
            base_bin,
            counts,
            total,
        }
    }
}

/// Parses a standalone `IIH1` histogram binary payload.
pub fn parse_hist_bin(bytes: &[u8]) -> Option<HistogramBin> {
    if bytes.len() < 22 || bytes[0..4] != HISTOGRAM_MAGIC {
        return None;
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().ok()?);
    if version != BINARY_FORMAT_VERSION {
        return None;
    }

    let base = f32::from_le_bytes(bytes[6..10].try_into().ok()?);
    let min = f32::from_le_bytes(bytes[10..14].try_into().ok()?);
    let max = f32::from_le_bytes(bytes[14..18].try_into().ok()?);
    let bins = u32::from_le_bytes(bytes[18..22].try_into().ok()?) as usize;

    let payload = bytes.get(22..)?;
    if payload.len() != bins * 4 {
        return None;
    }

    let mut counts = Vec::with_capacity(bins);
    for chunk in payload.chunks_exact(4) {
        counts.push(u32::from_le_bytes(chunk.try_into().ok()?));
    }

    Some(HistogramBin::new(min, max, base, counts))
}

/// Parses a standalone `IIM1` heatmap binary payload.
pub fn parse_heat_bin(bytes: &[u8]) -> Option<HeatmapBin> {
    if bytes.len() < 38 || bytes[0..4] != HEATMAP_MAGIC {
        return None;
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().ok()?);
    if version != BINARY_FORMAT_VERSION {
        return None;
    }

    let base_x = f32::from_le_bytes(bytes[6..10].try_into().ok()?);
    let base_y = f32::from_le_bytes(bytes[10..14].try_into().ok()?);
    let min_x = f32::from_le_bytes(bytes[14..18].try_into().ok()?);
    let max_x = f32::from_le_bytes(bytes[18..22].try_into().ok()?);
    let min_y = f32::from_le_bytes(bytes[22..26].try_into().ok()?);
    let max_y = f32::from_le_bytes(bytes[26..30].try_into().ok()?);
    let width = u32::from_le_bytes(bytes[30..34].try_into().ok()?) as usize;
    let height = u32::from_le_bytes(bytes[34..38].try_into().ok()?) as usize;

    let payload = bytes.get(38..)?;
    if payload.len() != width * height * 4 {
        return None;
    }

    let mut grid = Vec::with_capacity(width * height);
    for chunk in payload.chunks_exact(4) {
        grid.push(u32::from_le_bytes(chunk.try_into().ok()?));
    }

    Some(HeatmapBin {
        min_x,
        max_x,
        min_y,
        max_y,
        base_x,
        base_y,
        width,
        height,
        grid,
    })
}

/// Parses a combined IIC1 binary payload into a histogram and heatmap.
///
/// The IIC1 format stores both payloads in a single file:
/// `[IIC1][version u16 LE][hist_len u32 LE][IIH1 blob (hist_len bytes)][IIM1 blob (remainder)]`
pub fn parse_combined_bin(bytes: &[u8]) -> Option<(HistogramBin, HeatmapBin)> {
    if bytes.len() < 10 || bytes[0..4] != COMBINED_MAGIC {
        return None;
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().ok()?);
    if version != BINARY_FORMAT_VERSION {
        return None;
    }
    let hist_len = u32::from_le_bytes(bytes[6..10].try_into().ok()?) as usize;
    let hist_bytes = bytes.get(10..10 + hist_len)?;
    let heat_bytes = bytes.get(10 + hist_len..)?;
    let hist = parse_hist_bin(hist_bytes)?;
    let heat = parse_heat_bin(heat_bytes)?;
    Some((hist, heat))
}
