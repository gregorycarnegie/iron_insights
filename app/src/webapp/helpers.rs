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

pub(super) fn tier_for_percentile(pct: f32) -> &'static str {
    if pct >= 0.99 {
        "Legend"
    } else if pct >= 0.95 {
        "Elite"
    } else if pct >= 0.8 {
        "Advanced"
    } else if pct >= 0.6 {
        "Intermediate"
    } else {
        "Novice"
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

pub(super) fn build_share_url(params: &[(&str, String)]) -> Option<String> {
    let window = web_sys::window()?;
    let search = web_sys::UrlSearchParams::new().ok()?;
    for (key, value) in params {
        search.append(key, value);
    }
    let origin = window.location().origin().ok()?;
    Some(format!("{origin}/?{}", search.to_string()))
}
