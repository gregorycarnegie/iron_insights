// Polars-specific scoring expressions
use iron_scoring::DotsCoefficients;
use polars::prelude::*;

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
