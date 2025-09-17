use maud::{Markup, PreEscaped};

pub mod base;
pub mod charts;
pub mod components;
pub mod layout;
pub mod responsive;
pub mod tables;
pub mod theme;

pub use base::render_base_styles;
pub use charts::render_chart_styles;
pub use components::render_component_styles;
pub use layout::render_layout_styles;
pub use responsive::render_responsive_styles;
pub use tables::render_table_styles;
pub use theme::render_theme_styles;

pub fn render_styles() -> Markup {
    PreEscaped(format!(
        "{}{}{}{}{}{}{}",
        render_base_styles().into_string(),
        render_layout_styles().into_string(),
        render_component_styles().into_string(),
        render_chart_styles().into_string(),
        render_table_styles().into_string(),
        render_responsive_styles().into_string(),
        render_theme_styles().into_string()
    ))
}
