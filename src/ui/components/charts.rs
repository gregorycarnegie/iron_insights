// src/ui/components/charts.rs - Chart container components
use maud::{html, Markup};

/// Responsive chart grid layout with all four charts
pub fn render_chart_grid() -> Markup {
    html! {
        div.chart-grid {
            (render_histogram_chart())
            (render_dots_histogram_chart())
            (render_scatter_chart())
            (render_dots_scatter_chart())
        }
    }
}

/// Raw weight histogram chart container
fn render_histogram_chart() -> Markup {
    html! {
        div {
            h3.chart-title { "Raw Weight Distribution" }
            div #histogram .chart {
                div #histogramError .chart-error style="display: none;" {
                    "No data available for this lift type"
                }
            }
        }
    }
}

/// DOTS score histogram chart container
fn render_dots_histogram_chart() -> Markup {
    html! {
        div {
            h3.chart-title { "DOTS Score Distribution" }
            div #dotsHistogram .chart {
                div #dotsHistogramError .chart-error style="display: none;" {
                    "No DOTS data available - check data processing"
                }
            }
        }
    }
}

/// Raw weight vs bodyweight scatter plot container
fn render_scatter_chart() -> Markup {
    html! {
        div {
            h3.chart-title { "Raw Weight vs Bodyweight" }
            div #scatter .chart {
                div #scatterError .chart-error style="display: none;" {
                    "No scatter data available"
                }
            }
        }
    }
}

/// DOTS vs bodyweight scatter plot container
fn render_dots_scatter_chart() -> Markup {
    html! {
        div {
            h3.chart-title { "DOTS vs Bodyweight" }
            div #dotsScatter .chart {
                div #dotsScatterError .chart-error style="display: none;" {
                    "No DOTS scatter data available"
                }
            }
        }
    }
}

/// Individual chart container with error handling
pub fn render_chart_container(id: &str, title: &str, error_message: &str) -> Markup {
    html! {
        div {
            h3.chart-title { (title) }
            div id=(id) class="chart" {
                div id=(format!("{}Error", id)) class="chart-error" style="display: none;" {
                    (error_message)
                }
            }
        }
    }
}

/// Reusable chart error display component
pub fn render_chart_error(chart_id: &str, message: &str) -> Markup {
    html! {
        div id=(format!("{}Error", chart_id)) class="chart-error" {
            (message)
        }
    }
}