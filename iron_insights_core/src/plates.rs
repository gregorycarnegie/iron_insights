// ===== PLATE CALCULATOR =====

/// Standard IPF competition plates (kg), largest first.
pub const IPF_PLATES_KG: &[f32] = &[25.0, 20.0, 15.0, 10.0, 5.0, 2.5, 1.25];

/// Greedily selects plates for one side of the bar.
///
/// Returns `(plates, remainder_kg)` where `plates` is a list of `(plate_weight_kg, count)`
/// pairs and `remainder_kg` is the unloadable shortfall (ideally 0).
///
/// ```
/// use iron_insights_core::plates_per_side;
/// let (plates, rem) = plates_per_side(90.0);
/// assert_eq!(plates, vec![(25.0, 3), (15.0, 1)]);
/// assert!(rem.abs() < 1e-4);
/// // negative input treated as zero
/// let (plates, rem) = plates_per_side(-5.0);
/// assert!(plates.is_empty());
/// assert!(rem.abs() < 1e-4);
/// ```
pub fn plates_per_side(per_side_needed_kg: f32) -> (Vec<(f32, usize)>, f32) {
    let mut remaining = per_side_needed_kg.max(0.0);
    let mut plates = Vec::new();
    for &plate in IPF_PLATES_KG {
        let count = (remaining / plate + 1e-6).floor() as usize;
        if count > 0 {
            remaining -= plate * count as f32;
            plates.push((plate, count));
        }
    }
    (plates, remaining)
}
