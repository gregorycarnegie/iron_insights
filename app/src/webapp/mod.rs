mod charts;
mod components;
mod cross_sex;
mod data;
mod helpers;
mod models;
mod persistence;
mod selectors;
mod share;
mod slices;
mod state;
mod ui;

use self::charts::draw_heatmap;
use self::cross_sex::{
    CrossSexComparisonCtx, CrossSexHeatCtx, CrossSexHistCtx, CrossSexLiftComparisonCtx,
    CrossSexRowsCtx, choose_cross_sex_slice, make_cross_sex_comparison,
    setup_cross_sex_heat_effect, setup_cross_sex_hist_effect,
    setup_cross_sex_lift_comparison_effect, setup_cross_sex_rows_effect,
};
use self::helpers::{ComparableLifter, comparable_lift_value, tier_for_percentile};
use self::persistence::{
    HashNavCtx, QueryLoadCtx, StatePersistCtx, UnitPrefCtx, setup_hash_nav_effects,
    setup_query_load_effect, setup_state_persist_effect, setup_unit_pref_effects,
};
use self::models::{
    CrossSexLiftComparison, LatestJson, RootIndex, SliceRow, SliceSummary, TrendSeries,
};
use self::selectors::{
    age_options, equip_options, lift_options, metric_options, sex_options, slice_selector_index,
    tested_options, wc_options,
};
use self::state::{
    init_dataset_load, setup_default_selection_effects, setup_distribution_effect,
    setup_slice_rows_effect, setup_slice_summary_effect, setup_trends_effect,
};
use self::ui::metric_label;
use crate::core::{HeatmapBin, HistogramBin, percentile_for_value, rebin_1d, rebin_2d};
use leptos::ev;
use leptos::html::Canvas;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::mount::mount_to;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

pub fn run() {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        mount_to_body(|| view! { <App /> });
        return;
    };

    let Some(app_root) = document
        .get_element_by_id("app")
        .and_then(|el| el.dyn_into::<HtmlElement>().ok())
    else {
        mount_to_body(|| view! { <App /> });
        return;
    };

    let owner = mount_to(app_root, || view! { <App /> });
    if let Some(shell) = document.get_element_by_id("app-shell") {
        shell.remove();
    }
    owner.forget();
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AppPage {
    Ranking,
    Nerds,
    MenVsWomen,
    OneRm,
    PlateCalc,
    Bodyfat,
}

impl AppPage {
    fn label(self) -> &'static str {
        match self {
            AppPage::Ranking => "RANKING",
            AppPage::Nerds => "STATS FOR NERDS",
            AppPage::MenVsWomen => "MEN VS WOMEN",
            AppPage::OneRm => "1RM CALCULATOR",
            AppPage::PlateCalc => "PLATE CALCULATOR",
            AppPage::Bodyfat => "BODYFAT %",
        }
    }

    fn hash(self) -> &'static str {
        match self {
            AppPage::Ranking => "#ranking",
            AppPage::Nerds => "#nerds",
            AppPage::MenVsWomen => "#men-vs-women",
            AppPage::OneRm => "#1rm",
            AppPage::PlateCalc => "#plate-calc",
            AppPage::Bodyfat => "#bodyfat",
        }
    }
}


