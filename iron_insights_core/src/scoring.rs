/// Calculates the DOTS score for a given sex, bodyweight (kg), and total (kg).
///
/// ```
/// use iron_insights_core::dots_points;
/// assert_eq!(dots_points("M", 83.0, 0.0), 0.0);
/// assert!(dots_points("M", 83.0, 500.0) > dots_points("M", 83.0, 400.0));
/// assert!(dots_points("F", 63.0, 300.0) > 0.0);
/// ```
#[allow(clippy::excessive_precision)]
pub fn dots_points(sex: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let bw = match sex {
        "F" => bodyweight_kg.clamp(40.0, 150.0),
        _ => bodyweight_kg.clamp(40.0, 210.0),
    };
    let denom = if sex == "F" {
        -57.96288 + 13.6175032 * bw - 0.1126655495 * bw.powi(2) + 0.0005158568 * bw.powi(3)
            - 0.0000010706 * bw.powi(4)
    } else {
        -307.75076 + 24.0900756 * bw - 0.1918759221 * bw.powi(2) + 0.0007391293 * bw.powi(3)
            - 0.0000010930 * bw.powi(4)
    };
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 500.0 / denom
    }
}

/// Calculates the Wilks score for a given sex, bodyweight (kg), and total (kg).
///
/// Uses the updated 2020 Wilks coefficients (600-point scale) as published by
/// OpenPowerlifting (`crates/coefficients/src/wilks2020.rs`).
///
/// ```
/// use iron_insights_core::wilks_points;
/// assert_eq!(wilks_points("M", 83.0, 0.0), 0.0);
/// assert!(wilks_points("M", 83.0, 500.0) > wilks_points("M", 83.0, 400.0));
/// assert!(wilks_points("F", 63.0, 300.0) > 0.0);
/// ```
#[allow(clippy::excessive_precision)]
pub fn wilks_points(sex: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let bw = match sex {
        "F" => bodyweight_kg.clamp(40.0, 150.95),
        _ => bodyweight_kg.clamp(40.0, 200.95),
    };
    let denom = if sex == "F" {
        -125.425539779509 + 13.7121941940668 * bw
            - 0.0330725063103405 * bw.powi(2)
            - 0.0010504000506583 * bw.powi(3)
            + 0.00000938773881462799 * bw.powi(4)
            - 0.000000023334613884954 * bw.powi(5)
    } else {
        47.4617885411949 + 8.47206137941125 * bw + 0.073694103462609 * bw.powi(2)
            - 0.00139583381094385 * bw.powi(3)
            + 0.00000707665973070743 * bw.powi(4)
            - 0.0000000120804336482315 * bw.powi(5)
    };
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 600.0 / denom
    }
}

/// Calculates the IPF GL (Goodlift) points for a given sex, equipment, bodyweight (kg), and total (kg).
///
/// ```
/// use iron_insights_core::goodlift_points;
/// assert_eq!(goodlift_points("M", "Raw", 83.0, 0.0), 0.0);
/// assert!(goodlift_points("M", "Raw", 83.0, 500.0) > goodlift_points("M", "Raw", 83.0, 400.0));
/// assert!(goodlift_points("F", "Single-ply", 63.0, 300.0) > 0.0);
/// ```
#[allow(clippy::excessive_precision)]
pub fn goodlift_points(sex: &str, equipment: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let classic = matches!(equipment, "Raw" | "Wraps" | "Straps");
    let (a, b, c) = match (sex, classic) {
        ("F", true) => (610.32796, 1045.59282, 0.03048),
        ("F", false) => (758.63878, 949.31382, 0.02435),
        ("M", true) => (1199.72839, 1025.18162, 0.00921),
        _ => (1236.25115, 1449.21864, 0.01644),
    };
    let denom = a - (b * (-c * bodyweight_kg).exp());
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 100.0 / denom
    }
}
