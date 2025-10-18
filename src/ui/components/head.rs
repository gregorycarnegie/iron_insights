// src/ui/components/head.rs - Modern HTML head with optimized resources
use super::styles::render_styles;
use maud::{Markup, html};

pub fn render_head() -> Markup {
    html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta name="description" content="Professional powerlifting analytics platform with DOTS scoring, performance tracking, and comprehensive competition data analysis.";
            meta name="theme-color" content="#2563eb";
            meta property="og:title" content="Iron Insights - Powerlifting Analytics";
            meta property="og:description" content="Explore pro-grade analytics, visualizations, calculators, and shareable cards for lifters.";
            meta property="og:type" content="website";

            title { "Iron Insights - Professional Powerlifting Analytics" }

            // Fonts (Inter) and preconnects
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="";
            link rel="dns-prefetch" href="https://cdnjs.cloudflare.com";

            // Critical resource hints for JavaScript modules
            link rel="modulepreload" href="/static/wasm/iron_insights_wasm.js";
            link rel="modulepreload" href="/static/js/lazy-loader.js";

            // Font loading with display=swap for better performance
            link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet";

            // Lazy loading module - loads first to enable on-demand loading
            script src="/static/js/lazy-loader.js" defer {}

            // Service Worker registration
            script {
                r#"
                if ('serviceWorker' in navigator) {
                    window.addEventListener('load', () => {
                        navigator.serviceWorker.register('/static/sw.js')
                            .then(reg => console.log('✅ Service Worker registered'))
                            .catch(err => console.log('❌ Service Worker registration failed:', err));
                    });
                }
                "#
            }

            // Inline critical CSS for faster initial paint
            style { (render_styles()) }

            // Favicon (inline SVG barbell)
            link rel="icon" type="image/svg+xml" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='white' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Crect x='1' y='9' width='3' height='6'/%3E%3Crect x='20' y='9' width='3' height='6'/%3E%3Crect x='6' y='8' width='2' height='8'/%3E%3Crect x='16' y='8' width='2' height='8'/%3E%3Cline x1='8' y1='12' x2='16' y2='12'/%3E%3C/svg%3E";
        }
    }
}

// Minimal head for non-analytics pages: avoids webfonts and heavy libs to prevent CLS/TBT
pub fn render_head_minimal() -> Markup {
    html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta name="description" content="Professional powerlifting analytics platform with DOTS scoring, performance tracking, and comprehensive competition data analysis.";
            meta name="theme-color" content="#2563eb";
            meta property="og:title" content="Iron Insights - Powerlifting Analytics";
            meta property="og:description" content="Explore pro-grade analytics, visualizations, calculators, and shareable cards for lifters.";
            meta property="og:type" content="website";

            title { "Iron Insights - Professional Powerlifting Analytics" }

            // Lazy loading module for optimal performance
            script src="/static/js/lazy-loader.js" defer {}

            // Inline critical CSS only (system font stack used from base styles)
            style { (render_styles()) }

            // Favicon (inline SVG barbell)
            link rel="icon" type="image/svg+xml" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='white' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Crect x='1' y='9' width='3' height='6'/%3E%3Crect x='20' y='9' width='3' height='6'/%3E%3Crect x='6' y='8' width='2' height='8'/%3E%3Crect x='16' y='8' width='2' height='8'/%3E%3Cline x1='8' y1='12' x2='16' y2='12'/%3E%3C/svg%3E";
        }
    }
}
