use iron_insights::scoring::*;
use polars::prelude::*;

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
        .with_columns([calculate_ipf_weight_class_expr()])
        .collect();

    assert!(result.is_ok());
    let df_with_wc = result.unwrap();

    // Check that the column was created
    assert!(df_with_wc.column("IPFWeightClassKg").is_ok());

    let wc_col = df_with_wc.column("IPFWeightClassKg").unwrap();
    let wc_values: Vec<&str> = wc_col.str().unwrap().into_no_null_iter().collect();

    // Check expected weight classes
    // 75kg male -> 83kg class, 60kg female -> 63kg class, 105kg male -> 105kg class, 55kg female -> 57kg class
    assert_eq!(wc_values, vec!["83kg", "63kg", "105kg", "57kg"]);

    println!("Weight class values: {:?}", wc_values);
}
