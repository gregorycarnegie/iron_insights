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

// ===== BODYFAT (US NAVY METHOD) =====

#[derive(Clone, Copy, PartialEq)]
pub(super) struct BodyfatResult {
    pub(super) body_fat_pct: f32,
    pub(super) lean_mass_kg: f32,
    pub(super) fat_mass_kg: f32,
}

pub(super) fn calc_bodyfat_male(
    height_cm: f32,
    weight_kg: f32,
    neck_cm: f32,
    waist_cm: f32,
) -> Option<BodyfatResult> {
    if height_cm <= 0.0 || neck_cm <= 0.0 || waist_cm <= neck_cm {
        return None;
    }
    let diff = waist_cm - neck_cm;
    if diff <= 0.0 {
        return None;
    }
    let bf = 495.0
        / (1.0324 - 0.19077 * (diff as f64).log10() as f32
            + 0.15456 * (height_cm as f64).log10() as f32)
        - 450.0;
    let bf = bf.clamp(2.0, 60.0);
    let fat_mass = weight_kg * bf / 100.0;
    let lean_mass = weight_kg - fat_mass;
    Some(BodyfatResult {
        body_fat_pct: bf,
        lean_mass_kg: lean_mass,
        fat_mass_kg: fat_mass,
    })
}

pub(super) fn calc_bodyfat_female(
    height_cm: f32,
    weight_kg: f32,
    neck_cm: f32,
    waist_cm: f32,
    hip_cm: f32,
) -> Option<BodyfatResult> {
    if height_cm <= 0.0 || neck_cm <= 0.0 {
        return None;
    }
    let diff = waist_cm + hip_cm - neck_cm;
    if diff <= 0.0 {
        return None;
    }
    let bf = 495.0
        / (1.29579 - 0.35004 * (diff as f64).log10() as f32
            + 0.22100 * (height_cm as f64).log10() as f32)
        - 450.0;
    let bf = bf.clamp(8.0, 60.0);
    let fat_mass = weight_kg * bf / 100.0;
    let lean_mass = weight_kg - fat_mass;
    Some(BodyfatResult {
        body_fat_pct: bf,
        lean_mass_kg: lean_mass,
        fat_mass_kg: fat_mass,
    })
}

pub(super) fn bodyfat_category(pct: f32, is_male: bool) -> &'static str {
    if is_male {
        if pct < 6.0 {
            "Essential"
        } else if pct < 11.0 {
            "Elite Athlete"
        } else if pct < 15.0 {
            "Athlete"
        } else if pct < 20.0 {
            "Fitness"
        } else if pct < 25.0 {
            "Average"
        } else {
            "Obese"
        }
    } else {
        if pct < 14.0 {
            "Essential"
        } else if pct < 18.0 {
            "Elite Athlete"
        } else if pct < 22.0 {
            "Athlete"
        } else if pct < 26.0 {
            "Fitness"
        } else if pct < 32.0 {
            "Average"
        } else {
            "Obese"
        }
    }
}
