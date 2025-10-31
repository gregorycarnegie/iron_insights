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
        let denominator = DOTS_MALE_A
            + DOTS_MALE_B * bodyweight_kg
            + DOTS_MALE_C * bodyweight_kg.powi(2)
            + DOTS_MALE_D * bodyweight_kg.powi(3)
            + DOTS_MALE_E * bodyweight_kg.powi(4);
        lift_kg * 500.0 / denominator
    } else {
        let denominator = DOTS_FEMALE_A
            + DOTS_FEMALE_B * bodyweight_kg
            + DOTS_FEMALE_C * bodyweight_kg.powi(2)
            + DOTS_FEMALE_D * bodyweight_kg.powi(3)
            + DOTS_FEMALE_E * bodyweight_kg.powi(4);
        lift_kg * 500.0 / denominator
    }
}

/// Calculate Wilks 2020 score for a given lift, bodyweight, and sex
#[wasm_bindgen]
pub fn calculate_wilks(lift_kg: f64, bodyweight_kg: f64, is_male: bool) -> f64 {
    let bw = bodyweight_kg;

    if is_male {
        let denominator = WILKS_MALE_A
            + WILKS_MALE_B * bw
            + WILKS_MALE_C * bw.powi(2)
            + WILKS_MALE_D * bw.powi(3)
            + WILKS_MALE_E * bw.powi(4)
            + WILKS_MALE_F * bw.powi(5);
        lift_kg * 600.0 / denominator
    } else {
        let denominator = WILKS_FEMALE_A
            + WILKS_FEMALE_B * bw
            + WILKS_FEMALE_C * bw.powi(2)
            + WILKS_FEMALE_D * bw.powi(3)
            + WILKS_FEMALE_E * bw.powi(4)
            + WILKS_FEMALE_F * bw.powi(5);
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
    calculate_strength_level_for_lift_with_gender(dots_score, lift_type, true)
}

/// Gender-aware strength level calculation with realistic thresholds
#[wasm_bindgen]
pub fn calculate_strength_level_for_lift_with_gender(
    dots_score: f64,
    lift_type: &str,
    is_male: bool,
) -> String {
    match lift_type {
        "squat" => {
            let thresholds = if is_male {
                (61.0, 102.0, 132.0, 163.0, 187.0)
            } else {
                (58.0, 96.0, 125.0, 154.0, 177.0)
            };

            if dots_score < thresholds.0 {
                "Beginner".to_string()
            } else if dots_score < thresholds.1 {
                "Novice".to_string()
            } else if dots_score < thresholds.2 {
                "Intermediate".to_string()
            } else if dots_score < thresholds.3 {
                "Advanced".to_string()
            } else if dots_score < thresholds.4 {
                "Elite".to_string()
            } else {
                "World Class".to_string()
            }
        }
        "bench" => {
            let thresholds = if is_male {
                (41.0, 69.0, 89.0, 110.0, 127.0)
            } else {
                (39.0, 65.0, 85.0, 104.0, 120.0)
            };

            if dots_score < thresholds.0 {
                "Beginner".to_string()
            } else if dots_score < thresholds.1 {
                "Novice".to_string()
            } else if dots_score < thresholds.2 {
                "Intermediate".to_string()
            } else if dots_score < thresholds.3 {
                "Advanced".to_string()
            } else if dots_score < thresholds.4 {
                "Elite".to_string()
            } else {
                "World Class".to_string()
            }
        }
        "deadlift" => {
            let thresholds = if is_male {
                (63.0, 105.0, 136.0, 167.0, 192.0)
            } else {
                (59.0, 99.0, 128.0, 158.0, 182.0)
            };

            if dots_score < thresholds.0 {
                "Beginner".to_string()
            } else if dots_score < thresholds.1 {
                "Novice".to_string()
            } else if dots_score < thresholds.2 {
                "Intermediate".to_string()
            } else if dots_score < thresholds.3 {
                "Advanced".to_string()
            } else if dots_score < thresholds.4 {
                "Elite".to_string()
            } else {
                "World Class".to_string()
            }
        }
        _ => {
            // "total" and default
            let thresholds = if is_male {
                (200.0, 300.0, 400.0, 500.0, 600.0)
            } else {
                (180.0, 270.0, 360.0, 450.0, 540.0)
            };

            if dots_score < thresholds.0 {
                "Beginner".to_string()
            } else if dots_score < thresholds.1 {
                "Novice".to_string()
            } else if dots_score < thresholds.2 {
                "Intermediate".to_string()
            } else if dots_score < thresholds.3 {
                "Advanced".to_string()
            } else if dots_score < thresholds.4 {
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
pub fn calculate_dots_and_level_for_lift(
    lift_kg: f64,
    bodyweight_kg: f64,
    lift_type: &str,
) -> JsValue {
    calculate_dots_and_level_for_lift_with_gender(lift_kg, bodyweight_kg, true, lift_type)
}

/// Combined function to calculate DOTS and lift-specific strength level with gender
#[wasm_bindgen]
pub fn calculate_dots_and_level_for_lift_with_gender(
    lift_kg: f64,
    bodyweight_kg: f64,
    is_male: bool,
    lift_type: &str,
) -> JsValue {
    let dots = calculate_dots_with_gender(lift_kg, bodyweight_kg, is_male);
    let level = calculate_strength_level_for_lift_with_gender(dots, lift_type, is_male);
    let color = get_strength_level_color(&level);

    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from("dots"), &JsValue::from(dots)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("level"), &JsValue::from(level)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("color"), &JsValue::from(color)).unwrap();

    result.into()
}

/// Combined function to calculate all scoring systems at once
#[wasm_bindgen]
pub fn calculate_all_scores(
    lift_kg: f64,
    bodyweight_kg: f64,
    is_male: bool,
    percentile: f64,
) -> JsValue {
    let dots = calculate_dots_with_gender(lift_kg, bodyweight_kg, is_male);
    let wilks = calculate_wilks(lift_kg, bodyweight_kg, is_male);
    let ipf_gl_points = calculate_ipf_gl_points(lift_kg, bodyweight_kg, is_male);
    let level = calculate_strength_level_from_percentile(percentile);
    let color = get_strength_level_color(&level);

    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from("dots"), &JsValue::from(dots)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("wilks"), &JsValue::from(wilks)).unwrap();
    js_sys::Reflect::set(
        &result,
        &JsValue::from("ipf_gl_points"),
        &JsValue::from(ipf_gl_points),
    )
    .unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("level"), &JsValue::from(level)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from("color"), &JsValue::from(color)).unwrap();

    result.into()
}
