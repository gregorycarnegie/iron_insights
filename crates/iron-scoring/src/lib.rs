// iron-scoring: Pure DOTS scoring calculations (no dependencies on data processing libraries)

/// DOTS coefficients (gender-specific)
#[derive(Debug, Clone)]
pub struct DotsCoefficients {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
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

    let denominator = coeffs.a
        + coeffs.b * bodyweight_kg
        + coeffs.c * bodyweight_kg.powi(2)
        + coeffs.d * bodyweight_kg.powi(3)
        + coeffs.e * bodyweight_kg.powi(4);

    lift_kg * 500.0 / denominator
}
