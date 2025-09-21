// scoring.rs - Fixed with proper column aliases
use polars::prelude::*;

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

/// Create Polars expression for calculating gender-specific DOTS scores
pub fn calculate_dots_expr(lift_col: &str, output_col: &str) -> Expr {
    let male_coeffs = DotsCoefficients::male();
    let female_coeffs = DotsCoefficients::female();

    // Create conditional DOTS calculation based on sex
    when(col("Sex").eq(lit("M")))
        .then(
            // Male DOTS calculation
            col(lift_col) * lit(500.0)
                / (lit(male_coeffs.a)
                    + lit(male_coeffs.b) * col("BodyweightKg")
                    + lit(male_coeffs.c) * col("BodyweightKg").pow(2)
                    + lit(male_coeffs.d) * col("BodyweightKg").pow(3)
                    + lit(male_coeffs.e) * col("BodyweightKg").pow(4)),
        )
        .otherwise(
            // Female DOTS calculation
            col(lift_col) * lit(500.0)
                / (lit(female_coeffs.a)
                    + lit(female_coeffs.b) * col("BodyweightKg")
                    + lit(female_coeffs.c) * col("BodyweightKg").pow(2)
                    + lit(female_coeffs.d) * col("BodyweightKg").pow(3)
                    + lit(female_coeffs.e) * col("BodyweightKg").pow(4)),
        )
        .alias(output_col)
}

/// Calculate weight class based on bodyweight and sex
pub fn calculate_weight_class_expr() -> Expr {
    when(col("Sex").eq(lit("M")))
        .then(
            when(col("BodyweightKg").lt_eq(59.0))
                .then(lit("59kg"))
                .when(col("BodyweightKg").lt_eq(66.0))
                .then(lit("66kg"))
                .when(col("BodyweightKg").lt_eq(74.0))
                .then(lit("74kg"))
                .when(col("BodyweightKg").lt_eq(83.0))
                .then(lit("83kg"))
                .when(col("BodyweightKg").lt_eq(93.0))
                .then(lit("93kg"))
                .when(col("BodyweightKg").lt_eq(105.0))
                .then(lit("105kg"))
                .when(col("BodyweightKg").lt_eq(120.0))
                .then(lit("120kg"))
                .otherwise(lit("120kg+")),
        )
        .otherwise(
            when(col("BodyweightKg").lt_eq(47.0))
                .then(lit("47kg"))
                .when(col("BodyweightKg").lt_eq(52.0))
                .then(lit("52kg"))
                .when(col("BodyweightKg").lt_eq(57.0))
                .then(lit("57kg"))
                .when(col("BodyweightKg").lt_eq(63.0))
                .then(lit("63kg"))
                .when(col("BodyweightKg").lt_eq(69.0))
                .then(lit("69kg"))
                .when(col("BodyweightKg").lt_eq(76.0))
                .then(lit("76kg"))
                .when(col("BodyweightKg").lt_eq(84.0))
                .then(lit("84kg"))
                .otherwise(lit("84kg+")),
        )
        .alias("WeightClassKg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dots_calculation() {
        // Test with some realistic values
        let male_100kg_500kg_total = calculate_dots_score(500.0, 100.0, "M");
        let female_60kg_300kg_total = calculate_dots_score(300.0, 60.0, "F");

        // DOTS scores should be reasonable (typically 300-600 for competitive lifters)
        assert!(male_100kg_500kg_total > 200.0 && male_100kg_500kg_total < 800.0);
        assert!(female_60kg_300kg_total > 200.0 && female_60kg_300kg_total < 800.0);

        println!(
            "Male 100kg, 500kg total: {:.1} DOTS",
            male_100kg_500kg_total
        );
        println!(
            "Female 60kg, 300kg total: {:.1} DOTS",
            female_60kg_300kg_total
        );

        // Test that male and female have different scores for same lift/bodyweight
        let same_lift_male = calculate_dots_score(400.0, 80.0, "M");
        let same_lift_female = calculate_dots_score(400.0, 80.0, "F");
        assert!((same_lift_male - same_lift_female).abs() > 10.0); // Should be different

        println!(
            "Same lift (400kg) at 80kg - Male: {:.1}, Female: {:.1}",
            same_lift_male, same_lift_female
        );
    }

    #[test]
    fn test_same_dots_different_bodyweights() {
        // Higher absolute weight should be needed for heavier lifters to achieve same DOTS
        let light_lifter_dots = calculate_dots_score(400.0, 70.0, "M");
        let heavy_lifter_dots = calculate_dots_score(500.0, 120.0, "M");

        // Should be roughly similar DOTS scores
        let difference = (light_lifter_dots - heavy_lifter_dots).abs();
        assert!(difference < 50.0); // Allow some reasonable difference

        println!("Light lifter (70kg, 400kg): {:.1} DOTS", light_lifter_dots);
        println!("Heavy lifter (120kg, 500kg): {:.1} DOTS", heavy_lifter_dots);
    }

    #[test]
    fn test_dots_expr_creation() {
        // Test that the expression can be created without panicking
        let expr = calculate_dots_expr("Best3SquatKg", "SquatDOTS");

        // Basic test with sample data - needs Sex column for DOTS calculation
        let df = df! {
            "Best3SquatKg" => [180.0f32, 200.0, 220.0],
            "BodyweightKg" => [75.0f32, 85.0, 95.0],
            "Sex" => ["M", "M", "M"],
        }
        .unwrap();

        let result = df.lazy().with_columns([expr]).collect();

        assert!(result.is_ok());
        let df_with_dots = result.unwrap();

        // Check that the column was created
        assert!(df_with_dots.column("SquatDOTS").is_ok());

        // Check that values are reasonable
        let dots_col = df_with_dots.column("SquatDOTS").unwrap();
        let dots_values: Vec<f32> = dots_col.f32().unwrap().into_no_null_iter().collect();

        for &dots in &dots_values {
            assert!(dots > 0.0 && dots < 1000.0 && dots.is_finite());
        }

        println!("DOTS values from expression: {:?}", dots_values);
    }

    #[test]
    fn test_weight_class_expr() {
        let df = df! {
            "Sex" => ["M", "F", "M", "F"],
            "BodyweightKg" => [75.0f32, 60.0, 105.0, 55.0],
        }
        .unwrap();

        let result = df
            .lazy()
            .with_columns([calculate_weight_class_expr()])
            .collect();

        assert!(result.is_ok());
        let df_with_wc = result.unwrap();

        // Check that the column was created
        assert!(df_with_wc.column("WeightClassKg").is_ok());

        let wc_col = df_with_wc.column("WeightClassKg").unwrap();
        let wc_values: Vec<&str> = wc_col.str().unwrap().into_no_null_iter().collect();

        // Check expected weight classes
        // 75kg male -> 83kg class, 60kg female -> 63kg class, 105kg male -> 105kg class, 55kg female -> 57kg class
        assert_eq!(wc_values, vec!["83kg", "63kg", "105kg", "57kg"]);

        println!("Weight class values: {:?}", wc_values);
    }
}
