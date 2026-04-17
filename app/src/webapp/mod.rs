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
use self::data::{fetch_binary_first, fetch_json_first};
use self::helpers::{
    ComparableLifter, build_share_url, comparable_lift_value, display_to_kg, format_input_bound,
    kg_to_display, parse_query_f32, tier_for_percentile,
};
use self::models::{
    CrossSexComparison, LatestJson, RootIndex, SavedUiState, SliceIndex, SliceIndexEntries,
    SliceRow, SliceSummary, TrendSeries,
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
use self::ui::{age_label, lift_label, metric_label};
use crate::core::{
    HeatmapBin, HistogramBin, bodyweight_conditioned_percentile,
    equivalent_value_for_same_percentile, histogram_density_for_value, parse_combined_bin,
    percentile_for_value, rebin_1d, rebin_2d, value_for_percentile,
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
enum AppPage {
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

fn dataset_file_url(version: &str, path: &str) -> String {
    let trimmed = path.trim_start_matches('/');
    format!("data/{version}/{trimmed}")
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
    let (hist_load_ms, set_hist_load_ms) = signal(None::<u32>);
    let (heat_load_ms, set_heat_load_ms) = signal(None::<u32>);
    let (summary_load_ms, set_summary_load_ms) = signal(None::<u32>);

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

    // Cross-sex shard loading
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

        let male_shard_rel = root
            .shards
            .get(&format!("sex=M|equip={equip_value}"))
            .cloned();
        let female_shard_rel = root
            .shards
            .get(&format!("sex=F|equip={equip_value}"))
            .cloned();
        set_cross_sex_rows_error.set(None);

        spawn_local(async move {
            let mut male_rows = Vec::new();
            let mut female_rows = Vec::new();
            let mut issues = Vec::new();

            if let Some(rel) = male_shard_rel {
                let url = dataset_file_url(&latest_v.version, &rel);
                match fetch_json_first::<SliceIndex>(&[&url]).await {
                    Ok(index) => {
                        if cross_sex_rows_request_id.get_untracked() != next_request_id {
                            return;
                        }
                        male_rows = rows_from_slice_index(index);
                    }
                    Err(err) => issues.push(format!("Failed male shard: {err}")),
                }
            } else {
                issues.push("Missing male shard for selected equipment.".to_string());
            }

            if let Some(rel) = female_shard_rel {
                let url = dataset_file_url(&latest_v.version, &rel);
                match fetch_json_first::<SliceIndex>(&[&url]).await {
                    Ok(index) => {
                        if cross_sex_rows_request_id.get_untracked() != next_request_id {
                            return;
                        }
                        female_rows = rows_from_slice_index(index);
                    }
                    Err(err) => issues.push(format!("Failed female shard: {err}")),
                }
            } else {
                issues.push("Missing female shard for selected equipment.".to_string());
            }

            if cross_sex_rows_request_id.get_untracked() != next_request_id {
                return;
            }
            set_male_slice_rows.set(male_rows);
            set_female_slice_rows.set(female_rows);
            if !issues.is_empty() {
                set_cross_sex_rows_error.set(Some(issues.join(" ")));
            }
        });
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

    Effect::new(move |_| {
        let next_id = cross_sex_hist_request_id.get_untracked().wrapping_add(1);
        set_cross_sex_hist_request_id.set(next_id);
        if !cross_sex_page_active.get() || !calculated.get() {
            set_male_cross_hist.set(None);
            set_female_cross_hist.set(None);
            set_cross_sex_hist_loading.set(false);
            set_cross_sex_hist_error.set(None);
            return;
        }
        let (Some(latest_v), Some(male_c), Some(female_c)) = (
            latest.get(),
            male_cross_choice.get(),
            female_cross_choice.get(),
        ) else {
            set_male_cross_hist.set(None);
            set_female_cross_hist.set(None);
            set_cross_sex_hist_loading.set(false);
            set_cross_sex_hist_error.set(None);
            return;
        };
        let selected_bin = current_row.get().map(|r| r.entry.bin);
        let current_hist = hist.get();
        let mut pre_male = None;
        let mut pre_female = None;
        if let (Some(bin), Some(h)) = (selected_bin, current_hist) {
            if bin == male_c.row.entry.bin {
                pre_male = Some(h.clone());
            }
            if bin == female_c.row.entry.bin {
                pre_female = Some(h);
            }
        }
        if let Some(h) = pre_male.clone() {
            set_male_cross_hist.set(Some(h));
        } else {
            set_male_cross_hist.set(None);
        }
        if let Some(h) = pre_female.clone() {
            set_female_cross_hist.set(Some(h));
        } else {
            set_female_cross_hist.set(None);
        }
        if pre_male.is_some() && pre_female.is_some() {
            set_cross_sex_hist_loading.set(false);
            set_cross_sex_hist_error.set(None);
            return;
        }
        set_cross_sex_hist_loading.set(true);
        set_cross_sex_hist_error.set(None);
        spawn_local(async move {
            let mut issues = Vec::new();
            if pre_male.is_none() {
                let url = dataset_file_url(&latest_v.version, &male_c.row.entry.bin);
                match fetch_binary_first(&[&url]).await {
                    Ok(bytes) => match parse_combined_bin(&bytes).map(|(h, _)| h) {
                        Some(h) => {
                            if cross_sex_hist_request_id.get_untracked() != next_id {
                                return;
                            }
                            set_male_cross_hist.set(Some(h));
                        }
                        None => issues.push("Invalid men's payload.".to_string()),
                    },
                    Err(e) => issues.push(format!("Men's bin error: {e}")),
                }
            }
            if pre_female.is_none() {
                let url = dataset_file_url(&latest_v.version, &female_c.row.entry.bin);
                match fetch_binary_first(&[&url]).await {
                    Ok(bytes) => match parse_combined_bin(&bytes).map(|(h, _)| h) {
                        Some(h) => {
                            if cross_sex_hist_request_id.get_untracked() != next_id {
                                return;
                            }
                            set_female_cross_hist.set(Some(h));
                        }
                        None => issues.push("Invalid women's payload.".to_string()),
                    },
                    Err(e) => issues.push(format!("Women's bin error: {e}")),
                }
            }
            if cross_sex_hist_request_id.get_untracked() != next_id {
                return;
            }
            set_cross_sex_hist_loading.set(false);
            if !issues.is_empty() {
                set_cross_sex_hist_error.set(Some(issues.join(" ")));
            }
        });
    });

    Effect::new(move |_| {
        let next_id = cross_sex_heat_request_id.get_untracked().wrapping_add(1);
        set_cross_sex_heat_request_id.set(next_id);
        if !cross_sex_page_active.get() || !calculated.get() {
            set_male_cross_heat.set(None);
            set_female_cross_heat.set(None);
            set_cross_sex_heat_loading.set(false);
            set_cross_sex_heat_error.set(None);
            return;
        }
        let (Some(latest_v), Some(male_c), Some(female_c)) = (
            latest.get(),
            male_cross_choice.get(),
            female_cross_choice.get(),
        ) else {
            set_male_cross_heat.set(None);
            set_female_cross_heat.set(None);
            set_cross_sex_heat_loading.set(false);
            return;
        };
        set_cross_sex_heat_loading.set(true);
        set_cross_sex_heat_error.set(None);
        spawn_local(async move {
            let mut issues = Vec::new();
            let male_url = dataset_file_url(&latest_v.version, &male_c.row.entry.bin);
            match fetch_binary_first(&[&male_url]).await {
                Ok(bytes) => match parse_combined_bin(&bytes).map(|(_, h)| h) {
                    Some(h) => {
                        if cross_sex_heat_request_id.get_untracked() != next_id {
                            return;
                        }
                        set_male_cross_heat.set(Some(h));
                    }
                    None => issues.push("Invalid men's payload.".to_string()),
                },
                Err(e) => issues.push(e),
            }
            let female_url = dataset_file_url(&latest_v.version, &female_c.row.entry.bin);
            match fetch_binary_first(&[&female_url]).await {
                Ok(bytes) => match parse_combined_bin(&bytes).map(|(_, h)| h) {
                    Some(h) => {
                        if cross_sex_heat_request_id.get_untracked() != next_id {
                            return;
                        }
                        set_female_cross_heat.set(Some(h));
                    }
                    None => issues.push("Invalid women's payload.".to_string()),
                },
                Err(e) => issues.push(e),
            }
            if cross_sex_heat_request_id.get_untracked() != next_id {
                return;
            }
            set_cross_sex_heat_loading.set(false);
            if !issues.is_empty() {
                set_cross_sex_heat_error.set(Some(issues.join(" ")));
            }
        });
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

    // Unit pref persistence
    Effect::new(move |_| {
        if unit_pref_loaded.get() {
            return;
        }
        let Some(w) = web_sys::window() else {
            return;
        };
        let Ok(Some(storage)) = w.local_storage() else {
            set_unit_pref_loaded.set(true);
            return;
        };
        if let Ok(Some(saved)) = storage.get_item("ironscale_units")
            && saved == "lb"
        {
            set_use_lbs.set(true);
        }
        set_unit_pref_loaded.set(true);
    });

    Effect::new(move |_| {
        if !unit_pref_loaded.get() {
            return;
        }
        let Some(w) = web_sys::window() else {
            return;
        };
        let Ok(Some(storage)) = w.local_storage() else {
            return;
        };
        let units = if use_lbs.get() { "lb" } else { "kg" };
        let _ = storage.set_item("ironscale_units", units);
    });

    // Query/hash load
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
                && let Ok(Some(raw)) = storage.get_item("ironscale_last_state")
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
                set_calculated.set(saved.calculated);
            }
            set_query_loaded.set(true);
            return;
        }
        let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) else {
            set_query_loaded.set(true);
            return;
        };
        if let Some(v) = params.get("sex") {
            set_sex.set(v);
        }
        if let Some(v) = params.get("equip") {
            set_equip.set(v);
        }
        if let Some(v) = params.get("wc") {
            set_wc.set(v);
        }
        if let Some(v) = params.get("age") {
            set_age.set(v);
        }
        if let Some(v) = params.get("tested") {
            set_tested.set(v);
        }
        if let Some(v) = params.get("lift") {
            set_lift.set(v);
        }
        if let Some(v) = params.get("metric") {
            set_metric.set(v);
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
        set_squat_delta.set(parse_query_f32(params.get("sd"), 0.0, -50.0, 50.0));
        set_bench_delta.set(parse_query_f32(params.get("bd"), 0.0, -50.0, 50.0));
        set_deadlift_delta.set(parse_query_f32(params.get("dd"), 0.0, -50.0, 50.0));
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
            let p = if hash.eq_ignore_ascii_case("#nerds") {
                AppPage::Nerds
            } else if hash.eq_ignore_ascii_case("#men-vs-women") {
                AppPage::MenVsWomen
            } else if hash.eq_ignore_ascii_case("#1rm") {
                AppPage::OneRm
            } else if hash.eq_ignore_ascii_case("#plate-calc") {
                AppPage::PlateCalc
            } else if hash.eq_ignore_ascii_case("#bodyfat") {
                AppPage::Bodyfat
            } else {
                AppPage::Ranking
            };
            set_active_page.set(p);
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
        let _ = window.location().set_hash(active_page.get().hash());
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
            share_handle: String::new(),
            calculated: calculated.get(),
        };
        if let Ok(raw) = serde_json::to_string(&snapshot) {
            let _ = storage.set_item("ironscale_last_state", &raw);
        }
    });

    // Cross-sex comparison derivation
    let cross_sex_comparison = Memo::new(move |_| -> Result<CrossSexComparison, String> {
        if !calculated.get() {
            return Err("Calculate first.".to_string());
        }
        if let Some(err) = cross_sex_rows_error.get() {
            return Err(err);
        }
        let mc = male_cross_choice
            .get()
            .ok_or("No matching men's cohort.".to_string())?;
        let fc = female_cross_choice
            .get()
            .ok_or("No matching women's cohort.".to_string())?;
        let ms = mc
            .row
            .entry
            .summary
            .as_ref()
            .ok_or("Men's summary missing.".to_string())?;
        let fs = fc
            .row
            .entry
            .summary
            .as_ref()
            .ok_or("Women's summary missing.".to_string())?;
        if ms.total < MIN_CROSS_SEX_COHORT_TOTAL || fs.total < MIN_CROSS_SEX_COHORT_TOTAL {
            return Err(format!(
                "Cohort too small (<{} lifters).",
                MIN_CROSS_SEX_COHORT_TOTAL
            ));
        }
        if let Some(err) = cross_sex_hist_error.get() {
            return Err(err);
        }
        let mh = male_cross_hist
            .get()
            .ok_or("Men's distribution not loaded.".to_string())?;
        let fh = female_cross_hist
            .get()
            .ok_or("Women's distribution not loaded.".to_string())?;
        let e = equip.get();
        let l = lift.get();
        let m = metric.get();
        let male_val = comparable_lift_value(
            ComparableLifter {
                sex: "M",
                equipment: &e,
                bodyweight: bodyweight.get(),
                squat: squat.get(),
                bench: bench.get(),
                deadlift: deadlift.get(),
            },
            &l,
            &m,
        );
        let female_val = comparable_lift_value(
            ComparableLifter {
                sex: "F",
                equipment: &e,
                bodyweight: bodyweight.get(),
                squat: squat.get(),
                bench: bench.get(),
                deadlift: deadlift.get(),
            },
            &l,
            &m,
        );
        let (male_pct, female_at_male_pct) =
            crate::core::equivalent_value_for_same_percentile(Some(&mh), Some(&fh), male_val)
                .ok_or("Could not compute women's equivalent.".to_string())?;
        let (female_pct, male_at_female_pct) =
            crate::core::equivalent_value_for_same_percentile(Some(&fh), Some(&mh), female_val)
                .ok_or("Could not compute men's equivalent.".to_string())?;
        let caveat = if m == "Kg" {
            Some(
                "Raw kg cross-sex comparisons are descriptive. Prefer Dots, Wilks, or Goodlift."
                    .to_string(),
            )
        } else {
            None
        };
        Ok(CrossSexComparison {
            male_percentile: male_pct,
            female_percentile: female_pct,
            male_total: ms.total,
            female_total: fs.total,
            male_input_value: male_val,
            female_input_value: female_val,
            female_value_at_male_percentile: female_at_male_pct,
            male_value_at_female_percentile: male_at_female_pct,
            metric: m,
            male_weight_class: mc.row.key.wc,
            female_weight_class: fc.row.key.wc,
            male_wc_fallback: mc.weight_class_fallback,
            female_wc_fallback: fc.weight_class_fallback,
            caveat,
        })
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
