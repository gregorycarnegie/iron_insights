use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

const MAGIC_HIST: [u8; 4] = *b"IIH1";
const MAGIC_HEAT: [u8; 4] = *b"IIM1";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistogramHeader {
    pub version: u16,
    pub base_bin_size: f32,
    pub min: f32,
    pub max: f32,
    pub bins: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatmapHeader {
    pub version: u16,
    pub base_bin_size_x: f32,
    pub base_bin_size_y: f32,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub width: u32,
    pub height: u32,
}

pub fn write_histogram_bin(path: &Path, header: HistogramHeader, counts: &[u32]) -> Result<()> {
    if counts.len() != header.bins as usize {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "counts length does not match header bins",
        ));
    }

    let mut bytes = Vec::with_capacity(4 + 2 + (3 * 4) + 4 + counts.len() * 4);
    bytes.extend_from_slice(&MAGIC_HIST);
    bytes.extend_from_slice(&header.version.to_le_bytes());
    bytes.extend_from_slice(&header.base_bin_size.to_le_bytes());
    bytes.extend_from_slice(&header.min.to_le_bytes());
    bytes.extend_from_slice(&header.max.to_le_bytes());
    bytes.extend_from_slice(&header.bins.to_le_bytes());

    for value in counts {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fs::write(path, bytes)?;
    Ok(())
}

pub fn read_histogram_bin(path: &Path) -> Result<(HistogramHeader, Vec<u32>)> {
    let bytes = fs::read(path)?;
    if bytes.len() < 22 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "file too small to contain histogram header",
        ));
    }
    if bytes[0..4] != MAGIC_HIST {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "invalid histogram magic header",
        ));
    }

    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let base_bin_size = f32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
    let min = f32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
    let max = f32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
    let bins = u32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]);

    let payload = &bytes[22..];
    if payload.len() != bins as usize * 4 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "histogram payload length does not match header bins",
        ));
    }

    let mut counts = Vec::with_capacity(bins as usize);
    for chunk in payload.chunks_exact(4) {
        counts.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }

    Ok((
        HistogramHeader {
            version,
            base_bin_size,
            min,
            max,
            bins,
        },
        counts,
    ))
}

pub fn write_heatmap_bin(path: &Path, header: HeatmapHeader, grid: &[u32]) -> Result<()> {
    if grid.len() != (header.width as usize * header.height as usize) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "grid length does not match header width*height",
        ));
    }

    let mut bytes = Vec::with_capacity(4 + 2 + (7 * 4) + (2 * 4) + grid.len() * 4);
    bytes.extend_from_slice(&MAGIC_HEAT);
    bytes.extend_from_slice(&header.version.to_le_bytes());
    bytes.extend_from_slice(&header.base_bin_size_x.to_le_bytes());
    bytes.extend_from_slice(&header.base_bin_size_y.to_le_bytes());
    bytes.extend_from_slice(&header.min_x.to_le_bytes());
    bytes.extend_from_slice(&header.max_x.to_le_bytes());
    bytes.extend_from_slice(&header.min_y.to_le_bytes());
    bytes.extend_from_slice(&header.max_y.to_le_bytes());
    bytes.extend_from_slice(&header.width.to_le_bytes());
    bytes.extend_from_slice(&header.height.to_le_bytes());

    for value in grid {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fs::write(path, bytes)?;
    Ok(())
}

pub fn read_heatmap_bin(path: &Path) -> Result<(HeatmapHeader, Vec<u32>)> {
    let bytes = fs::read(path)?;
    if bytes.len() < 38 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "file too small to contain heatmap header",
        ));
    }
    if bytes[0..4] != MAGIC_HEAT {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "invalid heatmap magic header",
        ));
    }

    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let base_bin_size_x = f32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
    let base_bin_size_y = f32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
    let min_x = f32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
    let max_x = f32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]);
    let min_y = f32::from_le_bytes([bytes[22], bytes[23], bytes[24], bytes[25]]);
    let max_y = f32::from_le_bytes([bytes[26], bytes[27], bytes[28], bytes[29]]);
    let width = u32::from_le_bytes([bytes[30], bytes[31], bytes[32], bytes[33]]);
    let height = u32::from_le_bytes([bytes[34], bytes[35], bytes[36], bytes[37]]);

    let payload = &bytes[38..];
    if payload.len() != width as usize * height as usize * 4 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "heatmap payload length does not match width*height",
        ));
    }

    let mut grid = Vec::with_capacity(width as usize * height as usize);
    for chunk in payload.chunks_exact(4) {
        grid.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }

    Ok((
        HeatmapHeader {
            version,
            base_bin_size_x,
            base_bin_size_y,
            min_x,
            max_x,
            min_y,
            max_y,
            width,
            height,
        },
        grid,
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        HeatmapHeader, HistogramHeader, read_heatmap_bin, read_histogram_bin, write_heatmap_bin,
        write_histogram_bin,
    };
    use std::path::PathBuf;

    #[test]
    fn histogram_roundtrip() {
        let mut path = std::env::temp_dir();
        path.push("ii_hist_roundtrip.bin");

        let header = HistogramHeader {
            version: 1,
            base_bin_size: 2.5,
            min: 0.0,
            max: 200.0,
            bins: 4,
        };
        let counts = vec![10, 20, 30, 40];

        write_histogram_bin(&path, header, &counts).expect("write should succeed");
        let (read_header, read_counts) = read_histogram_bin(&path).expect("read should succeed");

        assert_eq!(read_header, header);
        assert_eq!(read_counts, counts);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn heatmap_roundtrip() {
        let mut path = std::env::temp_dir();
        path.push("ii_heat_roundtrip.bin");

        let header = HeatmapHeader {
            version: 1,
            base_bin_size_x: 2.5,
            base_bin_size_y: 1.0,
            min_x: 0.0,
            max_x: 300.0,
            min_y: 40.0,
            max_y: 200.0,
            width: 2,
            height: 3,
        };
        let grid = vec![1, 2, 3, 4, 5, 6];

        write_heatmap_bin(&path, header, &grid).expect("write should succeed");
        let (read_header, read_grid) = read_heatmap_bin(&path).expect("read should succeed");

        assert_eq!(read_header, header);
        assert_eq!(read_grid, grid);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn invalid_hist_payload_is_rejected() {
        let mut path = std::env::temp_dir();
        path.push("ii_hist_invalid.bin");

        std::fs::write(
            &path,
            [
                b'I', b'I', b'H', b'1', // magic
                1, 0, // version
                0, 0, 32, 64, // base_bin_size (2.5)
                0, 0, 0, 0, // min
                0, 0, 72, 67, // max (200)
                2, 0, 0, 0, // bins=2
                1, 0, 0, 0, // only one u32 provided
            ],
        )
        .expect("write should succeed");

        let result = read_histogram_bin(&path);
        assert!(result.is_err());
        let _ = std::fs::remove_file(PathBuf::from(path));
    }
}
