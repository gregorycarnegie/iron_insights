/// Current version of the binary payload format for histograms and heatmaps.
///
/// v2 replaced the fixed-width `u32` count payload with [`encode_counts`].
pub const BINARY_FORMAT_VERSION: u16 = 2;

/// Magic byte sequence identifying a standalone Iron Insights Histogram (`IIH1`).
pub const HISTOGRAM_MAGIC: [u8; 4] = *b"IIH1";
/// Magic byte sequence identifying a standalone Iron Insights Heatmap (`IIM1`).
pub const HEATMAP_MAGIC: [u8; 4] = *b"IIM1";
/// Magic for the combined histogram+heatmap binary (IIC1 = Iron Insights Combined v1).
/// Layout: `[IIC1][version u16 LE][hist_len u32 LE][IIH1 blob][IIM1 blob]`
pub const COMBINED_MAGIC: [u8; 4] = *b"IIC1";

/// Upper bound on the cell count a header may declare.
///
/// The payload is variable-length, so a header's declared length is no longer
/// implied by the byte count and a corrupt one could otherwise drive a huge
/// allocation. The largest real grid is ~71k cells, so this is ample headroom.
const MAX_CELLS: usize = 1 << 22;

fn push_varint(out: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

/// Reads one LEB128 varint, advancing `pos`. Returns `None` if truncated or
/// over-long (a `u32` never needs more than five groups).
fn read_varint(bytes: &[u8], pos: &mut usize) -> Option<u32> {
    let mut result: u32 = 0;
    for shift in [0u32, 7, 14, 21, 28] {
        let byte = *bytes.get(*pos)?;
        *pos += 1;
        // The final group carries only the 4 bits that still fit in a u32.
        let payload = u32::from(byte & 0x7F);
        if shift == 28 && payload > 0x0F {
            return None;
        }
        result |= payload << shift;
        if byte & 0x80 == 0 {
            return Some(result);
        }
    }
    None
}

/// Encodes bin counts as varints with zero runs collapsed.
///
/// A zero varint marks a run and is followed by the run length; any other
/// varint is a literal count. Counts are small and grids are ~87% zeros, so
/// this stores the published data in roughly 8% of the fixed-width `u32` size.
pub fn encode_counts(values: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len());
    let mut i = 0;
    while i < values.len() {
        if values[i] == 0 {
            let start = i;
            while i < values.len() && values[i] == 0 {
                i += 1;
            }
            push_varint(&mut out, 0);
            push_varint(&mut out, (i - start) as u32);
        } else {
            push_varint(&mut out, values[i]);
            i += 1;
        }
    }
    out
}

/// Decodes an [`encode_counts`] payload holding exactly `len` counts.
///
/// Returns `None` for anything malformed: truncation, a run that overruns
/// `len`, or trailing bytes after the last count.
pub fn decode_counts(bytes: &[u8], len: usize) -> Option<Vec<u32>> {
    if len > MAX_CELLS {
        return None;
    }
    let mut out = Vec::with_capacity(len);
    let mut pos = 0usize;
    while out.len() < len {
        let value = read_varint(bytes, &mut pos)?;
        if value == 0 {
            let run = read_varint(bytes, &mut pos)? as usize;
            if run == 0 || run > len - out.len() {
                return None;
            }
            out.resize(out.len() + run, 0);
        } else {
            out.push(value);
        }
    }
    if pos != bytes.len() {
        return None;
    }
    Some(out)
}

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
    /// Sum of `counts`, computed once by [`HistogramBin::new`].
    pub total: u32,
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
pub(crate) fn parse_hist_bin(bytes: &[u8]) -> Option<HistogramBin> {
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

    let counts = decode_counts(bytes.get(22..)?, bins)?;

    Some(HistogramBin::new(min, max, base, counts))
}

/// Parses a standalone `IIM1` heatmap binary payload.
pub(crate) fn parse_heat_bin(bytes: &[u8]) -> Option<HeatmapBin> {
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

    let grid = decode_counts(bytes.get(38..)?, width.checked_mul(height)?)?;

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
