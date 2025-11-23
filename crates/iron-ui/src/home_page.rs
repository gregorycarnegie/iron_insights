use crate::AssetManifest;
use crate::components::*;
use maud::{DOCTYPE, Markup, html};

pub fn render_home_page(manifest: &AssetManifest) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head_minimal(manifest))
            body.no-js {
                div.container {
                    (render_header(Some("/")))
                    main #main-content.page-transition role="main" {
                        div.home-layout {
                            // Hero and feature cards
                            section.hero-section aria-labelledby="hero-heading" {
                                h1 #hero-heading { "Iron Insights" }
                                p.hero-description {
                                    "Advanced powerlifting analytics and visualization platform. Track your progress, analyze your lifts, and share your achievements."
                                }

                                div.feature-grid.stagger-children role="group" aria-label="Available features" {
                                    // Analytics
                                    (render_feature_card(
                                        analytics_icon(),
                                        "Analytics",
                                        "Deep dive into your lifting data with advanced visualizations and statistical analysis.",
                                        "/analytics",
                                        "btn-primary",
                                        "View Analytics",
                                        "analytics-description",
                                        "Navigate to the analytics page to view detailed lifting statistics and charts"
                                    ))

                                    // Share Cards
                                    (render_feature_card(
                                        share_icon(),
                                        "Share Cards",
                                        "Create beautiful social media cards to share your lifting achievements.",
                                        "/sharecard",
                                        "btn-secondary",
                                        "Create Share Card",
                                        "sharecard-description",
                                        "Navigate to the share card creator to make social media posts"
                                    ))

                                    // 1RM Calculator
                                    (render_feature_card(
                                        one_rm_icon(),
                                        "1RM Calculator",
                                        "Estimate your one-rep max using proven formulas (Epley, Brzycki, Lombardi).",
                                        "/1rm",
                                        "btn-tertiary",
                                        "Open 1RM Calculator",
                                        "one-rm-description",
                                        "Navigate to the one-repetition maximum calculator"
                                    ))

                                    // About
                                    (render_feature_card(
                                        about_icon(),
                                        "About",
                                        "Learn more about the project, the methodology, and the developer behind Iron Insights.",
                                        "/about",
                                        "btn-tertiary",
                                        "Read About",
                                        "about-description",
                                        "Navigate to the about page"
                                    ))

                                    // Donate
                                    (render_feature_card(
                                        donate_icon(),
                                        "Donate",
                                        "Support the development and maintenance of this open-source powerlifting analytics platform.",
                                        "/donate",
                                        "btn-primary",
                                        "Support Project",
                                        "donate-description",
                                        "Navigate to the donation page"
                                    ))
                                }
                            }
                        }
                    }
                }
                script {
                    r#"
                    function connectWebSocket() {
                        if (typeof window.connectToWebSocket === 'function') {
                            window.connectToWebSocket();
                        } else {
                            alert('WebSocket functionality not available');
                        }
                    }

                    // Progressive enhancement setup
                    document.addEventListener('DOMContentLoaded', function() {
                        // Remove no-js class and add js class for progressive enhancement
                        document.body.classList.remove('no-js');
                        document.body.classList.add('js');
                        
                        // Show WebSocket button when JS is available
                        const wsButton = document.querySelector('.js-websocket-btn');
                        if (wsButton) {
                            wsButton.style.display = 'inline-flex';
                        }
                    });
                    "#
                }
                style {
                    r#"
                    .home-layout {
                        width: 100%;
                        max-width: 1440px;
                        margin: 0 auto;
                        padding: 0 1rem;
                        min-height: calc(100vh - 65px);
                    }

                    .hero-section {
                        text-align: center;
                        padding: 3.5rem 1.25rem;
                        border-radius: 16px;
                        margin: 2rem auto 2.25rem;
                        max-width: 100%;
                        position: relative;
                        background: radial-gradient(1200px 500px at -10% -40%, rgba(var(--primary-rgb),0.18), transparent 70%),
                                    radial-gradient(900px 500px at 110% -30%, rgba(16,185,129,0.18), transparent 70%),
                                    linear-gradient(135deg, rgba(255,255,255,0.02), rgba(255,255,255,0));
                        border: 1px solid var(--border);
                    }

                    .hero-section h1 { font-size: 2.5rem; margin-bottom: 0.5rem; color: var(--text-primary); }

                    .hero-description {
                        max-width: 700px;
                        margin: 0.75rem auto 2.25rem;
                        color: var(--text-secondary);
                        background: rgba(2,6,23,0.35);
                        border: 1px solid var(--border);
                        padding: 0.75rem 1rem;
                        border-radius: 10px;
                    }

                    .feature-grid {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
                        gap: 1.25rem;
                        max-width: 100%;
                        margin: 0 auto;
                        align-items: stretch;
                    }

                    .feature-card {
                        background: var(--surface);
                        padding: 2rem;
                        border-radius: 14px;
                        border: 1px solid var(--border);
                        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
                        transition: transform 0.2s ease, box-shadow 0.2s ease;
                        text-align: left;
                        display: flex;
                        flex-direction: column;
                        align-items: flex-start;
                        gap: 0.5rem;
                        height: 100%;
                        min-height: 220px;
                    }
                    .feature-card:hover { transform: translateY(-4px); box-shadow: 0 8px 24px rgba(0,0,0,0.15); }
                    .feature-card .icon-wrap { margin-bottom: 0.5rem; }
                    .feature-card .feature-title { margin-bottom: 0.5rem; color: var(--text-primary); font-size: 1.5rem; }
                    .feature-card .feature-desc { color: var(--text-secondary); margin-bottom: 1rem; line-height: 1.6; }
                    .feature-card .feature-cta { margin-top: auto; align-self: stretch; width: 100%; display: flex; justify-content: center; }

                    .btn { display: inline-block; padding: 0.75rem 1.5rem; border-radius: 8px; text-decoration: none; font-weight: 600; transition: all 0.2s ease; border: none; cursor: pointer; font-size: 1rem; }
                    .btn-primary { background: var(--primary); color: white; }
                    .btn-primary:hover { background: var(--primary-dark); transform: translateY(-2px); }
                    .btn-secondary { background: var(--secondary); color: white; }
                    .btn-secondary:hover { background: var(--secondary-dark); transform: translateY(-2px); }
                    .btn-tertiary { background: var(--surface-secondary); color: var(--text-primary); border: 2px solid var(--border); }
                    .btn-tertiary:hover { background: var(--border); transform: translateY(-2px); }

                    @media (max-width: 768px) {
                        .hero-section h1 { font-size: 2rem; }
                        .hero-description { font-size: 1.1rem; }
                        .feature-grid { grid-template-columns: 1fr; }
                    }
                    "#
                }
            }
        }
    }
}

