use iron_insights_core::{dots_points, goodlift_points, wilks_points};

use super::constants::{LIFT_BIN_BASE_KG, SCORE_BIN_BASE_POINTS};

#[derive(Debug, Clone, Copy)]
pub(super) enum Metric {
    Kg,
    Dots,
    Wilks,
    Gl,
}

pub(super) fn tested_bucket(tested: &str) -> &'static str {
    if tested == "tested" { "Yes" } else { "All" }
}

pub(super) fn lift_code(lift: &str) -> &'static str {
    match lift {
        "squat" => "S",
        "bench" => "B",
        "deadlift" => "D",
        "total" => "T",
        _ => "U",
    }
}

pub(super) fn metrics_for_lift(lift: &str) -> &'static [Metric] {
    if lift == "total" {
        &[Metric::Kg, Metric::Dots, Metric::Wilks, Metric::Gl]
    } else {
        &[Metric::Kg]
    }
}

pub(super) fn metric_base_bin(metric: Metric) -> f32 {
    match metric {
        Metric::Kg => LIFT_BIN_BASE_KG,
        Metric::Dots | Metric::Wilks | Metric::Gl => SCORE_BIN_BASE_POINTS,
    }
}

pub(super) fn metric_slug(metric: Metric) -> &'static str {
    match metric {
        Metric::Kg => "kg",
        Metric::Dots => "dots",
        Metric::Wilks => "wilks",
        Metric::Gl => "gl",
    }
}

pub(super) fn metric_code(metric: Metric) -> &'static str {
    match metric {
        Metric::Kg => "Kg",
        Metric::Dots => "Dots",
        Metric::Wilks => "Wilks",
        Metric::Gl => "GL",
    }
}

pub(super) fn metric_value(
    metric: Metric,
    lift: &str,
    sex: &str,
    equipment: &str,
    lift_value: f32,
    bodyweight_kg: Option<f32>,
) -> Option<f32> {
    match metric {
        Metric::Kg => Some(lift_value),
        Metric::Dots => {
            if lift != "total" {
                return None;
            }
            Some(dots_points(sex, bodyweight_kg?, lift_value))
        }
        Metric::Wilks => {
            if lift != "total" {
                return None;
            }
            Some(wilks_points(sex, bodyweight_kg?, lift_value))
        }
        Metric::Gl => {
            if lift != "total" {
                return None;
            }
            Some(goodlift_points(sex, equipment, bodyweight_kg?, lift_value))
        }
    }
}
