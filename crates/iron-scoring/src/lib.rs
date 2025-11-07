// iron-scoring: Pure DOTS scoring calculations (no dependencies on data processing libraries)

/// DOTS coefficients (gender-specific)
#[derive(Debug, Clone)]
pub struct DotsCoefficients {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
}

impl DotsCoefficients {
    /// Get coefficients for male lifters
    pub fn male() -> Self {
        Self {
            a: -307.75076,
            b: 24.0900756,
            c: -0.1918759221,
            d: 0.0007391293,
            e: -0.000001093,
        }
    }

    /// Get coefficients for female lifters
    pub fn female() -> Self {
        Self {
            a: -57.96288,
            b: 13.6175032,
            c: -0.1126655495,
            d: 0.0005158568,
            e: -0.0000010706,
        }
    }
}

impl Default for DotsCoefficients {
    fn default() -> Self {
        Self::male()
    }
}

/// Calculate DOTS score for a given lift, bodyweight, and sex
pub fn calculate_dots_score(lift_kg: f32, bodyweight_kg: f32, sex: &str) -> f32 {
    let coeffs = if sex == "M" || sex == "Male" {
        DotsCoefficients::male()
    } else {
        DotsCoefficients::female()
    };
    let x = bodyweight_kg as f64;
    let denominator = coeffs.a
        + coeffs.b * x
        + coeffs.c * x.powi(2)
        + coeffs.d * x.powi(3)
        + coeffs.e * x.powi(4);

    lift_kg * 500.0 / denominator as f32
}
