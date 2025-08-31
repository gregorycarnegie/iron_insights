// src/ui/mod.rs - Main UI module with modern layout
use maud::{html, Markup, DOCTYPE};

// Import all UI components
pub mod components;
use components::*;
pub mod sharecard_page;
pub mod home_page;

/// Home page - landing page with overview
pub fn render_index() -> Markup {
    home_page::render_home_page()
}

/// Analytics page - the original main functionality
pub fn render_analytics() -> Markup {
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
                (components::scripts::render_scripts())
            }
        }
    }
}
