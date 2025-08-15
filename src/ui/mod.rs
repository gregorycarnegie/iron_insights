// src/ui/mod.rs - Main UI module with maud HTML templating
use maud::{html, Markup, DOCTYPE};

// Import all UI components
pub mod components;
use components::*;

/// Main page template - clean, component-based structure
pub fn render_index() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head())
            body {
                div.container {
                    (render_header())
                    (render_controls())
                    (render_chart_grid())
                    (render_user_metrics())
                    (render_realtime_panel())
                    (render_percentiles())
                    (render_stats())
                    (render_share_card_section())
                }
                (render_scripts())
            }
        }
    }
}


/// Stats and percentiles placeholder containers
fn render_percentiles() -> Markup {
    html! {
        div #percentiles {}
    }
}

fn render_stats() -> Markup {
    html! {
        div #stats {}
    }
}