// src/ui/components/header.rs - Modern header with navigation
use maud::{html, Markup};

pub fn render_header() -> Markup {
    html! {
        div.header {
            div.header-content {
                h1 {
                    span.logo { "🏋️" }
                    "Iron Insights"
                }
                
                nav.header-nav {
                    a href="/" { "Home" }
                    a href="/analytics" { "Analytics" }
                    a href="/1rm" { "1RM Calculator" }
                    a href="/sharecard" { "Share Card" }
                    a href="#rankings" { "Rankings" }
                    
                    button.mobile-menu-toggle onclick="toggleSidebar()" {
                        svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" {}
                        }
                    }
                }
            }
        }
    }
}
