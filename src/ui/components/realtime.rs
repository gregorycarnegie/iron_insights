// src/ui/components/realtime.rs - Real-time WebSocket activity panel
use maud::{html, Markup};

pub fn render_realtime_panel() -> Markup {
    html! {
        div #realtimePanel .realtime-panel {
            div.realtime-title {
                span #connectionStatus .connection-status {}
                "🌐 Live Activity Feed"
                span #userCount style="margin-left: auto; font-size: 14px;" {
                    "- users online"
                }
            }
            
            div #activityFeed .activity-feed {
                div.activity-item {
                    "Connecting to real-time updates..."
                }
            }
            
            div #liveStats .live-stats {
                div.live-stat {
                    div #totalConnections .live-stat-value { "-" }
                    div.live-stat-label { "Total Connections" }
                }
                div.live-stat {
                    div #calculationsCount .live-stat-value { "-" }
                    div.live-stat-label { "Recent Calculations" }
                }
                div.live-stat {
                    div #serverLoad .live-stat-value { "-" }
                    div.live-stat-label { "Server Load" }
                }
            }
        }
    }
}