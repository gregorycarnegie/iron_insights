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
    ChartsPanel, CompareModePanel, FaqPanel, LogoMark, MeetDayPanel, OnboardingPanel,
    OneRepMaxPanel, PercentilePanel, ProgressPanel, ResultCardPanel, SimulatorPanel, TrendsPanel,
};
use self::helpers::{
    build_share_url, comparable_lift_value, kg_to_display, parse_query_f32, tier_for_percentile,
};
use self::models::{
    CompareMode, LatestJson, RootIndex, SavedUiState, SliceRow, SliceSummary, TrendSeries,
};
use self::selectors::{
    age_options, equip_options, lift_options, metric_options, sex_options, tested_options,
    wc_options,
};
use self::state::{
    init_dataset_load, setup_default_selection_effects, setup_distribution_effect,
    setup_slice_rows_effect, setup_slice_summary_effect, setup_trends_effect,
};
use self::ui::{age_label, metric_label};
use crate::core::{
    HeatmapBin, HistogramBin, percentile_for_value, rebin_1d, rebin_2d, value_for_percentile,
};
use leptos::html::Canvas;
use leptos::prelude::*;

pub fn run() {
    mount_to_body(|| view! { <App /> });
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppPage {
    Rank,
    OneRm,
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
    let (active_page, set_active_page) = signal(AppPage::Rank);
    let (page_loaded, set_page_loaded) = signal(false);

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
    let (target_percentile, set_target_percentile) = signal(90.0f32);

    let (lift_mult, set_lift_mult) = signal(4usize);
    let (bw_mult, set_bw_mult) = signal(5usize);

    let (hist, set_hist) = signal(None::<HistogramBin>);
    let (heat, set_heat) = signal(None::<HeatmapBin>);
    let (slice_request_id, set_slice_request_id) = signal(0u64);
    let (summary_request_id, set_summary_request_id) = signal(0u64);
    let (dist_request_id, set_dist_request_id) = signal(0u64);
    let (trends_request_id, set_trends_request_id) = signal(0u64);
    let (slice_summary, set_slice_summary) = signal(None::<SliceSummary>);
    let (summary_load_ms, set_summary_load_ms) = signal(None::<u32>);
    let (hist_load_ms, set_hist_load_ms) = signal(None::<u32>);
    let (heat_load_ms, set_heat_load_ms) = signal(None::<u32>);

    let canvas_ref: NodeRef<Canvas> = NodeRef::new();

    init_dataset_load(
        set_latest,
        set_root_index,
        set_sex,
        set_equip,
        set_load_error,
    );
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
    setup_trends_effect(
        latest,
        set_trend_series,
        trends_request_id,
        set_trends_request_id,
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

    {
        let set_wc = set_wc;
        let set_age = set_age;
        let set_tested = set_tested;
        let set_lift = set_lift;
        let set_metric = set_metric;
        Effect::new(move |_| {
            if !calculated.get() || current_row.get().is_some() {
                return;
            }

            let s = sex.get();
            let e = equip.get();
            let t = tested.get();
            let l = lift.get();
            let m = metric.get();
            let w = wc.get();
            let a = age.get();

            let mut candidates: Vec<SliceRow> = slice_rows
                .get()
                .into_iter()
                .filter(|row| row.key.sex == s && row.key.equip == e)
                .collect();
            if candidates.is_empty() {
                return;
            }

            // Prefer current tested/lift/metric, then gracefully broaden.
            let exact: Vec<SliceRow> = candidates
                .iter()
                .filter(|row| row.key.tested == t && row.key.lift == l && row.key.metric == m)
                .cloned()
                .collect();
            if !exact.is_empty() {
                candidates = exact;
            } else {
                let tested_lift: Vec<SliceRow> = candidates
                    .iter()
                    .filter(|row| row.key.tested == t && row.key.lift == l)
                    .cloned()
                    .collect();
                if !tested_lift.is_empty() {
                    candidates = tested_lift;
                }
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

    setup_distribution_effect(
        current_row,
        latest,
        calculated,
        show_main_charts,
        set_hist,
        set_heat,
        set_hist_load_ms,
        set_heat_load_ms,
        set_load_error,
        dist_request_id,
        set_dist_request_id,
    );
    setup_slice_summary_effect(
        current_row,
        latest,
        set_slice_summary,
        set_summary_load_ms,
        set_load_error,
        summary_request_id,
        set_summary_request_id,
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
    let projected_total = Memo::new(move |_| {
        projected_squat.get() + projected_bench.get() + projected_deadlift.get()
    });

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
    let metric_is_kg_comparable = Memo::new(move |_| lift.get() != "T" || metric.get() == "Kg");

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
    let percentile_delta =
        Memo::new(
            move |_| match (percentile.get(), projected_percentile.get()) {
                (Some((current, _, _)), Some((projected, _, _))) => Some(projected - current),
                _ => None,
            },
        );
    let rank_tier =
        Memo::new(move |_| percentile.get().map(|(pct, _, _)| tier_for_percentile(pct)));
    let projected_rank_tier = Memo::new(move |_| {
        projected_percentile
            .get()
            .map(|(pct, _, _)| tier_for_percentile(pct))
    });
    let target_lift_value = Memo::new(move |_| {
        value_for_percentile(
            rebinned_hist.get().as_ref(),
            target_percentile.get().clamp(50.0, 99.0) / 100.0,
        )
    });
    let target_kg_needed = Memo::new(move |_| {
        if !calculated.get() || !metric_is_kg_comparable.get() {
            return None;
        }
        let target = target_lift_value.get()?;
        Some((target - user_lift.get()).max(0.0))
    });
    let target_summary = Memo::new(move |_| {
        if !calculated.get() {
            return "Press Calculate to estimate a target lift.".to_string();
        }
        if !metric_is_kg_comparable.get() {
            return "Target planner is available when metric is kg-based.".to_string();
        }
        match (target_lift_value.get(), target_kg_needed.get()) {
            (Some(target), Some(needed)) if needed <= 0.0 => format!(
                "You are already at or above the {:.0}% target (~{:.1} kg).",
                target_percentile.get(),
                target
            ),
            (Some(target), Some(needed)) => format!(
                "To reach ~{:.0}% in this slice, aim for about {:.1} kg ({:.1} kg more).",
                target_percentile.get(),
                target,
                needed
            ),
            _ => "Target estimate unavailable for this slice.".to_string(),
        }
    });
    let has_input_error = Memo::new(move |_| {
        squat_error.get().is_some()
            || bench_error.get().is_some()
            || deadlift_error.get().is_some()
            || bodyweight_error.get().is_some()
    });
    let unit_label = Memo::new(move |_| if use_lbs.get() { "lb" } else { "kg" });
    let percentile_percent = Memo::new(move |_| percentile.get().map(|(pct, _, _)| pct * 100.0));
    let result_unavailable_reason = Memo::new(move |_| {
        if !calculated.get() || percentile.get().is_some() {
            return None::<String>;
        }
        if current_row.get().is_none() {
            return Some(format!(
                "No slice found for sex={}, equip={}, wc={}, age={}, tested={}, lift={}, metric={}.",
                sex.get(),
                equip.get(),
                wc.get(),
                age.get(),
                tested.get(),
                lift.get(),
                metric.get()
            ));
        }
        if hist.get().is_none() {
            return Some(load_error.get().unwrap_or_else(|| {
                "Histogram payload was not loaded for this slice.".to_string()
            }));
        }
        Some("Distribution exists but percentile could not be computed.".to_string())
    });
    let summary_blurb = Memo::new(move |_| match slice_summary.get() {
        Some(summary) => format!(
            "Cohort summary: {} lifters, {}-{:.1} kg range.",
            summary.total, summary.min_kg, summary.max_kg
        ),
        None => "Cohort summary loading...".to_string(),
    });
    let load_timing_blurb = Memo::new(move |_| {
        let mut parts = Vec::new();
        if let Some(ms) = summary_load_ms.get() {
            parts.push(format!("summary {ms}ms"));
        }
        if let Some(ms) = hist_load_ms.get() {
            parts.push(format!("hist {ms}ms"));
        }
        if let Some(ms) = heat_load_ms.get() {
            parts.push(format!("heat {ms}ms"));
        }
        if parts.is_empty() {
            "Load timings: pending".to_string()
        } else {
            format!("Load timings: {}", parts.join(" | "))
        }
    });
    let compare_summary = Memo::new(move |_| match (compare_mode.get(), percentile.get()) {
        (CompareMode::AllLifters, Some((pct, _, _))) => {
            format!(
                "Across all lifters, you're stronger than {:.1}%.",
                pct * 100.0
            )
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
        (CompareMode::SameTestedStatus, Some((pct, _, _))) => format!(
            "Within {} meets, you're stronger than {:.1}%.",
            tested.get(),
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
    let trend_key = Memo::new(move |_| {
        format!(
            "sex={}|equip={}|tested={}|lift={}|metric={}",
            sex.get(),
            equip.get(),
            tested.get(),
            lift.get(),
            metric.get()
        )
    });
    let selected_trend_points = Memo::new(move |_| {
        let key = trend_key.get();
        trend_series
            .get()
            .into_iter()
            .find(|series| series.key == key)
            .map(|series| series.points)
            .unwrap_or_default()
    });
    let trend_note = Memo::new(move |_| {
        let points = selected_trend_points.get();
        if let (Some(first), Some(last)) = (points.first(), points.last()) {
            format!(
                "Year buckets {}-{} for {} / {} / {} / {} / {}. Cohort size {} -> {}.",
                first.year,
                last.year,
                sex.get(),
                equip.get(),
                tested.get(),
                lift.get(),
                metric.get(),
                first.total,
                last.total
            )
        } else {
            "Time buckets use yearly snapshots from best-lift records; sparse cohorts may have missing years.".to_string()
        }
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

        set_squat.set(parse_query_f32(
            params.get("s"),
            squat.get_untracked(),
            0.0,
            600.0,
        ));
        set_bench.set(parse_query_f32(
            params.get("b"),
            bench.get_untracked(),
            0.0,
            600.0,
        ));
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
        if page_loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        if let Ok(hash) = window.location().hash() {
            if hash.eq_ignore_ascii_case("#1rm") {
                set_active_page.set(AppPage::OneRm);
            } else {
                set_active_page.set(AppPage::Rank);
            }
        }
        set_page_loaded.set(true);
    });

    Effect::new(move |_| {
        if !page_loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let hash = match active_page.get() {
            AppPage::Rank => "#rank",
            AppPage::OneRm => "#1rm",
        };
        let _ = window.location().set_hash(hash);
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
            <header class="panel topbar">
                <div class="brand">
                    <LogoMark />
                    <div class="brand-copy">
                        <p class="brand-title">"Iron Insights"</p>
                        <p class="brand-subtitle">"Powerlifting data explorer"</p>
                    </div>
                </div>
                <nav class="page-nav" aria-label="Main pages">
                    <button
                        type="button"
                        class:chip=true
                        class:active=move || active_page.get() == AppPage::Rank
                        on:click=move |_| set_active_page.set(AppPage::Rank)
                    >
                        "Ranking"
                    </button>
                    <button
                        type="button"
                        class:chip=true
                        class:active=move || active_page.get() == AppPage::OneRm
                        on:click=move |_| set_active_page.set(AppPage::OneRm)
                    >
                        "1RM Calculator"
                    </button>
                </nav>
            </header>

            <Show when=move || active_page.get() == AppPage::Rank>
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
                    <p class="muted">{move || summary_blurb.get()}</p>
                    <p class="muted">{move || load_timing_blurb.get()}</p>
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
                    load_error=load_error
                    unavailable_reason=result_unavailable_reason
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
                <ProgressPanel
                    calculated=calculated
                    percentile=percentile
                    sex=sex
                    equip=equip
                    wc=wc
                    age=age
                    tested=tested
                    lift=lift
                    metric=metric
                    squat=squat
                    bench=bench
                    deadlift=deadlift
                    bodyweight=bodyweight
                    use_lbs=use_lbs
                    unit_label=unit_label
                />

                <CompareModePanel
                    compare_mode=compare_mode
                    set_compare_mode=set_compare_mode
                    tested=tested
                    set_tested=set_tested
                    equip=equip
                    set_equip=set_equip
                    age=age
                    set_age=set_age
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
                    target_percentile=target_percentile
                    set_target_percentile=set_target_percentile
                    target_kg_needed=target_kg_needed
                    target_summary=target_summary
                />
                <MeetDayPanel
                    squat=squat
                    bench=bench
                    deadlift=deadlift
                    use_lbs=use_lbs
                    unit_label=unit_label
                />
                <TrendsPanel
                    calculated=calculated
                    trend_points=selected_trend_points
                    trend_note=trend_note
                />
                <FaqPanel />
            </Show>

            <Show when=move || active_page.get() == AppPage::OneRm>
                <header class="hero">
                    <h1>"Estimate Your 1-Rep Max"</h1>
                    <p>"Use a training set to estimate your max with common formulas."</p>
                </header>
                <OneRepMaxPanel />
            </Show>

            <footer class="panel attribution">
                <p>
                    "Data powered by "
                    <a href="https://www.openpowerlifting.org/" target="_blank" rel="noopener noreferrer">
                        "OpenPowerlifting.org"
                    </a>
                    ". Huge appreciation to their team for keeping this public dataset free."
                </p>
            </footer>
        </div>
    }
}

#[cfg(debug_assertions)]
fn debug_log(message: &str) {
    web_sys::console::debug_1(&message.into());
}

#[cfg(not(debug_assertions))]
fn debug_log(_message: &str) {}
