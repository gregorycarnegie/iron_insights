// src/ui/components/header.rs - Header with title and DOTS info
use maud::{html, Markup};

pub fn render_header() -> Markup {
    html! {
        div.header {
            h1 { "🏋️ Iron Insights" }
            p { "High-Performance Powerlifting Analytics with DOTS Scoring" }
            
            div.dots-info {
                strong { "DOTS (Dots Total)" }
                " is the modern replacement for Wilks, providing more accurate strength comparisons across different bodyweights and genders using a single, unified formula."
            }
            
            div #debugInfo .debug-info style="display: none;" {}
            
            button.debug-toggle onclick="toggleDebug()" {
                "Toggle Debug Info"
            }
        }
    }
}