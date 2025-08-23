// src/ui/mod.rs - Main UI module with modern layout
use maud::{html, Markup, DOCTYPE};

// Import all UI components
pub mod components;
use components::*;

/// Main page template with modern design
pub fn render_index() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head())
            body {
                div.container {
                    (render_header())
                    div.main-content {
                        (render_controls())
                        (render_main_content())
                    }
                }
                (render_scripts())
            }
        }
    }
}