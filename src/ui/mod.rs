// src/ui/mod.rs - Main UI module with modern layout
use maud::{DOCTYPE, Markup, html};

// Import all UI components
pub mod components;
use components::*;
pub mod about_page;
pub mod donate_page;
pub mod home_page;
pub mod onerepmax_page;
pub mod rankings_page;
pub mod sharecard_page;

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

/// 1RM Calculator page
pub fn render_onerepmax() -> Markup {
    onerepmax_page::render_onerepmax_page()
}

/// About page
pub fn render_about() -> Markup {
    about_page::render_about_page()
}

/// Donation page
pub fn render_donate() -> Markup {
    donate_page::render_donate_page()
}

/// Rankings page
pub fn render_rankings(
    rankings: Option<&crate::models::RankingsResponse>,
    params: &crate::models::RankingsParams
) -> Markup {
    rankings_page::render_rankings_page(rankings, params)
}

/// Share Card page (kept for convenience)
pub fn render_sharecard() -> Markup {
    sharecard_page::render_sharecard_page()
}
