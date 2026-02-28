pub fn rebin_1d(counts: Vec<u32>, k: usize) -> Vec<u32> {
    assert!(k > 0, "k must be > 0");

    if k == 1 {
        return counts;
    }

    counts
        .chunks(k)
        .map(|chunk| chunk.iter().copied().sum())
        .collect()
}

pub fn rebin_2d(
    grid: Vec<u32>,
    w: usize,
    h: usize,
    kx: usize,
    ky: usize,
) -> (Vec<u32>, usize, usize) {
    assert!(w * h == grid.len(), "grid length must equal w*h");
    assert!(kx > 0, "kx must be > 0");
    assert!(ky > 0, "ky must be > 0");

    let w2 = w.div_ceil(kx);
    let h2 = h.div_ceil(ky);
    let mut out = vec![0u32; w2 * h2];

    for y in 0..h {
        let y2 = y / ky;
        for x in 0..w {
            let x2 = x / kx;
            let src_idx = y * w + x;
            let dst_idx = y2 * w2 + x2;
            out[dst_idx] = out[dst_idx].saturating_add(grid[src_idx]);
        }
    }

    (out, w2, h2)
}

#[cfg(test)]
mod tests {
    use super::{rebin_1d, rebin_2d};

    #[test]
    fn rebin_1d_groups_chunks() {
        let counts = vec![1, 2, 3, 4, 5];
        let rebinned = rebin_1d(counts, 2);
        assert_eq!(rebinned, vec![3, 7, 5]);
    }

    #[test]
    fn rebin_1d_identity_when_k_is_one() {
        let counts = vec![10, 20, 30];
        assert_eq!(rebin_1d(counts.clone(), 1), counts);
    }

    #[test]
    fn rebin_2d_reduces_even_grid() {
        // 4x2 grid:
        // [1, 2, 3, 4,
        //  5, 6, 7, 8]
        let grid = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let (out, w2, h2) = rebin_2d(grid, 4, 2, 2, 1);

        assert_eq!((w2, h2), (2, 2));
        assert_eq!(out, vec![3, 7, 11, 15]);
    }

    #[test]
    fn rebin_2d_keeps_partial_edge_bins() {
        // 3x3 with kx=2, ky=2 => 2x2 output with partial edge bins.
        let grid = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let (out, w2, h2) = rebin_2d(grid, 3, 3, 2, 2);

        assert_eq!((w2, h2), (2, 2));
        assert_eq!(out, vec![12, 9, 15, 9]);
    }
}
