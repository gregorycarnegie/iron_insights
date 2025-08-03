// scoring.rs
use polars::prelude::*;

/// DOTS coefficients (gender-neutral)
pub struct DotsCoefficients {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
}

impl Default for DotsCoefficients {
    fn default() -> Self {
        Self {
            a: -307.75076,
            b: 24.0900756,
            c: -0.1918759221,
            d: 0.0007391293,
            e: -0.000001093,
        }
    }
}

/// Calculate DOTS score for a given lift and bodyweight
pub fn calculate_dots_score(lift_kg: f64, bodyweight_kg: f64) -> f64 {
    let coeffs = DotsCoefficients::default();
    
    let denominator = coeffs.a + 
        coeffs.b * bodyweight_kg +
        coeffs.c * bodyweight_kg.powi(2) +
        coeffs.d * bodyweight_kg.powi(3) +
        coeffs.e * bodyweight_kg.powi(4);

    lift_kg * 500.0 / denominator
}

/// Create Polars expression for calculating DOTS scores
pub fn calculate_dots_expr(lift_col: &str, output_col: &str) -> Expr {
    let coeffs = DotsCoefficients::default();
    
    // DOTS = Total × (500 / (A + B × BW + C × BW² + D × BW³ + E × BW⁴))
    col(lift_col) * lit(500.0) / 
    (lit(coeffs.a) + 
     lit(coeffs.b) * col("BodyweightKg") +
     lit(coeffs.c) * col("BodyweightKg").pow(2) +
     lit(coeffs.d) * col("BodyweightKg").pow(3) +
     lit(coeffs.e) * col("BodyweightKg").pow(4))
    .alias(output_col)
}

/// Calculate weight class based on bodyweight and sex
pub fn calculate_weight_class_expr() -> Expr {
    when(col("Sex").eq(lit("M")))
        .then(
            when(col("BodyweightKg").lt_eq(59.0)).then(lit("59kg"))
            .when(col("BodyweightKg").lt_eq(66.0)).then(lit("66kg"))
            .when(col("BodyweightKg").lt_eq(74.0)).then(lit("74kg"))
            .when(col("BodyweightKg").lt_eq(83.0)).then(lit("83kg"))
            .when(col("BodyweightKg").lt_eq(93.0)).then(lit("93kg"))
            .when(col("BodyweightKg").lt_eq(105.0)).then(lit("105kg"))
            .when(col("BodyweightKg").lt_eq(120.0)).then(lit("120kg"))
            .otherwise(lit("120kg+"))
        )
        .otherwise(
            when(col("BodyweightKg").lt_eq(47.0)).then(lit("47kg"))
            .when(col("BodyweightKg").lt_eq(52.0)).then(lit("52kg"))
            .when(col("BodyweightKg").lt_eq(57.0)).then(lit("57kg"))
            .when(col("BodyweightKg").lt_eq(63.0)).then(lit("63kg"))
            .when(col("BodyweightKg").lt_eq(69.0)).then(lit("69kg"))
            .when(col("BodyweightKg").lt_eq(76.0)).then(lit("76kg"))
            .when(col("BodyweightKg").lt_eq(84.0)).then(lit("84kg"))
            .otherwise(lit("84kg+"))
        )
        .alias("WeightClassKg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dots_calculation() {
        // Test with some realistic values
        let male_100kg_500kg_total = calculate_dots_score(500.0, 100.0);
        let female_60kg_300kg_total = calculate_dots_score(300.0, 60.0);
        
        // DOTS scores should be reasonable (typically 300-600 for competitive lifters)
        assert!(male_100kg_500kg_total > 200.0 && male_100kg_500kg_total < 800.0);
        assert!(female_60kg_300kg_total > 200.0 && female_60kg_300kg_total < 800.0);
    }
    
    #[test]
    fn test_same_dots_different_bodyweights() {
        // Higher absolute weight should be needed for heavier lifters to achieve same DOTS
        let light_lifter_dots = calculate_dots_score(400.0, 70.0);
        let heavy_lifter_dots = calculate_dots_score(500.0, 120.0);
        
        // Should be roughly similar DOTS scores
        let difference = (light_lifter_dots - heavy_lifter_dots).abs();
        assert!(difference < 50.0); // Allow some reasonable difference
    }
}