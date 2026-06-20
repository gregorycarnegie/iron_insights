// ===== 1RM FORMULAS =====

/// Estimates the one-rep max from a submaximal set.
///
/// Returns `weight` unchanged for `reps <= 1.0`.
/// Supported `formula` values: `"epley"` (default), `"brzycki"`, `"mayhew"`, `"lander"`,
/// `"lombardi"`, `"oconner"`.
///
/// ```
/// use iron_insights_core::calc_1rm;
/// // single rep is always the 1RM
/// assert_eq!(calc_1rm(100.0, 1.0, "epley"), 100.0);
/// // Epley: weight * (1 + reps/30)
/// assert!((calc_1rm(100.0, 10.0, "epley") - 133.33).abs() < 0.1);
/// // higher rep count → higher estimated 1RM
/// assert!(calc_1rm(100.0, 10.0, "brzycki") > calc_1rm(100.0, 5.0, "brzycki"));
/// ```
pub fn calc_1rm(weight: f32, reps: f32, formula: &str) -> f32 {
    if reps <= 1.0 {
        return weight;
    }
    match formula {
        "brzycki" => weight / (1.0278 - 0.0278 * reps),
        "mayhew" => (100.0 * weight) / (52.2 + 41.9 * (-0.055 * reps).exp()),
        "lander" => (100.0 * weight) / (101.3 - 2.67123 * reps),
        "lombardi" => weight * reps.powf(0.1),
        "oconner" => weight * (1.0 + reps / 40.0),
        _ => weight * (1.0 + reps / 30.0),
    }
}
