use wasm_bindgen::prelude::*;

/// DOTS coefficients (gender-neutral)
const DOTS_A: f64 = -307.75076;
const DOTS_B: f64 = 24.0900756;
const DOTS_C: f64 = -0.1918759221;
const DOTS_D: f64 = 0.0007391293;
const DOTS_E: f64 = -0.000001093;

/// Calculate DOTS score for a given lift and bodyweight
#[wasm_bindgen]
pub fn calculate_dots(lift_kg: f64, bodyweight_kg: f64) -> f64 {
    let denominator = DOTS_A + 
        DOTS_B * bodyweight_kg +
        DOTS_C * bodyweight_kg.powi(2) +
        DOTS_D * bodyweight_kg.powi(3) +
        DOTS_E * bodyweight_kg.powi(4);

    lift_kg * 500.0 / denominator
}

/// Strength levels based on DOTS scores
#[wasm_bindgen]
pub fn calculate_strength_level(dots_score: f64) -> String {
    if dots_score < 200.0 {
        "Beginner".to_string()
    } else if dots_score < 300.0 {
        "Novice".to_string()
    } else if dots_score < 400.0 {
        "Intermediate".to_string()
    } else if dots_score < 500.0 {
        "Advanced".to_string()
    } else if dots_score < 600.0 {
        "Elite".to_string()
    } else {
        "World Class".to_string()
    }
}

/// Get strength level color for UI styling
#[wasm_bindgen]
pub fn get_strength_level_color(level: &str) -> String {
    match level {
        "Beginner" => "#6c757d".to_string(),
        "Novice" => "#28a745".to_string(),
        "Intermediate" => "#17a2b8".to_string(),
        "Advanced" => "#ffc107".to_string(),
        "Elite" => "#fd7e14".to_string(),
        "World Class" => "#dc3545".to_string(),
        _ => "#6c757d".to_string(),
    }
}

/// Combined function to calculate DOTS and strength level
#[wasm_bindgen]
pub fn calculate_dots_and_level(lift_kg: f64, bodyweight_kg: f64) -> JsValue {
    let dots = calculate_dots(lift_kg, bodyweight_kg);
    let level = calculate_strength_level(dots);
    let color = get_strength_level_color(&level);
    
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from("dots"), &JsValue::from(dots)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("level"), &JsValue::from(level)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("color"), &JsValue::from(color)).unwrap();
    
    result.into()
}