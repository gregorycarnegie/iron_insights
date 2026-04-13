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
    LogoMark, MenVsWomenPage, NerdsPage, OneRmPage, PlateCalcPage, RankingPage,
};
use self::data::{fetch_binary_first, fetch_json_first};
use self::helpers::{
    ComparableLifter, build_share_url, comparable_lift_value, kg_to_display, parse_query_f32,
    tier_for_percentile,
};
use self::models::{
    CohortComparisonRow, CompareMode, CrossSexComparison, LatestJson, RootIndex, SavedUiState,
    SliceIndex, SliceIndexEntries, SliceRow, SliceSummary, TrendSeries,
};
use self::selectors::{
    age_options, equip_options, lift_options, metric_options, sex_options, slice_selector_index,
    tested_options, wc_options,
};
use self::slices::{entry_from_slice_key, parse_slice_key};
use self::state::{
    init_dataset_load, setup_default_selection_effects, setup_distribution_effect,
    setup_slice_rows_effect, setup_slice_summary_effect, setup_trends_effect,
};
use self::ui::lift_label;
use self::ui::{age_label, metric_label};
use crate::core::{
    HeatmapBin, HistogramBin, bodyweight_conditioned_percentile,
    equivalent_value_for_same_percentile, histogram_density_for_value, histogram_diagnostics,
    parse_combined_bin, percentile_for_value, rebin_1d, rebin_2d, value_for_percentile,
};
use leptos::ev;
use leptos::html::Canvas;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::mount::mount_to;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::BTreeMap;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

pub fn run() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
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
enum AppPage {
    Rank,
    StatsForNerds,
    MenVsWomen,
    OneRm,
    PlateCalc,
}

const MIN_CROSS_SEX_COHORT_TOTAL: u32 = 50;

#[derive(Clone, PartialEq)]
struct CrossSexSliceChoice {
    row: SliceRow,
    weight_class_fallback: bool,
}

#[derive(Clone, Copy)]
struct ComparisonSliceFilter<'a> {
    sex: &'a str,
    equip: &'a str,
    wc: &'a str,
    age: &'a str,
    tested: &'a str,
    lift: &'a str,
    metric: &'a str,
}

struct CohortComparisonContext<'a> {
    sex: &'a str,
    equip: &'a str,
    lift: &'a str,
    base_total: Option<u32>,
}

struct CohortComparisonVariant<'a> {
    label: &'a str,
    wc: String,
    age: String,
    tested: String,
    metric: String,
    is_current: bool,
}

fn kg_needed_for_percentile_step(
    hist: Option<&HistogramBin>,
    current_value: f32,
    current_pct: f32,
    percentile_step: f32,
) -> Option<f32> {
    let hist = hist?;
    let target_pct = (current_pct + percentile_step / 100.0).clamp(0.0, 0.999);
    if target_pct <= current_pct {
        return Some(0.0);
    }
    let target_value = value_for_percentile(Some(hist), target_pct)?;
    Some((target_value - current_value).max(0.0))
}

fn percentile_points_for_kg_gain(
    hist: Option<&HistogramBin>,
    current_value: f32,
    current_pct: f32,
    kg_gain: f32,
) -> Option<f32> {
    let next_pct = percentile_for_value(hist, current_value + kg_gain)?.0;
    Some(((next_pct - current_pct) * 100.0).max(0.0))
}

fn dataset_file_url(version: &str, path: &str) -> String {
    let trimmed = path.trim_start_matches('/');
    format!("data/{version}/{trimmed}")
}

fn find_comparison_slice<'a>(
    rows: &'a [SliceRow],
    filter: ComparisonSliceFilter<'_>,
) -> Option<&'a SliceRow> {
    rows.iter().find(|row| {
        row.key.sex == filter.sex
            && row.key.equip == filter.equip
            && row.key.wc == filter.wc
            && row.key.age == filter.age
            && row.key.tested == filter.tested
            && row.key.lift == filter.lift
            && row.key.metric == filter.metric
    })
}

fn rows_from_slice_index(index: SliceIndex) -> Vec<SliceRow> {
    let mut rows = Vec::new();
    match index.slices {
        SliceIndexEntries::Map(entries) => {
            rows.reserve(entries.len());
            for (raw_key, entry) in entries {
                if let Some(key) = parse_slice_key(&raw_key) {
                    rows.push(SliceRow { key, entry });
                }
            }
        }
        SliceIndexEntries::Keys(keys) => {
            rows.reserve(keys.len());
            for raw_key in keys {
                if let Some((key, entry)) = entry_from_slice_key(&raw_key) {
                    rows.push(SliceRow { key, entry });
                }
            }
        }
    }
    rows.sort_by(|a, b| a.key.cmp(&b.key));
    rows
}

fn choose_cross_sex_slice(
    rows: &[SliceRow],
    equip: &str,
    wc: &str,
    age: &str,
    tested: &str,
    lift: &str,
    metric: &str,
) -> Option<CrossSexSliceChoice> {
    let exact = rows
        .iter()
        .find(|row| {
            row.key.equip == equip
                && row.key.wc == wc
                && row.key.age == age
                && row.key.tested == tested
                && row.key.lift == lift
                && row.key.metric == metric
        })
        .cloned();
    if let Some(row) = exact {
        return Some(CrossSexSliceChoice {
            row,
            weight_class_fallback: false,
        });
    }

    rows.iter()
        .find(|row| {
            row.key.equip == equip
                && row.key.wc == "All"
                && row.key.age == age
                && row.key.tested == tested
                && row.key.lift == lift
                && row.key.metric == metric
        })
        .cloned()
        .map(|row| CrossSexSliceChoice {
            row,
            weight_class_fallback: true,
        })
}

