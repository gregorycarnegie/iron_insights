use crate::webapp::models::TrendPoint;
use leptos::prelude::*;

const CHART_W: f32 = 680.0;
const CHART_H: f32 = 190.0;
const PAD_LEFT: f32 = 52.0;
const PAD_RIGHT: f32 = 14.0;
const PAD_TOP: f32 = 10.0;
const PAD_BOTTOM: f32 = 26.0;

#[component]
pub fn TrendsPanel(
    calculated: ReadSignal<bool>,
    trend_points: Memo<Vec<TrendPoint>>,
    trend_note: Memo<String>,
    current_value: Memo<f32>,
) -> impl IntoView {
    let total_path = Memo::new(move |_| line_path(&trend_points.get(), |p| p.total as f32));
    let total_min = Memo::new(move |_| value_range(&trend_points.get(), |p| p.total as f32).0);
    let total_max = Memo::new(move |_| value_range(&trend_points.get(), |p| p.total as f32).1);

    let p50_path = Memo::new(move |_| line_path(&trend_points.get(), |p| p.p50));
    let p90_path = Memo::new(move |_| line_path(&trend_points.get(), |p| p.p90));
    let pct_min = Memo::new(move |_| {
        let points = trend_points.get();
        let (a_min, _) = value_range(&points, |p| p.p50);
        let (b_min, _) = value_range(&points, |p| p.p90);
        a_min.min(b_min)
    });
    let pct_max = Memo::new(move |_| {
        let points = trend_points.get();
        let (_, a_max) = value_range(&points, |p| p.p50);
        let (_, b_max) = value_range(&points, |p| p.p90);
        a_max.max(b_max)
    });

    let years = Memo::new(move |_| {
        let points = trend_points.get();
        match (points.first(), points.last()) {
            (Some(first), Some(last)) => Some((first.year, last.year)),
            _ => None,
        }
    });
    let growth_summary = Memo::new(move |_| {
        let points = trend_points.get();
        match (points.first(), points.last()) {
            (Some(first), Some(last)) => {
                let delta = i64::from(last.total) - i64::from(first.total);
                let pct = if first.total == 0 {
                    0.0
                } else {
                    (delta as f32 / first.total as f32) * 100.0
                };
                format!(
                    "{}-{}: {:+} lifters ({:+.1}%).",
                    first.year, last.year, delta, pct
                )
            }
            _ => "Not enough points for growth summary.".to_string(),
        }
    });
    let p50_drift_summary = Memo::new(move |_| {
        let points = trend_points.get();
        match (points.first(), points.last()) {
            (Some(first), Some(last)) => {
                format!("{}-{}: {:+.1}", first.year, last.year, last.p50 - first.p50)
            }
            _ => "Not enough points for p50 drift.".to_string(),
        }
    });
    let p90_drift_summary = Memo::new(move |_| {
        let points = trend_points.get();
        match (points.first(), points.last()) {
            (Some(first), Some(last)) => {
                format!("{}-{}: {:+.1}", first.year, last.year, last.p90 - first.p90)
            }
            _ => "Not enough points for p90 drift.".to_string(),
        }
    });
    let historical_clear_summary = Memo::new(move |_| {
        let points = trend_points.get();
        if points.len() < 2 {
            return "Need at least two yearly points for prior-year threshold checks.".to_string();
        }
        let current = current_value.get();
        let prior = &points[..points.len() - 1];
        let last = points.last();
        let prior_p50_hits = prior.iter().filter(|point| current >= point.p50).count();
        let prior_p90_hits = prior.iter().filter(|point| current >= point.p90).count();

        match last {
            Some(last) => format!(
                "Current input {:.1} would clear prior-year p50 in {}/{} years and prior-year p90 in {}/{} years. Against latest year {}, p50={} and p90={}.",
                current,
                prior_p50_hits,
                prior.len(),
                prior_p90_hits,
                prior.len(),
                last.year,
                if current >= last.p50 { "yes" } else { "no" },
                if current >= last.p90 { "yes" } else { "no" }
            ),
            None => "No latest year available for threshold check.".to_string(),
        }
    });

    view! {
        <section class="panel trends">
            <h2>"Trends Over Time"</h2>
            <p class="muted">{move || trend_note.get()}</p>
            <Show
                when=move || calculated.get() && (trend_points.get().len() >= 2)
                fallback=move || {
                    view! {
                        <p class="muted">
                            "Calculate first. Trend lines appear when historical buckets exist for this cohort."
                        </p>
                    }
                }
            >
                <div class="nerd-metrics-grid">
                    <p><strong>"Cohort size growth"</strong>{move || growth_summary.get()}</p>
                    <p><strong>"p50 drift"</strong>{move || p50_drift_summary.get()}</p>
                    <p><strong>"p90 drift"</strong>{move || p90_drift_summary.get()}</p>
                    <p><strong>"Current input vs historical thresholds"</strong>{move || historical_clear_summary.get()}</p>
                </div>
                <div class="trend-card">
                    <div class="trend-head">
                        <h3>"Cohort Size by Year"</h3>
                        <div class="trend-legend">
                            <span class="legend-swatch legend-total"></span>
                            <span>"Total lifters"</span>
                        </div>
                    </div>
                    <svg
                        viewBox="0 0 680 190"
                        role="img"
                        aria-label="Cohort size trend chart with year and lifter count axes"
                    >
                        <line x1={PAD_LEFT} y1={CHART_H - PAD_BOTTOM} x2={CHART_W - PAD_RIGHT} y2={CHART_H - PAD_BOTTOM} class="trend-axis"></line>
                        <line x1={PAD_LEFT} y1={PAD_TOP} x2={PAD_LEFT} y2={CHART_H - PAD_BOTTOM} class="trend-axis"></line>
                        <path d={move || total_path.get()} class="trend-line trend-line-total"></path>

                        <text x="6" y={PAD_TOP + 4.0} class="trend-tick">{move || format!("{:.0}", total_max.get())}</text>
                        <text x="6" y={(PAD_TOP + (CHART_H - PAD_BOTTOM)) / 2.0 + 4.0} class="trend-tick">{move || format!("{:.0}", (total_min.get() + total_max.get()) / 2.0)}</text>
                        <text x="6" y={CHART_H - PAD_BOTTOM + 4.0} class="trend-tick">{move || format!("{:.0}", total_min.get())}</text>

                        <Show when=move || years.get().is_some()>
                            <text x={PAD_LEFT} y={CHART_H - 4.0} class="trend-tick">{move || years.get().map(|(start, _)| start.to_string()).unwrap_or_default()}</text>
                            <text x={CHART_W - PAD_RIGHT} y={CHART_H - 4.0} text-anchor="end" class="trend-tick">{move || years.get().map(|(_, end)| end.to_string()).unwrap_or_default()}</text>
                        </Show>
                    </svg>
                </div>

                <div class="trend-card">
                    <div class="trend-head">
                        <h3>"Percentile Thresholds by Year"</h3>
                        <div class="trend-legend multi">
                            <span class="legend-swatch legend-p50"></span>
                            <span>"p50"</span>
                            <span class="legend-swatch legend-p90"></span>
                            <span>"p90"</span>
                        </div>
                    </div>
                    <svg
                        viewBox="0 0 680 190"
                        role="img"
                        aria-label="Percentile threshold trend chart with year and threshold axes"
                    >
                        <line x1={PAD_LEFT} y1={CHART_H - PAD_BOTTOM} x2={CHART_W - PAD_RIGHT} y2={CHART_H - PAD_BOTTOM} class="trend-axis"></line>
                        <line x1={PAD_LEFT} y1={PAD_TOP} x2={PAD_LEFT} y2={CHART_H - PAD_BOTTOM} class="trend-axis"></line>
                        <path d={move || p50_path.get()} class="trend-line trend-line-p50"></path>
                        <path d={move || p90_path.get()} class="trend-line trend-line-p90"></path>

                        <text x="6" y={PAD_TOP + 4.0} class="trend-tick">{move || format!("{:.1}", pct_max.get())}</text>
                        <text x="6" y={(PAD_TOP + (CHART_H - PAD_BOTTOM)) / 2.0 + 4.0} class="trend-tick">{move || format!("{:.1}", (pct_min.get() + pct_max.get()) / 2.0)}</text>
                        <text x="6" y={CHART_H - PAD_BOTTOM + 4.0} class="trend-tick">{move || format!("{:.1}", pct_min.get())}</text>

                        <Show when=move || years.get().is_some()>
                            <text x={PAD_LEFT} y={CHART_H - 4.0} class="trend-tick">{move || years.get().map(|(start, _)| start.to_string()).unwrap_or_default()}</text>
                            <text x={CHART_W - PAD_RIGHT} y={CHART_H - 4.0} text-anchor="end" class="trend-tick">{move || years.get().map(|(_, end)| end.to_string()).unwrap_or_default()}</text>
                        </Show>
                    </svg>
                    <p class="muted">"Lines show p50 and p90 thresholds for each year bucket."</p>
                </div>
                <div class="trend-caveats">
                    <p class="muted">"Interpretation caveats:"</p>
                    <ul>
                        <li>"Trend lines are cohort-specific to your exact filter set."</li>
                        <li>
                            "Changing equipment, tested status, age class, lift, or metric creates a different cohort and is not a like-for-like time comparison."
                        </li>
                        <li>
                            "Missing years or sharp jumps usually indicate sparse samples in that cohort, not necessarily abrupt population shifts."
                        </li>
                    </ul>
                </div>
            </Show>
        </section>
    }
}

