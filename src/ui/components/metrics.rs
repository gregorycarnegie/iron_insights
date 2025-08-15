// src/ui/components/metrics.rs - User metrics and performance display
use maud::{html, Markup};

pub fn render_user_metrics() -> Markup {
    html! {
        div #userMetrics .user-input-section style="display: none;" {
            h3 { "Your Performance" }
            div #strengthBadge {}
            
            div.user-metrics {
                div.metric-display {
                    div #userDotsValue .metric-value { "-" }
                    div.metric-label { "DOTS Score" }
                }
                div.metric-display {
                    div #userStrengthLevel .metric-value { "-" }
                    div.metric-label { "Strength Level" }
                }
            }
        }
    }
}