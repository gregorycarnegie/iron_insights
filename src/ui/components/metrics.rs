// src/ui/components/metrics.rs - Modern metrics display component
use maud::{html, Markup};

/// Renders the user metrics section with modern card design
pub fn render_user_metrics() -> Markup {
    html! {
        div #userMetricsSection style="display: none;" {
            // Performance Summary Card
            div.performance-summary {
                div.summary-header {
                    h2 { "Performance Summary" }
                    div #competitionClass {
                        span.tag.primary { "Raw" }
                    }
                }
                
                div.summary-grid {
                    div.summary-item {
                        div.summary-label { "Competition Total" }
                        div.summary-value {
                            span #totalKg { "—" }
                            span.summary-unit { " kg" }
                        }
                    }
                    div.summary-item {
                        div.summary-label { "DOTS Score" }
                        div.summary-value #dotsValue { "—" }
                    }
                    div.summary-item {
                        div.summary-label { "Classification" }
                        div #classificationBadge {}
                    }
                    div.summary-item {
                        div.summary-label { "National Rank" }
                        div.summary-value #nationalRank { "—" }
                    }
                }
            }
            
            // Comparison to Standards
            div.standards-comparison {
                h3 { "Qualifying Standards" }
                div.standards-grid {
                    div.standard-item {
                        div.standard-name { "Local Meet" }
                        div.standard-progress {
                            div.progress-bar {
                                div.progress-fill #localProgress style="width: 0%;" {}
                            }
                            span.progress-text #localPercentage { "0%" }
                        }
                    }
                    div.standard-item {
                        div.standard-name { "Regional" }
                        div.standard-progress {
                            div.progress-bar {
                                div.progress-fill #regionalProgress style="width: 0%;" {}
                            }
                            span.progress-text #regionalPercentage { "0%" }
                        }
                    }
                    div.standard-item {
                        div.standard-name { "National" }
                        div.standard-progress {
                            div.progress-bar {
                                div.progress-fill #nationalProgress style="width: 0%;" {}
                            }
                            span.progress-text #nationalPercentage { "0%" }
                        }
                    }
                    div.standard-item {
                        div.standard-name { "International" }
                        div.standard-progress {
                            div.progress-bar {
                                div.progress-fill #internationalProgress style="width: 0%;" {}
                            }
                            span.progress-text #internationalPercentage { "0%" }
                        }
                    }
                }
            }
        }
    }
}

/// Additional CSS for metrics components
pub fn render_metrics_styles() -> &'static str {
    r#"
        .performance-summary {
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.5rem;
            padding: 1.5rem;
            margin-bottom: 1.5rem;
        }
        
        .summary-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1.5rem;
        }
        
        .summary-header h2 {
            font-size: 1.25rem;
            font-weight: 600;
            color: var(--text-primary);
            margin: 0;
        }
        
        .summary-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1.5rem;
        }
        
        .summary-item {
            text-align: center;
        }
        
        .summary-label {
            font-size: 0.75rem;
            font-weight: 500;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-tertiary);
            margin-bottom: 0.5rem;
        }
        
        .summary-value {
            font-size: 2rem;
            font-weight: 700;
            color: var(--text-primary);
        }
        
        .summary-unit {
            font-size: 1rem;
            font-weight: 400;
            color: var(--text-secondary);
        }
        
        .standards-comparison {
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.5rem;
            padding: 1.5rem;
        }
        
        .standards-comparison h3 {
            font-size: 1rem;
            font-weight: 600;
            color: var(--text-primary);
            margin-bottom: 1rem;
        }
        
        .standards-grid {
            display: grid;
            gap: 1rem;
        }
        
        .standard-item {
            display: flex;
            align-items: center;
            justify-content: space-between;
        }
        
        .standard-name {
            font-size: 0.875rem;
            font-weight: 500;
            color: var(--text-primary);
            min-width: 100px;
        }
        
        .standard-progress {
            flex: 1;
            display: flex;
            align-items: center;
            gap: 1rem;
            margin-left: 1rem;
        }
        
        .progress-bar {
            flex: 1;
            height: 8px;
            background: var(--light-secondary);
            border-radius: 4px;
            overflow: hidden;
        }
        
        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, var(--primary), var(--primary-light));
            border-radius: 4px;
            transition: width 0.3s ease;
        }
        
        .progress-text {
            font-size: 0.875rem;
            font-weight: 600;
            color: var(--text-primary);
            min-width: 40px;
            text-align: right;
        }
    "#
}