fn build_cohort_comparison_row(
    rows: &[SliceRow],
    context: &CohortComparisonContext<'_>,
    variant: CohortComparisonVariant<'_>,
) -> CohortComparisonRow {
    let CohortComparisonVariant {
        label,
        wc,
        age,
        tested,
        metric,
        is_current,
    } = variant;
    let row = find_comparison_slice(
        rows,
        ComparisonSliceFilter {
            sex: context.sex,
            equip: context.equip,
            wc: &wc,
            age: &age,
            tested: &tested,
            lift: context.lift,
            metric: &metric,
        },
    );

    let (total, min_kg, max_kg, bin_path, status, status_ok) = match row {
        Some(found) => match found.entry.summary.as_ref() {
            Some(summary) => (
                Some(summary.total),
                Some(summary.min_kg),
                Some(summary.max_kg),
                Some(found.entry.bin.clone()),
                "embedded summary".to_string(),
                true,
            ),
            None => (
                None,
                None,
                None,
                Some(found.entry.bin.clone()),
                "slice found; summary missing".to_string(),
                false,
            ),
        },
        None => (None, None, None, None, "slice missing".to_string(), false),
    };

    let total_delta = match (total, context.base_total) {
        (Some(_), Some(_)) if is_current => Some(0),
        (Some(candidate), Some(base)) => Some(i64::from(candidate) - i64::from(base)),
        _ => None,
    };

    CohortComparisonRow {
        id: format!(
            "wc={wc}|age={age}|tested={tested}|lift={}|metric={metric}",
            context.lift
        ),
        label: label.to_string(),
        wc,
        age,
        tested,
        metric,
        total,
        total_delta,
        min_kg,
        max_kg,
        status,
        status_ok,
        bin_path,
        is_current,
    }
}

