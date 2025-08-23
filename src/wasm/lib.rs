use wasm_bindgen::prelude::*;

/// Strength levels enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrengthLevel {
    Beginner = 0,
    Novice = 1,
    Intermediate = 2,
    Advanced = 3,
    Elite = 4,
    WorldClass = 5,
}

impl StrengthLevel {
    pub fn to_string(&self) -> String {
        match self {
            StrengthLevel::Beginner => "Beginner".to_string(),
            StrengthLevel::Novice => "Novice".to_string(),
            StrengthLevel::Intermediate => "Intermediate".to_string(),
            StrengthLevel::Advanced => "Advanced".to_string(),
            StrengthLevel::Elite => "Elite".to_string(),
            StrengthLevel::WorldClass => "World Class".to_string(),
        }
    }
}

/// DOTS coefficients for males
const DOTS_MALE_A: f64 = -307.75076;
const DOTS_MALE_B: f64 = 24.0900756;
const DOTS_MALE_C: f64 = -0.1918759221;
const DOTS_MALE_D: f64 = 0.0007391293;
const DOTS_MALE_E: f64 = -0.000001093;

/// DOTS coefficients for females
const DOTS_FEMALE_A: f64 = -57.96288;
const DOTS_FEMALE_B: f64 = 13.6175032;
const DOTS_FEMALE_C: f64 = -0.1126655495;
const DOTS_FEMALE_D: f64 = 0.0005158568;
const DOTS_FEMALE_E: f64 = -0.0000010706;

/// Wilks 2020 coefficients for males
const WILKS_MALE_A: f64 = 47.46178854;
const WILKS_MALE_B: f64 = 8.472061379;
const WILKS_MALE_C: f64 = 0.07369410346;
const WILKS_MALE_D: f64 = -0.001395833811;
const WILKS_MALE_E: f64 = 7.07665973070743e-06;
const WILKS_MALE_F: f64 = -1.20804336482315e-08;

/// Wilks 2020 coefficients for females  
const WILKS_FEMALE_A: f64 = -125.4255398;
const WILKS_FEMALE_B: f64 = 13.71219419;
const WILKS_FEMALE_C: f64 = -0.03307250631;
const WILKS_FEMALE_D: f64 = -0.001050400051;
const WILKS_FEMALE_E: f64 = 9.38773881462799e-06;
const WILKS_FEMALE_F: f64 = -2.3334613884954e-08;

/// IPF GL coefficients for males
const IPF_GL_MALE_A: f64 = 1199.72839;
const IPF_GL_MALE_B: f64 = 1025.18162;
const IPF_GL_MALE_C: f64 = 0.00921;

/// IPF GL coefficients for females
const IPF_GL_FEMALE_A: f64 = 610.32796;
const IPF_GL_FEMALE_B: f64 = 1045.59282;
const IPF_GL_FEMALE_C: f64 = 0.03048;

/// Calculate DOTS score for a given lift and bodyweight (legacy function - defaults to male)
#[wasm_bindgen]
pub fn calculate_dots(lift_kg: f64, bodyweight_kg: f64) -> f64 {
    calculate_dots_with_gender(lift_kg, bodyweight_kg, true)
}

/// Calculate DOTS score for a given lift, bodyweight, and gender
#[wasm_bindgen]
pub fn calculate_dots_with_gender(lift_kg: f64, bodyweight_kg: f64, is_male: bool) -> f64 {
    if is_male {
        let denominator = DOTS_MALE_A + 
            DOTS_MALE_B * bodyweight_kg +
            DOTS_MALE_C * bodyweight_kg.powi(2) +
            DOTS_MALE_D * bodyweight_kg.powi(3) +
            DOTS_MALE_E * bodyweight_kg.powi(4);
        lift_kg * 500.0 / denominator
    } else {
        let denominator = DOTS_FEMALE_A + 
            DOTS_FEMALE_B * bodyweight_kg +
            DOTS_FEMALE_C * bodyweight_kg.powi(2) +
            DOTS_FEMALE_D * bodyweight_kg.powi(3) +
            DOTS_FEMALE_E * bodyweight_kg.powi(4);
        lift_kg * 500.0 / denominator
    }
}

/// Calculate Wilks 2020 score for a given lift, bodyweight, and sex
#[wasm_bindgen]
pub fn calculate_wilks(lift_kg: f64, bodyweight_kg: f64, is_male: bool) -> f64 {
    let bw = bodyweight_kg;
    
    if is_male {
        let denominator = WILKS_MALE_A + 
            WILKS_MALE_B * bw +
            WILKS_MALE_C * bw.powi(2) +
            WILKS_MALE_D * bw.powi(3) +
            WILKS_MALE_E * bw.powi(4) +
            WILKS_MALE_F * bw.powi(5);
        lift_kg * 600.0 / denominator
    } else {
        let denominator = WILKS_FEMALE_A + 
            WILKS_FEMALE_B * bw +
            WILKS_FEMALE_C * bw.powi(2) +
            WILKS_FEMALE_D * bw.powi(3) +
            WILKS_FEMALE_E * bw.powi(4) +
            WILKS_FEMALE_F * bw.powi(5);
        lift_kg * 600.0 / denominator
    }
}

