// src/ui/components/head.rs - Modern HTML head with optimized resources
use maud::{html, Markup};
use super::render_styles;

pub fn render_head() -> Markup {
    html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta name="description" content="Professional powerlifting analytics platform with DOTS scoring, performance tracking, and comprehensive competition data analysis.";
            meta name="theme-color" content="#2563eb";
            
            title { "Iron Insights - Professional Powerlifting Analytics" }
            
            // Preconnect to external domains for performance
            link rel="preconnect" href="https://cdn.plot.ly";
            link rel="dns-prefetch" href="https://cdn.plot.ly";
            
            // External libraries
            script src="https://cdn.plot.ly/plotly-2.27.0.min.js" charset="utf-8" {}
            
            // Inline critical CSS for faster initial paint
            style { (render_styles()) }
            
            // Favicon
            link rel="icon" type="image/svg+xml" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='0.9em' font-size='90'>🏋️</text></svg>";
        }
    }
}