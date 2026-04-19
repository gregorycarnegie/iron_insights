use crate::core::{dots_points, goodlift_points, wilks_points};
pub(super) use crate::core::{
    BodyfatResult, bodyfat_category, calc_bodyfat_female, calc_bodyfat_male, tier_for_percentile,
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
        format!("{:.0}", value)
    } else {
        format!("{:.1}", value)
    }
}

#[allow(dead_code)]
pub(super) fn build_share_url(params: &[(&str, String)]) -> Option<String> {
    let window = web_sys::window()?;
    let search = web_sys::UrlSearchParams::new().ok()?;
    for (key, value) in params {
        search.append(key, value);
    }
    let origin = window.location().origin().ok()?;
    let pathname = window.location().pathname().ok()?;
    let hash = window.location().hash().ok().unwrap_or_default();
    Some(format!("{origin}{pathname}?{}{}", search.to_string(), hash))
}

