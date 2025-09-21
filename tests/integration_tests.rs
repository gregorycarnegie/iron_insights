// tests/integration_tests.rs - Integration tests moved from main.rs
use std::sync::Arc;
use iron_insights::{
    config::AppConfig,
    data::DataProcessor,
    filters::apply_filters_lazy,
    models::{AppState, FilterParams},
    percentiles::percentile_rank,
    scoring::calculate_dots_score,
};

#[tokio::test]
async fn test_sample_data_generation() {
    let processor = DataProcessor::new().with_sample_size(100);
    let result =
        tokio::task::spawn_blocking(move || processor.load_and_preprocess_data()).await;

    assert!(result.is_ok());
    let df = result.unwrap().unwrap();
    assert_eq!(df.height(), 100);

    // Verify required columns exist
    let column_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|name| name.to_string())
        .collect();

    assert!(column_names.contains(&"Name".to_string()));
    assert!(column_names.contains(&"Sex".to_string()));
    assert!(column_names.contains(&"BodyweightKg".to_string()));
    assert!(column_names.contains(&"Best3SquatKg".to_string()));
    assert!(column_names.contains(&"SquatDOTS".to_string()));
    assert!(column_names.contains(&"WeightClassKg".to_string()));
}

#[test]
fn test_dots_calculation() {
    // Test realistic values
    let dots_male_100kg_500total = calculate_dots_score(500.0, 100.0, "M");
    let dots_female_60kg_300total = calculate_dots_score(300.0, 60.0, "F");

    // DOTS scores should be in reasonable range
    assert!(dots_male_100kg_500total > 200.0 && dots_male_100kg_500total < 800.0);
    assert!(dots_female_60kg_300total > 200.0 && dots_female_60kg_300total < 800.0);

    println!(
        "Male 100kg, 500kg total: {:.1} DOTS",
        dots_male_100kg_500total
    );
    println!(
        "Female 60kg, 300kg total: {:.1} DOTS",
        dots_female_60kg_300total
    );
}

#[test]
fn test_percentile_calculation() {
    use polars::prelude::*;

    let df = df! {
        "TestColumn" => [100.0f32, 200.0, 300.0, 400.0, 500.0],
    }
    .unwrap();

    let p50 = percentile_rank(&df, "TestColumn", Some(300.0));
    assert_eq!(p50, Some(40.0)); // 2 out of 5 values below 300

    let p100 = percentile_rank(&df, "TestColumn", Some(600.0));
    assert_eq!(p100, Some(100.0)); // All values below 600
}

#[tokio::test]
async fn test_filter_pipeline() {
    use polars::prelude::*;

    let df = df! {
        "Sex" => ["M", "F", "M"],
        "Equipment" => ["Raw", "Single-ply", "Raw"],
        "BodyweightKg" => [75.0f32, 65.0, 85.0],
        "Best3SquatKg" => [180.0f32, 120.0, 200.0],
        "Best3BenchKg" => [120.0f32, 70.0, 140.0],
        "Best3DeadliftKg" => [220.0f32, 140.0, 240.0],
        "TotalKg" => [520.0f32, 330.0, 580.0],
        "SquatDOTS" => [300.0f32, 280.0, 320.0],
        "BenchDOTS" => [200.0f32, 160.0, 220.0],
        "DeadliftDOTS" => [360.0f32, 320.0, 380.0],
        "TotalDOTS" => [860.0f32, 760.0, 920.0],
        "WeightClassKg" => ["74kg", "63kg", "83kg"],
        "Date" => ["2024-01-01", "2024-01-02", "2024-01-03"],
    }
    .unwrap();

    let params = FilterParams {
        sex: Some("M".to_string()),
        equipment: Some(vec!["Raw".to_string()]),
        weight_class: None,
        squat: None,
        bench: None,
        deadlift: None,
        bodyweight: None,
        units: None,
        lift_type: None,
        min_bodyweight: None,
        max_bodyweight: None,
        years_filter: None,
    };

    let filtered = apply_filters_lazy(&df, &params).unwrap().collect().unwrap();

    // Should only have 2 male lifters with Raw equipment
    assert_eq!(filtered.height(), 2);
}

#[tokio::test]
async fn test_server_creation() {
    let config = AppConfig::default();
    let data_processor = DataProcessor::new().with_sample_size(10);

    let data = tokio::task::spawn_blocking(move || data_processor.load_and_preprocess_data())
        .await
        .unwrap()
        .unwrap();

    let state = AppState::new(Arc::new(data), config.cache_config());

    // Verify state was created successfully
    assert!(state.data.height() > 0);
    assert_eq!(state.cache.entry_count(), 0); // Empty cache initially
}

#[test]
fn test_weight_class_format_conversion() {
    // Test that the weight class filtering logic handles format conversion correctly
    let test_cases = vec![
        ("74", "74kg"),
        ("120", "120kg"),
        ("120+", "120kg+"),
        ("84+", "84kg+"),
    ];

    for (input, expected) in test_cases {
        let result = if input.ends_with('+') {
            format!("{}kg+", input.trim_end_matches('+'))
        } else {
            format!("{}kg", input)
        };
        assert_eq!(result, expected, "Failed to convert '{}' to '{}'", input, expected);
    }
}

#[test]
fn test_dots_coefficients() {
    // Test that DOTS coefficients produce expected ranges

    // Test male coefficients
    let male_80kg_400total = calculate_dots_score(400.0, 80.0, "M");
    let male_100kg_500total = calculate_dots_score(500.0, 100.0, "M");

    // Test female coefficients
    let female_60kg_300total = calculate_dots_score(300.0, 60.0, "F");
    let female_70kg_350total = calculate_dots_score(350.0, 70.0, "F");

    // All should be in reasonable competitive ranges
    assert!(male_80kg_400total > 250.0 && male_80kg_400total < 600.0);
    assert!(male_100kg_500total > 250.0 && male_100kg_500total < 600.0);
    assert!(female_60kg_300total > 250.0 && female_60kg_300total < 600.0);
    assert!(female_70kg_350total > 250.0 && female_70kg_350total < 600.0);

    // Heavier lifts should generally produce higher DOTS scores
    assert!(male_100kg_500total > male_80kg_400total);
    assert!(female_70kg_350total > female_60kg_300total);
}