// Helper function to render feature cards
fn render_feature_card(
    icon: Markup,
    title: &str,
    desc: &str,
    link: &str,
    btn_class: &str,
    btn_text: &str,
    desc_id: &str,
    sr_desc: &str,
) -> Markup {
    html! {
        article.feature-card.glass-card.card-hover {
            div.icon-wrap aria-hidden="true" { (icon) }
            h3 class="feature-title" { (title) }
            p class="feature-desc" { (desc) }
            a href=(link) class=(format!("btn {} feature-cta", btn_class)) aria-describedby=(desc_id) { (btn_text) }
            span.sr-only id=(desc_id) { (sr_desc) }
        }
    }
}

// Inline SVG icons (ASCII-only) to avoid encoding issues
fn analytics_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--primary);" {
            rect x="3" y="10" width="4" height="10" rx="1" {}
            rect x="10" y="6" width="4" height="14" rx="1" {}
            rect x="17" y="3" width="4" height="17" rx="1" {}
        }
    }
}

fn share_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--secondary);" {
            circle cx="18" cy="5" r="3" {}
            circle cx="6" cy="12" r="3" {}
            circle cx="18" cy="19" r="3" {}
            line x1="8.59" y1="13.51" x2="15.42" y2="17.49" {}
            line x1="15.41" y1="6.51" x2="8.59" y2="10.49" {}
        }
    }
}

// Simple calculator/dumbbell hybrid icon for 1RM
fn one_rm_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--text-secondary);" {
            // Calculator body
            rect x="3" y="3" width="10" height="14" rx="2" {}
            // Display
            rect x="5" y="5" width="6" height="3" rx="1" {}
            // Buttons
            circle cx="6.5" cy="10" r="0.8" {}
            circle cx="9.5" cy="10" r="0.8" {}
            circle cx="6.5" cy="12.5" r="0.8" {}
            circle cx="9.5" cy="12.5" r="0.8" {}
            // Barbell to hint 1RM
            line x1="16" y1="10" x2="22" y2="10" {}
            rect x="16.5" y="8" width="1" height="4" {}
            rect x="20.5" y="8" width="1" height="4" {}
        }
    }
}

fn about_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--text-primary);" {
            circle cx="12" cy="12" r="9" {}
            line x1="12" y1="11" x2="12" y2="16" {}
            circle cx="12" cy="8" r="0.8" {}
        }
    }
}

fn donate_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--primary);" {
            path d="M12 21l-1.45-1.32C6 15 4 12.5 4 10a4.5 4.5 0 0 1 8-2.5A4.5 4.5 0 0 1 20 10c0 2.5-2 5-6.55 9.68L12 21z" {}
        }
    }
}
