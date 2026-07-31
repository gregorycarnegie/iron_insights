pub(super) use crate::core::{
    JacksonPollock7SiteSkinfolds, bodyfat_category, calc_bodyfat_female, calc_bodyfat_jp3,
    calc_bodyfat_jp7, calc_bodyfat_male, calc_bodyfat_ymca, tier_for_percentile,
};
use crate::core::{dots_points, goodlift_points, wilks_points};

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
    use proptest::{
        prelude::*,
        test_runner::{Config, TestRunner},
    };
    use wasm_bindgen_test::wasm_bindgen_test;

    fn property_runner() -> TestRunner {
        TestRunner::new(Config::with_cases(128))
    }

    #[wasm_bindgen_test]
    fn kg_display_converts_both_directions() {
        // (kg, use_lbs, expected display value)
        for (kg, use_lbs, expected) in [
            (100.0, false, 100.0),
            (100.0, true, 220.462),
            (0.0, true, 0.0),
        ] {
            let shown = kg_to_display(kg, use_lbs);
            assert!((shown - expected).abs() < 0.01, "{kg} kg -> {shown}");
            assert!((display_to_kg(shown, use_lbs) - kg).abs() < 0.01, "{shown}");
        }
    }

    #[wasm_bindgen_test]
    fn kg_display_round_trips_for_generated_values() {
        property_runner()
            .run(&(0.0f32..1000.0, any::<bool>()), |(kg, use_lbs)| {
                let converted = kg_to_display(kg, use_lbs);
                let back = display_to_kg(converted, use_lbs);

                prop_assert!((back - kg).abs() <= 1e-4);
                Ok(())
            })
            .expect("kg/display conversion property should hold");
    }

    #[wasm_bindgen_test]
    fn parse_query_f32_parses_clamps_and_falls_back() {
        // (raw param, default, expected)
        for (raw, default, expected) in [
            (None, 50.0, 50.0),                    // absent
            (Some("75.5"), 0.0, 75.5),             // valid
            (Some("200.0"), 0.0, 100.0),           // above max
            (Some("-10.0"), 0.0, 0.0),             // below min
            (Some("abc"), 42.0, 42.0),             // unparseable
        ] {
            let got = parse_query_f32(raw.map(Into::into), default, 0.0, 100.0);
            assert!((got - expected).abs() < 0.001, "{raw:?} -> {got}");
        }
    }

    #[wasm_bindgen_test]
    fn parse_query_f32_clamps_generated_values() {
        property_runner()
            .run(
                &(-1000.0f32..1000.0, -1000.0f32..1000.0, 0.0f32..1000.0),
                |(value, min, span)| {
                    let max = min + span;
                    let parsed = parse_query_f32(Some(value.to_string()), 0.0, min, max);

                    prop_assert!((min..=max).contains(&parsed));
                    Ok(())
                },
            )
            .expect("query parsing clamp property should hold");
    }

    #[wasm_bindgen_test]
    fn comparable_lift_value_selects_the_named_lift() {
        let lifter = ComparableLifter {
            sex: "M",
            equipment: "Raw",
            bodyweight: 83.0,
            squat: 200.0,
            bench: 130.0,
            deadlift: 240.0,
        };
        // (lift code, expected kg); "X" is an unknown code.
        for (lift, expected) in [
            ("S", 200.0),
            ("B", 130.0),
            ("D", 240.0),
            ("T", 570.0),
            ("X", 0.0),
        ] {
            let got = comparable_lift_value(lifter, lift, "Kg");
            assert!((got - expected).abs() < 0.001, "{lift} -> {got}");
        }
    }

    #[wasm_bindgen_test]
    #[expect(
        clippy::float_cmp,
        reason = "these lifts are returned verbatim, so exact equality is the property under test: \
                  an epsilon would let a value that had been rounded or scaled slip through"
    )]
    fn comparable_lift_value_matches_generated_lift_fields() {
        property_runner()
            .run(
                &(30.0f32..200.0, 0.0f32..500.0, 0.0f32..350.0, 0.0f32..500.0),
                |(bodyweight, squat, bench, deadlift)| {
                    let lifter = ComparableLifter {
                        sex: "M",
                        equipment: "Raw",
                        bodyweight,
                        squat,
                        bench,
                        deadlift,
                    };

                    prop_assert_eq!(comparable_lift_value(lifter, "S", "Kg"), squat);
                    prop_assert_eq!(comparable_lift_value(lifter, "B", "Kg"), bench);
                    prop_assert_eq!(comparable_lift_value(lifter, "D", "Kg"), deadlift);
                    prop_assert_eq!(
                        comparable_lift_value(lifter, "T", "Kg"),
                        squat + bench + deadlift
                    );
                    Ok(())
                },
            )
            .expect("comparable lift property should hold");
    }

    #[wasm_bindgen_test]
    fn format_input_bound_drops_trailing_zero_decimals() {
        assert_eq!(format_input_bound(100.0, false), "100");
        assert_eq!(format_input_bound(102.3, false), "102.3");
    }
}