#[component]
fn App() -> impl IntoView {
    let (calculated, set_calculated) = signal(false);
    let (reveal_tick, set_reveal_tick) = signal(0u64);
    let (use_lbs, set_use_lbs) = signal(false);
    let (calculating, set_calculating) = signal(false);
    let (active_page, set_active_page) = signal(AppPage::Ranking);
    let (page_loaded, set_page_loaded) = signal(false);
    let (query_loaded, set_query_loaded) = signal(false);
    let (unit_pref_loaded, set_unit_pref_loaded) = signal(false);

    let (latest, set_latest) = signal(None::<LatestJson>);
    let (root_index, set_root_index) = signal(None::<RootIndex>);
    let (slice_rows, set_slice_rows) = signal(Vec::<SliceRow>::new());
    let (trend_series, set_trend_series) = signal(Vec::<TrendSeries>::new());
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
    let (summary_request_id, set_summary_request_id) = signal(0u64);
    let (dist_request_id, set_dist_request_id) = signal(0u64);
    let (trends_request_id, set_trends_request_id) = signal(0u64);
    let (slice_summary, set_slice_summary) = signal(None::<SliceSummary>);
    let (_hist_load_ms, set_hist_load_ms) = signal(None::<u32>);
    let (_heat_load_ms, set_heat_load_ms) = signal(None::<u32>);
    let (_summary_load_ms, set_summary_load_ms) = signal(None::<u32>);

    // Cross-sex
    let (male_slice_rows, set_male_slice_rows) = signal(Vec::<SliceRow>::new());
    let (female_slice_rows, set_female_slice_rows) = signal(Vec::<SliceRow>::new());
    let (cross_sex_rows_error, set_cross_sex_rows_error) = signal(None::<String>);
    let (cross_sex_rows_request_id, set_cross_sex_rows_request_id) = signal(0u64);
    let (male_cross_hist, set_male_cross_hist) = signal(None::<HistogramBin>);
    let (female_cross_hist, set_female_cross_hist) = signal(None::<HistogramBin>);
    let (male_cross_heat, set_male_cross_heat) = signal(None::<HeatmapBin>);
    let (female_cross_heat, set_female_cross_heat) = signal(None::<HeatmapBin>);
    let (cross_sex_hist_loading, set_cross_sex_hist_loading) = signal(false);
    let (cross_sex_hist_error, set_cross_sex_hist_error) = signal(None::<String>);
    let (cross_sex_hist_request_id, set_cross_sex_hist_request_id) = signal(0u64);
    let (cross_sex_heat_loading, set_cross_sex_heat_loading) = signal(false);
    let (cross_sex_heat_error, set_cross_sex_heat_error) = signal(None::<String>);
    let (cross_sex_heat_request_id, set_cross_sex_heat_request_id) = signal(0u64);
    let (cross_sex_lift_comparisons, set_cross_sex_lift_comparisons) =
        signal(Vec::<CrossSexLiftComparison>::new());
    let (cross_sex_lift_comparison_loading, set_cross_sex_lift_comparison_loading) = signal(false);
    let (cross_sex_lift_comparison_error, set_cross_sex_lift_comparison_error) =
        signal(None::<String>);
    let (cross_sex_lift_comparison_request_id, set_cross_sex_lift_comparison_request_id) =
        signal(0u64);
    let (heatmap_resize_tick, set_heatmap_resize_tick) = signal(0u32);

    let canvas_ref: NodeRef<Canvas> = NodeRef::new();
    let nerds_page_active = Memo::new(move |_| active_page.get() == AppPage::Nerds);
    let cross_sex_page_active = Memo::new(move |_| active_page.get() == AppPage::MenVsWomen);

    let heatmap_resize_handle = window_event_listener(ev::resize, move |_| {
        set_heatmap_resize_tick.update(|tick| *tick = tick.wrapping_add(1));
    });
    on_cleanup(move || heatmap_resize_handle.remove());

    init_dataset_load(
        set_latest,
        set_root_index,
        set_sex,
        set_equip,
        set_load_error,
    );

    setup_slice_rows_effect(state::SliceRowsEffectContext {
        latest,
        root_index,
        selection: state::SliceRowsSelection { sex, equip },
        outputs: state::SliceRowsOutputs {
            set_slice_rows,
            set_load_error,
        },
        request: state::RequestTracker {
            current: slice_request_id,
            set: set_slice_request_id,
        },
    });

    setup_trends_effect(
        latest,
        root_index,
        sex,
        equip,
        nerds_page_active,
        set_trend_series,
        trends_request_id,
        set_trends_request_id,
    );

    setup_cross_sex_rows_effect(CrossSexRowsCtx {
        page_active: cross_sex_page_active,
        latest,
        root_index,
        equip,
        set_male_rows: set_male_slice_rows,
        set_female_rows: set_female_slice_rows,
        set_error: set_cross_sex_rows_error,
        request: state::RequestTracker {
            current: cross_sex_rows_request_id,
            set: set_cross_sex_rows_request_id,
        },
    });

    let selector_index = slice_selector_index(slice_rows);
    let current_row = Memo::new(move |_| {
        let w = wc.get();
        let a = age.get();
        let t = tested.get();
        let l = lift.get();
        let m = metric.get();
        selector_index.with(|index| index.current_row(&w, &a, &t, &l, &m))
    });

    {
        let fallback_index = selector_index;
        Effect::new(move |_| {
            if !calculated.get() || current_row.get().is_some() {
                return;
            }
            let t = tested.get();
            let l = lift.get();
            let m = metric.get();
            let w = wc.get();
            let a = age.get();
            let candidates = fallback_index.with(|index| index.candidate_rows(&t, &l, &m));
            if candidates.is_empty() {
                return;
            }
            let best = candidates.into_iter().min_by_key(|row| {
                (
                    if row.key.wc == w {
                        0
                    } else if row.key.wc == "All" {
                        1
                    } else {
                        2
                    },
                    if row.key.age == a {
                        0
                    } else if row.key.age == "All Ages" {
                        1
                    } else {
                        2
                    },
                    if row.key.tested == t {
                        0
                    } else if row.key.tested == "All" {
                        1
                    } else {
                        2
                    },
                    if row.key.metric == m { 0 } else { 1 },
                )
            });
            if let Some(row) = best {
                if row.key.wc != w {
                    set_wc.set(row.key.wc.clone());
                }
                if row.key.age != a {
                    set_age.set(row.key.age.clone());
                }
                if row.key.tested != t {
                    set_tested.set(row.key.tested.clone());
                }
                if row.key.lift != l {
                    set_lift.set(row.key.lift.clone());
                }
                if row.key.metric != m {
                    set_metric.set(row.key.metric.clone());
                }
            }
        });
    }

    setup_distribution_effect(state::DistributionEffectContext {
        current_row,
        latest,
        should_load_hist: calculated,
        should_load_heat: nerds_page_active,
        outputs: state::DistributionOutputs {
            set_hist,
            set_heat,
            set_hist_load_ms,
            set_heat_load_ms,
            set_load_error,
        },
        request: state::RequestTracker {
            current: dist_request_id,
            set: set_dist_request_id,
        },
    });

    setup_slice_summary_effect(
        current_row,
        latest,
        set_slice_summary,
        set_summary_load_ms,
        set_load_error,
        summary_request_id,
        set_summary_request_id,
    );

    let sex_opts = sex_options(root_index);
    let equip_opts = equip_options(root_index, sex);
    let tested_opts = tested_options(selector_index, wc, age);
    let wc_opts = wc_options(selector_index);
    let age_opts = age_options(selector_index, wc);
    let lift_opts = lift_options(selector_index, wc, age, tested);
    let metric_opts = metric_options(selector_index, wc, age, tested, lift);

    setup_default_selection_effects(
        state::DefaultSelectionOptions {
            equip: equip_opts,
            wc: wc_opts,
            age: age_opts,
            tested: tested_opts,
            lift: lift_opts,
            metric: metric_opts,
        },
        state::DefaultSelectionSignals {
            equip,
            wc,
            age,
            tested,
            lift,
            metric,
        },
        state::DefaultSelectionSetters {
            equip: set_equip,
            wc: set_wc,
            age: set_age,
            tested: set_tested,
            lift: set_lift,
            metric: set_metric,
        },
    );

    let rebinned_hist = Memo::new(move |_| {
        hist.get().map(|h| {
            let k = lift_mult.get();
            let counts = rebin_1d(h.counts, k);
            let bin = h.base_bin * k as f32;
            HistogramBin::new(h.min, h.max, bin, counts)
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

    let hist_x_label = Memo::new(move |_| {
        if lift.get() != "T" || metric.get() == "Kg" {
            "Lift (kg)".to_string()
        } else {
            format!("{} Points", metric_label(&metric.get()))
        }
    });

    let user_lift = Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let l = lift.get();
        let m = metric.get();
        comparable_lift_value(
            ComparableLifter {
                sex: &s,
                equipment: &e,
                bodyweight: bodyweight.get(),
                squat: squat.get(),
                bench: bench.get(),
                deadlift: deadlift.get(),
            },
            &l,
            &m,
        )
    });

    let percentile =
        Memo::new(move |_| percentile_for_value(rebinned_hist.get().as_ref(), user_lift.get()));
    let rank_tier =
        Memo::new(move |_| percentile.get().map(|(pct, _, _)| tier_for_percentile(pct)));
    let unit_label = Memo::new(move |_| if use_lbs.get() { "lb" } else { "kg" });
    let has_input_error = Memo::new(move |_| {
        squat_error.get().is_some()
            || bench_error.get().is_some()
            || deadlift_error.get().is_some()
            || bodyweight_error.get().is_some()
    });

    let dataset_blurb = Memo::new(move |_| {
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
    });

    let ranking_cohort_blurb = Memo::new(move |_| match slice_summary.get() {
        Some(s) => format!("{} lifters in cohort", s.total),
        None => "Loading cohort...".to_string(),
    });

    // Cross-sex selection and loading
    let male_cross_choice = Memo::new(move |_| {
        choose_cross_sex_slice(
            &male_slice_rows.get(),
            &equip.get(),
            &wc.get(),
            &age.get(),
            &tested.get(),
            &lift.get(),
            &metric.get(),
        )
    });
    let female_cross_choice = Memo::new(move |_| {
        choose_cross_sex_slice(
            &female_slice_rows.get(),
            &equip.get(),
            &wc.get(),
            &age.get(),
            &tested.get(),
            &lift.get(),
            &metric.get(),
        )
    });

    let current_bin = Memo::new(move |_| current_row.get().map(|r| r.entry.bin));

    setup_cross_sex_hist_effect(CrossSexHistCtx {
        page_active: cross_sex_page_active,
        calculated,
        latest,
        male_choice: male_cross_choice,
        female_choice: female_cross_choice,
        current_bin,
        current_hist: hist,
        set_male_hist: set_male_cross_hist,
        set_female_hist: set_female_cross_hist,
        set_loading: set_cross_sex_hist_loading,
        set_error: set_cross_sex_hist_error,
        request: state::RequestTracker {
            current: cross_sex_hist_request_id,
            set: set_cross_sex_hist_request_id,
        },
    });

    setup_cross_sex_heat_effect(CrossSexHeatCtx {
        page_active: cross_sex_page_active,
        calculated,
        latest,
        male_choice: male_cross_choice,
        female_choice: female_cross_choice,
        set_male_heat: set_male_cross_heat,
        set_female_heat: set_female_cross_heat,
        set_loading: set_cross_sex_heat_loading,
        set_error: set_cross_sex_heat_error,
        request: state::RequestTracker {
            current: cross_sex_heat_request_id,
            set: set_cross_sex_heat_request_id,
        },
    });

    setup_cross_sex_lift_comparison_effect(CrossSexLiftComparisonCtx {
        page_active: cross_sex_page_active,
        calculated,
        latest,
        male_rows: male_slice_rows,
        female_rows: female_slice_rows,
        equip,
        wc,
        age,
        tested,
        set_comparisons: set_cross_sex_lift_comparisons,
        set_loading: set_cross_sex_lift_comparison_loading,
        set_error: set_cross_sex_lift_comparison_error,
        request: state::RequestTracker {
            current: cross_sex_lift_comparison_request_id,
            set: set_cross_sex_lift_comparison_request_id,
        },
    });

    // Heatmap canvas drawing
    Effect::new(move |_| {
        let _ = heatmap_resize_tick.get();
        let Some(canvas) = canvas_ref.get() else {
            return;
        };
        let Some(h) = rebinned_heat.get() else {
            return;
        };
        draw_heatmap(
            &canvas,
            &h,
            nerds_page_active.get().then(|| user_lift.get()),
            bodyweight.get(),
            &hist_x_label.get(),
        );
    });

    setup_unit_pref_effects(UnitPrefCtx {
        loaded: unit_pref_loaded,
        set_loaded: set_unit_pref_loaded,
        use_lbs,
        set_use_lbs,
    });

    setup_query_load_effect(QueryLoadCtx {
        query_loaded,
        set_query_loaded,
        set_sex,
        set_equip,
        set_wc,
        set_age,
        set_tested,
        set_lift,
        set_metric,
        squat,
        set_squat,
        bench,
        set_bench,
        deadlift,
        set_deadlift,
        bodyweight,
        set_bodyweight,
        set_squat_delta,
        set_bench_delta,
        set_deadlift_delta,
        set_lift_mult,
        set_bw_mult,
        set_calculated,
    });

    setup_hash_nav_effects(HashNavCtx {
        page_loaded,
        set_page_loaded,
        active_page,
        set_active_page,
    });

    setup_state_persist_effect(StatePersistCtx {
        query_loaded,
        sex,
        equip,
        wc,
        age,
        tested,
        lift,
        metric,
        squat,
        bench,
        deadlift,
        bodyweight,
        squat_delta,
        bench_delta,
        deadlift_delta,
        lift_mult,
        bw_mult,
        calculated,
    });

    let cross_sex_comparison = make_cross_sex_comparison(CrossSexComparisonCtx {
        calculated,
        rows_error: cross_sex_rows_error,
        male_choice: male_cross_choice,
        female_choice: female_cross_choice,
        hist_error: cross_sex_hist_error,
        male_hist: male_cross_hist,
        female_hist: female_cross_hist,
        equip,
        lift,
        metric,
        bodyweight,
        squat,
        bench,
        deadlift,
    });

    // Build context structs for page components
    let ranking_ctx = components::RankingCtx {
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
        heat,
        rebinned_heat,
        canvas_ref: canvas_ref.clone(),
        set_squat_delta,
        set_bench_delta,
        set_deadlift_delta,
        set_lift_mult,
        set_bw_mult,
    };

    let nerds_ctx = components::NerdsCtx {
        dataset_blurb,
        sex_opts: sex_options(root_index),
        sex,
        set_sex,
        equip_opts: equip_options(root_index, sex),
        equip,
        set_equip,
        unit_label,
        use_lbs,
        set_use_lbs,
        wc_opts: wc_options(selector_index),
        wc,
        set_wc,
        age_opts: age_options(selector_index, wc),
        age,
        set_age,
        tested_opts: tested_options(selector_index, wc, age),
        tested,
        set_tested,
        lift_opts: lift_options(selector_index, wc, age, tested),
        lift,
        set_lift,
        metric_opts: metric_options(selector_index, wc, age, tested, lift),
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
        heat,
        rebinned_heat,
        canvas_ref,
        set_squat_delta,
        set_bench_delta,
        set_deadlift_delta,
        set_lift_mult,
        set_bw_mult,
        slice_summary,
        trend_series,
    };

    let mvw_ctx = components::MenVsWomenCtx {
        dataset_blurb,
        calculated,
        cross_sex_comparison,
        male_hist: male_cross_hist,
        female_hist: female_cross_hist,
        male_heat: male_cross_heat,
        female_heat: female_cross_heat,
        hist_loading: cross_sex_hist_loading,
        hist_error: cross_sex_hist_error,
        heat_loading: cross_sex_heat_loading,
        heat_error: cross_sex_heat_error,
        user_lift,
        bodyweight,
        hist_x_label,
        use_lbs,
        unit_label,
        slice_summary,
        lift_comparisons: cross_sex_lift_comparisons,
        lift_comparison_loading: cross_sex_lift_comparison_loading,
        lift_comparison_error: cross_sex_lift_comparison_error,
    };

    view! {
        <div class="app">
            // Sidebar
            <aside class="nav">
                <div class="brand">
                    <div class="brand-mark"><span class="bar"></span>"IRONSCALE"</div>
                    <div class="brand-sub">"// WHERE YOU STAND · EST. 2026"</div>
                </div>
                <ul class="nav-list">
                    {[
                        (AppPage::Ranking, "01", "Ranking"),
                        (AppPage::Nerds, "02", "Stats for Nerds"),
                        (AppPage::MenVsWomen, "03", "Men vs Women"),
                        (AppPage::OneRm, "04", "1RM Calculator"),
                        (AppPage::PlateCalc, "05", "Plate Calculator"),
                        (AppPage::Bodyfat, "06", "Bodyfat %"),
                    ]
                    .into_iter()
                    .map(|(page, num, label)| {
                        view! {
                            <li
                                class="nav-item"
                                class:active=move || active_page.get() == page
                                on:click=move |_| set_active_page.set(page)
                            >
                                <span class="num">{num}</span>
                                {label}
                            </li>
                        }
                    })
                    .collect_view()}
                </ul>
                <div class="nav-foot">
                    {move || dataset_blurb.get()}
                    <br />
                    "OPENPOWERLIFTING"
                </div>
            </aside>

            // Main column
            <main class="main">
                <div class="topbar">
                    <div class="crumb">
                        <span>"IRONSCALE"</span>
                        <span class="sep">"/"</span>
                        <span class="cur">{move || active_page.get().label()}</span>
                    </div>
                    <div class="topbar-right">
                        <span>
                            <span class="live-dot"></span>
                            "LIVE DATA"
                        </span>
                        <span style="color:var(--ink-mute)">"v1.0"</span>
                    </div>
                </div>

                <Show when=move || active_page.get() == AppPage::Ranking>
                    <components::RankingPage ctx=ranking_ctx.clone() />
                </Show>
                <Show when=move || active_page.get() == AppPage::Nerds>
                    <components::NerdsPage ctx=nerds_ctx.clone() />
                </Show>
                <Show when=move || active_page.get() == AppPage::MenVsWomen>
                    <components::MenVsWomenPage ctx=mvw_ctx.clone() />
                </Show>
                <Show when=move || active_page.get() == AppPage::OneRm>
                    <components::OneRmPage />
                </Show>
                <Show when=move || active_page.get() == AppPage::PlateCalc>
                    <components::PlateCalcPage />
                </Show>
                <Show when=move || active_page.get() == AppPage::Bodyfat>
                    <components::BodyfatPage />
                </Show>
            </main>
        </div>
    }
}

#[cfg(debug_assertions)]
fn debug_log(message: &str) {
    web_sys::console::debug_1(&message.into());
}

#[cfg(not(debug_assertions))]
fn debug_log(_message: &str) {}
