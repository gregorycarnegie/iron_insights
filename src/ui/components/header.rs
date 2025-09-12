// src/ui/components/header.rs - Modern header with navigation
use maud::{html, Markup};

pub fn render_header() -> Markup {
    html! {
        a.skip-link href="#main-content" { "Skip to main content" }
        header.header role="banner" {
            div.header-content {
                h1 {
                    span.logo {
                        // Inline barbell icon (ASCII-only)
                        svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" {
                            rect x="1" y="9" width="3" height="6" {}
                            rect x="20" y="9" width="3" height="6" {}
                            rect x="6" y="8" width="2" height="8" {}
                            rect x="16" y="8" width="2" height="8" {}
                            line x1="8" y1="12" x2="16" y2="12" {}
                        }
                    }
                    "Iron Insights"
                }

                nav.header-nav role="navigation" aria-label="Main navigation" {
                    a href="/" aria-current="page" { "Home" }
                    a href="/analytics" { "Analytics" }
                    a href="/1rm" { "1RM Calculator" }
                    a href="/sharecard" { "Share Card" }
                    a href="#rankings" { "Rankings" }

                    button.mobile-menu-toggle onclick="toggleSidebar()" aria-label="Toggle mobile menu" aria-expanded="false" aria-controls="sidebar" {
                        svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" {}
                        }
                    }
                }
            }
        }
    }
}

