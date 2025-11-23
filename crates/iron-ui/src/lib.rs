// iron-ui: UI components and templates
use maud::{DOCTYPE, Markup, html};
// Re-export AssetManifest from iron-core
pub use iron_core::models::AssetManifest;

pub mod about_page;
pub mod components;
pub mod donate_page;
pub mod home_page;
pub mod onerepmax_page;
pub mod sharecard_page;

// Re-export components
use components::*;

/// Home page - landing page with overview
pub fn render_index(manifest: &AssetManifest) -> Markup {
    home_page::render_home_page(manifest)
}

/// Analytics page - the original main functionality
pub fn render_analytics(manifest: &AssetManifest) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head(manifest))
            body {
                div.container {
                    (render_header(Some("/analytics")))
                    div.main-content {
                        (render_controls())
                        (render_main_content())
                    }
                }
                (components::scripts::render_scripts(manifest))
            }
        }
    }
}

/// 1RM Calculator page
pub fn render_onerepmax(manifest: &AssetManifest) -> Markup {
    onerepmax_page::render_onerepmax_page(manifest)
}

/// About page
pub fn render_about(manifest: &AssetManifest) -> Markup {
    about_page::render_about_page(manifest)
}

/// Donation page
pub fn render_donate(manifest: &AssetManifest) -> Markup {
    donate_page::render_donate_page(manifest)
}

/// Share Card page (kept for convenience)
pub fn render_sharecard(manifest: &AssetManifest) -> Markup {
    sharecard_page::render_sharecard_page(manifest)
}
