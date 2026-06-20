// ===== PERCENTILE TIER =====

/// Maps a percentile (0.0 to 1.0) to a descriptive performance tier label.
///
/// ```
/// use iron_insights_core::tier_for_percentile;
/// assert_eq!(tier_for_percentile(0.99), "Legend");
/// assert_eq!(tier_for_percentile(0.97), "Elite");
/// assert_eq!(tier_for_percentile(0.85), "Advanced");
/// assert_eq!(tier_for_percentile(0.70), "Intermediate");
/// assert_eq!(tier_for_percentile(0.30), "Novice");
/// ```
pub fn tier_for_percentile(pct: f32) -> &'static str {
    if pct >= 0.99 {
        "Legend"
    } else if pct >= 0.95 {
        "Elite"
    } else if pct >= 0.8 {
        "Advanced"
    } else if pct >= 0.6 {
        "Intermediate"
    } else {
        "Novice"
    }
}

// ===== BODYFAT (US NAVY METHOD) =====
