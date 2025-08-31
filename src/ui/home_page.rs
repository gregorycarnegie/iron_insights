use maud::{html, Markup, DOCTYPE};
use crate::ui::components::*;

pub fn render_home_page() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head())
            body {
                div.container {
                    (render_header())
                    div.main-content {
                        div.hero-section {
                            h1 { "Iron Insights" }
                            p.hero-description { 
                                "Advanced powerlifting analytics and visualization platform. Track your progress, analyze your lifts, and share your achievements." 
                            }
                            
                            div.feature-grid {
                                div.feature-card {
                                    h3 { "📊 Analytics" }
                                    p { "Deep dive into your lifting data with advanced visualizations and statistical analysis." }
                                    a href="/analytics" class="btn btn-primary" { "View Analytics" }
                                }
                                
                                div.feature-card {
                                    h3 { "🎨 Share Cards" }
                                    p { "Create beautiful social media cards to share your lifting achievements." }
                                    a href="/sharecard" class="btn btn-secondary" { "Create Share Card" }
                                }
                                
                                div.feature-card {
                                    h3 { "📈 Real-time Data" }
                                    p { "Connect via WebSocket for live data streaming and real-time updates." }
                                    button class="btn btn-tertiary" onclick="connectWebSocket()" { "Connect WebSocket" }
                                }
                            }
                        }
                        
                        div.quick-stats {
                            h2 { "Quick Stats Overview" }
                            div.stats-grid {
                                div.stat-card {
                                    span.stat-number { "—" }
                                    span.stat-label { "Total Records" }
                                }
                                div.stat-card {
                                    span.stat-number { "—" }
                                    span.stat-label { "Avg Wilks Score" }
                                }
                                div.stat-card {
                                    span.stat-number { "—" }
                                    span.stat-label { "Top Percentile" }
                                }
                            }
                        }
                    }
                }
                (crate::ui::components::scripts::render_scripts())
                script {
                    r#"
                    function connectWebSocket() {
                        if (typeof window.connectToWebSocket === 'function') {
                            window.connectToWebSocket();
                        } else {
                            alert('WebSocket functionality not available');
                        }
                    }
                    
                    // Load quick stats on page load
                    window.addEventListener('load', async function() {
                        try {
                            const response = await fetch('/api/stats');
                            if (response.ok) {
                                const stats = await response.json();
                                document.querySelector('.stat-card:nth-child(1) .stat-number').textContent = stats.total_records?.toLocaleString() || '—';
                                document.querySelector('.stat-card:nth-child(2) .stat-number').textContent = stats.avg_wilks?.toFixed(1) || '—';
                                document.querySelector('.stat-card:nth-child(3) .stat-number').textContent = stats.top_percentile?.toFixed(1) + '%' || '—';
                            }
                        } catch (e) {
                            console.warn('Could not load quick stats:', e);
                        }
                    });
                    "#
                }
                style {
                    r#"
                    .hero-section {
                        text-align: center;
                        padding: 3rem 1rem;
                        background: linear-gradient(135deg, var(--primary-light), var(--secondary-light));
                        border-radius: 12px;
                        margin-bottom: 2rem;
                    }
                    
                    .hero-section h1 {
                        font-size: 3rem;
                        margin-bottom: 1rem;
                        background: linear-gradient(135deg, var(--primary), var(--secondary));
                        -webkit-background-clip: text;
                        -webkit-text-fill-color: transparent;
                        background-clip: text;
                    }
                    
                    .hero-description {
                        font-size: 1.25rem;
                        color: var(--text-secondary);
                        margin-bottom: 2rem;
                        max-width: 600px;
                        margin-left: auto;
                        margin-right: auto;
                    }
                    
                    .feature-grid {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                        gap: 2rem;
                        margin-top: 2rem;
                    }
                    
                    .feature-card {
                        background: var(--surface);
                        padding: 2rem;
                        border-radius: 12px;
                        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
                        transition: transform 0.2s ease, box-shadow 0.2s ease;
                        text-align: left;
                    }
                    
                    .feature-card:hover {
                        transform: translateY(-4px);
                        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
                    }
                    
                    .feature-card h3 {
                        margin-bottom: 1rem;
                        color: var(--text-primary);
                        font-size: 1.5rem;
                    }
                    
                    .feature-card p {
                        color: var(--text-secondary);
                        margin-bottom: 1.5rem;
                        line-height: 1.6;
                    }
                    
                    .btn {
                        display: inline-block;
                        padding: 0.75rem 1.5rem;
                        border-radius: 8px;
                        text-decoration: none;
                        font-weight: 600;
                        transition: all 0.2s ease;
                        border: none;
                        cursor: pointer;
                        font-size: 1rem;
                    }
                    
                    .btn-primary {
                        background: var(--primary);
                        color: white;
                    }
                    
                    .btn-primary:hover {
                        background: var(--primary-dark);
                        transform: translateY(-2px);
                    }
                    
                    .btn-secondary {
                        background: var(--secondary);
                        color: white;
                    }
                    
                    .btn-secondary:hover {
                        background: var(--secondary-dark);
                        transform: translateY(-2px);
                    }
                    
                    .btn-tertiary {
                        background: var(--surface-secondary);
                        color: var(--text-primary);
                        border: 2px solid var(--border);
                    }
                    
                    .btn-tertiary:hover {
                        background: var(--border);
                        transform: translateY(-2px);
                    }
                    
                    .quick-stats {
                        margin-top: 3rem;
                    }
                    
                    .quick-stats h2 {
                        text-align: center;
                        margin-bottom: 2rem;
                        color: var(--text-primary);
                    }
                    
                    .stats-grid {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                        gap: 1.5rem;
                    }
                    
                    .stat-card {
                        background: var(--surface);
                        padding: 2rem;
                        border-radius: 12px;
                        text-align: center;
                        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
                        transition: transform 0.2s ease;
                    }
                    
                    .stat-card:hover {
                        transform: translateY(-2px);
                    }
                    
                    .stat-number {
                        display: block;
                        font-size: 2.5rem;
                        font-weight: 700;
                        color: var(--primary);
                        margin-bottom: 0.5rem;
                    }
                    
                    .stat-label {
                        display: block;
                        color: var(--text-secondary);
                        font-weight: 500;
                    }
                    
                    @media (max-width: 768px) {
                        .hero-section h1 {
                            font-size: 2rem;
                        }
                        
                        .hero-description {
                            font-size: 1.1rem;
                        }
                        
                        .feature-grid {
                            grid-template-columns: 1fr;
                        }
                    }
                    "#
                }
            }
        }
    }
}