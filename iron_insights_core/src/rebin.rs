/// Condenses a 1D counts array by summing adjacent `k` bins.
///
/// ```
/// use iron_insights_core::rebin_1d;
/// assert_eq!(rebin_1d(vec![1, 2, 3, 4], 2), vec![3, 7]);
/// assert_eq!(rebin_1d(vec![1, 2, 3], 1), vec![1, 2, 3]); // identity
/// assert_eq!(rebin_1d(vec![1, 2, 3], 2), vec![3, 3]);    // partial tail kept
/// ```
pub fn rebin_1d(counts: Vec<u32>, k: usize) -> Vec<u32> {
    if k <= 1 {
        return counts;
    }
    counts
        .chunks(k)
        .map(|chunk| chunk.iter().copied().sum())
        .collect()
}

/// Condenses a 2D heatmap grid by pooling cells into `kx` by `ky` blocks.
///
/// ```
/// use iron_insights_core::rebin_2d;
/// // 2×2 grid pooled into 1×1
/// let (out, w, h) = rebin_2d(vec![1, 2, 3, 4], 2, 2, 2, 2);
/// assert_eq!((out, w, h), (vec![10], 1, 1));
/// // identity: k=1 in each dimension
/// let (out, w, h) = rebin_2d(vec![1, 2, 3, 4], 2, 2, 1, 1);
/// assert_eq!((out, w, h), (vec![1, 2, 3, 4], 2, 2));
/// ```
pub fn rebin_2d(
    grid: Vec<u32>,
    width: usize,
    height: usize,
    kx: usize,
    ky: usize,
) -> (Vec<u32>, usize, usize) {
    if kx <= 1 && ky <= 1 {
        return (grid, width, height);
    }

    let w2 = width.div_ceil(kx.max(1));
    let h2 = height.div_ceil(ky.max(1));
    let mut out = vec![0u32; w2 * h2];

    for y in 0..height {
        for x in 0..width {
            let src = y * width + x;
            let dst = (y / ky.max(1)) * w2 + (x / kx.max(1));
            out[dst] = out[dst].saturating_add(grid[src]);
        }
    }

    (out, w2, h2)
}
