// src/ui/components/charts.rs - Modern chart containers
use maud::{html, Markup};

/// Main content area with stats and charts
pub fn render_main_content() -> Markup {
    html! {
        main.content {
            // Stats Overview
            div.stats-grid {
                div.stat-card {
                    div.stat-label { "Total Athletes" }
                    div.stat-value #totalAthletes { "—" }
                    div.stat-change.positive { "↑ 12%" }
                }
                div.stat-card {
                    div.stat-label { "Avg DOTS Score" }
                    div.stat-value #avgDots { "—" }
                    div.stat-change.positive { "↑ 3.5" }
                }
                div.stat-card {
                    div.stat-label { "Records Analyzed" }
                    div.stat-value #recordsAnalyzed { "—" }
                }
                div.stat-card {
                    div.stat-label { "Processing Time" }
                    div.stat-value #processingTime { "—" }
                }
            }
            
            // User Performance Card
            div #userPerformance {
                (render_user_performance_card())
            }
            
            // Percentile Cards
            div.percentile-grid #percentileGrid style="display: none;" {
                div.percentile-card {
                    div.percentile-value #rawPercentile { "—" }
                    div.percentile-label { "Raw Weight Percentile" }
                }
                div.percentile-card.dots {
                    div.percentile-value #dotsPercentile { "—" }
                    div.percentile-label { "DOTS Score Percentile" }
                }
            }
            
            // Global Export Controls
            div.export-controls {
                div.export-section {
                    h4 { "Export Options" }
                    div.export-buttons {
                        button.export-all-btn onclick="exportAllCharts('png')" { "📊 Export All as PNG" }
                        button.export-all-btn onclick="exportAllCharts('svg')" { "📄 Export All as SVG" }
                        button.export-data-btn onclick="exportDataAsCSV()" { "📋 Export Data as CSV" }
                    }
                }
            }

            // Chart Grid
            div.chart-grid {
                (render_chart_container("weightDistribution", "Weight Distribution", true))
                (render_chart_container("dotsDistribution", "DOTS Distribution", true))
                (render_chart_container("bodyweightScatter", "Performance vs Bodyweight", false))
                (render_chart_container("dotsScatter", "DOTS vs Bodyweight", false))
            }
            
            // Rankings Table
            div.chart-container {
                div.chart-header {
                    h3.chart-title { "Top Performances" }
                    div.chart-options {
                        button.chart-option.active data-type="dots" onclick="switchRankings(this)" { "DOTS" }
                        button.chart-option data-type="raw" onclick="switchRankings(this)" { "Raw" }
                    }
                }
                div style="padding: 1rem;" {
                    table.data-table #rankingsTable {
                        thead {
                            tr {
                                th { "Rank" }
                                th { "Name" }
                                th { "BW" }
                                th { "Squat" }
                                th { "Bench" }
                                th { "Deadlift" }
                                th { "Total" }
                                th { "DOTS" }
                            }
                        }
                        tbody #rankingsBody {
                            // Populated dynamically
                        }
                    }
                }
            }
        }
    }
}

/// Individual chart container
fn render_chart_container(id: &str, title: &str, is_histogram: bool) -> Markup {
    html! {
        div.chart-container {
            div.chart-header {
                h3.chart-title { (title) }
                div.chart-options {
                    @if is_histogram {
                        button.chart-option.active data-bins="30" onclick=(format!("changeBins(this, '{}')", id)) { "30" }
                        button.chart-option data-bins="50" onclick=(format!("changeBins(this, '{}')", id)) { "50" }
                        button.chart-option data-bins="100" onclick=(format!("changeBins(this, '{}')", id)) { "100" }
                    } @else {
                        button.chart-option onclick=(format!("toggleTrendline('{}')", id)) { "Trend" }
                        button.chart-option onclick=(format!("togglePoints('{}')", id)) { "Points" }
                    }
                    div.export-dropdown {
                        button.chart-option.export-btn onclick="toggleExportDropdown(this)" { "Export ↓" }
                        div.export-menu style="display: none;" {
                            button onclick=(format!("exportChart('{}', 'png')", id)) { "PNG" }
                            button onclick=(format!("exportChart('{}', 'svg')", id)) { "SVG" }
                            button onclick=(format!("exportChart('{}', 'jpeg')", id)) { "JPEG" }
                        }
                    }
                }
            }
            // Chart with overlay skeleton (absolute, fades out on load)
            div.chart id=(id) {
                div.skeleton-chart id=(format!("{}-skeleton", id)) {}
            }
            div.chart-error id=(format!("{}Error", id)) style="display: none;" {
                "No data available"
            }
        }
    }
}

/// User performance card
fn render_user_performance_card() -> Markup {
    html! {
        div.user-metrics-card {
            div.user-metrics-header {
                h3.user-metrics-title { "Your Performance Analysis" }
                div #strengthLevel {}
            }
            div.user-metrics-grid {
                div.metric-display {
                    div.metric-value #userDotsScore { "—" }
                    div.metric-label { "DOTS Score" }
                }
                div.metric-display {
                    div.metric-value #userWilks { "—" }
                    div.metric-label { "Wilks Score" }
                }
                div.metric-display {
                    div.metric-value #userIPFGLPoints { "—" }
                    div.metric-label { "IPF GL Points" }
                }
            }
        }
    }
}
