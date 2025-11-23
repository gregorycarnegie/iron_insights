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

/// Calculate IPF weight class based on bodyweight and sex
pub fn calculate_ipf_weight_class_expr() -> Expr {
    when(col("Sex").eq(lit("M")))
        .then(
            when(col("BodyweightKg").lt_eq(53.0))
                .then(lit("53kg"))
                .when(col("BodyweightKg").lt_eq(59.0))
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
            when(col("BodyweightKg").lt_eq(43.0))
                .then(lit("43kg"))
                .when(col("BodyweightKg").lt_eq(47.0))
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
        .alias("IPFWeightClassKg")
}

/// Calculate Para powerlifting weight class based on bodyweight and sex
pub fn calculate_para_weight_class_expr() -> Expr {
    when(col("Sex").eq(lit("M")))
        .then(
            when(col("BodyweightKg").lt_eq(49.0))
                .then(lit("49kg"))
                .when(col("BodyweightKg").lt_eq(54.0))
                .then(lit("54kg"))
                .when(col("BodyweightKg").lt_eq(59.0))
                .then(lit("59kg"))
                .when(col("BodyweightKg").lt_eq(65.0))
                .then(lit("65kg"))
                .when(col("BodyweightKg").lt_eq(72.0))
                .then(lit("72kg"))
                .when(col("BodyweightKg").lt_eq(80.0))
                .then(lit("80kg"))
                .when(col("BodyweightKg").lt_eq(88.0))
                .then(lit("88kg"))
                .when(col("BodyweightKg").lt_eq(97.0))
                .then(lit("97kg"))
                .when(col("BodyweightKg").lt_eq(107.0))
                .then(lit("107kg"))
                .otherwise(lit("107kg+")),
        )
        .otherwise(
            when(col("BodyweightKg").lt_eq(41.0))
                .then(lit("41kg"))
                .when(col("BodyweightKg").lt_eq(45.0))
                .then(lit("45kg"))
                .when(col("BodyweightKg").lt_eq(50.0))
                .then(lit("50kg"))
                .when(col("BodyweightKg").lt_eq(55.0))
                .then(lit("55kg"))
                .when(col("BodyweightKg").lt_eq(61.0))
                .then(lit("61kg"))
                .when(col("BodyweightKg").lt_eq(67.0))
                .then(lit("67kg"))
                .when(col("BodyweightKg").lt_eq(73.0))
                .then(lit("73kg"))
                .when(col("BodyweightKg").lt_eq(79.0))
                .then(lit("79kg"))
                .when(col("BodyweightKg").lt_eq(86.0))
                .then(lit("86kg"))
                .otherwise(lit("86kg+")),
        )
        .alias("ParaWeightClassKg")
}

/// Calculate WP weight class based on bodyweight and sex
pub fn calculate_wp_weight_class_expr() -> Expr {
    when(col("Sex").eq(lit("M")))
        .then(
            when(col("BodyweightKg").lt_eq(62.0))
                .then(lit("62kg"))
                .when(col("BodyweightKg").lt_eq(69.0))
                .then(lit("69kg"))
                .when(col("BodyweightKg").lt_eq(77.0))
                .then(lit("77kg"))
                .when(col("BodyweightKg").lt_eq(85.0))
                .then(lit("85kg"))
                .when(col("BodyweightKg").lt_eq(94.0))
                .then(lit("94kg"))
                .when(col("BodyweightKg").lt_eq(105.0))
                .then(lit("105kg"))
                .when(col("BodyweightKg").lt_eq(120.0))
                .then(lit("120kg"))
                .otherwise(lit("120kg+")),
        )
        .otherwise(
            when(col("BodyweightKg").lt_eq(48.0))
                .then(lit("48kg"))
                .when(col("BodyweightKg").lt_eq(53.0))
                .then(lit("53kg"))
                .when(col("BodyweightKg").lt_eq(58.0))
                .then(lit("58kg"))
                .when(col("BodyweightKg").lt_eq(64.0))
                .then(lit("64kg"))
                .when(col("BodyweightKg").lt_eq(72.0))
                .then(lit("72kg"))
                .when(col("BodyweightKg").lt_eq(84.0))
                .then(lit("84kg"))
                .when(col("BodyweightKg").lt_eq(100.0))
                .then(lit("100kg"))
                .otherwise(lit("100kg+")),
        )
        .alias("WPWeightClassKg")
}
