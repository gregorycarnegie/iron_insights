use maud::{html, Markup, DOCTYPE};
use crate::ui::components::*;

pub fn render_home_page() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head_minimal())
            body.no-js {
                div.container {
                    (render_header())
                    main #main-content.page-transition role="main" {
                        div.main-content {
                            // Quick stats section (as in mock)
                            section.quick-stats aria-labelledby="stats-heading" {
                                h2 #stats-heading { "Quick Stats Overview" }
                                
                                // Skeleton loading state
                                div.skeleton-container #stats-skeleton {
                                    div.stats-grid {
                                        div.skeleton.skeleton-stat {}
                                        div.skeleton.skeleton-stat {}
                                        div.skeleton.skeleton-stat {}
                                    }
                                }
                                
                                // Actual content (hidden by default)
                                div.content-container #stats-content {
                                    div.stats-grid role="group" aria-label="Statistics summary" {
                                        div.stat-card.card-hover role="img" aria-labelledby="total-records-label" aria-describedby="total-records-value" {
                                            span.stat-number #total-records-value aria-live="polite" { "-" }
                                            span.stat-label #total-records-label { "Total Records" }
                                        }
                                        div.stat-card.card-hover role="img" aria-labelledby="avg-wilks-label" aria-describedby="avg-wilks-value" {
                                            span.stat-number #avg-wilks-value aria-live="polite" { "-" }
                                            span.stat-label #avg-wilks-label { "Avg Wilks Score" }
                                        }
                                        div.stat-card.card-hover role="img" aria-labelledby="top-percentile-label" aria-describedby="top-percentile-value" {
                                            span.stat-number #top-percentile-value aria-live="polite" { "-" }
                                            span.stat-label #top-percentile-label { "Top Percentile" }
                                        }
                                    }
                                }
                            }

                            // Hero and feature cards
                            section.hero-section aria-labelledby="hero-heading" {
                                h1 #hero-heading { "Iron Insights" }
                                p.hero-description {
                                    "Advanced powerlifting analytics and visualization platform. Track your progress, analyze your lifts, and share your achievements."
                                }

                                div.feature-grid.stagger-children role="group" aria-label="Available features" {
                                    // Analytics
                                    article.feature-card.glass-card.card-hover {
                                        div.icon-wrap aria-hidden="true" { (analytics_icon()) }
                                        h3 { "Analytics" }
                                        p { "Deep dive into your lifting data with advanced visualizations and statistical analysis." }
                                        a href="/analytics" class="btn btn-primary" aria-describedby="analytics-description" { "View Analytics" }
                                        span.sr-only #analytics-description { "Navigate to the analytics page to view detailed lifting statistics and charts" }
                                    }

                                    // Share Cards
                                    article.feature-card.glass-card.card-hover {
                                        div.icon-wrap aria-hidden="true" { (share_icon()) }
                                        h3 { "Share Cards" }
                                        p { "Create beautiful social media cards to share your lifting achievements." }
                                        a href="/sharecard" class="btn btn-secondary" aria-describedby="sharecard-description" { "Create Share Card" }
                                        span.sr-only #sharecard-description { "Navigate to the share card creator to make social media posts" }
                                    }

                                    // Real-time Data - Progressive enhancement
                                    article.feature-card.glass-card.card-hover {
                                        div.icon-wrap aria-hidden="true" { (realtime_icon()) }
                                        h3 { "Real-time Data" }
                                        p { "Connect via WebSocket for live data streaming and real-time updates." }
                                        noscript {
                                            p.text-warning { "JavaScript is required for real-time data connections." }
                                        }
                                        button class="btn btn-tertiary js-websocket-btn" onclick="connectWebSocket()" aria-describedby="websocket-description" style="display: none;" { "Connect WebSocket" }
                                        span.sr-only #websocket-description { "Establish a real-time connection for live data updates" }
                                    }
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
                        
                        // Show stats skeleton initially, hide content
                        const skeleton = document.getElementById('stats-skeleton');
                        const content = document.getElementById('stats-content');
                        if (skeleton && content) {
                            skeleton.classList.remove('loaded');
                            content.classList.remove('loaded');
                        }
                    });

                    // Enhanced stats loading with skeleton transition
                    window.addEventListener('load', async function() {
                        const skeleton = document.getElementById('stats-skeleton');
                        const content = document.getElementById('stats-content');
                        
                        // Minimum loading time for smooth UX
                        const minLoadTime = 800;
                        const startTime = Date.now();
                        
                        try {
                            const response = await fetch('/api/stats');
                            if (response.ok) {
                                const stats = await response.json();
                                
                                // Update content
                                const totalRecords = document.getElementById('total-records-value');
                                const avgWilks = document.getElementById('avg-wilks-value');
                                const topPercentile = document.getElementById('top-percentile-value');
                                
                                if (totalRecords) totalRecords.textContent = stats.total_records?.toLocaleString() || '-';
                                if (avgWilks) avgWilks.textContent = stats.avg_wilks?.toFixed(1) || '-';
                                if (topPercentile) topPercentile.textContent = (stats.top_percentile?.toFixed(1) + '%') || '-';
                            }
                        } catch (e) {
                            console.warn('Could not load quick stats:', e);
                        }
                        
                        // Ensure minimum loading time, then transition
                        const elapsedTime = Date.now() - startTime;
                        const remainingTime = Math.max(0, minLoadTime - elapsedTime);
                        
                        setTimeout(() => {
                            if (skeleton) {
                                skeleton.classList.add('loaded');
                            }
                            if (content) {
                                content.classList.add('loaded');
                            }
                        }, remainingTime);
                    });
                    "#
                }
                style {
                    r#"
                    .hero-section {
                        text-align: center;
                        padding: 3.5rem 1.25rem;
                        border-radius: 16px;
                        margin: 2rem auto 2.25rem;
                        max-width: 1100px;
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
                        max-width: 1100px;
                        margin: 0 auto;
                    }

                    .feature-card {
                        background: var(--surface);
                        padding: 2rem;
                        border-radius: 14px;
                        border: 1px solid var(--border);
                        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
                        transition: transform 0.2s ease, box-shadow 0.2s ease;
                        text-align: left;
                    }
                    .feature-card:hover { transform: translateY(-4px); box-shadow: 0 8px 24px rgba(0,0,0,0.15); }
                    .feature-card .icon-wrap { margin-bottom: 0.5rem; }
                    .feature-card h3 { margin-bottom: 1rem; color: var(--text-primary); font-size: 1.5rem; }
                    .feature-card p { color: var(--text-secondary); margin-bottom: 1.5rem; line-height: 1.6; }

                    .btn { display: inline-block; padding: 0.75rem 1.5rem; border-radius: 8px; text-decoration: none; font-weight: 600; transition: all 0.2s ease; border: none; cursor: pointer; font-size: 1rem; }
                    .btn-primary { background: var(--primary); color: white; }
                    .btn-primary:hover { background: var(--primary-dark); transform: translateY(-2px); }
                    .btn-secondary { background: var(--secondary); color: white; }
                    .btn-secondary:hover { background: var(--secondary-dark); transform: translateY(-2px); }
                    .btn-tertiary { background: var(--surface-secondary); color: var(--text-primary); border: 2px solid var(--border); }
                    .btn-tertiary:hover { background: var(--border); transform: translateY(-2px); }

                    .quick-stats { margin-top: 2rem; }
                    .quick-stats h2 { text-align: center; margin-bottom: 2rem; color: var(--text-primary); }

                    .stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1.25rem; }
                    .stat-card { background: var(--surface); padding: 2rem; border-radius: 12px; text-align: center; box-shadow: 0 2px 8px rgba(0,0,0,0.1); transition: transform 0.2s ease; border: 1px solid var(--border); }
                    .stat-card:hover { transform: translateY(-2px); }
                    .stat-number { display: block; font-size: 2.5rem; font-weight: 700; color: var(--primary); margin-bottom: 0.5rem; }
                    .stat-label { display: block; color: var(--text-secondary); font-weight: 500; }

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

fn realtime_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--text-secondary);" {
            path d="M22 12h-4l-3 9-6-18-3 9H2" {}
        }
    }
}
