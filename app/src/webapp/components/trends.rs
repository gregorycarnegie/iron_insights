use crate::webapp::models::TrendPoint;
use leptos::prelude::*;
use std::cmp::Ordering;

const CHART_W: f32 = 680.0;
const CHART_H: f32 = 210.0;
const PAD_LEFT: f32 = 68.0;
const PAD_RIGHT: f32 = 14.0;
const PAD_TOP: f32 = 12.0;
const PAD_BOTTOM: f32 = 44.0;
const Y_AXIS_LABEL_X: f32 = 16.0;

#[derive(Clone, PartialEq)]
struct AxisTick {
    key: String,
    y: f32,
    label: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scale {
    Linear,
    Log,
}

impl Scale {
    fn apply(self, value: f32) -> f32 {
        match self {
            Self::Linear => value,
            Self::Log => value.max(1.0).log10(),
        }
    }
}

#[derive(Clone, PartialEq)]
struct TrendRenderState {
    total_path: String,
    total_ticks: Vec<AxisTick>,
    p50_path: String,
    p90_path: String,
    pct_ticks: Vec<AxisTick>,
    years: Option<(i32, i32)>,
    growth_summary: String,
    p50_drift_summary: String,
    p90_drift_summary: String,
}

impl TrendRenderState {
    fn from_points(points: &[TrendPoint]) -> Self {
        let (total_min, total_max) = log_value_range(points, |p| p.total as f32);
        let (pct_min, pct_max) = percentile_range(points);

        Self {
            total_path: line_path_scaled(
                points,
                |p| p.total as f32,
                total_min,
                total_max,
                Scale::Log,
            ),
            total_ticks: log_axis_ticks(total_min, total_max),
            p50_path: line_path_scaled(points, |p| p.p50, pct_min, pct_max, Scale::Linear),
            p90_path: line_path_scaled(points, |p| p.p90, pct_min, pct_max, Scale::Linear),
            pct_ticks: linear_axis_ticks(pct_min, pct_max, 1),
            years: year_span(points),
            growth_summary: growth_summary(points),
            p50_drift_summary: drift_summary(points, |p| p.p50, "Not enough points for p50 drift."),
            p90_drift_summary: drift_summary(points, |p| p.p90, "Not enough points for p90 drift."),
        }
    }
}

#[component]
pub fn TrendsPanel(
    calculated: ReadSignal<bool>,
    trend_points: Memo<Vec<TrendPoint>>,
    trend_note: Memo<String>,
    current_value: Memo<f32>,
    threshold_axis_label: Memo<String>,
) -> impl IntoView {
    let trend_state = Memo::new(move |_| TrendRenderState::from_points(&trend_points.get()));
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
                {metric_grid! {
                    "Cohort size growth" => move || trend_state.get().growth_summary.clone(),
                    "p50 drift" => move || trend_state.get().p50_drift_summary.clone(),
                    "p90 drift" => move || trend_state.get().p90_drift_summary.clone(),
                    "Current input vs historical thresholds" => move || historical_clear_summary.get(),
                }}
                <div class="trend-card">
                    <div class="trend-head">
                        <h3>"Cohort Size by Year"</h3>
                        <div class="trend-legend">
                            <span class="legend-swatch legend-total"></span>
                            <span>"Total lifters"</span>
                        </div>
                    </div>
                    <svg
                        viewBox={format!("0 0 {:.0} {:.0}", CHART_W, CHART_H)}
                        role="img"
                        aria-label="Cohort size trend chart with year on the x-axis and total lifters on a logarithmic y-axis"
                    >
                        <For
                            each=move || trend_state.get().total_ticks.clone()
                            key=|tick| tick.key.clone()
                            let:tick
                        >
                            <g>
                                <line
                                    x1={PAD_LEFT}
                                    y1={tick.y}
                                    x2={CHART_W - PAD_RIGHT}
                                    y2={tick.y}
                                    class="trend-grid"
                                ></line>
                                <text
                                    x={PAD_LEFT - 6.0}
                                    y={tick.y + 3.0}
                                    text-anchor="end"
                                    class="trend-tick"
                                >
                                    {tick.label}
                                </text>
                            </g>
                        </For>
                        <line x1={PAD_LEFT} y1={CHART_H - PAD_BOTTOM} x2={CHART_W - PAD_RIGHT} y2={CHART_H - PAD_BOTTOM} class="trend-axis"></line>
                        <line x1={PAD_LEFT} y1={PAD_TOP} x2={PAD_LEFT} y2={CHART_H - PAD_BOTTOM} class="trend-axis"></line>
                        <path d={move || trend_state.get().total_path.clone()} class="trend-line trend-line-total"></path>
                        <text
                            x={Y_AXIS_LABEL_X}
                            y={(PAD_TOP + (CHART_H - PAD_BOTTOM)) / 2.0}
                            text-anchor="middle"
                            transform={format!(
                                "rotate(-90 {:.1} {:.1})",
                                Y_AXIS_LABEL_X,
                                (PAD_TOP + (CHART_H - PAD_BOTTOM)) / 2.0
                            )}
                            class="trend-axis-label"
                        >
                            "Total Lifters (log scale)"
                        </text>

                        <Show when=move || trend_state.get().years.is_some()>
                            <text x={PAD_LEFT} y={CHART_H - PAD_BOTTOM + 18.0} class="trend-tick">{move || trend_state.get().years.map(|(start, _)| start.to_string()).unwrap_or_default()}</text>
                            <text x={CHART_W - PAD_RIGHT} y={CHART_H - PAD_BOTTOM + 18.0} text-anchor="end" class="trend-tick">{move || trend_state.get().years.map(|(_, end)| end.to_string()).unwrap_or_default()}</text>
                        </Show>
                        <text
                            x={(PAD_LEFT + (CHART_W - PAD_RIGHT)) / 2.0}
                            y={CHART_H - 8.0}
                            text-anchor="middle"
                            class="trend-axis-label"
                        >
                            "Year"
                        </text>
                    </svg>
                    <p class="muted">
                        "Y-axis uses a log scale so early sparse years stay visible beside recent large cohorts."
                    </p>
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
                        viewBox={format!("0 0 {:.0} {:.0}", CHART_W, CHART_H)}
                        role="img"
                        aria-label="Percentile threshold trend chart with year on the x-axis and threshold value on the y-axis"
                    >
                        <For
                            each=move || trend_state.get().pct_ticks.clone()
                            key=|tick| tick.key.clone()
                            let:tick
                        >
                            <g>
                                <line
                                    x1={PAD_LEFT}
                                    y1={tick.y}
                                    x2={CHART_W - PAD_RIGHT}
                                    y2={tick.y}
                                    class="trend-grid"
                                ></line>
                                <text
                                    x={PAD_LEFT - 6.0}
                                    y={tick.y + 3.0}
                                    text-anchor="end"
                                    class="trend-tick"
                                >
                                    {tick.label}
                                </text>
                            </g>
                        </For>
                        <line x1={PAD_LEFT} y1={CHART_H - PAD_BOTTOM} x2={CHART_W - PAD_RIGHT} y2={CHART_H - PAD_BOTTOM} class="trend-axis"></line>
                        <line x1={PAD_LEFT} y1={PAD_TOP} x2={PAD_LEFT} y2={CHART_H - PAD_BOTTOM} class="trend-axis"></line>
                        <path d={move || trend_state.get().p50_path.clone()} class="trend-line trend-line-p50"></path>
                        <path d={move || trend_state.get().p90_path.clone()} class="trend-line trend-line-p90"></path>
                        <text
                            x={Y_AXIS_LABEL_X}
                            y={(PAD_TOP + (CHART_H - PAD_BOTTOM)) / 2.0}
                            text-anchor="middle"
                            transform={format!(
                                "rotate(-90 {:.1} {:.1})",
                                Y_AXIS_LABEL_X,
                                (PAD_TOP + (CHART_H - PAD_BOTTOM)) / 2.0
                            )}
                            class="trend-axis-label"
                        >
                            {move || threshold_axis_label.get()}
                        </text>

                        <Show when=move || trend_state.get().years.is_some()>
                            <text x={PAD_LEFT} y={CHART_H - PAD_BOTTOM + 18.0} class="trend-tick">{move || trend_state.get().years.map(|(start, _)| start.to_string()).unwrap_or_default()}</text>
                            <text x={CHART_W - PAD_RIGHT} y={CHART_H - PAD_BOTTOM + 18.0} text-anchor="end" class="trend-tick">{move || trend_state.get().years.map(|(_, end)| end.to_string()).unwrap_or_default()}</text>
                        </Show>
                        <text
                            x={(PAD_LEFT + (CHART_W - PAD_RIGHT)) / 2.0}
                            y={CHART_H - 8.0}
                            text-anchor="middle"
                            class="trend-axis-label"
                        >
                            "Year"
                        </text>
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

fn year_span(points: &[TrendPoint]) -> Option<(i32, i32)> {
    match (points.first(), points.last()) {
        (Some(first), Some(last)) => Some((first.year, last.year)),
        _ => None,
    }
}

fn growth_summary(points: &[TrendPoint]) -> String {
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
}

fn drift_summary(
    points: &[TrendPoint],
    select: impl Fn(&TrendPoint) -> f32,
    empty_message: &str,
) -> String {
    match (points.first(), points.last()) {
        (Some(first), Some(last)) => {
            format!(
                "{}-{}: {:+.1}",
                first.year,
                last.year,
                select(last) - select(first)
            )
        }
        _ => empty_message.to_string(),
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

fn log_value_range(points: &[TrendPoint], select: impl Fn(&TrendPoint) -> f32) -> (f32, f32) {
    if points.is_empty() {
        return (1.0, 10.0);
    }

    let min = points
        .iter()
        .map(&select)
        .reduce(f32::min)
        .unwrap_or(1.0)
        .max(1.0);
    let max = points
        .iter()
        .map(&select)
        .reduce(f32::max)
        .unwrap_or(10.0)
        .max(min);
    if (max - min).abs() < f32::EPSILON {
        (min, (min * 10.0).max(min + 1.0))
    } else {
        (min, max)
    }
}

fn percentile_range(points: &[TrendPoint]) -> (f32, f32) {
    let (a_min, a_max) = value_range(points, |p| p.p50);
    let (b_min, b_max) = value_range(points, |p| p.p90);
    (a_min.min(b_min), a_max.max(b_max))
}

fn line_path_scaled(
    points: &[TrendPoint],
    select: impl Fn(&TrendPoint) -> f32,
    min: f32,
    max: f32,
    scale: Scale,
) -> String {
    if points.len() < 2 {
        return String::new();
    }

    let plot_w = CHART_W - PAD_LEFT - PAD_RIGHT;

    let mut d = String::new();
    let len = (points.len() - 1) as f32;
    for (idx, point) in points.iter().enumerate() {
        let x = PAD_LEFT
            + if len <= 0.0 {
                0.0
            } else {
                (idx as f32 / len) * plot_w
            };
        let y = scaled_y(select(point), min, max, scale);
        if idx == 0 {
            d.push_str(&format!("M{:.2},{:.2}", x, y));
        } else {
            d.push_str(&format!(" L{:.2},{:.2}", x, y));
        }
    }
    d
}

fn scaled_y(value: f32, min: f32, max: f32, scale: Scale) -> f32 {
    let plot_h = CHART_H - PAD_TOP - PAD_BOTTOM;
    let scaled_min = scale.apply(min);
    let scaled_max = scale.apply(max);
    let span = (scaled_max - scaled_min).max(1.0);
    PAD_TOP + (plot_h - ((scale.apply(value) - scaled_min) / span) * plot_h)
}

fn linear_axis_ticks(min: f32, max: f32, precision: usize) -> Vec<AxisTick> {
    [max, (min + max) / 2.0, min]
        .into_iter()
        .enumerate()
        .map(|(idx, value)| AxisTick {
            key: format!("linear-{idx}-{value:.3}"),
            y: scaled_y(value, min, max, Scale::Linear),
            label: format!("{value:.precision$}", precision = precision),
        })
        .collect()
}

fn log_axis_ticks(min: f32, max: f32) -> Vec<AxisTick> {
    let mut values = Vec::new();
    push_unique_tick(&mut values, min);

    let start_exp = min.log10().floor() as i32;
    let end_exp = max.log10().floor() as i32;
    for exp in start_exp..=end_exp {
        let tick = 10f32.powi(exp);
        if tick > min && tick < max {
            values.push(tick);
        }
    }

    push_unique_tick(&mut values, max);
    values.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

    values
        .into_iter()
        .map(|value| AxisTick {
            key: format!("log-{value:.3}"),
            y: scaled_y(value, min, max, Scale::Log),
            label: format_count_label(value),
        })
        .collect()
}

fn push_unique_tick(values: &mut Vec<f32>, value: f32) {
    if !values
        .iter()
        .any(|existing| (*existing - value).abs() < f32::EPSILON)
    {
        values.push(value);
    }
}

fn format_count_label(value: f32) -> String {
    let raw = value.round().max(0.0) as i64;
    let digits = raw.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);

    for (idx, ch) in digits.chars().enumerate() {
        if idx > 0 && (digits.len() - idx) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }

    out
}