#[component]
fn App() -> impl IntoView {
    let (calculated, set_calculated) = signal(false);
    let (reveal_tick, set_reveal_tick) = signal(0u64);
    let (show_share, set_show_share) = signal(false);
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
    let (show_nerd_heat_indicator, set_show_nerd_heat_indicator) = signal(true);

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
    let (cohort_exact_deltas_enabled, set_cohort_exact_deltas_enabled) = signal(false);
    let (cohort_exact_percentiles, set_cohort_exact_percentiles) =
        signal(BTreeMap::<String, Option<f32>>::new());
    let (cohort_exact_loading, set_cohort_exact_loading) = signal(false);
    let (cohort_exact_error, set_cohort_exact_error) = signal(None::<String>);
    let (cohort_exact_request_id, set_cohort_exact_request_id) = signal(0u64);
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
    let (heatmap_resize_tick, set_heatmap_resize_tick) = signal(0u32);

    let canvas_ref: NodeRef<Canvas> = NodeRef::new();
    let nerds_page_active = Memo::new(move |_| active_page.get() == AppPage::StatsForNerds);
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
    Effect::new(move |_| {
        let next_request_id = cross_sex_rows_request_id.get_untracked().wrapping_add(1);
        set_cross_sex_rows_request_id.set(next_request_id);

        if !cross_sex_page_active.get() {
            set_male_slice_rows.set(Vec::new());
            set_female_slice_rows.set(Vec::new());
            set_cross_sex_rows_error.set(None);
            return;
        }

        let (Some(latest_v), Some(root)) = (latest.get(), root_index.get()) else {
            set_male_slice_rows.set(Vec::new());
            set_female_slice_rows.set(Vec::new());
            set_cross_sex_rows_error.set(None);
            return;
        };
        let equip_value = equip.get();
        if equip_value.is_empty() {
            set_male_slice_rows.set(Vec::new());
            set_female_slice_rows.set(Vec::new());
            set_cross_sex_rows_error.set(None);
            return;
        }

        let male_shard_key = format!("sex=M|equip={equip_value}");
        let female_shard_key = format!("sex=F|equip={equip_value}");
        let male_shard_rel = root.shards.get(&male_shard_key).cloned();
        let female_shard_rel = root.shards.get(&female_shard_key).cloned();

        let cross_sex_rows_request_id = cross_sex_rows_request_id;
        let set_male_slice_rows = set_male_slice_rows;
        let set_female_slice_rows = set_female_slice_rows;
        let set_cross_sex_rows_error = set_cross_sex_rows_error;
        set_cross_sex_rows_error.set(None);

        spawn_local(async move {
            let mut male_rows = Vec::new();
            let mut female_rows = Vec::new();
            let mut issues = Vec::new();

            if let Some(male_rel) = male_shard_rel {
                let male_url = dataset_file_url(&latest_v.version, &male_rel);
                match fetch_json_first::<SliceIndex>(&[&male_url]).await {
                    Ok(index) => {
                        if cross_sex_rows_request_id.get_untracked() != next_request_id {
                            debug_log(&format!(
                                "Ignored stale male cross-sex shard for request {next_request_id}"
                            ));
                            return;
                        }
                        male_rows = rows_from_slice_index(index);
                    }
                    Err(err) => issues.push(format!("Failed male shard load: {err}")),
                }
            } else {
                issues.push("Missing male shard for selected equipment.".to_string());
            }

            if let Some(female_rel) = female_shard_rel {
                let female_url = dataset_file_url(&latest_v.version, &female_rel);
                match fetch_json_first::<SliceIndex>(&[&female_url]).await {
                    Ok(index) => {
                        if cross_sex_rows_request_id.get_untracked() != next_request_id {
                            debug_log(&format!(
                                "Ignored stale female cross-sex shard for request {next_request_id}"
                            ));
                            return;
                        }
                        female_rows = rows_from_slice_index(index);
                    }
                    Err(err) => issues.push(format!("Failed female shard load: {err}")),
                }
            } else {
                issues.push("Missing female shard for selected equipment.".to_string());
            }

            if cross_sex_rows_request_id.get_untracked() != next_request_id {
                debug_log(&format!(
                    "Ignored stale cross-sex shard completion for request {next_request_id}"
                ));
                return;
            }
            set_male_slice_rows.set(male_rows);
            set_female_slice_rows.set(female_rows);
            if issues.is_empty() {
                set_cross_sex_rows_error.set(None);
            } else {
                set_cross_sex_rows_error.set(Some(issues.join(" ")));
            }
        });
    });

    let selector_index = slice_selector_index(slice_rows);
    let current_row_index = selector_index;
    let current_row = Memo::new(move |_| {
        let w = wc.get();
        let a = age.get();
        let t = tested.get();
        let l = lift.get();
        let m = metric.get();

        current_row_index.with(|index| index.current_row(&w, &a, &t, &l, &m))
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

    let sex_options = sex_options(root_index);
    let equip_options = equip_options(root_index, sex);
    let tested_options = tested_options(selector_index, wc, age);
    let wc_options = wc_options(selector_index);
    let age_options = age_options(selector_index, wc);
    let lift_options = lift_options(selector_index, wc, age, tested);
    let metric_options = metric_options(selector_index, wc, age, tested, lift);
    setup_default_selection_effects(
        state::DefaultSelectionOptions {
            equip: equip_options,
            wc: wc_options,
            age: age_options,
            tested: tested_options,
            lift: lift_options,
            metric: metric_options,
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

    let projected_squat = Memo::new(move |_| (squat.get() + squat_delta.get()).clamp(0.0, 600.0));
    let projected_bench = Memo::new(move |_| (bench.get() + bench_delta.get()).clamp(0.0, 600.0));
    let projected_deadlift =
        Memo::new(move |_| (deadlift.get() + deadlift_delta.get()).clamp(0.0, 600.0));
    let projected_total = Memo::new(move |_| {
        projected_squat.get() + projected_bench.get() + projected_deadlift.get()
    });

    let user_lift = Memo::new(move |_| {
        let current_sex = sex.get();
        let current_equip = equip.get();
        let current_lift = lift.get();
        let current_metric = metric.get();
        comparable_lift_value(
            ComparableLifter {
                sex: &current_sex,
                equipment: &current_equip,
                bodyweight: bodyweight.get(),
                squat: squat.get(),
                bench: bench.get(),
                deadlift: deadlift.get(),
            },
            &current_lift,
            &current_metric,
        )
    });
    let projected_user_lift = Memo::new(move |_| {
        let current_sex = sex.get();
        let current_equip = equip.get();
        let current_lift = lift.get();
        let current_metric = metric.get();
        comparable_lift_value(
            ComparableLifter {
                sex: &current_sex,
                equipment: &current_equip,
                bodyweight: bodyweight.get(),
                squat: projected_squat.get(),
                bench: projected_bench.get(),
                deadlift: projected_deadlift.get(),
            },
            &current_lift,
            &current_metric,
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
    let distribution_diagnostics =
        Memo::new(move |_| histogram_diagnostics(rebinned_hist.get().as_ref()));
    let rarity_snapshot = Memo::new(move |_| {
        histogram_density_for_value(rebinned_hist.get().as_ref(), user_lift.get())
    });
    let bodyweight_conditioned = Memo::new(move |_| {
        bodyweight_conditioned_percentile(
            rebinned_heat.get().as_ref(),
            user_lift.get(),
            bodyweight.get(),
        )
    });
    let kg_for_next_1pct = Memo::new(move |_| {
        if !calculated.get() || !metric_is_kg_comparable.get() {
            return None;
        }
        let (pct, _, _) = percentile.get()?;
        kg_needed_for_percentile_step(rebinned_hist.get().as_ref(), user_lift.get(), pct, 1.0)
    });
    let kg_for_next_5pct = Memo::new(move |_| {
        if !calculated.get() || !metric_is_kg_comparable.get() {
            return None;
        }
        let (pct, _, _) = percentile.get()?;
        kg_needed_for_percentile_step(rebinned_hist.get().as_ref(), user_lift.get(), pct, 5.0)
    });
    let kg_for_next_10pct = Memo::new(move |_| {
        if !calculated.get() || !metric_is_kg_comparable.get() {
            return None;
        }
        let (pct, _, _) = percentile.get()?;
        kg_needed_for_percentile_step(rebinned_hist.get().as_ref(), user_lift.get(), pct, 10.0)
    });
    let pct_gain_plus_2_5kg = Memo::new(move |_| {
        if !calculated.get() || !metric_is_kg_comparable.get() {
            return None;
        }
        let (pct, _, _) = percentile.get()?;
        percentile_points_for_kg_gain(rebinned_hist.get().as_ref(), user_lift.get(), pct, 2.5)
    });
    let pct_gain_plus_5kg = Memo::new(move |_| {
        if !calculated.get() || !metric_is_kg_comparable.get() {
            return None;
        }
        let (pct, _, _) = percentile.get()?;
        percentile_points_for_kg_gain(rebinned_hist.get().as_ref(), user_lift.get(), pct, 5.0)
    });
    let pct_gain_plus_10kg = Memo::new(move |_| {
        if !calculated.get() || !metric_is_kg_comparable.get() {
            return None;
        }
        let (pct, _, _) = percentile.get()?;
        percentile_points_for_kg_gain(rebinned_hist.get().as_ref(), user_lift.get(), pct, 10.0)
    });
    let result_unavailable_reason = Memo::new(move |_| {
        if !calculated.get() || percentile.get().is_some() {
            return None::<String>;
        }
        if current_row.get().is_none() {
            return Some(
                "No matching comparison cohort was found for these filters. Try broader options like All weight classes, All ages, or a different tested status."
                    .to_string(),
            );
        }
        if hist.get().is_none() {
            return Some(load_error.get().unwrap_or_else(|| {
                "We found a matching cohort, but its distribution data is not available yet. Try again or widen the filters."
                    .to_string()
            }));
        }
        Some(
            "We found a matching cohort, but could not compute a percentile for this setup. Try recalculating or adjusting the filters."
                .to_string(),
        )
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
        Some(summary) => format!("Compared against {} lifters in this cohort.", summary.total),
        None => "Loading cohort size...".to_string(),
    });
    let nerd_cohort_summary = Memo::new(move |_| match slice_summary.get() {
        Some(summary) => format!(
            "Current cohort: {} lifters, {:.1}-{:.1} kg observed range.",
            summary.total, summary.min_kg, summary.max_kg
        ),
        None => "Current cohort summary is loading...".to_string(),
    });
    let cohort_comparison_rows = Memo::new(move |_| {
        let rows = slice_rows.get();
        let s = sex.get();
        let e = equip.get();
        let w = wc.get();
        let a = age.get();
        let t = tested.get();
        let l = lift.get();
        let m = metric.get();

        let base_total = find_comparison_slice(
            &rows,
            ComparisonSliceFilter {
                sex: &s,
                equip: &e,
                wc: &w,
                age: &a,
                tested: &t,
                lift: &l,
                metric: &m,
            },
        )
        .and_then(|row| row.entry.summary.as_ref().map(|summary| summary.total))
        .or_else(|| slice_summary.get().map(|summary| summary.total));
        let context = CohortComparisonContext {
            sex: &s,
            equip: &e,
            lift: &l,
            base_total,
        };

        vec![
            build_cohort_comparison_row(
                &rows,
                &context,
                CohortComparisonVariant {
                    label: "Current slice",
                    wc: w.clone(),
                    age: a.clone(),
                    tested: t.clone(),
                    metric: m.clone(),
                    is_current: true,
                },
            ),
            build_cohort_comparison_row(
                &rows,
                &context,
                CohortComparisonVariant {
                    label: "All Ages",
                    wc: w.clone(),
                    age: "All Ages".to_string(),
                    tested: t.clone(),
                    metric: m.clone(),
                    is_current: false,
                },
            ),
            build_cohort_comparison_row(
                &rows,
                &context,
                CohortComparisonVariant {
                    label: "All weight classes",
                    wc: "All".to_string(),
                    age: a.clone(),
                    tested: t.clone(),
                    metric: m.clone(),
                    is_current: false,
                },
            ),
            build_cohort_comparison_row(
                &rows,
                &context,
                CohortComparisonVariant {
                    label: "All tested statuses",
                    wc: w.clone(),
                    age: a.clone(),
                    tested: "All".to_string(),
                    metric: m.clone(),
                    is_current: false,
                },
            ),
            build_cohort_comparison_row(
                &rows,
                &context,
                CohortComparisonVariant {
                    label: "Metric: Kg",
                    wc: w.clone(),
                    age: a.clone(),
                    tested: t.clone(),
                    metric: "Kg".to_string(),
                    is_current: false,
                },
            ),
            build_cohort_comparison_row(
                &rows,
                &context,
                CohortComparisonVariant {
                    label: "Metric: Dots",
                    wc: w.clone(),
                    age: a.clone(),
                    tested: t.clone(),
                    metric: "Dots".to_string(),
                    is_current: false,
                },
            ),
            build_cohort_comparison_row(
                &rows,
                &context,
                CohortComparisonVariant {
                    label: "Metric: Wilks",
                    wc: w.clone(),
                    age: a.clone(),
                    tested: t.clone(),
                    metric: "Wilks".to_string(),
                    is_current: false,
                },
            ),
            build_cohort_comparison_row(
                &rows,
                &context,
                CohortComparisonVariant {
                    label: "Metric: Goodlift",
                    wc: w,
                    age: a,
                    tested: t,
                    metric: "GL".to_string(),
                    is_current: false,
                },
            ),
        ]
    });
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
    let cross_sex_comparison = Memo::new(move |_| -> Result<CrossSexComparison, String> {
        if !calculated.get() {
            return Err("Calculate first to compare men and women side-by-side.".to_string());
        }
        if let Some(err) = cross_sex_rows_error.get() {
            return Err(err);
        }
        let male_choice = male_cross_choice.get().ok_or_else(|| {
            "No matching men's cohort for this equipment/tested/age/lift/metric combination."
                .to_string()
        })?;
        let female_choice = female_cross_choice.get().ok_or_else(|| {
            "No matching women's cohort for this equipment/tested/age/lift/metric combination."
                .to_string()
        })?;

        let male_summary = male_choice
            .row
            .entry
            .summary
            .as_ref()
            .ok_or_else(|| "Men's cohort summary is missing in the shard index.".to_string())?;
        let female_summary =
            female_choice.row.entry.summary.as_ref().ok_or_else(|| {
                "Women's cohort summary is missing in the shard index.".to_string()
            })?;
        if male_summary.total < MIN_CROSS_SEX_COHORT_TOTAL
            || female_summary.total < MIN_CROSS_SEX_COHORT_TOTAL
        {
            return Err(format!(
                "Cross-sex comparison skipped because one cohort is too small (<{} lifters).",
                MIN_CROSS_SEX_COHORT_TOTAL
            ));
        }

        if let Some(err) = cross_sex_hist_error.get() {
            return Err(err);
        }
        let male_hist = male_cross_hist
            .get()
            .ok_or_else(|| "Men's distribution is not loaded yet.".to_string())?;
        let female_hist = female_cross_hist
            .get()
            .ok_or_else(|| "Women's distribution is not loaded yet.".to_string())?;

        let equip_value = equip.get();
        let lift_value = lift.get();
        let metric_value = metric.get();
        let male_input_value = comparable_lift_value(
            ComparableLifter {
                sex: "M",
                equipment: &equip_value,
                bodyweight: bodyweight.get(),
                squat: squat.get(),
                bench: bench.get(),
                deadlift: deadlift.get(),
            },
            &lift_value,
            &metric_value,
        );
        let female_input_value = comparable_lift_value(
            ComparableLifter {
                sex: "F",
                equipment: &equip_value,
                bodyweight: bodyweight.get(),
                squat: squat.get(),
                bench: bench.get(),
                deadlift: deadlift.get(),
            },
            &lift_value,
            &metric_value,
        );

        let (male_percentile, female_value_at_male_percentile) =
            equivalent_value_for_same_percentile(
                Some(&male_hist),
                Some(&female_hist),
                male_input_value,
            )
            .ok_or_else(|| {
                "Could not compute women's equivalent value at men's percentile.".to_string()
            })?;
        let (female_percentile, male_value_at_female_percentile) =
            equivalent_value_for_same_percentile(
                Some(&female_hist),
                Some(&male_hist),
                female_input_value,
            )
            .ok_or_else(|| {
                "Could not compute men's equivalent value at women's percentile.".to_string()
            })?;

        let caveat = if metric_value == "Kg" {
            Some("Raw kg cross-sex comparisons are descriptive, not apples-to-apples. Prefer Dots, Wilks, or Goodlift when available.".to_string())
        } else {
            None
        };

        Ok(CrossSexComparison {
            male_percentile,
            female_percentile,
            male_total: male_summary.total,
            female_total: female_summary.total,
            male_input_value,
            female_input_value,
            female_value_at_male_percentile,
            male_value_at_female_percentile,
            metric: metric_value,
            male_weight_class: male_choice.row.key.wc,
            female_weight_class: female_choice.row.key.wc,
            male_wc_fallback: male_choice.weight_class_fallback,
            female_wc_fallback: female_choice.weight_class_fallback,
            caveat,
        })
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
    let exact_slice_key = Memo::new(move |_| {
        if let Some(row) = current_row.get() {
            format!(
                "sex={}|equip={}|wc={}|age={}|tested={}|lift={}|metric={}",
                row.key.sex,
                row.key.equip,
                row.key.wc,
                row.key.age,
                row.key.tested,
                row.key.lift,
                row.key.metric
            )
        } else {
            format!(
                "sex={}|equip={}|wc={}|age={}|tested={}|lift={}|metric={}",
                sex.get(),
                equip.get(),
                wc.get(),
                age.get(),
                tested.get(),
                lift.get(),
                metric.get()
            )
        }
    });
    let shard_key = Memo::new(move |_| format!("sex={}|equip={}", sex.get(), equip.get()));
    let dataset_version = Memo::new(move |_| {
        latest
            .get()
            .map(|data| data.version)
            .unwrap_or_else(|| "loading".to_string())
    });
    let dataset_revision = Memo::new(move |_| {
        latest
            .get()
            .and_then(|data| data.revision)
            .unwrap_or_else(|| "n/a".to_string())
    });
    let histogram_bin_width = Memo::new(move |_| rebinned_hist.get().map(|h| h.base_bin));
    let heatmap_dims = Memo::new(move |_| {
        rebinned_heat
            .get()
            .map(|h| (h.width, h.height, h.base_x, h.base_y))
    });
    let summary_stats = Memo::new(move |_| {
        slice_summary
            .get()
            .map(|summary| (summary.total, summary.min_kg, summary.max_kg))
    });
    let compare_summary = Memo::new(move |_| match compare_mode.get() {
        CompareMode::AllLifters => match percentile.get() {
            Some((pct, _, _)) => format!(
                "Across all lifters, you're stronger than {:.1}%.",
                pct * 100.0
            ),
            None => "Comparison summary appears after a matching slice loads.".to_string(),
        },
        CompareMode::SameBodyweightRange => match bodyweight_conditioned.get() {
            Some(stats) => {
                let low = kg_to_display(stats.bw_window_low, use_lbs.get());
                let high = kg_to_display(stats.bw_window_high, use_lbs.get());
                format!(
                    "Among nearby bodyweights ({:.1}-{:.1}{}), you're stronger than {:.1}% ({} nearby lifters; local neighborhood {:.2}% of heatmap mass).",
                    low,
                    high,
                    unit_label.get(),
                    stats.percentile * 100.0,
                    stats.total_nearby,
                    stats.neighborhood_share * 100.0
                )
            }
            None => "Bodyweight-conditioned summary appears after calculation and heatmap load."
                .to_string(),
        },
        CompareMode::SameWeightClass => match percentile.get() {
            Some((pct, _, _)) => format!(
                "In weight class {}, you're stronger than {:.1}%.",
                wc.get(),
                pct * 100.0
            ),
            None => "Comparison summary appears after a matching slice loads.".to_string(),
        },
        CompareMode::SameAgeClass => match percentile.get() {
            Some((pct, _, _)) => format!(
                "In age class {}, you're stronger than {:.1}%.",
                age_label(&age.get()),
                pct * 100.0
            ),
            None => "Comparison summary appears after a matching slice loads.".to_string(),
        },
        CompareMode::SameTestedStatus => match percentile.get() {
            Some((pct, _, _)) => format!(
                "Within {} meets, you're stronger than {:.1}%.",
                tested.get(),
                pct * 100.0
            ),
            None => "Comparison summary appears after a matching slice loads.".to_string(),
        },
    });
    let cross_sex_selection_summary = Memo::new(move |_| {
        format!(
            "Current alignment: {} equipment, {}, {} tested status, {}, {}, requested {} weight class.",
            equip.get(),
            age_label(&age.get()),
            tested.get(),
            lift_label(&lift.get()),
            metric_label(&metric.get()),
            wc.get()
        )
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

    Effect::new(move |_| {
        let next_request_id = cohort_exact_request_id.get_untracked().wrapping_add(1);
        set_cohort_exact_request_id.set(next_request_id);

        let should_load =
            cohort_exact_deltas_enabled.get() && calculated.get() && nerds_page_active.get();
        if !should_load {
            set_cohort_exact_percentiles.set(BTreeMap::new());
            set_cohort_exact_loading.set(false);
            set_cohort_exact_error.set(None);
            return;
        }

        let Some(latest_v) = latest.get() else {
            set_cohort_exact_percentiles.set(BTreeMap::new());
            set_cohort_exact_loading.set(false);
            set_cohort_exact_error.set(Some(
                "Dataset metadata is still loading; exact deltas will appear shortly.".to_string(),
            ));
            return;
        };

        let rows = cohort_comparison_rows.get();
        let current_hist = hist.get();
        let current_sex = sex.get();
        let current_equip = equip.get();
        let current_lift = lift.get();
        let current_bodyweight = bodyweight.get();
        let current_squat = squat.get();
        let current_bench = bench.get();
        let current_deadlift = deadlift.get();
        let current_lifter = ComparableLifter {
            sex: &current_sex,
            equipment: &current_equip,
            bodyweight: current_bodyweight,
            squat: current_squat,
            bench: current_bench,
            deadlift: current_deadlift,
        };

        let mut prefetched = BTreeMap::<String, Option<f32>>::new();
        let mut hist_groups: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        for row in rows {
            if row.is_current
                && let Some(histogram) = current_hist.as_ref()
            {
                let lift_value = comparable_lift_value(current_lifter, &current_lift, &row.metric);
                let percentile = percentile_for_value(Some(histogram), lift_value)
                    .map(|(pct, _, _)| pct * 100.0);
                prefetched.insert(row.id, percentile);
                continue;
            }

            if let Some(bin_path) = row.bin_path {
                hist_groups
                    .entry(bin_path)
                    .or_default()
                    .push((row.id, row.metric));
            } else {
                prefetched.insert(row.id, None);
            }
        }

        if hist_groups.is_empty() {
            set_cohort_exact_percentiles.set(prefetched);
            set_cohort_exact_loading.set(false);
            set_cohort_exact_error.set(None);
            return;
        }

        let cohort_exact_request_id = cohort_exact_request_id;
        let set_cohort_exact_percentiles = set_cohort_exact_percentiles;
        let set_cohort_exact_loading = set_cohort_exact_loading;
        let set_cohort_exact_error = set_cohort_exact_error;
        set_cohort_exact_loading.set(true);
        set_cohort_exact_percentiles.set(BTreeMap::new());
        set_cohort_exact_error.set(None);

        spawn_local(async move {
            let mut resolved = prefetched;
            let mut errors = Vec::new();

            for (bin_path, targets) in hist_groups {
                if cohort_exact_request_id.get_untracked() != next_request_id {
                    debug_log(&format!(
                        "Ignored stale cohort comparison response for request {next_request_id}"
                    ));
                    return;
                }

                let bin_url = dataset_file_url(&latest_v.version, &bin_path);
                match fetch_binary_first(&[&bin_url]).await {
                    Ok(bytes) => match parse_combined_bin(&bytes).map(|(h, _)| h) {
                        Some(histogram) => {
                            let current_lifter = ComparableLifter {
                                sex: &current_sex,
                                equipment: &current_equip,
                                bodyweight: current_bodyweight,
                                squat: current_squat,
                                bench: current_bench,
                                deadlift: current_deadlift,
                            };
                            for (row_id, row_metric) in targets {
                                let lift_value = comparable_lift_value(
                                    current_lifter,
                                    &current_lift,
                                    &row_metric,
                                );
                                let percentile = percentile_for_value(Some(&histogram), lift_value)
                                    .map(|(pct, _, _)| pct * 100.0);
                                resolved.insert(row_id, percentile);
                            }
                        }
                        None => {
                            errors.push(format!("{bin_path}: invalid binary payload"));
                            for (row_id, _) in targets {
                                resolved.insert(row_id, None);
                            }
                        }
                    },
                    Err(err) => {
                        errors.push(format!("{bin_path}: {err}"));
                        for (row_id, _) in targets {
                            resolved.insert(row_id, None);
                        }
                    }
                }
            }

            if cohort_exact_request_id.get_untracked() != next_request_id {
                debug_log(&format!(
                    "Ignored stale cohort comparison completion for request {next_request_id}"
                ));
                return;
            }

            set_cohort_exact_percentiles.set(resolved);
            set_cohort_exact_loading.set(false);
            if errors.is_empty() {
                set_cohort_exact_error.set(None);
            } else {
                set_cohort_exact_error.set(Some(
                    "Some exact percentile rows could not be loaded for this view.".to_string(),
                ));
                debug_log(&format!(
                    "Cohort exact percentile row errors: {}",
                    errors.join(" | ")
                ));
            }
        });
    });

    Effect::new(move |_| {
        let next_request_id = cross_sex_hist_request_id.get_untracked().wrapping_add(1);
        set_cross_sex_hist_request_id.set(next_request_id);

        let should_load = cross_sex_page_active.get() && calculated.get();
        if !should_load {
            set_male_cross_hist.set(None);
            set_female_cross_hist.set(None);
            set_cross_sex_hist_loading.set(false);
            set_cross_sex_hist_error.set(None);
            return;
        }

        let Some(latest_v) = latest.get() else {
            set_male_cross_hist.set(None);
            set_female_cross_hist.set(None);
            set_cross_sex_hist_loading.set(false);
            set_cross_sex_hist_error.set(Some(
                "Dataset metadata is still loading for cross-sex comparison.".to_string(),
            ));
            return;
        };
        let Some(male_choice) = male_cross_choice.get() else {
            set_male_cross_hist.set(None);
            set_female_cross_hist.set(None);
            set_cross_sex_hist_loading.set(false);
            set_cross_sex_hist_error.set(None);
            return;
        };
        let Some(female_choice) = female_cross_choice.get() else {
            set_male_cross_hist.set(None);
            set_female_cross_hist.set(None);
            set_cross_sex_hist_loading.set(false);
            set_cross_sex_hist_error.set(None);
            return;
        };

        let selected_row_bin = current_row.get().map(|row| row.entry.bin);
        let selected_hist = hist.get();

        let mut prefetched_male = None;
        let mut prefetched_female = None;
        if let (Some(selected_row_bin), Some(selected_hist)) = (selected_row_bin, selected_hist) {
            if selected_row_bin == male_choice.row.entry.bin {
                prefetched_male = Some(selected_hist.clone());
            }
            if selected_row_bin == female_choice.row.entry.bin {
                prefetched_female = Some(selected_hist);
            }
        }

        if prefetched_male.is_some() && prefetched_female.is_some() {
            set_male_cross_hist.set(prefetched_male);
            set_female_cross_hist.set(prefetched_female);
            set_cross_sex_hist_loading.set(false);
            set_cross_sex_hist_error.set(None);
            return;
        }

        let cross_sex_hist_request_id = cross_sex_hist_request_id;
        let set_male_cross_hist = set_male_cross_hist;
        let set_female_cross_hist = set_female_cross_hist;
        let set_cross_sex_hist_loading = set_cross_sex_hist_loading;
        let set_cross_sex_hist_error = set_cross_sex_hist_error;
        set_cross_sex_hist_loading.set(true);
        set_cross_sex_hist_error.set(None);

        spawn_local(async move {
            let mut resolved_male = prefetched_male;
            let mut resolved_female = prefetched_female;
            let mut issues = Vec::new();

            if resolved_male.is_none() {
                let male_url = dataset_file_url(&latest_v.version, &male_choice.row.entry.bin);
                match fetch_binary_first(&[&male_url]).await {
                    Ok(bytes) => match parse_combined_bin(&bytes).map(|(h, _)| h) {
                        Some(histogram) => resolved_male = Some(histogram),
                        None => issues.push("Invalid men's binary payload.".to_string()),
                    },
                    Err(err) => issues.push(format!("Failed men's bin fetch: {err}")),
                }
            }

            if cross_sex_hist_request_id.get_untracked() != next_request_id {
                debug_log(&format!(
                    "Ignored stale cross-sex male histogram for request {next_request_id}"
                ));
                return;
            }

            if resolved_female.is_none() {
                let female_url = dataset_file_url(&latest_v.version, &female_choice.row.entry.bin);
                match fetch_binary_first(&[&female_url]).await {
                    Ok(bytes) => match parse_combined_bin(&bytes).map(|(h, _)| h) {
                        Some(histogram) => resolved_female = Some(histogram),
                        None => issues.push("Invalid women's binary payload.".to_string()),
                    },
                    Err(err) => issues.push(format!("Failed women's bin fetch: {err}")),
                }
            }

            if cross_sex_hist_request_id.get_untracked() != next_request_id {
                debug_log(&format!(
                    "Ignored stale cross-sex female histogram for request {next_request_id}"
                ));
                return;
            }

            set_male_cross_hist.set(resolved_male);
            set_female_cross_hist.set(resolved_female);
            set_cross_sex_hist_loading.set(false);
            if issues.is_empty() {
                set_cross_sex_hist_error.set(None);
            } else {
                set_cross_sex_hist_error.set(Some(issues.join(" ")));
            }
        });
    });

    Effect::new(move |_| {
        let next_request_id = cross_sex_heat_request_id.get_untracked().wrapping_add(1);
        set_cross_sex_heat_request_id.set(next_request_id);

        let should_load = cross_sex_page_active.get() && calculated.get();
        if !should_load {
            set_male_cross_heat.set(None);
            set_female_cross_heat.set(None);
            set_cross_sex_heat_loading.set(false);
            set_cross_sex_heat_error.set(None);
            return;
        }

        let Some(latest_v) = latest.get() else {
            set_male_cross_heat.set(None);
            set_female_cross_heat.set(None);
            set_cross_sex_heat_loading.set(false);
            set_cross_sex_heat_error.set(Some(
                "Dataset metadata is still loading for cross-sex heatmaps.".to_string(),
            ));
            return;
        };
        let Some(male_choice) = male_cross_choice.get() else {
            set_male_cross_heat.set(None);
            set_female_cross_heat.set(None);
            set_cross_sex_heat_loading.set(false);
            set_cross_sex_heat_error.set(None);
            return;
        };
        let Some(female_choice) = female_cross_choice.get() else {
            set_male_cross_heat.set(None);
            set_female_cross_heat.set(None);
            set_cross_sex_heat_loading.set(false);
            set_cross_sex_heat_error.set(None);
            return;
        };

        let cross_sex_heat_request_id = cross_sex_heat_request_id;
        let set_male_cross_heat = set_male_cross_heat;
        let set_female_cross_heat = set_female_cross_heat;
        let set_cross_sex_heat_loading = set_cross_sex_heat_loading;
        let set_cross_sex_heat_error = set_cross_sex_heat_error;
        set_cross_sex_heat_loading.set(true);
        set_cross_sex_heat_error.set(None);

        spawn_local(async move {
            let mut resolved_male = None;
            let mut resolved_female = None;
            let mut issues = Vec::new();

            let male_url = dataset_file_url(&latest_v.version, &male_choice.row.entry.bin);
            match fetch_binary_first(&[&male_url]).await {
                Ok(bytes) => match parse_combined_bin(&bytes).map(|(_, h)| h) {
                    Some(heatmap) => resolved_male = Some(heatmap),
                    None => issues.push("Invalid men's binary payload.".to_string()),
                },
                Err(err) => issues.push(format!("Failed men's bin fetch: {err}")),
            }

            if cross_sex_heat_request_id.get_untracked() != next_request_id {
                debug_log(&format!(
                    "Ignored stale cross-sex male heatmap for request {next_request_id}"
                ));
                return;
            }

            let female_url = dataset_file_url(&latest_v.version, &female_choice.row.entry.bin);
            match fetch_binary_first(&[&female_url]).await {
                Ok(bytes) => match parse_combined_bin(&bytes).map(|(_, h)| h) {
                    Some(heatmap) => resolved_female = Some(heatmap),
                    None => issues.push("Invalid women's binary payload.".to_string()),
                },
                Err(err) => issues.push(format!("Failed women's bin fetch: {err}")),
            }

            if cross_sex_heat_request_id.get_untracked() != next_request_id {
                debug_log(&format!(
                    "Ignored stale cross-sex female heatmap for request {next_request_id}"
                ));
                return;
            }

            set_male_cross_heat.set(resolved_male);
            set_female_cross_heat.set(resolved_female);
            set_cross_sex_heat_loading.set(false);
            if issues.is_empty() {
                set_cross_sex_heat_error.set(None);
            } else {
                set_cross_sex_heat_error.set(Some(issues.join(" ")));
            }
        });
    });

    Effect::new(move |_| {
        let _ = heatmap_resize_tick.get();
        let Some(canvas) = canvas_ref.get() else {
            return;
        };
        let Some(heat) = rebinned_heat.get() else {
            return;
        };
        draw_heatmap(
            &canvas,
            &heat,
            show_nerd_heat_indicator.get().then(|| user_lift.get()),
            bodyweight.get(),
            &hist_x_label.get(),
        );
    });

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
            } else if hash.eq_ignore_ascii_case("#men-vs-women") {
                set_active_page.set(AppPage::MenVsWomen);
            } else if hash.eq_ignore_ascii_case("#nerds") {
                set_active_page.set(AppPage::StatsForNerds);
            } else if hash.eq_ignore_ascii_case("#plate-calc") {
                set_active_page.set(AppPage::PlateCalc);
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
            AppPage::StatsForNerds => "#nerds",
            AppPage::MenVsWomen => "#men-vs-women",
            AppPage::OneRm => "#1rm",
            AppPage::PlateCalc => "#plate-calc",
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

    let onboarding_sections = components::OnboardingSections {
        identity: components::OnboardingIdentitySection {
            sex_options,
            sex,
            set_sex,
            equip_options,
            equip,
            set_equip,
            unit_label,
            use_lbs,
            set_use_lbs,
        },
        lifts: components::OnboardingLiftSection {
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
        },
        filters: components::OnboardingFilterSection {
            tested_options,
            tested,
            set_tested,
            age_options,
            age,
            set_age,
            wc_options,
            wc,
            set_wc,
            lift_options,
            lift,
            set_lift,
            metric_options,
            metric,
            set_metric,
        },
        actions: components::OnboardingActionSection {
            set_squat_delta,
            set_bench_delta,
            set_deadlift_delta,
            set_share_handle,
            set_calculated,
            set_reveal_tick,
            has_input_error,
            calculating,
            set_calculating,
            set_share_status,
            set_lift_mult,
            set_bw_mult,
        },
    };

    let ranking_page = components::RankingPageSections {
        header: components::RankingHeroSection {
            dataset_blurb,
            ranking_cohort_blurb,
        },
        onboarding: onboarding_sections.clone(),
        result: components::ResultCardSections {
            status: components::ResultCardStatusSection {
                calculated,
                percentile,
                rank_tier,
                reveal_tick,
                load_error,
                unavailable_reason: result_unavailable_reason,
            },
            share: components::ResultCardShareSection {
                show_share,
                set_show_share,
                share_url,
                share_status,
                set_share_status,
                share_handle,
                set_share_handle,
            },
            lifts: components::ResultCardLiftSection {
                bodyweight,
                squat,
                bench,
                deadlift,
                lift,
                use_lbs,
                unit_label,
            },
        },
        meet_day: components::MeetDaySection {
            squat,
            bench,
            deadlift,
            use_lbs,
            unit_label,
        },
    };

    let progress_sections = components::ProgressSections {
        result: components::ProgressResultSection {
            calculated,
            percentile,
        },
        selection: components::ProgressSelectionSection {
            sex,
            equip,
            wc,
            age,
            tested,
            lift,
            metric,
        },
        lifts: components::ProgressLiftSection {
            squat,
            bench,
            deadlift,
            bodyweight,
        },
        display: components::ProgressDisplaySection {
            use_lbs,
            unit_label,
        },
    };
    let distribution_controls = components::DistributionControlsSection {
        lift_mult,
        set_lift_mult,
        bw_mult,
        set_bw_mult,
    };

    let nerds_page = components::NerdsPageSections {
        header: components::NerdsHeaderSection {
            dataset_blurb,
            nerd_cohort_summary,
        },
        onboarding: onboarding_sections.clone(),
        cohort: components::NerdsCohortSection {
            compare_mode,
            set_compare_mode,
            tested,
            set_tested,
            equip,
            set_equip,
            age,
            set_age,
            compare_summary,
            cohort_comparison_rows,
            cohort_exact_deltas_enabled,
            set_cohort_exact_deltas_enabled,
            cohort_exact_percentiles,
            cohort_exact_loading,
            cohort_exact_error,
            progress: progress_sections.clone(),
        },
        distributions: components::NerdsDistributionSection {
            controls: distribution_controls.clone(),
            diagnostics: components::DistributionDiagnosticsSection {
                distribution_diagnostics,
                hist_x_label,
            },
            ladder: components::PercentileLadderData {
                calculated,
                metric_is_kg_comparable,
                estimates: components::PercentileLadderEstimates {
                    kg_for_next_1pct,
                    kg_for_next_5pct,
                    kg_for_next_10pct,
                    pct_gain_plus_2_5kg,
                    pct_gain_plus_5kg,
                    pct_gain_plus_10kg,
                },
            },
            rarity_snapshot,
            bodyweight_conditioned,
            rebinned_hist,
            user_lift,
            canvas_ref,
            show_heat_indicator: show_nerd_heat_indicator,
            set_show_heat_indicator: set_show_nerd_heat_indicator,
            calculated,
            use_lbs,
            unit_label,
        },
        targets: components::NerdsTargetsSection {
            squat_delta,
            set_squat_delta,
            bench_delta,
            set_bench_delta,
            deadlift_delta,
            set_deadlift_delta,
            projected_total,
            projected_squat,
            projected_bench,
            projected_deadlift,
            use_lbs,
            unit_label,
            projected_percentile,
            projected_rank_tier,
            percentile_delta,
            target_percentile,
            set_target_percentile,
            target_kg_needed,
            target_summary,
        },
        trends: components::NerdsTrendsSection {
            calculated,
            selected_trend_points,
            trend_note,
            user_lift,
            hist_x_label,
        },
        methodology: components::NerdsMethodologySection {
            exact_slice_key,
            shard_key,
            dataset_version,
            dataset_revision,
            histogram_bin_width,
            heatmap_dims,
            summary_stats,
            load_timing_blurb,
        },
    };

    let men_vs_women_page = components::MenVsWomenPageSections {
        dataset_blurb,
        selection_summary: cross_sex_selection_summary,
        onboarding: onboarding_sections,
        controls: distribution_controls,
        comparison_loading: cross_sex_hist_loading,
        comparison: cross_sex_comparison,
        heat_loading: cross_sex_heat_loading,
        heat_error: cross_sex_heat_error,
        male_hist: male_cross_hist,
        female_hist: female_cross_hist,
        male_heat: male_cross_heat,
        female_heat: female_cross_heat,
        hist_x_label,
        user_lift,
        bodyweight,
        use_lbs,
        unit_label,
        progress: progress_sections,
    };

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
                        aria-pressed=move || (active_page.get() == AppPage::Rank).to_string()
                        on:click=move |_| set_active_page.set(AppPage::Rank)
                    >
                        "Ranking"
                    </button>
                    <button
                        type="button"
                        class:chip=true
                        class:active=move || active_page.get() == AppPage::StatsForNerds
                        aria-pressed=move || {
                            (active_page.get() == AppPage::StatsForNerds).to_string()
                        }
                        on:click=move |_| set_active_page.set(AppPage::StatsForNerds)
                    >
                        "Stats for Nerds"
                    </button>
                    <button
                        type="button"
                        class:chip=true
                        class:active=move || active_page.get() == AppPage::MenVsWomen
                        aria-pressed=move || (active_page.get() == AppPage::MenVsWomen).to_string()
                        on:click=move |_| set_active_page.set(AppPage::MenVsWomen)
                    >
                        "Men vs Women"
                    </button>
                    <button
                        type="button"
                        class:chip=true
                        class:active=move || active_page.get() == AppPage::OneRm
                        aria-pressed=move || (active_page.get() == AppPage::OneRm).to_string()
                        on:click=move |_| set_active_page.set(AppPage::OneRm)
                    >
                        "1RM Calculator"
                    </button>
                    <button
                        type="button"
                        class:chip=true
                        class:active=move || active_page.get() == AppPage::PlateCalc
                        aria-pressed=move || (active_page.get() == AppPage::PlateCalc).to_string()
                        on:click=move |_| set_active_page.set(AppPage::PlateCalc)
                    >
                        "Plate Calculator"
                    </button>
                    <a class="chip" href="./landing/index.html">
                        "Guides"
                    </a>
                </nav>
            </header>

            <Show when=move || active_page.get() == AppPage::Rank>
                <RankingPage page=ranking_page.clone() />
            </Show>

            <Show when=move || active_page.get() == AppPage::StatsForNerds>
                <NerdsPage page=nerds_page.clone() />
            </Show>

            <Show when=move || active_page.get() == AppPage::MenVsWomen>
                <MenVsWomenPage page=men_vs_women_page.clone() />
            </Show>

            <Show when=move || active_page.get() == AppPage::OneRm>
                <OneRmPage />
            </Show>

            <Show when=move || active_page.get() == AppPage::PlateCalc>
                <PlateCalcPage />
            </Show>

            <footer class="panel attribution">
                <p>
                    "Data powered by "
                    <a href="https://www.openpowerlifting.org/" target="_blank" rel="noopener noreferrer">
                        "OpenPowerlifting.org"
                    </a>
                    ". Huge appreciation to their team for keeping this public dataset free."
                </p>
                <p>
                    "If Iron Insights helps your training, "
                    <a href="./landing/help-make-it-better.html">
                        "help make it better"
                    </a>
                    "."
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
