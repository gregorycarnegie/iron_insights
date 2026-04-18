use super::shared::{Corners, InputForm, InputFormCtx};
use crate::core::{HeatmapBin, HistogramBin};
use crate::webapp::charts::render_histogram_svg;
use crate::webapp::models::{SliceSummary, TrendSeries};
use leptos::html::Canvas;
use leptos::prelude::*;

#[allow(dead_code)]
#[derive(Clone)]
pub struct NerdsCtx {
    pub dataset_blurb: Memo<String>,
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
    pub slice_summary: ReadSignal<Option<SliceSummary>>,
    pub trend_series: ReadSignal<Vec<TrendSeries>>,
}

#[component]
pub fn NerdsPage(ctx: NerdsCtx) -> impl IntoView {
    let NerdsCtx {
        dataset_blurb,
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
        load_error: _,
        rebinned_hist,
        hist_x_label,
        heat: _,
        rebinned_heat,
        canvas_ref,
        set_squat_delta,
        set_bench_delta,
        set_deadlift_delta,
        set_lift_mult,
        set_bw_mult,
        slice_summary,
        trend_series: _,
    } = ctx;

    let total_kg = Memo::new(move |_| squat.get() + bench.get() + deadlift.get());
    let squat_pct = Memo::new(move |_| {
        let t = total_kg.get();
        if t > 0.0 {
            squat.get() / t * 100.0
        } else {
            0.0
        }
    });
    let bench_pct = Memo::new(move |_| {
        let t = total_kg.get();
        if t > 0.0 {
            bench.get() / t * 100.0
        } else {
            0.0
        }
    });
    let dl_pct = Memo::new(move |_| {
        let t = total_kg.get();
        if t > 0.0 {
            deadlift.get() / t * 100.0
        } else {
            0.0
        }
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
        <section class="page active" id="page-nerds">
            <div class="page-head">
                <h1 class="page-title">
                    "Stats for " <span class="accent">"nerds"</span> "."
                </h1>
                <p class="page-lede">
                    <span class="serif">"Distribution, heatmap, and lift ratios"</span>
                    " for your selected cohort. Data from " {move || dataset_blurb.get()} "."
                </p>
            </div>

            <div class="nerd-grid">
                // Left: heatmap panel
                <div>
                    <div class="panel" style="margin-bottom:24px">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"MAP"</span>" LIFT × BODYWEIGHT HEATMAP"</span>
                            <span>"DENSITY"</span>
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
                                                {if calculated.get() { "Loading heatmap..." } else { "Compute first to load heatmap." }}
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                    // Histogram
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"DIST"</span>" DISTRIBUTION CURVE"</span>
                            <span>"LIFTER COUNT"</span>
                        </div>
                        <div class="panel-body">
                            <div class="hist-wrap">
                                {move || match rebinned_hist.get() {
                                    Some(h) => render_histogram_svg(
                                        &h,
                                        calculated.get().then(|| user_lift.get()),
                                        &hist_x_label.get(),
                                    ),
                                    None => view! {
                                        <div class="notice">
                                            {if calculated.get() { "Loading distribution..." } else { "Compute first." }}
                                        </div>
                                    }.into_any(),
                                }}
                            </div>
                        </div>
                    </div>
                </div>

                // Right: stats panels
                <div class="nerd-side">
                    // Your lift ratios (live from inputs)
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"μ"</span>" LIFT RATIOS"</span>
                            <span>"OF YOUR TOTAL"</span>
                        </div>
                        <div class="panel-body">
                            <div class="dist-bar">
                                <div class="lb">
                                    <span>"SQUAT"</span>
                                    <span>{move || format!("{:.1}%", squat_pct.get())}</span>
                                </div>
                                <div class="tr">
                                    <div class="fl" style=move || format!("width:{:.1}%", squat_pct.get())></div>
                                </div>
                            </div>
                            <div class="dist-bar">
                                <div class="lb">
                                    <span>"BENCH"</span>
                                    <span>{move || format!("{:.1}%", bench_pct.get())}</span>
                                </div>
                                <div class="tr">
                                    <div class="fl" style=move || format!("width:{:.1}%", bench_pct.get())></div>
                                </div>
                            </div>
                            <div class="dist-bar">
                                <div class="lb">
                                    <span>"DEADLIFT"</span>
                                    <span>{move || format!("{:.1}%", dl_pct.get())}</span>
                                </div>
                                <div class="tr">
                                    <div class="fl" style=move || format!("width:{:.1}%", dl_pct.get())></div>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Population stats from cohort
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"P"</span>" COHORT"</span>
                            <span>"SUMMARY"</span>
                        </div>
                        <div class="panel-body" style="font-size:12px;line-height:1.9">
                            {move || match slice_summary.get() {
                                Some(s) => view! {
                                    <div>
                                        <div style="display:flex;justify-content:space-between">
                                            <span style="color:var(--ink-dim)">"TOTAL LIFTERS"</span>
                                            <span style="font-family:'Archivo Black',sans-serif">{s.total.to_string()}</span>
                                        </div>
                                        <div style="display:flex;justify-content:space-between">
                                            <span style="color:var(--ink-dim)">"MIN LIFT"</span>
                                            <span style="font-family:'Archivo Black',sans-serif">{format!("{:.1} KG", s.min_kg)}</span>
                                        </div>
                                        <div style="display:flex;justify-content:space-between">
                                            <span style="color:var(--ink-dim)">"MAX LIFT"</span>
                                            <span style="font-family:'Archivo Black',sans-serif">{format!("{:.1} KG", s.max_kg)}</span>
                                        </div>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="notice">
                                        {if calculated.get() { "Loading cohort summary..." } else { "Compute first." }}
                                    </div>
                                }.into_any(),
                            }}
                        </div>
                    </div>

                    // Your percentile result
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"PCT"</span>" YOUR PERCENTILE"</span>
                            <span>"RESULT"</span>
                        </div>
                        <div class="panel-body">
                            {move || match percentile.get() {
                                Some((p, _, _)) => view! {
                                    <div>
                                        <div class="stat-big">
                                            <div class="label">"PERCENTILE"</div>
                                            <div class="num">{format!("{:.1}", p * 100.0)}<span style="font-size:28px;color:var(--ink-dim)">"ᵗʰ"</span></div>
                                            <div class="sub">{rank_tier.get().unwrap_or("—").to_uppercase()}</div>
                                            <div class="bar-track">
                                                <div class="bar-fill" style=format!("width:{:.1}%", p * 100.0)></div>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="notice">
                                        {if calculated.get() { "Awaiting distribution data..." } else { "Compute first." }}
                                    </div>
                                }.into_any(),
                            }}
                        </div>
                    </div>

                    // Input form
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"IN"</span>" YOUR NUMBERS"</span>
                            <span>"FILTER"</span>
                        </div>
                        <div class="panel-body">
                            <InputForm ctx=form_ctx />
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
