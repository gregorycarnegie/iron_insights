pub(super) const LIFT_BIN_BASE_KG: f32 = 2.5;
pub(super) const BW_BIN_BASE_KG: f32 = 1.0;
pub(super) const SCORE_BIN_BASE_POINTS: f32 = 2.5;
/// Plausible bodyweight range (kg). Sizes the heatmap's y axis, so corrupt
/// rows outside it would stretch the grid with empty rows.
pub(super) const VALID_BW_RANGE_KG: std::ops::RangeInclusive<f32> = 20.0..=300.0;
pub(super) const ALL: &str = "All";
pub(super) const ALL_AGES: &str = "All Ages";
