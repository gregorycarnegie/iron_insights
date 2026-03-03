use crate::core::{dots_points, goodlift_points, wilks_points};

pub(super) fn comparable_lift_value(
    sex: &str,
    equipment: &str,
    lift: &str,
    metric: &str,
    bodyweight: f32,
    squat: f32,
    bench: f32,
    deadlift: f32,
) -> f32 {
    let total = squat + bench + deadlift;
    match (lift, metric) {
        ("S", _) => squat,
        ("B", _) => bench,
        ("D", _) => deadlift,
        ("T", "Dots") => dots_points(sex, bodyweight, total),
        ("T", "Wilks") => wilks_points(sex, bodyweight, total),
        ("T", "GL") => goodlift_points(sex, equipment, bodyweight, total),
        ("T", _) => total,
        _ => 0.0,
    }
}

pub(super) fn tier_for_percentile(pct: f32) -> &'static str {
    if pct >= 0.999 {
        "Mythical"
    } else if pct >= 0.99 {
        "Legendary"
    } else if pct >= 0.95 {
        "Elite"
    } else if pct >= 0.8 {
        "Advanced"
    } else if pct >= 0.6 {
        "Intermediate"
    } else if pct >= 0.35 {
        "Novice"
    } else {
        "Beginner"
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
