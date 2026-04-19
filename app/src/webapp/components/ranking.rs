use super::shared::{Corners, InputForm};
use crate::core::value_for_percentile;
use crate::webapp::charts::draw_ranking_distribution_canvas;
use crate::webapp::helpers::kg_to_display;
use crate::webapp::state::AppState;
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

#[cfg(test)]
mod tests {
    use super::{format_count, next_unlock};
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn format_count_zero() {
        assert_eq!(format_count(0), "0");
    }

    #[wasm_bindgen_test]
    fn format_count_below_thousand() {
        assert_eq!(format_count(999), "999");
    }

    #[wasm_bindgen_test]
    fn format_count_thousands() {
        assert_eq!(format_count(1_000), "1,000");
        assert_eq!(format_count(12_345), "12,345");
    }

    #[wasm_bindgen_test]
    fn format_count_millions() {
        assert_eq!(format_count(1_234_567), "1,234,567");
    }

    #[wasm_bindgen_test]
    fn next_unlock_at_zero_is_intermediate() {
        let (pct, label) = next_unlock(0.0).expect("should have a next unlock");
        assert!((pct - 0.60).abs() < 0.001);
        assert_eq!(label, "INTERMEDIATE");
    }

    #[wasm_bindgen_test]
    fn next_unlock_past_intermediate_is_advanced() {
        let (_, label) = next_unlock(0.61).expect("should have a next unlock");
        assert_eq!(label, "ADVANCED");
    }

    #[wasm_bindgen_test]
    fn next_unlock_past_advanced_is_elite() {
        let (_, label) = next_unlock(0.81).expect("should have a next unlock");
        assert_eq!(label, "ELITE");
    }

    #[wasm_bindgen_test]
    fn next_unlock_past_elite_is_legend() {
        let (_, label) = next_unlock(0.96).expect("should have a next unlock");
        assert_eq!(label, "LEGEND");
    }

    #[wasm_bindgen_test]
    fn next_unlock_at_legend_is_none() {
        assert!(next_unlock(0.99).is_none());
    }
}

#[component]
pub fn RankingPage() -> impl IntoView {
    let app = use_context::<AppState>().expect("AppState must be provided by App");
    let sel = app.selection;
    let inp = app.input;
    let cmp = app.compute;
    let dataset_blurb = cmp.dataset_blurb;
    let ranking_cohort_blurb = cmp.ranking_cohort_blurb;
    let percentile = cmp.percentile;
    let rank_tier = cmp.rank_tier;
    let user_lift = cmp.user_lift;
    let load_error = cmp.load_error;
    let rebinned_hist = cmp.rebinned_hist;
    let hist_x_label = cmp.hist_x_label;
    let calculated = cmp.calculated;
    let use_lbs = inp.use_lbs;
    let unit_label = inp.unit_label;
    let lift = sel.lift;
    let metric = sel.metric;
    let wc = sel.wc;
    let sex = sel.sex;
    let equip = sel.equip;
    let squat = inp.squat;
    let bench = inp.bench;
    let deadlift = inp.deadlift;
    let bodyweight = inp.bodyweight;

    let total_kg = Memo::new(move |_| squat.get() + bench.get() + deadlift.get());
    let pct_num = Memo::new(move |_| percentile.get().map(|(p, _, _)| p * 100.0));
    let pct_fraction = Memo::new(move |_| percentile.get().map(|(p, _, _)| p));
    let pct_display = Memo::new(move |_| {
        pct_num
            .get()
            .map(|p| format!("{:.1}", p))
            .unwrap_or_else(|| "--".to_string())
    });
    let showing_sample_result = Memo::new(move |_| {
        calculated.get()
            && (bodyweight.get() - 90.0).abs() < 0.001
            && (squat.get() - 180.0).abs() < 0.001
            && (bench.get() - 120.0).abs() < 0.001
            && (deadlift.get() - 220.0).abs() < 0.001
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
                    <span class="blurb-meta">{move || dataset_blurb.get()}</span>
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
                        <InputForm />
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

                    <div class="stat-big stat-big-wrap">
                        <div class="stat-row">
                            <div>
                                <div class="label">"OVERALL PERCENTILE / " {move || metric.get().to_uppercase()}</div>
                                <div class="num">
                                    {move || pct_display.get()}
                                    <span class="stat-ordinal">"th"</span>
                                </div>
                                <div class="sub">
                                    {move || {
                                        if !calculated.get() {
                                            "Enter numbers and compute to see your rank.".to_string()
                                        } else if showing_sample_result.get() {
                                            "Sample result loaded. Change any number to see your own rank.".to_string()
                                        } else {
                                            match pct_num.get() {
                                                Some(p) => format!("Ahead of {:.1}% of lifters in this cohort", p),
                                                None => load_error.get().unwrap_or_else(|| "Awaiting data...".to_string()),
                                            }
                                        }
                                    }}
                                </div>
                            </div>
                            <div class="stat-right">
                                <div class="label">"STRENGTH LEVEL"</div>
                                <div class="serif stat-tier">
                                    {move || rank_tier.get().unwrap_or("-").to_uppercase()}
                                </div>
                                <div class="stat-cohort">
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

                    <div class="panel panel-mb">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"02"</span>" DISTRIBUTION CURVE"</span>
                            <span>"LIFTER COUNT BY LIFT VALUE"</span>
                        </div>
                        <div class="panel-body">
                            <p class="chart-summary">
                                {move || {
                                    if !calculated.get() {
                                        "Compute your lifts to see where your mark sits in the cohort distribution.".to_string()
                                    } else {
                                        match pct_num.get() {
                                            Some(p) => format!(
                                                "Your {} {} mark out-lifts {:.1}% of lifters in this selected cohort.",
                                                current_score.get(),
                                                current_score_unit.get(),
                                                p,
                                            ),
                                            None => "Loading the cohort distribution for your selected lift.".to_string(),
                                        }
                                    }
                                }}
                            </p>
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
                            <p class="chart-summary">
                                {move || {
                                    if !calculated.get() {
                                        "Compute to see how far you have climbed through the percentile tiers.".to_string()
                                    } else {
                                        match percentile.get() {
                                            Some((p, _, total)) => format!(
                                                "You are at P{:.1}, ahead of {} of {} lifters in this selected cohort.",
                                                p * 100.0,
                                                beaten_lifters.get().map(format_count).unwrap_or_else(|| "0".to_string()),
                                                format_count(total),
                                            ),
                                            None => "Loading the percentile ladder for your selected cohort.".to_string(),
                                        }
                                    }
                                }}
                            </p>
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
