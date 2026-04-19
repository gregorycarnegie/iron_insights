use crate::core::{dots_points, goodlift_points, wilks_points};
pub(super) use crate::core::{
    bodyfat_category, calc_bodyfat_female, calc_bodyfat_male, tier_for_percentile,
};

#[derive(Clone, Copy)]
pub(super) struct ComparableLifter<'a> {
    pub(super) sex: &'a str,
    pub(super) equipment: &'a str,
    pub(super) bodyweight: f32,
    pub(super) squat: f32,
    pub(super) bench: f32,
    pub(super) deadlift: f32,
}

pub(super) fn comparable_lift_value(lifter: ComparableLifter<'_>, lift: &str, metric: &str) -> f32 {
    let total = lifter.squat + lifter.bench + lifter.deadlift;
    match (lift, metric) {
        ("S", _) => lifter.squat,
        ("B", _) => lifter.bench,
        ("D", _) => lifter.deadlift,
        ("T", "Dots") => dots_points(lifter.sex, lifter.bodyweight, total),
        ("T", "Wilks") => wilks_points(lifter.sex, lifter.bodyweight, total),
        ("T", "GL") => goodlift_points(lifter.sex, lifter.equipment, lifter.bodyweight, total),
        ("T", _) => total,
        _ => 0.0,
    }
}

pub(super) fn parse_query_f32(value: Option<String>, default: f32, min: f32, max: f32) -> f32 {
    value
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(default)
        .clamp(min, max)
}

pub(super) fn kg_to_display(kg: f32, use_lbs: bool) -> f32 {
    if use_lbs { kg * 2.204_622_5 } else { kg }
}

pub(super) fn display_to_kg(value: f32, use_lbs: bool) -> f32 {
    if use_lbs { value / 2.204_622_5 } else { value }
}

pub(super) fn format_input_bound(value_kg: f32, use_lbs: bool) -> String {
    let value = kg_to_display(value_kg, use_lbs);
    if (value - value.round()).abs() < 0.05 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn kg_to_display_kg_mode_is_identity() {
        assert!((kg_to_display(100.0, false) - 100.0).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn kg_to_display_converts_to_lbs() {
        let lbs = kg_to_display(100.0, true);
        assert!((lbs - 220.462).abs() < 0.01, "got {lbs}");
    }

    #[wasm_bindgen_test]
    fn display_to_kg_kg_mode_is_identity() {
        assert!((display_to_kg(100.0, false) - 100.0).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn display_to_kg_converts_from_lbs() {
        let kg = display_to_kg(220.462, true);
        assert!((kg - 100.0).abs() < 0.01, "got {kg}");
    }

    #[wasm_bindgen_test]
    fn kg_display_round_trips() {
        let original = 142.5_f32;
        let converted = kg_to_display(original, true);
        let back = display_to_kg(converted, true);
        assert!((back - original).abs() < 0.01, "round trip lost precision: {back}");
    }

    #[wasm_bindgen_test]
    fn parse_query_f32_none_returns_default() {
        assert!((parse_query_f32(None, 50.0, 0.0, 100.0) - 50.0).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn parse_query_f32_parses_valid_string() {
        assert!((parse_query_f32(Some("75.5".into()), 0.0, 0.0, 100.0) - 75.5).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn parse_query_f32_clamps_to_range() {
        assert!((parse_query_f32(Some("200.0".into()), 0.0, 0.0, 100.0) - 100.0).abs() < 0.001);
        assert!((parse_query_f32(Some("-10.0".into()), 0.0, 0.0, 100.0) - 0.0).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn parse_query_f32_invalid_string_returns_default() {
        assert!((parse_query_f32(Some("abc".into()), 42.0, 0.0, 100.0) - 42.0).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn comparable_lift_value_squat() {
        let lifter = ComparableLifter {
            sex: "M",
            equipment: "Raw",
            bodyweight: 83.0,
            squat: 200.0,
            bench: 130.0,
            deadlift: 240.0,
        };
        assert!((comparable_lift_value(lifter, "S", "Kg") - 200.0).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn comparable_lift_value_bench() {
        let lifter = ComparableLifter {
            sex: "M",
            equipment: "Raw",
            bodyweight: 83.0,
            squat: 200.0,
            bench: 130.0,
            deadlift: 240.0,
        };
        assert!((comparable_lift_value(lifter, "B", "Kg") - 130.0).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn comparable_lift_value_deadlift() {
        let lifter = ComparableLifter {
            sex: "M",
            equipment: "Raw",
            bodyweight: 83.0,
            squat: 200.0,
            bench: 130.0,
            deadlift: 240.0,
        };
        assert!((comparable_lift_value(lifter, "D", "Kg") - 240.0).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn comparable_lift_value_total_kg() {
        let lifter = ComparableLifter {
            sex: "M",
            equipment: "Raw",
            bodyweight: 83.0,
            squat: 200.0,
            bench: 130.0,
            deadlift: 240.0,
        };
        assert!((comparable_lift_value(lifter, "T", "Kg") - 570.0).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn comparable_lift_value_unknown_lift_returns_zero() {
        let lifter = ComparableLifter {
            sex: "M",
            equipment: "Raw",
            bodyweight: 83.0,
            squat: 200.0,
            bench: 130.0,
            deadlift: 240.0,
        };
        assert!((comparable_lift_value(lifter, "X", "Kg") - 0.0).abs() < 0.001);
    }

    #[wasm_bindgen_test]
    fn format_input_bound_whole_number() {
        assert_eq!(format_input_bound(100.0, false), "100");
    }

    #[wasm_bindgen_test]
    fn format_input_bound_fractional() {
        assert_eq!(format_input_bound(102.3, false), "102.3");
    }
}

