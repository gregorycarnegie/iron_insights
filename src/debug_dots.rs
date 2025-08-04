// debug_dots.rs - Run with: cargo run --bin debug_dots
use iron_insights::scoring::{calculate_dots_score, DotsCoefficients};
use polars::prelude::*;

fn main() -> PolarsResult<()> {
    println!("🔍 DOTS Debugging Script");
    println!("========================");
    
    // Test basic DOTS calculations
    println!("\n1. Testing basic DOTS calculations:");
    test_dots_calculations();
    
    // Test with sample data
    println!("\n2. Testing with sample DataFrame:");
    test_sample_dataframe()?;
    
    // Test edge cases
    println!("\n3. Testing edge cases:");
    test_edge_cases();
    
    // Test formula components
    test_dots_formula_components();
    
    Ok(())
}

fn test_dots_calculations() {
    let test_cases = vec![
        (180.0, 75.0, "Male 75kg, 180kg squat"),
        (300.0, 60.0, "Female 60kg, 300kg total"),
        (500.0, 100.0, "Male 100kg, 500kg total"),
        (120.0, 80.0, "80kg lifter, 120kg bench"),
        (200.0, 120.0, "Heavy lifter 120kg, 200kg deadlift"),
    ];
    
    for (lift, bodyweight, description) in test_cases {
        let dots = calculate_dots_score(lift, bodyweight);
        let is_valid = dots.is_finite() && dots > 0.0 && dots < 1000.0;
        let status = if is_valid { "✅" } else { "❌" };
        
        println!("  {} {}: DOTS = {:.2} (lift: {}kg, bw: {}kg)", 
                 status, description, dots, lift, bodyweight);
        
        if !is_valid {
            println!("    ⚠️  Invalid DOTS score detected!");
        }
    }
}

fn test_sample_dataframe() -> PolarsResult<()> {
    use iron_insights::scoring::{calculate_dots_expr, calculate_weight_class_expr};
    
    // Create sample data
    let df = df! {
        "Name" => ["Lifter1", "Lifter2", "Lifter3", "Lifter4", "Lifter5"],
        "Sex" => ["M", "F", "M", "F", "M"],
        "Equipment" => ["Raw", "Raw", "Wraps", "Raw", "Single-ply"],
        "BodyweightKg" => [75.0f32, 60.0, 100.0, 55.0, 90.0],
        "Best3SquatKg" => [180.0f32, 120.0, 250.0, 110.0, 220.0],
        "Best3BenchKg" => [120.0f32, 70.0, 180.0, 65.0, 150.0],
        "Best3DeadliftKg" => [220.0f32, 140.0, 280.0, 135.0, 250.0],
        "TotalKg" => [520.0f32, 330.0, 710.0, 310.0, 620.0],
    }?;
    
    println!("  📊 Original DataFrame:");
    println!("{}", df);
    
    // Add DOTS calculations
    let df_with_dots = df
        .lazy()
        .with_columns([
            calculate_weight_class_expr(),
            calculate_dots_expr("Best3SquatKg", "SquatDOTS"),
            calculate_dots_expr("Best3BenchKg", "BenchDOTS"),
            calculate_dots_expr("Best3DeadliftKg", "DeadliftDOTS"),
            calculate_dots_expr("TotalKg", "TotalDOTS"),
        ])
        .collect()?;
    
    println!("\n  📈 DataFrame with DOTS:");
    println!("{}", df_with_dots);
    
    // Check DOTS data validity
    println!("\n  🔍 DOTS Data Validation:");
    for col_name in ["SquatDOTS", "BenchDOTS", "DeadliftDOTS", "TotalDOTS"] {
        if let Ok(column) = df_with_dots.column(col_name) {
            if let Ok(f32_series) = column.f32() {
                let values: Vec<f32> = f32_series.into_no_null_iter().collect();
                let valid_count = values.iter()
                    .filter(|&&x| x.is_finite() && x > 0.0 && x < 1000.0)
                    .count();
                let avg = if !values.is_empty() {
                    values.iter().sum::<f32>() / values.len() as f32
                } else { 0.0 };
                
                println!("    {} {}: {}/{} valid, avg: {:.1}", 
                         if valid_count == values.len() { "✅" } else { "❌" },
                         col_name, valid_count, values.len(), avg);
                
                // Show individual values for debugging
                for (i, &val) in values.iter().enumerate() {
                    let status = if val.is_finite() && val > 0.0 && val < 1000.0 { "✅" } else { "❌" };
                    println!("      {} Row {}: {:.2}", status, i, val);
                }
            }
        }
    }
    
    Ok(())
}

fn test_edge_cases() {
    let edge_cases = vec![
        (0.0, 75.0, "Zero lift"),
        (180.0, 0.0, "Zero bodyweight"),
        (-50.0, 75.0, "Negative lift"),
        (180.0, -75.0, "Negative bodyweight"),
        (f32::INFINITY, 75.0, "Infinite lift"),
        (180.0, f32::INFINITY, "Infinite bodyweight"),
        (f32::NAN, 75.0, "NaN lift"),
        (180.0, f32::NAN, "NaN bodyweight"),
        (1000000.0, 75.0, "Extremely high lift"),
        (180.0, 1000.0, "Extremely high bodyweight"),
        (180.0, 30.0, "Very low bodyweight"),
        (180.0, 300.0, "Very high bodyweight"),
    ];
    
    for (lift, bodyweight, description) in edge_cases {
        let dots = calculate_dots_score(lift, bodyweight);
        let is_reasonable = dots.is_finite() && dots > 0.0 && dots < 1000.0;
        let status = if is_reasonable { "✅" } else { "❌" };
        
        println!("  {} {}: DOTS = {:.2}", status, description, dots);
        
        if !is_reasonable {
            println!("    ⚠️  Edge case produces unreasonable DOTS score");
        }
    }
}

// Additional helper functions for comprehensive testing
fn test_dots_formula_components() {
    println!("\n4. Testing DOTS formula components:");
    
    let bodyweight = 75.0f32;
    let coeffs = iron_insights::scoring::DotsCoefficients::default();
    
    let bw2 = bodyweight.powi(2);
    let bw3 = bodyweight.powi(3);
    let bw4 = bodyweight.powi(4);
    
    let denominator = coeffs.a + 
        coeffs.b * bodyweight +
        coeffs.c * bw2 +
        coeffs.d * bw3 +
        coeffs.e * bw4;
    
    println!("  For {}kg bodyweight:", bodyweight);
    println!("    A term: {:.6}", coeffs.a);
    println!("    B term: {:.6}", coeffs.b * bodyweight);
    println!("    C term: {:.6}", coeffs.c * bw2);
    println!("    D term: {:.6}", coeffs.d * bw3);
    println!("    E term: {:.6}", coeffs.e * bw4);
    println!("    Denominator: {:.6}", denominator);
    println!("    500/denominator: {:.6}", 500.0 / denominator);
    
    if denominator <= 0.0 || !denominator.is_finite() {
        println!("    ❌ PROBLEM: Invalid denominator!");
    } else {
        println!("    ✅ Denominator looks good");
    }
}