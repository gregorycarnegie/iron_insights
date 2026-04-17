use super::shared::{Corners, InputForm, InputFormCtx};
use crate::core::{HeatmapBin, HistogramBin};
use crate::webapp::charts::draw_ranking_distribution_canvas;
use crate::webapp::helpers::kg_to_display;
use leptos::ev;
use leptos::html::Canvas;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::prelude::*;

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
        rebinned_heat,
        canvas_ref,
        set_squat_delta,
        set_bench_delta,
        set_deadlift_delta,
        set_lift_mult,
        set_bw_mult,
    } = ctx;

    let total_kg = Memo::new(move |_| squat.get() + bench.get() + deadlift.get());
    let pct_num = Memo::new(move |_| percentile.get().map(|(p, _, _)| p * 100.0));
    let pct_display = Memo::new(move |_| {
        pct_num
            .get()
            .map(|p| format!("{:.1}", p))
            .unwrap_or_else(|| "--".to_string())
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
                // ---- INPUT PANEL ----
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

                // ---- RESULT COLUMN ----
                <div>
                    // Mini stats row
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

                    // Big percentile stat
                    <div class="stat-big" style="margin-bottom:24px">
                        <div style="display:flex;justify-content:space-between;align-items:baseline">
                            <div>
                                <div class="label">"OVERALL PERCENTILE · " {move || metric.get().to_uppercase()}</div>
                                <div class="num">
                                    {move || pct_display.get()}
                                    <span style="font-size:32px;color:var(--ink-dim)">"ᵗʰ"</span>
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
                                    {move || rank_tier.get().unwrap_or("—").to_uppercase()}
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

                    // Histogram panel
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

                    // Heatmap panel
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"03"</span>" LIFT × BODYWEIGHT HEATMAP"</span>
                            <span>"DENSITY · YOU = ●"</span>
                        </div>
                        <div class="panel-body">
                            <div class="heat-wrap">
                                {move || {
                                    if rebinned_heat.get().is_some() {
                                        view! {
                                            <canvas
                                                node_ref=canvas_ref.clone()
                                                style="width:100%;height:100%;display:block"
                                            ></canvas>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="notice">
                                                {if calculated.get() { "Heatmap loads on Stats for Nerds page." } else { "Compute first to load data." }}
                                            </div>
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
