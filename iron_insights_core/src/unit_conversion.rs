// ===== UNIT CONVERSION =====

/// Conversion factor from pounds to kilograms.
pub const KG_PER_LB: f32 = 0.453_592_37;

/// Converts kilograms to pounds.
///
/// ```
/// use iron_insights_core::kg_to_lbs;
/// assert!((kg_to_lbs(100.0) - 220.462).abs() < 0.01);
/// assert_eq!(kg_to_lbs(0.0), 0.0);
/// ```
pub fn kg_to_lbs(kg: f32) -> f32 {
    kg / KG_PER_LB
}

/// Converts pounds to kilograms.
///
/// ```
/// use iron_insights_core::lbs_to_kg;
/// assert!((lbs_to_kg(220.0) - 99.790).abs() < 0.01);
/// assert_eq!(lbs_to_kg(0.0), 0.0);
/// ```
pub fn lbs_to_kg(lbs: f32) -> f32 {
    lbs * KG_PER_LB
}
