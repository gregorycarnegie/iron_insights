// src/ui/components/head.rs - HTML head section with metadata and styles
use maud::{html, Markup};
use super::render_styles;

pub fn render_head() -> Markup {
    html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title { "Iron Insights - Powerlifting Analytics with DOTS" }
            
            // External libraries
            script src="https://cdn.plot.ly/plotly-3.0.3.min.js" {}
            
            // Embedded styles
            style { (render_styles()) }
        }
    }
}