fn value_range(points: &[TrendPoint], select: impl Fn(&TrendPoint) -> f32) -> (f32, f32) {
    if points.is_empty() {
        return (0.0, 1.0);
    }
    let min = points.iter().map(&select).reduce(f32::min).unwrap_or(0.0);
    let max = points.iter().map(&select).reduce(f32::max).unwrap_or(1.0);
    if (max - min).abs() < f32::EPSILON {
        (min, min + 1.0)
    } else {
        (min, max)
    }
}

fn line_path(points: &[TrendPoint], select: impl Fn(&TrendPoint) -> f32) -> String {
    if points.len() < 2 {
        return String::new();
    }

    let (min, max) = value_range(points, &select);
    let plot_w = CHART_W - PAD_LEFT - PAD_RIGHT;
    let plot_h = CHART_H - PAD_TOP - PAD_BOTTOM;
    let span = (max - min).max(1.0);

    let mut d = String::new();
    let len = (points.len() - 1) as f32;
    for (idx, point) in points.iter().enumerate() {
        let x = PAD_LEFT
            + if len <= 0.0 {
                0.0
            } else {
                (idx as f32 / len) * plot_w
            };
        let y = PAD_TOP + (plot_h - ((select(point) - min) / span) * plot_h);
        if idx == 0 {
            d.push_str(&format!("M{:.2},{:.2}", x, y));
        } else {
            d.push_str(&format!(" L{:.2},{:.2}", x, y));
        }
    }
    d
}
