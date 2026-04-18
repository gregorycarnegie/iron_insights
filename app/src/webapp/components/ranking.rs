use super::shared::{Corners, InputForm, InputFormCtx};
use crate::core::{HeatmapBin, HistogramBin, value_for_percentile};
use crate::webapp::charts::draw_ranking_distribution_canvas;
use crate::webapp::helpers::kg_to_display;
use crate::webapp::ui::metric_label;
use leptos::ev;
use leptos::html::Canvas;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::prelude::*;

const LADDER_DOTS: usize = 240;
const LADDER_MARKS: [(f32, &str, &str); 5] = [
    (0.50, "P50", "MEDIAN"),
    (0.75, "P75", "TOP 25%"),
    (0.90, "P90", "TOP 10%"),
    (0.95, "P95", "ELITE"),
    (0.99, "P99", "LEGEND"),
];
const RANK_UNLOCKS: [(f32, &str); 4] = [
    (0.60, "INTERMEDIATE"),
    (0.80, "ADVANCED"),
    (0.95, "ELITE"),
    (0.99, "LEGEND"),
];

fn format_count(value: u32) -> String {
    let raw = value.to_string();
    let mut out = String::with_capacity(raw.len() + raw.len() / 3);
    for (idx, ch) in raw.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn score_unit(lift: &str, metric: &str, use_lbs: bool) -> String {
    if lift != "T" || metric == "Kg" {
        if use_lbs {
            "LB".to_string()
        } else {
            "KG".to_string()
        }
    } else {
        metric_label(metric).to_uppercase()
    }
}

fn display_score(value: f32, lift: &str, metric: &str, use_lbs: bool) -> f32 {
    if lift != "T" || metric == "Kg" {
        kg_to_display(value, use_lbs)
    } else {
        value
    }
}

fn format_score(value: f32, lift: &str, metric: &str, use_lbs: bool) -> String {
    let shown = display_score(value, lift, metric, use_lbs);
    if (shown - shown.round()).abs() < 0.05 {
        format!("{shown:.0}")
    } else {
        format!("{shown:.1}")
    }
}

fn sex_label(code: &str) -> &'static str {
    match code {
        "M" => "MALE",
        "F" => "FEMALE",
        _ => "ALL",
    }
}

fn weight_class_label(wc: &str) -> String {
    if wc == "All" {
        "ALL CLASSES".to_string()
    } else {
        format!("{wc}KG")
    }
}

fn next_unlock(current_pct: f32) -> Option<(f32, &'static str)> {
    RANK_UNLOCKS
        .into_iter()
        .find(|(target_pct, _)| current_pct < target_pct - 0.0001)
}

