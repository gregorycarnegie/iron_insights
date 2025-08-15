// src/ui/components/controls.rs - Control panel with form inputs
use maud::{html, Markup};

pub fn render_controls() -> Markup {
    html! {
        div.controls {
            (render_sex_control())
            (render_lift_type_control())
            (render_bodyweight_control())
            (render_lift_input_control())
            (render_update_button())
        }
    }
}

fn render_sex_control() -> Markup {
    html! {
        div.control-group {
            label { "Sex:" }
            select #sex {
                option value="M" { "Male" }
                option value="F" { "Female" }
                option value="All" { "All" }
            }
        }
    }
}

fn render_lift_type_control() -> Markup {
    html! {
        div.control-group {
            label { "Lift Type:" }
            select #liftType {
                option value="squat" { "Squat" }
                option value="bench" { "Bench Press" }
                option value="deadlift" { "Deadlift" }
                option value="total" { "Total" }
            }
        }
    }
}

fn render_bodyweight_control() -> Markup {
    html! {
        div.control-group {
            label { "Your Bodyweight (kg):" }
            input #bodyweight type="number" placeholder="75" step="0.1";
        }
    }
}

fn render_lift_input_control() -> Markup {
    html! {
        div.control-group {
            label { "Your Lift (kg):" }
            input #userLift type="number" placeholder="150" step="0.5";
        }
    }
}

fn render_update_button() -> Markup {
    html! {
        div.control-group {
            button onclick="updateCharts()" {
                "Update Analytics"
            }
        }
    }
}