/// Calculate IPF GL Points (Good-Lift Points) for a given lift, bodyweight, and sex
#[wasm_bindgen]
pub fn calculate_ipf_gl_points(lift_kg: f64, bodyweight_kg: f64, is_male: bool) -> f64 {
    if is_male {
        IPF_GL_MALE_A / (IPF_GL_MALE_B - IPF_GL_MALE_C * bodyweight_kg) * lift_kg
    } else {
        IPF_GL_FEMALE_A / (IPF_GL_FEMALE_B - IPF_GL_FEMALE_C * bodyweight_kg) * lift_kg
    }
}

/// Calculate strength level based on percentile
#[wasm_bindgen]
pub fn calculate_strength_level_from_percentile(percentile: f64) -> String {
    let level = if percentile < 20.0 {
        StrengthLevel::Beginner
    } else if percentile < 40.0 {
        StrengthLevel::Novice
    } else if percentile < 60.0 {
        StrengthLevel::Intermediate
    } else if percentile < 80.0 {
        StrengthLevel::Advanced
    } else if percentile < 95.0 {
        StrengthLevel::Elite
    } else {
        StrengthLevel::WorldClass
    };
    level.to_string()
}

/// Legacy strength level calculation (for backward compatibility)
#[wasm_bindgen]
pub fn calculate_strength_level(dots_score: f64) -> String {
    // This is a fallback - in practice, percentiles should be used
    calculate_strength_level_for_lift(dots_score, "total")
}

/// Legacy lift-specific strength level calculations (for backward compatibility)
#[wasm_bindgen]
pub fn calculate_strength_level_for_lift(dots_score: f64, lift_type: &str) -> String {
    // This is a fallback - in practice, percentiles should be used
    match lift_type {
        "squat" => {
            if dots_score < 150.0 {
                "Beginner".to_string()
            } else if dots_score < 225.0 {
                "Novice".to_string()
            } else if dots_score < 300.0 {
                "Intermediate".to_string()
            } else if dots_score < 375.0 {
                "Advanced".to_string()
            } else if dots_score < 450.0 {
                "Elite".to_string()
            } else {
                "World Class".to_string()
            }
        },
        "bench" => {
            if dots_score < 100.0 {
                "Beginner".to_string()
            } else if dots_score < 150.0 {
                "Novice".to_string()
            } else if dots_score < 200.0 {
                "Intermediate".to_string()
            } else if dots_score < 250.0 {
                "Advanced".to_string()
            } else if dots_score < 300.0 {
                "Elite".to_string()
            } else {
                "World Class".to_string()
            }
        },
        "deadlift" => {
            if dots_score < 175.0 {
                "Beginner".to_string()
            } else if dots_score < 262.5 {
                "Novice".to_string()
            } else if dots_score < 350.0 {
                "Intermediate".to_string()
            } else if dots_score < 437.5 {
                "Advanced".to_string()
            } else if dots_score < 525.0 {
                "Elite".to_string()
            } else {
                "World Class".to_string()
            }
        },
        _ => { // "total" and default
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

/// Combined function to calculate DOTS and strength level (defaults to total)
#[wasm_bindgen]
pub fn calculate_dots_and_level(lift_kg: f64, bodyweight_kg: f64) -> JsValue {
    calculate_dots_and_level_for_lift(lift_kg, bodyweight_kg, "total")
}

/// Combined function to calculate DOTS and lift-specific strength level
#[wasm_bindgen]
pub fn calculate_dots_and_level_for_lift(lift_kg: f64, bodyweight_kg: f64, lift_type: &str) -> JsValue {
    calculate_dots_and_level_for_lift_with_gender(lift_kg, bodyweight_kg, true, lift_type)
}

/// Combined function to calculate DOTS and lift-specific strength level with gender
#[wasm_bindgen]
pub fn calculate_dots_and_level_for_lift_with_gender(lift_kg: f64, bodyweight_kg: f64, is_male: bool, lift_type: &str) -> JsValue {
    let dots = calculate_dots_with_gender(lift_kg, bodyweight_kg, is_male);
    let level = calculate_strength_level_for_lift(dots, lift_type);
    let color = get_strength_level_color(&level);
    
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from("dots"), &JsValue::from(dots)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("level"), &JsValue::from(level)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("color"), &JsValue::from(color)).unwrap();
    
    result.into()
}

/// Combined function to calculate all scoring systems at once
#[wasm_bindgen]
pub fn calculate_all_scores(lift_kg: f64, bodyweight_kg: f64, is_male: bool, percentile: f64) -> JsValue {
    let dots = calculate_dots_with_gender(lift_kg, bodyweight_kg, is_male);
    let wilks = calculate_wilks(lift_kg, bodyweight_kg, is_male);
    let ipf_gl_points = calculate_ipf_gl_points(lift_kg, bodyweight_kg, is_male);
    let level = calculate_strength_level_from_percentile(percentile);
    let color = get_strength_level_color(&level);
    
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from("dots"), &JsValue::from(dots)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("wilks"), &JsValue::from(wilks)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("ipf_gl_points"), &JsValue::from(ipf_gl_points)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("level"), &JsValue::from(level)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("color"), &JsValue::from(color)).unwrap();
    
    result.into()
}