#[derive(Clone)]
pub struct RankingCtx {
    pub dataset_blurb: Memo<String>,
    pub ranking_cohort_blurb: Memo<String>,
    pub sex_opts: Memo<Vec<String>>,
    pub sex: ReadSignal<String>,
    pub set_sex: WriteSignal<String>,
    pub equip_opts: Memo<Vec<String>>,
    pub equip: ReadSignal<String>,
    pub set_equip: WriteSignal<String>,
    pub unit_label: Memo<&'static str>,
    pub use_lbs: ReadSignal<bool>,
    pub set_use_lbs: WriteSignal<bool>,
    pub wc_opts: Memo<Vec<String>>,
    pub wc: ReadSignal<String>,
    pub set_wc: WriteSignal<String>,
    pub age_opts: Memo<Vec<String>>,
    pub age: ReadSignal<String>,
    pub set_age: WriteSignal<String>,
    pub tested_opts: Memo<Vec<String>>,
    pub tested: ReadSignal<String>,
    pub set_tested: WriteSignal<String>,
    pub lift_opts: Memo<Vec<String>>,
    pub lift: ReadSignal<String>,
    pub set_lift: WriteSignal<String>,
    pub metric_opts: Memo<Vec<String>>,
    pub metric: ReadSignal<String>,
    pub set_metric: WriteSignal<String>,
    pub squat: ReadSignal<f32>,
    pub set_squat: WriteSignal<f32>,
    pub squat_error: ReadSignal<Option<String>>,
    pub set_squat_error: WriteSignal<Option<String>>,
    pub bench: ReadSignal<f32>,
    pub set_bench: WriteSignal<f32>,
    pub bench_error: ReadSignal<Option<String>>,
    pub set_bench_error: WriteSignal<Option<String>>,
    pub deadlift: ReadSignal<f32>,
    pub set_deadlift: WriteSignal<f32>,
    pub deadlift_error: ReadSignal<Option<String>>,
    pub set_deadlift_error: WriteSignal<Option<String>>,
    pub bodyweight: ReadSignal<f32>,
    pub set_bodyweight: WriteSignal<f32>,
    pub bodyweight_error: ReadSignal<Option<String>>,
    pub set_bodyweight_error: WriteSignal<Option<String>>,
    pub calculated: ReadSignal<bool>,
    pub set_calculated: WriteSignal<bool>,
    pub calculating: ReadSignal<bool>,
    pub set_calculating: WriteSignal<bool>,
    pub has_input_error: Memo<bool>,
    pub reveal_tick: ReadSignal<u64>,
    pub set_reveal_tick: WriteSignal<u64>,
    pub percentile: Memo<Option<(f32, usize, u32)>>,
    pub rank_tier: Memo<Option<&'static str>>,
    pub user_lift: Memo<f32>,
    pub load_error: ReadSignal<Option<String>>,
    pub rebinned_hist: Memo<Option<HistogramBin>>,
    pub hist_x_label: Memo<String>,
    pub heat: ReadSignal<Option<HeatmapBin>>,
    pub rebinned_heat: Memo<Option<HeatmapBin>>,
    pub canvas_ref: NodeRef<Canvas>,
    pub set_squat_delta: WriteSignal<f32>,
    pub set_bench_delta: WriteSignal<f32>,
    pub set_deadlift_delta: WriteSignal<f32>,
    pub set_lift_mult: WriteSignal<usize>,
    pub set_bw_mult: WriteSignal<usize>,
}

