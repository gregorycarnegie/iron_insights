use iron_insights::percentiles::*;
use polars::prelude::*;

#[test]
fn test_percentile_rank() {
    let df = df! {
        "values" => [100.0f32, 150.0, 200.0, 250.0, 300.0],
    }
    .unwrap();

    // Test middle value
    let p = percentile_rank(&df, "values", Some(200.0));
    assert_eq!(p, Some(40.0)); // 2 out of 5 values are below 200

    // Test lowest value
    let p = percentile_rank(&df, "values", Some(50.0));
    assert_eq!(p, Some(0.0)); // No values below 50

    // Test highest value
    let p = percentile_rank(&df, "values", Some(350.0));
    assert_eq!(p, Some(100.0)); // All values below 350
}

#[test]
fn test_percentile_with_invalid_values() {
    let df = df! {
        "values" => [100.0f32, f32::NAN, 200.0, f32::INFINITY, 300.0, -50.0, 0.0],
    }
    .unwrap();

    // Should only count valid positive finite values (100, 200, 300)
    let p = percentile_rank(&df, "values", Some(250.0));
    assert_eq!(p, Some(67.0)); // 2 out of 3 valid values are below 250
}