mod charts;
mod components;
mod data;
mod helpers;
mod models;
mod selectors;
mod share;
mod slices;
mod state;
mod ui;

use self::charts::draw_heatmap;
use self::components::{
    ChartsPanel, CompareModePanel, FaqPanel, OnboardingPanel, PercentilePanel, ResultCardPanel,
    SimulatorPanel,
};
use self::helpers::{
    build_share_url, comparable_lift_value, kg_to_display, parse_query_f32, tier_for_percentile,
};
use self::models::{
    CompareMode, LatestJson, RootIndex, SavedUiState, SliceIndexEntry, SliceRow,
};
use self::selectors::{
    age_options, equip_options, lift_options, metric_options, sex_options, tested_options,
    wc_options,
};
use self::state::{
    init_dataset_load, setup_default_selection_effects, setup_distribution_effect,
    setup_slice_rows_effect,
};
use self::ui::{age_label, metric_label};
use leptos::html::Canvas;
use leptos::prelude::*;
use crate::core::{percentile_for_value, rebin_1d, rebin_2d, HeatmapBin, HistogramBin};

pub fn run() {
    mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    let (calculated, set_calculated) = signal(false);
    let (show_share, set_show_share) = signal(false);
    let (show_main_charts, set_show_main_charts) = signal(false);
    let (share_handle, set_share_handle) = signal(String::new());
    let (share_status, set_share_status) = signal(None::<String>);
    let (query_loaded, set_query_loaded) = signal(false);
    let (unit_pref_loaded, set_unit_pref_loaded) = signal(false);
    let (use_lbs, set_use_lbs) = signal(false);
    let (calculating, set_calculating) = signal(false);
    let (compare_mode, set_compare_mode) = signal(CompareMode::AllLifters);

    let (latest, set_latest) = signal(None::<LatestJson>);
    let (root_index, set_root_index) = signal(None::<RootIndex>);
    let (slice_rows, set_slice_rows) = signal(Vec::<SliceRow>::new());
    let (load_error, set_load_error) = signal(None::<String>);

    let (sex, set_sex) = signal(String::new());
    let (equip, set_equip) = signal(String::new());
    let (wc, set_wc) = signal(String::new());
    let (age, set_age) = signal(String::new());
    let (tested, set_tested) = signal(String::new());
    let (lift, set_lift) = signal(String::new());
    let (metric, set_metric) = signal(String::new());

    let (squat, set_squat) = signal(180.0f32);
    let (bench, set_bench) = signal(120.0f32);
    let (deadlift, set_deadlift) = signal(220.0f32);
    let (bodyweight, set_bodyweight) = signal(90.0f32);
    let (squat_error, set_squat_error) = signal(None::<String>);
    let (bench_error, set_bench_error) = signal(None::<String>);
    let (deadlift_error, set_deadlift_error) = signal(None::<String>);
    let (bodyweight_error, set_bodyweight_error) = signal(None::<String>);
    let (squat_delta, set_squat_delta) = signal(0.0f32);
    let (bench_delta, set_bench_delta) = signal(0.0f32);
    let (deadlift_delta, set_deadlift_delta) = signal(0.0f32);

    let (lift_mult, set_lift_mult) = signal(4usize);
    let (bw_mult, set_bw_mult) = signal(5usize);

    let (hist, set_hist) = signal(None::<HistogramBin>);
    let (heat, set_heat) = signal(None::<HeatmapBin>);
    let (slice_request_id, set_slice_request_id) = signal(0u64);
    let (dist_request_id, set_dist_request_id) = signal(0u64);

    let canvas_ref: NodeRef<Canvas> = NodeRef::new();

    init_dataset_load(set_latest, set_root_index, set_sex, set_equip, set_load_error);
    setup_slice_rows_effect(
        latest,
        root_index,
        sex,
        equip,
        set_slice_rows,
        set_load_error,
        slice_request_id,
        set_slice_request_id,
    );

    let current_row = Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let w = wc.get();
        let a = age.get();
        let t = tested.get();
        let l = lift.get();
        let m = metric.get();

        slice_rows.get().into_iter().find(|row| {
            row.key.sex == s
                && row.key.equip == e
                && row.key.wc == w
                && row.key.age == a
                && row.key.tested == t
                && row.key.lift == l
                && row.key.metric == m
        })
    });

    setup_distribution_effect(
        current_row,
        latest,
        set_hist,
        set_heat,
        set_load_error,
        dist_request_id,
        set_dist_request_id,
    );

    let sex_options = sex_options(root_index);
    let equip_options = equip_options(root_index, sex);
    let tested_options = tested_options(slice_rows, sex, equip, wc, age);
    let wc_options = wc_options(slice_rows, sex, equip);
    let age_options = age_options(slice_rows, sex, equip, wc);
    let lift_options = lift_options(slice_rows, sex, equip, wc, age, tested);
    let metric_options = metric_options(slice_rows, sex, equip, wc, age, tested, lift);
    setup_default_selection_effects(
        equip_options,
        wc_options,
        age_options,
        tested_options,
        lift_options,
        metric_options,
        equip,
        wc,
        age,
        tested,
        lift,
        metric,
        set_equip,
        set_wc,
        set_age,
        set_tested,
        set_lift,
        set_metric,
    );

    let projected_squat = Memo::new(move |_| (squat.get() + squat_delta.get()).clamp(0.0, 600.0));
    let projected_bench = Memo::new(move |_| (bench.get() + bench_delta.get()).clamp(0.0, 600.0));
    let projected_deadlift =
        Memo::new(move |_| (deadlift.get() + deadlift_delta.get()).clamp(0.0, 600.0));
    let projected_total =
        Memo::new(move |_| projected_squat.get() + projected_bench.get() + projected_deadlift.get());

    let user_lift = Memo::new(move |_| {
        comparable_lift_value(
            &sex.get(),
            &equip.get(),
            &lift.get(),
            &metric.get(),
            bodyweight.get(),
            squat.get(),
            bench.get(),
            deadlift.get(),
        )
    });
    let projected_user_lift = Memo::new(move |_| {
        comparable_lift_value(
            &sex.get(),
            &equip.get(),
            &lift.get(),
            &metric.get(),
            bodyweight.get(),
            projected_squat.get(),
            projected_bench.get(),
            projected_deadlift.get(),
        )
    });

    let hist_x_label = Memo::new(move |_| {
        if lift.get() != "T" || metric.get() == "Kg" {
            "Lift (kg)".to_string()
        } else {
            format!("{} Points", metric_label(&metric.get()))
        }
    });

    let rebinned_hist = Memo::new(move |_| {
        hist.get().map(|h| {
            let k = lift_mult.get();
            let counts = rebin_1d(h.counts, k);
            let bin = h.base_bin * k as f32;
            HistogramBin {
                min: h.min,
                max: h.max,
                base_bin: bin,
                counts,
            }
        })
    });

    let rebinned_heat = Memo::new(move |_| {
        heat.get().map(|h| {
            let (grid, w2, h2) =
                rebin_2d(h.grid, h.width, h.height, lift_mult.get(), bw_mult.get());
            HeatmapBin {
                min_x: h.min_x,
                max_x: h.max_x,
                min_y: h.min_y,
                max_y: h.max_y,
                base_x: h.base_x * lift_mult.get() as f32,
                base_y: h.base_y * bw_mult.get() as f32,
                width: w2,
                height: h2,
                grid,
            }
        })
    });

    let percentile =
        Memo::new(move |_| percentile_for_value(rebinned_hist.get().as_ref(), user_lift.get()));
    let projected_percentile = Memo::new(move |_| {
        percentile_for_value(rebinned_hist.get().as_ref(), projected_user_lift.get())
    });
    let percentile_delta = Memo::new(move |_| match (percentile.get(), projected_percentile.get()) {
        (Some((current, _, _)), Some((projected, _, _))) => Some(projected - current),
        _ => None,
    });
    let rank_tier = Memo::new(move |_| percentile.get().map(|(pct, _, _)| tier_for_percentile(pct)));
    let projected_rank_tier =
        Memo::new(move |_| projected_percentile.get().map(|(pct, _, _)| tier_for_percentile(pct)));
    let has_input_error = Memo::new(move |_| {
        squat_error.get().is_some()
            || bench_error.get().is_some()
            || deadlift_error.get().is_some()
            || bodyweight_error.get().is_some()
    });
    let unit_label = Memo::new(move |_| if use_lbs.get() { "lb" } else { "kg" });
    let percentile_percent = Memo::new(move |_| percentile.get().map(|(pct, _, _)| pct * 100.0));
    let compare_summary = Memo::new(move |_| match (compare_mode.get(), percentile.get()) {
        (CompareMode::AllLifters, Some((pct, _, _))) => {
            format!("Across all lifters, you're stronger than {:.1}%.", pct * 100.0)
        }
        (CompareMode::SameBodyweightRange, Some((pct, _, _))) => {
            let low = kg_to_display((bodyweight.get() - 5.0).max(35.0), use_lbs.get());
            let high = kg_to_display((bodyweight.get() + 5.0).min(300.0), use_lbs.get());
            format!(
                "At {:.0}-{:.0}{} bodyweight, you're stronger than {:.1}%.",
                low,
                high,
                unit_label.get(),
                pct * 100.0
            )
        }
        (CompareMode::SameWeightClass, Some((pct, _, _))) => format!(
            "In weight class {}, you're stronger than {:.1}%.",
            wc.get(),
            pct * 100.0
        ),
        (CompareMode::SameAgeClass, Some((pct, _, _))) => format!(
            "In age class {}, you're stronger than {:.1}%.",
            age_label(&age.get()),
            pct * 100.0
        ),
        (_, None) => "Comparison summary appears after a matching slice loads.".to_string(),
    });
    let share_url = Memo::new(move |_| {
        build_share_url(&[
            ("calc", "1".to_string()),
            ("sex", sex.get()),
            ("equip", equip.get()),
            ("wc", wc.get()),
            ("age", age.get()),
            ("tested", tested.get()),
            ("lift", lift.get()),
            ("metric", metric.get()),
            ("s", format!("{:.1}", squat.get())),
            ("b", format!("{:.1}", bench.get())),
            ("d", format!("{:.1}", deadlift.get())),
            ("bw", format!("{:.1}", bodyweight.get())),
            ("sd", format!("{:.1}", squat_delta.get())),
            ("bd", format!("{:.1}", bench_delta.get())),
            ("dd", format!("{:.1}", deadlift_delta.get())),
            ("lm", lift_mult.get().to_string()),
            ("bm", bw_mult.get().to_string()),
            ("handle", share_handle.get()),
        ])
    });

    {
        let canvas_ref = canvas_ref;
        Effect::new(move |_| {
            let Some(canvas) = canvas_ref.get() else {
                return;
            };
            let Some(heat) = rebinned_heat.get() else {
                return;
            };
            draw_heatmap(
                &canvas,
                &heat,
                user_lift.get(),
                bodyweight.get(),
                &hist_x_label.get(),
            );
        });
    }

    Effect::new(move |_| {
        if unit_pref_loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(Some(storage)) = window.local_storage() else {
            set_unit_pref_loaded.set(true);
            return;
        };
        if let Ok(Some(saved_units)) = storage.get_item("iron_insights_units")
            && saved_units == "lb"
        {
            set_use_lbs.set(true);
        }
        set_unit_pref_loaded.set(true);
    });

    Effect::new(move |_| {
        if !unit_pref_loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(Some(storage)) = window.local_storage() else {
            return;
        };
        let units = if use_lbs.get() { "lb" } else { "kg" };
        let _ = storage.set_item("iron_insights_units", units);
    });

    Effect::new(move |_| {
        if query_loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(search) = window.location().search() else {
            return;
        };
        if search.is_empty() {
            if let Ok(Some(storage)) = window.local_storage()
                && let Ok(Some(raw)) = storage.get_item("iron_insights_last_state")
                && let Ok(saved) = serde_json::from_str::<SavedUiState>(&raw)
            {
                set_sex.set(saved.sex);
                set_equip.set(saved.equip);
                set_wc.set(saved.wc);
                set_age.set(saved.age);
                set_tested.set(saved.tested);
                set_lift.set(saved.lift);
                set_metric.set(saved.metric);
                set_squat.set(saved.squat.clamp(0.0, 600.0));
                set_bench.set(saved.bench.clamp(0.0, 600.0));
                set_deadlift.set(saved.deadlift.clamp(0.0, 600.0));
                set_bodyweight.set(saved.bodyweight.clamp(35.0, 300.0));
                set_squat_delta.set(saved.squat_delta.clamp(-50.0, 50.0));
                set_bench_delta.set(saved.bench_delta.clamp(-50.0, 50.0));
                set_deadlift_delta.set(saved.deadlift_delta.clamp(-50.0, 50.0));
                set_lift_mult.set(saved.lift_mult.clamp(1, 4));
                set_bw_mult.set(saved.bw_mult.clamp(1, 5));
                set_share_handle.set(saved.share_handle);
                set_calculated.set(saved.calculated);
            }
            set_query_loaded.set(true);
            return;
        }
        let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) else {
            set_query_loaded.set(true);
            return;
        };

        if let Some(value) = params.get("sex") {
            set_sex.set(value);
        }
        if let Some(value) = params.get("equip") {
            set_equip.set(value);
        }
        if let Some(value) = params.get("wc") {
            set_wc.set(value);
        }
        if let Some(value) = params.get("age") {
            set_age.set(value);
        }
        if let Some(value) = params.get("tested") {
            set_tested.set(value);
        }
        if let Some(value) = params.get("lift") {
            set_lift.set(value);
        }
        if let Some(value) = params.get("metric") {
            set_metric.set(value);
        }
        if let Some(value) = params.get("handle") {
            set_share_handle.set(value);
        }

        set_squat.set(parse_query_f32(params.get("s"), squat.get_untracked(), 0.0, 600.0));
        set_bench.set(parse_query_f32(params.get("b"), bench.get_untracked(), 0.0, 600.0));
        set_deadlift.set(parse_query_f32(
            params.get("d"),
            deadlift.get_untracked(),
            0.0,
            600.0,
        ));
        set_bodyweight.set(parse_query_f32(
            params.get("bw"),
            bodyweight.get_untracked(),
            35.0,
            300.0,
        ));
        set_squat_delta.set(parse_query_f32(
            params.get("sd"),
            squat_delta.get_untracked(),
            -50.0,
            50.0,
        ));
        set_bench_delta.set(parse_query_f32(
            params.get("bd"),
            bench_delta.get_untracked(),
            -50.0,
            50.0,
        ));
        set_deadlift_delta.set(parse_query_f32(
            params.get("dd"),
            deadlift_delta.get_untracked(),
            -50.0,
            50.0,
        ));
        if let Some(value) = params.get("lm")
            && let Ok(mult) = value.parse::<usize>()
        {
            set_lift_mult.set(mult.clamp(1, 4));
        }
        if let Some(value) = params.get("bm")
            && let Ok(mult) = value.parse::<usize>()
        {
            set_bw_mult.set(mult.clamp(1, 5));
        }
        if params.get("calc").as_deref() == Some("1") {
            set_calculated.set(true);
        }
        set_query_loaded.set(true);
    });

    Effect::new(move |_| {
        if !query_loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(Some(storage)) = window.local_storage() else {
            return;
        };
        let snapshot = SavedUiState {
            sex: sex.get(),
            equip: equip.get(),
            wc: wc.get(),
            age: age.get(),
            tested: tested.get(),
            lift: lift.get(),
            metric: metric.get(),
            squat: squat.get(),
            bench: bench.get(),
            deadlift: deadlift.get(),
            bodyweight: bodyweight.get(),
            squat_delta: squat_delta.get(),
            bench_delta: bench_delta.get(),
            deadlift_delta: deadlift_delta.get(),
            lift_mult: lift_mult.get(),
            bw_mult: bw_mult.get(),
            share_handle: share_handle.get(),
            calculated: calculated.get(),
        };
        if let Ok(raw) = serde_json::to_string(&snapshot) {
            let _ = storage.set_item("iron_insights_last_state", &raw);
        }
    });

    view! {
        <div class="page">
            <header class="hero">
                <h1>"How Strong Are You?"</h1>
                <p>
                    {move || {
                        if let Some(err) = load_error.get() {
                            err
                        } else if let Some(l) = latest.get() {
                            if let Some(r) = l.revision {
                                format!("Data version {} ({})", l.version, r)
                            } else {
                                format!("Data version {}", l.version)
                            }
                        } else {
                            "Loading data...".to_string()
                        }
                    }}
                </p>
                <p class="intro">
                    "Enter your lifts to see how you rank among lifters in this dataset. Higher percentile means stronger."
                </p>
            </header>
                <OnboardingPanel
                    sex_options=sex_options
                    sex=sex
                    set_sex=set_sex
                    equip_options=equip_options
                    equip=equip
                    set_equip=set_equip
                    unit_label=unit_label
                    use_lbs=use_lbs
                    set_use_lbs=set_use_lbs
                    squat=squat
                    set_squat=set_squat
                    squat_error=squat_error
                    set_squat_error=set_squat_error
                    bench=bench
                    set_bench=set_bench
                    bench_error=bench_error
                    set_bench_error=set_bench_error
                    deadlift=deadlift
                    set_deadlift=set_deadlift
                    deadlift_error=deadlift_error
                    set_deadlift_error=set_deadlift_error
                    bodyweight=bodyweight
                    set_bodyweight=set_bodyweight
                    bodyweight_error=bodyweight_error
                    set_bodyweight_error=set_bodyweight_error
                    set_squat_delta=set_squat_delta
                    set_bench_delta=set_bench_delta
                    set_deadlift_delta=set_deadlift_delta
                    set_share_handle=set_share_handle
                    set_calculated=set_calculated
                    has_input_error=has_input_error
                    calculating=calculating
                    set_calculating=set_calculating
                    set_share_status=set_share_status
                    tested_options=tested_options
                    tested=tested
                    set_tested=set_tested
                    age_options=age_options
                    age=age
                    set_age=set_age
                    wc_options=wc_options
                    wc=wc
                    set_wc=set_wc
                    lift_options=lift_options
                    lift=lift
                    set_lift=set_lift
                    metric_options=metric_options
                    metric=metric
                    set_metric=set_metric
                    lift_mult=lift_mult
                    set_lift_mult=set_lift_mult
                    bw_mult=bw_mult
                    set_bw_mult=set_bw_mult
                />
                <ResultCardPanel
                    calculated=calculated
                    percentile=percentile
                    rank_tier=rank_tier
                    show_share=show_share
                    set_show_share=set_show_share
                    share_url=share_url
                    share_status=share_status
                    set_share_status=set_share_status
                    share_handle=share_handle
                    set_share_handle=set_share_handle
                    bodyweight=bodyweight
                    squat=squat
                    bench=bench
                    deadlift=deadlift
                    lift=lift
                />

                <CompareModePanel
                    compare_mode=compare_mode
                    set_compare_mode=set_compare_mode
                    compare_summary=compare_summary
                />
                <PercentilePanel
                    percentile_percent=percentile_percent
                    show_main_charts=show_main_charts
                    set_show_main_charts=set_show_main_charts
                />
                <ChartsPanel
                    show_main_charts=show_main_charts
                    rebinned_hist=rebinned_hist
                    user_lift=user_lift
                    hist_x_label=hist_x_label
                    canvas_ref=canvas_ref
                />
                <SimulatorPanel
                    squat_delta=squat_delta
                    set_squat_delta=set_squat_delta
                    bench_delta=bench_delta
                    set_bench_delta=set_bench_delta
                    deadlift_delta=deadlift_delta
                    set_deadlift_delta=set_deadlift_delta
                    projected_total=projected_total
                    projected_squat=projected_squat
                    projected_bench=projected_bench
                    projected_deadlift=projected_deadlift
                    use_lbs=use_lbs
                    unit_label=unit_label
                    projected_percentile=projected_percentile
                    projected_rank_tier=projected_rank_tier
                    percentile_delta=percentile_delta
                />
                <FaqPanel />
        </div>
    }
}

#[cfg(debug_assertions)]
fn debug_log(message: &str) {
    web_sys::console::debug_1(&message.into());
}

#[cfg(not(debug_assertions))]
fn debug_log(_message: &str) {}