#[component]
pub fn RankingPage(ctx: RankingCtx) -> impl IntoView {
    let RankingCtx {
        dataset_blurb,
        ranking_cohort_blurb,
        sex_opts,
        sex,
        set_sex,
        equip_opts,
        equip,
        set_equip,
        unit_label,
        use_lbs,
        set_use_lbs,
        wc_opts,
        wc,
        set_wc,
        age_opts,
        age,
        set_age,
        tested_opts,
        tested,
        set_tested,
        lift_opts,
        lift,
        set_lift,
        metric_opts,
        metric,
        set_metric,
        squat,
        set_squat,
        squat_error,
        set_squat_error,
        bench,
        set_bench,
        bench_error,
        set_bench_error,
        deadlift,
        set_deadlift,
        deadlift_error,
        set_deadlift_error,
        bodyweight,
        set_bodyweight,
        bodyweight_error,
        set_bodyweight_error,
        calculated,
        set_calculated,
        calculating,
        set_calculating,
        has_input_error,
        reveal_tick,
        set_reveal_tick,
        percentile,
        rank_tier,
        user_lift,
        load_error,
        rebinned_hist,
        hist_x_label,
        heat: _heat,
        rebinned_heat: _rebinned_heat,
        canvas_ref: _canvas_ref,
        set_squat_delta,
        set_bench_delta,
        set_deadlift_delta,
        set_lift_mult,
        set_bw_mult,
    } = ctx;

    let total_kg = Memo::new(move |_| squat.get() + bench.get() + deadlift.get());
    let pct_num = Memo::new(move |_| percentile.get().map(|(p, _, _)| p * 100.0));
    let pct_fraction = Memo::new(move |_| percentile.get().map(|(p, _, _)| p));
    let pct_display = Memo::new(move |_| {
        pct_num
            .get()
            .map(|p| format!("{:.1}", p))
            .unwrap_or_else(|| "--".to_string())
    });
    let beaten_lifters = Memo::new(move |_| {
        percentile
            .get()
            .map(|(p, _, total)| ((p * total as f32).round() as u32).min(total))
    });
    let filled_dots = Memo::new(move |_| {
        pct_fraction
            .get()
            .map(|p| (p * LADDER_DOTS as f32).round() as usize)
            .unwrap_or(0)
            .min(LADDER_DOTS)
    });
    let ladder_cohort = Memo::new(move |_| {
        format!(
            "{} / {} / {} / {}",
            sex_label(&sex.get()),
            weight_class_label(&wc.get()),
            equip.get().to_uppercase(),
            metric_label(&metric.get()).to_uppercase()
        )
    });
    let current_score = Memo::new(move |_| {
        let l = lift.get();
        let m = metric.get();
        format_score(user_lift.get(), &l, &m, use_lbs.get())
    });
    let current_score_unit =
        Memo::new(move |_| score_unit(&lift.get(), &metric.get(), use_lbs.get()));
    let next_unlock_copy = Memo::new(move |_| {
        let Some(pct) = pct_fraction.get() else {
            return "Compute to reveal your next tier.".to_string();
        };
        let Some((target_pct, label)) = next_unlock(pct) else {
            return "Top tier reached.".to_string();
        };
        format!("P{:.0} / {}", target_pct * 100.0, label)
    });
    let next_unlock_delta = Memo::new(move |_| {
        let Some(pct) = pct_fraction.get() else {
            return "--".to_string();
        };
        let Some((target_pct, _)) = next_unlock(pct) else {
            return "LOCKED IN".to_string();
        };
        let Some(hist) = rebinned_hist.get() else {
            return "--".to_string();
        };
        let Some(target_value) = value_for_percentile(Some(&hist), target_pct) else {
            return "--".to_string();
        };
        let l = lift.get();
        let m = metric.get();
        let needed = display_score(
            (target_value - user_lift.get()).max(0.0),
            &l,
            &m,
            use_lbs.get(),
        );
        if needed <= 0.05 {
            "REACHED".to_string()
        } else if (needed - needed.round()).abs() < 0.05 {
            format!("+{needed:.0}")
        } else {
            format!("+{needed:.1}")
        }
    });

    let hist_canvas: NodeRef<Canvas> = NodeRef::new();
    let (chart_resize_tick, set_chart_resize_tick) = signal(0u32);
    let resize_handle = window_event_listener(ev::resize, move |_| {
        set_chart_resize_tick.update(|tick| *tick = tick.wrapping_add(1));
    });
    on_cleanup(move || resize_handle.remove());

    Effect::new(move |_| {
        let _ = chart_resize_tick.get();
        let Some(canvas) = hist_canvas.get() else {
            return;
        };
        let Some(hist) = rebinned_hist.get() else {
            return;
        };
        draw_ranking_distribution_canvas(
            &canvas,
            &hist,
            calculated.get().then(|| user_lift.get()),
            &hist_x_label.get(),
        );
    });

    let form_ctx = InputFormCtx {
        sex_opts,
        sex,
        set_sex,
        equip_opts,
        equip,
        set_equip,
        unit_label,
        use_lbs,
        set_use_lbs,
        wc_opts,
        wc,
        set_wc,
        age_opts,
        age,
        set_age,
        tested_opts,
        tested,
        set_tested,
        lift_opts,
        lift,
        set_lift,
        metric_opts,
        metric,
        set_metric,
        squat,
        set_squat,
        squat_error,
        set_squat_error,
        bench,
        set_bench,
        bench_error,
        set_bench_error,
        deadlift,
        set_deadlift,
        deadlift_error,
        set_deadlift_error,
        bodyweight,
        set_bodyweight,
        bodyweight_error,
        set_bodyweight_error,
        calculated,
        set_calculated,
        calculating,
        set_calculating,
        has_input_error,
        reveal_tick,
        set_reveal_tick,
        set_squat_delta,
        set_bench_delta,
        set_deadlift_delta,
        set_lift_mult,
        set_bw_mult,
    };

    view! {
        <section class="page active" id="page-ranking">
            <div class="page-head">
                <h1 class="page-title">
                    <span class="dim">"Where do"</span>
                    <br />
                    "you " <span class="accent">"stack"</span> "?"
                </h1>
                <p class="page-lede">
                    <span class="serif">"Enter your numbers."</span>
                    " We place you against 2M+ lifters from federated meets."
                    <br />
                    <span style="font-size:11px;color:var(--ink-mute)">{move || dataset_blurb.get()}</span>
                </p>
            </div>

            <div class="rank-grid">
                <div class="panel">
                    <Corners />
                    <div class="panel-head">
                        <span><span class="tag">"01"</span>" YOUR NUMBERS"</span>
                        <span>"INPUT"</span>
                    </div>
                    <div class="panel-body">
                        <InputForm ctx=form_ctx />
                    </div>
                </div>

                <div>
                    <div class="rank-stats">
                        <div class="mini-stat">
                            <div class="l">"TOTAL"</div>
                            <div class="v">
                                {move || format!("{:.0}", kg_to_display(total_kg.get(), use_lbs.get()))}
                                <span class="unit">{move || unit_label.get().to_uppercase()}</span>
                            </div>
                        </div>
                        <div class="mini-stat">
                            <div class="l">"SQUAT"</div>
                            <div class="v">
                                {move || format!("{:.0}", kg_to_display(squat.get(), use_lbs.get()))}
                                <span class="unit">{move || unit_label.get().to_uppercase()}</span>
                            </div>
                        </div>
                        <div class="mini-stat">
                            <div class="l">"BENCH"</div>
                            <div class="v">
                                {move || format!("{:.0}", kg_to_display(bench.get(), use_lbs.get()))}
                                <span class="unit">{move || unit_label.get().to_uppercase()}</span>
                            </div>
                        </div>
                        <div class="mini-stat">
                            <div class="l">"DEADLIFT"</div>
                            <div class="v">
                                {move || format!("{:.0}", kg_to_display(deadlift.get(), use_lbs.get()))}
                                <span class="unit">{move || unit_label.get().to_uppercase()}</span>
                            </div>
                        </div>
                    </div>

                    <div class="stat-big" style="margin-bottom:24px">
                        <div style="display:flex;justify-content:space-between;align-items:baseline">
                            <div>
                                <div class="label">"OVERALL PERCENTILE / " {move || metric.get().to_uppercase()}</div>
                                <div class="num">
                                    {move || pct_display.get()}
                                    <span style="font-size:32px;color:var(--ink-dim)">"th"</span>
                                </div>
                                <div class="sub">
                                    {move || {
                                        if !calculated.get() {
                                            "Enter numbers and compute to see your rank.".to_string()
                                        } else {
                                            match pct_num.get() {
                                                Some(p) => format!("Ahead of {:.1}% of lifters in this cohort", p),
                                                None => load_error.get().unwrap_or_else(|| "Awaiting data...".to_string()),
                                            }
                                        }
                                    }}
                                </div>
                            </div>
                            <div style="text-align:right">
                                <div class="label">"STRENGTH LEVEL"</div>
                                <div class="serif" style="font-size:24px;color:var(--chalk);margin-top:4px">
                                    {move || rank_tier.get().unwrap_or("-").to_uppercase()}
                                </div>
                                <div style="font-size:11px;color:var(--ink-mute);margin-top:4px">
                                    {move || ranking_cohort_blurb.get()}
                                </div>
                            </div>
                        </div>
                        <div class="bar-track">
                            <div
                                class="bar-fill"
                                style=move || format!("width:{}%", pct_num.get().unwrap_or(0.0))
                            ></div>
                        </div>
                    </div>

                    <div class="panel" style="margin-bottom:24px">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"02"</span>" DISTRIBUTION CURVE"</span>
                            <span>"LIFTER COUNT BY LIFT VALUE"</span>
                        </div>
                        <div class="panel-body">
                            <div class="hist-wrap">
                                {move || {
                                    match rebinned_hist.get() {
                                        Some(_) => view! {
                                            <canvas
                                                node_ref=hist_canvas.clone()
                                                class="hist"
                                                role="img"
                                                aria-label="Distribution curve with your input marker"
                                            ></canvas>
                                        }.into_any(),
                                        None => view! {
                                            <div class="notice">
                                                {if calculated.get() { "Loading distribution..." } else { "Compute to load distribution." }}
                                            </div>
                                        }.into_any(),
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"03"</span>" PERCENTILE CLIMB"</span>
                            <span>{move || ladder_cohort.get()}</span>
                        </div>
                        <div class="panel-body">
                            <div
                                class="tier-ladder"
                                role="img"
                                aria-label=move || {
                                    match percentile.get() {
                                        Some((p, _, total)) => format!(
                                            "Tier ladder showing your percentile at {:.1}, ahead of {} of {} lifters.",
                                            p * 100.0,
                                            beaten_lifters.get().map(format_count).unwrap_or_else(|| "0".to_string()),
                                            format_count(total)
                                        ),
                                        None => "Tier ladder waiting for calculation.".to_string(),
                                    }
                                }
                            >
                                <div class="tier-ladder-title">
                                    "You're climbing the " <span>"tier ladder"</span> "."
                                </div>
                                <div class="tier-ladder-track-wrap">
                                    <div class="tier-ladder-track">
                                        <div
                                            class="tier-ladder-fill"
                                            style=move || format!("width:{:.1}%", pct_num.get().unwrap_or(0.0))
                                        ></div>
                                    </div>

                                    {LADDER_MARKS
                                        .into_iter()
                                        .map(|(target_pct, tag, label)| {
                                            view! {
                                                <div class="tier-mark" style=format!("left:{:.1}%", target_pct * 100.0)>
                                                    <div class="tier-mark-label">
                                                        <span>{tag}</span>
                                                        {label}
                                                    </div>
                                                    <div class="tier-mark-score">
                                                        {move || {
                                                            rebinned_hist
                                                                .get()
                                                                .and_then(|hist| value_for_percentile(Some(&hist), target_pct))
                                                                .map(|value| {
                                                                    let l = lift.get();
                                                                    let m = metric.get();
                                                                    format!(
                                                                        "{} {}",
                                                                        current_score_unit.get(),
                                                                        format_score(value, &l, &m, use_lbs.get())
                                                                    )
                                                                })
                                                                .unwrap_or_else(|| "--".to_string())
                                                        }}
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect_view()}

                                    <div
                                        class="tier-you-marker"
                                        class:active=move || calculated.get() && pct_num.get().is_some()
                                        style=move || format!("left:{:.1}%", pct_num.get().unwrap_or(0.0).clamp(0.0, 100.0))
                                    >
                                        <div class="tier-you-plate">"YOU"</div>
                                        <div class="tier-you-stem"></div>
                                        <div class="tier-you-label">
                                            <span>{move || format!("{}th", pct_display.get())}</span>
                                            {move || format!("YOU / {} {}", current_score_unit.get(), current_score.get())}
                                        </div>
                                    </div>
                                </div>

                                {move || {
                                    if !calculated.get() || percentile.get().is_none() {
                                        view! {
                                            <div class="notice">
                                                "Compute to reveal the climb, lifters beaten, and next unlock."
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <>
                                                <div class="beat-grid">
                                                    <div>
                                                        <div class="beat-label">
                                                            "LIFTERS BENEATH YOU / SELECTED COHORT"
                                                        </div>
                                                        <div class="beat-dots">
                                                            {(0..LADDER_DOTS)
                                                                .map(|idx| {
                                                                    view! {
                                                                        <span
                                                                            class="beat-dot"
                                                                            class:on=move || idx < filled_dots.get()
                                                                            style=format!("animation-delay:{}ms", idx * 3)
                                                                        ></span>
                                                                    }
                                                                })
                                                                .collect_view()}
                                                        </div>
                                                    </div>
                                                    <div class="beat-readout">
                                                        <div class="beat-num">
                                                            {move || beaten_lifters.get().map(format_count).unwrap_or_else(|| "--".to_string())}
                                                        </div>
                                                        <div class="beat-caption">"LIFTERS BEATEN"</div>
                                                        <div class="beat-sub">
                                                            {move || percentile.get()
                                                                .map(|(_, _, total)| format!("OF {} TOTAL", format_count(total)))
                                                                .unwrap_or_else(|| "OF -- TOTAL".to_string())}
                                                        </div>
                                                    </div>
                                                </div>

                                                <div class="unlock-row">
                                                    <div class="unlock-copy">
                                                        "NEXT UNLOCK"
                                                        <span>{move || next_unlock_copy.get()}</span>
                                                    </div>
                                                    <div class="unlock-value">
                                                        {move || next_unlock_delta.get()}
                                                        <span>{move || current_score_unit.get()}</span>
                                                    </div>
                                                </div>
                                            </>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
