mod charts;
mod data;
mod selectors;
mod slices;
mod state;
mod ui;

use self::charts::{draw_heatmap, render_histogram_svg};
use self::selectors::{
    age_options, equip_options, lift_options, metric_options, sex_options, tested_options,
    wc_options,
};
use self::slices::SliceKey;
use self::state::{
    init_dataset_load, setup_default_selection_effects, setup_distribution_effect,
    setup_slice_rows_effect,
};
use self::ui::{age_label, lift_label, metric_label, parse_f32_input};
use leptos::html::Canvas;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wasm_bindgen::JsCast;
use crate::core::{
    dots_points, goodlift_points, percentile_for_value, rebin_1d, rebin_2d, wilks_points,
    HeatmapBin, HistogramBin,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompareMode {
    AllLifters,
    SameBodyweightRange,
    SameWeightClass,
    SameAgeClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedUiState {
    sex: String,
    equip: String,
    wc: String,
    age: String,
    tested: String,
    lift: String,
    metric: String,
    squat: f32,
    bench: f32,
    deadlift: f32,
    bodyweight: f32,
    squat_delta: f32,
    bench_delta: f32,
    deadlift_delta: f32,
    lift_mult: usize,
    bw_mult: usize,
    share_handle: String,
    calculated: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct LatestJson {
    version: String,
    revision: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RootIndex {
    shards: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SliceIndex {
    shard_key: String,
    slices: SliceIndexEntries,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SliceIndexEntries {
    Map(BTreeMap<String, SliceIndexEntry>),
    Keys(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct SliceIndexEntry {
    meta: String,
    hist: String,
    heat: String,
}

#[derive(Debug, Clone, PartialEq)]
struct SliceRow {
    key: SliceKey,
    entry: SliceIndexEntry,
}

pub fn run() {
    mount_to_body(|| view! { <App /> });
}

fn comparable_lift_value(
    sex: &str,
    equipment: &str,
    lift: &str,
    metric: &str,
    bodyweight: f32,
    squat: f32,
    bench: f32,
    deadlift: f32,
) -> f32 {
    let total = squat + bench + deadlift;
    match (lift, metric) {
        ("S", _) => squat,
        ("B", _) => bench,
        ("D", _) => deadlift,
        ("T", "Dots") => dots_points(sex, bodyweight, total),
        ("T", "Wilks") => wilks_points(sex, bodyweight, total),
        ("T", "GL") => goodlift_points(sex, equipment, bodyweight, total),
        ("T", _) => total,
        _ => 0.0,
    }
}

fn tier_for_percentile(pct: f32) -> &'static str {
    if pct >= 0.999 {
        "Mythical"
    } else if pct >= 0.99 {
        "Legendary"
    } else if pct >= 0.95 {
        "Elite"
    } else if pct >= 0.8 {
        "Advanced"
    } else if pct >= 0.6 {
        "Intermediate"
    } else if pct >= 0.35 {
        "Novice"
    } else {
        "Beginner"
    }
}

fn parse_query_f32(value: Option<String>, default: f32, min: f32, max: f32) -> f32 {
    value
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(default)
        .clamp(min, max)
}

fn kg_to_display(kg: f32, use_lbs: bool) -> f32 {
    if use_lbs { kg * 2.204_622_5 } else { kg }
}

fn display_to_kg(value: f32, use_lbs: bool) -> f32 {
    if use_lbs { value / 2.204_622_5 } else { value }
}

fn build_share_url(params: &[(&str, String)]) -> Option<String> {
    let window = web_sys::window()?;
    let search = web_sys::UrlSearchParams::new().ok()?;
    for (key, value) in params {
        search.append(key, value);
    }
    let origin = window.location().origin().ok()?;
    Some(format!("{origin}/?{}", search.to_string()))
}

#[allow(clippy::too_many_arguments)]
fn download_share_png(
    handle: &str,
    bodyweight: f32,
    squat: f32,
    bench: f32,
    deadlift: f32,
    lift_focus: &str,
    percentile: f32,
    tier: &str,
) -> Result<(), String> {
    let Some(window) = web_sys::window() else {
        return Err("No browser window.".to_string());
    };
    let Some(document) = window.document() else {
        return Err("No browser document.".to_string());
    };

    let canvas = document
        .create_element("canvas")
        .map_err(|_| "Failed to create canvas.")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| "Failed to create share canvas.")?;
    canvas.set_width(1200);
    canvas.set_height(630);

    let context = canvas
        .get_context("2d")
        .map_err(|_| "Failed to get 2d context.")?
        .ok_or_else(|| "No 2d context available.".to_string())?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| "Invalid canvas rendering context.")?;

    context.set_fill_style(&wasm_bindgen::JsValue::from_str("#0f2e24"));
    context.fill_rect(0.0, 0.0, 1200.0, 630.0);
    context.set_fill_style(&wasm_bindgen::JsValue::from_str("#153f31"));
    context.fill_rect(40.0, 40.0, 1120.0, 550.0);
    context.set_fill_style(&wasm_bindgen::JsValue::from_str("#f6f2e8"));
    context.set_font("700 60px 'Space Grotesk', sans-serif");
    context
        .fill_text("Iron Insights Ranking", 80.0, 130.0)
        .map_err(|_| "Failed to render heading text.")?;

    context.set_font("500 36px 'Space Grotesk', sans-serif");
    let who = if handle.trim().is_empty() {
        "Anonymous Lifter".to_string()
    } else {
        handle.trim().to_string()
    };
    context
        .fill_text(&who, 80.0, 200.0)
        .map_err(|_| "Failed to render handle.")?;

    context.set_font("400 30px 'IBM Plex Mono', monospace");
    context
        .fill_text(
            &format!(
                "BW {:.1}kg | S {:.1} | B {:.1} | D {:.1} | Focus {}",
                bodyweight, squat, bench, deadlift, lift_focus
            ),
            80.0,
            265.0,
        )
        .map_err(|_| "Failed to render lift line.")?;
    context
        .fill_text(
            &format!(
                "Stronger than {:.1}% of lifters | Tier {}",
                percentile * 100.0,
                tier
            ),
            80.0,
            320.0,
        )
        .map_err(|_| "Failed to render rank line.")?;

    context.set_fill_style(&wasm_bindgen::JsValue::from_str("#d4c6a9"));
    context.fill_rect(80.0, 370.0, 760.0, 14.0);
    context.set_fill_style(&wasm_bindgen::JsValue::from_str("#f6f2e8"));
    context.fill_rect(80.0, 370.0, (percentile * 760.0).clamp(0.0, 760.0) as f64, 14.0);

    context.set_fill_style(&wasm_bindgen::JsValue::from_str("#f6f2e8"));
    context.set_font("400 24px 'IBM Plex Mono', monospace");
    context
        .fill_text("iron-insights", 80.0, 555.0)
        .map_err(|_| "Failed to render watermark.")?;

    let data_url = canvas
        .to_data_url_with_type("image/png")
        .map_err(|_| "Failed to export PNG.")?;
    let anchor = document
        .create_element("a")
        .map_err(|_| "Failed to create download link.")?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "Failed to create download anchor.")?;
    anchor.set_href(&data_url);
    anchor.set_download("iron-insights-ranking.png");
    anchor.click();
    Ok(())
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
                <section class="panel onboarding">
                    <h2>"Your Numbers"</h2>
                    <div class="grid simple">
                        <label>"Sex"
                            <select on:change=move |ev| set_sex.set(event_target_value(&ev))>
                                <For each=move || sex_options.get() key=|v| v.clone() let:value>
                                    <option
                                        selected={
                                            let selected_value = value.clone();
                                            move || sex.get() == selected_value
                                        }
                                        value={value.clone()}
                                    >
                                        {value.clone()}
                                    </option>
                                </For>
                            </select>
                        </label>
                        <label>"Equipment"
                            <select on:change=move |ev| set_equip.set(event_target_value(&ev))>
                                <For each=move || equip_options.get() key=|v| v.clone() let:value>
                                    <option
                                        selected={
                                            let selected_value = value.clone();
                                            move || equip.get() == selected_value
                                        }
                                        value={value.clone()}
                                    >
                                        {value.clone()}
                                    </option>
                                </For>
                            </select>
                        </label>
                    </div>
                    <div class="grid numbers">
                        <label>{move || format!("Squat ({})", unit_label.get())}
                            <input
                                type="number"
                                min="0"
                                max="600"
                                step="0.5"
                                prop:value=move || format!("{:.1}", kg_to_display(squat.get(), use_lbs.get()))
                                on:input=move |ev| {
                                    let raw = event_target_value(&ev);
                                    match raw.parse::<f32>() {
                                        Ok(value) => {
                                            let value_kg = display_to_kg(value, use_lbs.get());
                                            if (0.0..=600.0).contains(&value_kg) {
                                                set_squat.set(value_kg);
                                                set_squat_error.set(None);
                                            } else {
                                                set_squat_error.set(Some(format!(
                                                    "Squat must be {:.0}-{:.0}{}.",
                                                    kg_to_display(0.0, use_lbs.get()),
                                                    kg_to_display(600.0, use_lbs.get()),
                                                    unit_label.get()
                                                )));
                                            }
                                        }
                                        Err(_) => set_squat_error.set(Some("Enter a valid squat number.".to_string())),
                                    }
                                }
                            />
                        </label>
                        <label>{move || format!("Bench ({})", unit_label.get())}
                            <input
                                type="number"
                                min="0"
                                max="600"
                                step="0.5"
                                prop:value=move || format!("{:.1}", kg_to_display(bench.get(), use_lbs.get()))
                                on:input=move |ev| {
                                    let raw = event_target_value(&ev);
                                    match raw.parse::<f32>() {
                                        Ok(value) => {
                                            let value_kg = display_to_kg(value, use_lbs.get());
                                            if (0.0..=600.0).contains(&value_kg) {
                                                set_bench.set(value_kg);
                                                set_bench_error.set(None);
                                            } else {
                                                set_bench_error.set(Some(format!(
                                                    "Bench must be {:.0}-{:.0}{}.",
                                                    kg_to_display(0.0, use_lbs.get()),
                                                    kg_to_display(600.0, use_lbs.get()),
                                                    unit_label.get()
                                                )));
                                            }
                                        }
                                        Err(_) => set_bench_error.set(Some("Enter a valid bench number.".to_string())),
                                    }
                                }
                            />
                        </label>
                        <label>{move || format!("Deadlift ({})", unit_label.get())}
                            <input
                                type="number"
                                min="0"
                                max="600"
                                step="0.5"
                                prop:value=move || format!("{:.1}", kg_to_display(deadlift.get(), use_lbs.get()))
                                on:input=move |ev| {
                                    let raw = event_target_value(&ev);
                                    match raw.parse::<f32>() {
                                        Ok(value) => {
                                            let value_kg = display_to_kg(value, use_lbs.get());
                                            if (0.0..=600.0).contains(&value_kg) {
                                                set_deadlift.set(value_kg);
                                                set_deadlift_error.set(None);
                                            } else {
                                                set_deadlift_error.set(Some(format!(
                                                    "Deadlift must be {:.0}-{:.0}{}.",
                                                    kg_to_display(0.0, use_lbs.get()),
                                                    kg_to_display(600.0, use_lbs.get()),
                                                    unit_label.get()
                                                )));
                                            }
                                        }
                                        Err(_) => set_deadlift_error.set(Some("Enter a valid deadlift number.".to_string())),
                                    }
                                }
                            />
                        </label>
                        <label>{move || format!("Bodyweight ({})", unit_label.get())}
                            <input
                                type="number"
                                min="35"
                                max="300"
                                step="0.5"
                                prop:value=move || format!("{:.1}", kg_to_display(bodyweight.get(), use_lbs.get()))
                                on:input=move |ev| {
                                    let raw = event_target_value(&ev);
                                    match raw.parse::<f32>() {
                                        Ok(value) => {
                                            let value_kg = display_to_kg(value, use_lbs.get());
                                            if (35.0..=300.0).contains(&value_kg) {
                                                set_bodyweight.set(value_kg);
                                                set_bodyweight_error.set(None);
                                            } else {
                                                set_bodyweight_error.set(Some(format!(
                                                    "Bodyweight must be {:.0}-{:.0}{}.",
                                                    kg_to_display(35.0, use_lbs.get()),
                                                    kg_to_display(300.0, use_lbs.get()),
                                                    unit_label.get()
                                                )));
                                            }
                                        }
                                        Err(_) => set_bodyweight_error.set(Some("Enter a valid bodyweight number.".to_string())),
                                    }
                                }
                            />
                        </label>
                    </div>
                    <div class="input-errors">
                        <Show when=move || squat_error.get().is_some()>
                            <p class="input-error">{move || squat_error.get().unwrap_or_default()}</p>
                        </Show>
                        <Show when=move || bench_error.get().is_some()>
                            <p class="input-error">{move || bench_error.get().unwrap_or_default()}</p>
                        </Show>
                        <Show when=move || deadlift_error.get().is_some()>
                            <p class="input-error">{move || deadlift_error.get().unwrap_or_default()}</p>
                        </Show>
                        <Show when=move || bodyweight_error.get().is_some()>
                            <p class="input-error">{move || bodyweight_error.get().unwrap_or_default()}</p>
                        </Show>
                    </div>
                    <div class="control-row">
                        <label class="units-toggle">
                            "Units"
                            <div class="toggle-buttons">
                                <button
                                    type="button"
                                    class="chip"
                                    class:active=move || !use_lbs.get()
                                    on:click=move |_| set_use_lbs.set(false)
                                >
                                    "kg"
                                </button>
                                <button
                                    type="button"
                                    class="chip"
                                    class:active=move || use_lbs.get()
                                    on:click=move |_| set_use_lbs.set(true)
                                >
                                    "lb"
                                </button>
                            </div>
                        </label>
                        <button
                            type="button"
                            class="secondary"
                            on:click=move |_| {
                                set_squat.set(0.0);
                                set_bench.set(0.0);
                                set_deadlift.set(0.0);
                                set_bodyweight.set(90.0);
                                set_squat_delta.set(0.0);
                                set_bench_delta.set(0.0);
                                set_deadlift_delta.set(0.0);
                                set_share_handle.set(String::new());
                                set_squat_error.set(None);
                                set_bench_error.set(None);
                                set_deadlift_error.set(None);
                                set_bodyweight_error.set(None);
                                set_calculated.set(false);
                            }
                        >
                            "Clear all"
                        </button>
                        <button
                            type="button"
                            class="secondary"
                            on:click=move |_| {
                                let Some(window) = web_sys::window() else {
                                    set_share_status.set(Some("No browser window.".to_string()));
                                    return;
                                };
                                let Ok(Some(storage)) = window.local_storage() else {
                                    set_share_status.set(Some("Local storage unavailable.".to_string()));
                                    return;
                                };
                                let Ok(Some(raw)) = storage.get_item("iron_insights_last_state") else {
                                    set_share_status.set(Some("No saved numbers found.".to_string()));
                                    return;
                                };
                                let Ok(saved) = serde_json::from_str::<SavedUiState>(&raw) else {
                                    set_share_status.set(Some("Saved numbers are invalid.".to_string()));
                                    return;
                                };
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
                                set_share_status.set(Some("Loaded saved numbers.".to_string()));
                            }
                        >
                            "Use my last numbers"
                        </button>
                    </div>
                    <button
                        type="button"
                        class="primary"
                        prop:disabled=move || has_input_error.get() || calculating.get()
                        on:click=move |_| {
                            set_calculating.set(true);
                            set_share_status.set(None);
                            let set_calculating = set_calculating;
                            let set_calculated = set_calculated;
                            gloo_timers::callback::Timeout::new(420, move || {
                                set_calculating.set(false);
                                set_calculated.set(true);
                            })
                            .forget();
                        }
                    >
                        {move || if calculating.get() { "Calculating..." } else { "Calculate my ranking" }}
                    </button>
                    <details class="advanced">
                        <summary>"Advanced Options"</summary>
                        <div class="grid">
                            <label title="Filter by drug-tested or untested meets.">
                                "Drug tested"
                                <select on:change=move |ev| set_tested.set(event_target_value(&ev))>
                                    <For each=move || tested_options.get() key=|v| v.clone() let:value>
                                        <option
                                            selected={
                                                let selected_value = value.clone();
                                                move || tested.get() == selected_value
                                            }
                                            value={value.clone()}
                                        >
                                            {value.clone()}
                                        </option>
                                    </For>
                                </select>
                            </label>
                            <label title="Compare only with this age class, or all ages.">
                                "Age class"
                                <select on:change=move |ev| set_age.set(event_target_value(&ev))>
                                    <For each=move || age_options.get() key=|v| v.clone() let:value>
                                        <option
                                            selected={
                                                let selected_value = value.clone();
                                                move || age.get() == selected_value
                                            }
                                            value={value.clone()}
                                        >
                                            {age_label(&value).to_string()}
                                        </option>
                                    </For>
                                </select>
                            </label>
                            <label title="Compare only to this bodyweight class, or choose All.">
                                "IPF class"
                                <select on:change=move |ev| set_wc.set(event_target_value(&ev))>
                                    <For each=move || wc_options.get() key=|v| v.clone() let:value>
                                        <option
                                            selected={
                                                let selected_value = value.clone();
                                                move || wc.get() == selected_value
                                            }
                                            value={value.clone()}
                                        >
                                            {value.clone()}
                                        </option>
                                    </For>
                                </select>
                            </label>
                            <label title="Pick squat, bench, deadlift, or total.">
                                "Lift focus"
                                <select on:change=move |ev| set_lift.set(event_target_value(&ev))>
                                    <For each=move || lift_options.get() key=|v| v.clone() let:value>
                                        <option
                                            selected={
                                                let selected_value = value.clone();
                                                move || lift.get() == selected_value
                                            }
                                            value={value.clone()}
                                        >
                                            {lift_label(&value).to_string()}
                                        </option>
                                    </For>
                                </select>
                            </label>
                            <label title="Use kilograms or points for comparison.">
                                "Compare by"
                                <select on:change=move |ev| set_metric.set(event_target_value(&ev))>
                                    <For each=move || metric_options.get() key=|v| v.clone() let:value>
                                        <option
                                            selected={
                                                let selected_value = value.clone();
                                                move || metric.get() == selected_value
                                            }
                                            value={value.clone()}
                                        >
                                            {metric_label(&value).to_string()}
                                        </option>
                                    </For>
                                </select>
                            </label>
                            <label title="Grouping size used for lift distributions.">
                                "Grouping size"
                                <select
                                    prop:value=move || lift_mult.get().to_string()
                                    on:change=move |ev| set_lift_mult.set(event_target_value(&ev).parse::<usize>().unwrap_or(4))
                                >
                                    <option value="1">"1x base"</option>
                                    <option value="2">"2x base"</option>
                                    <option value="4">"4x base"</option>
                                </select>
                            </label>
                            <label title="Bodyweight bucket size used in the heatmap.">
                                "Bodyweight grouping"
                                <select
                                    prop:value=move || bw_mult.get().to_string()
                                    on:change=move |ev| set_bw_mult.set(event_target_value(&ev).parse::<usize>().unwrap_or(5))
                                >
                                    <option value="1">"1kg"</option>
                                    <option value="2">"2kg"</option>
                                    <option value="5">"5kg"</option>
                                </select>
                            </label>
                        </div>
                        <button
                            type="button"
                            class="secondary"
                            on:click=move |_| {
                                set_tested.set("All".to_string());
                                set_age.set("All Ages".to_string());
                                set_wc.set("All".to_string());
                                set_lift.set("T".to_string());
                                set_metric.set("Kg".to_string());
                                set_lift_mult.set(4);
                                set_bw_mult.set(5);
                            }
                        >
                            "Reset to defaults"
                        </button>
                    </details>
                </section>

                <section class="panel result-card">
                    <h2>"Result"</h2>
                    <Show
                        when=move || calculated.get()
                        fallback=move || view! { <p class="muted">"Press Calculate my ranking to load your headline result."</p> }
                    >
                        <p class="big">
                            {move || match percentile.get() {
                                Some((pct, _, _)) => format!("You are stronger than {:.1}% of lifters", pct * 100.0),
                                None => "No matching distribution found for this slice.".to_string(),
                            }}
                        </p>
                        <p class="topline">
                            {move || match percentile.get() {
                                Some((pct, _, total)) => format!("Top {:.1}% | Compared against {} lifters", (1.0 - pct).max(0.0) * 100.0, total),
                                None => "Top tier unavailable.".to_string(),
                            }}
                        </p>
                        <p class="tier">
                            {move || match rank_tier.get() {
                                Some(tier) => format!("Tier: {}", tier),
                                None => "Tier: unavailable".to_string(),
                            }}
                        </p>
                        <p class="muted">"Higher is stronger."</p>
                        <div class="share-row">
                            <button
                                type="button"
                                class="secondary"
                                on:click=move |_| set_show_share.update(|open| *open = !*open)
                            >
                                "Share my ranking"
                            </button>
                            <button
                                type="button"
                                class="secondary"
                                on:click=move |_| {
                                    let Some(url) = share_url.get() else {
                                        set_share_status.set(Some("Unable to generate share link.".to_string()));
                                        return;
                                    };
                                    let Some(window) = web_sys::window() else {
                                        set_share_status.set(Some("Clipboard unavailable.".to_string()));
                                        return;
                                    };
                                    let clipboard = window.navigator().clipboard();
                                    let _ = clipboard.write_text(&url);
                                    set_share_status.set(Some("Link copied.".to_string()));
                                }
                            >
                                "Copy link"
                            </button>
                        </div>
                        <Show when=move || show_share.get()>
                            <div class="share-card">
                                <label>
                                    "Name / handle (optional)"
                                    <input
                                        type="text"
                                        placeholder="@lifter"
                                        prop:value=move || share_handle.get()
                                        on:input=move |ev| set_share_handle.set(event_target_value(&ev))
                                    />
                                </label>
                                <button
                                    type="button"
                                    class="secondary"
                                    on:click=move |_| {
                                        let Some((pct, _, _)) = percentile.get() else {
                                            set_share_status.set(Some("Calculate first to generate an image.".to_string()));
                                            return;
                                        };
                                        let tier = rank_tier.get().unwrap_or("Unknown");
                                        let result = download_share_png(
                                            &share_handle.get(),
                                            bodyweight.get(),
                                            squat.get(),
                                            bench.get(),
                                            deadlift.get(),
                                            lift_label(&lift.get()),
                                            pct,
                                            tier,
                                        );
                                        match result {
                                            Ok(()) => set_share_status.set(Some("PNG downloaded.".to_string())),
                                            Err(err) => set_share_status.set(Some(err)),
                                        }
                                    }
                                >
                                    "Download PNG"
                                </button>
                            </div>
                        </Show>
                        <Show when=move || share_status.get().is_some()>
                            <p class="muted">{move || share_status.get().unwrap_or_default()}</p>
                        </Show>
                    </Show>
                </section>

                <section class="panel">
                    <h2>"Compare mode"</h2>
                    <div class="compare-tabs">
                        <button
                            type="button"
                            class="chip"
                            class:active=move || compare_mode.get() == CompareMode::AllLifters
                            on:click=move |_| set_compare_mode.set(CompareMode::AllLifters)
                        >
                            "All lifters"
                        </button>
                        <button
                            type="button"
                            class="chip"
                            class:active=move || compare_mode.get() == CompareMode::SameBodyweightRange
                            on:click=move |_| set_compare_mode.set(CompareMode::SameBodyweightRange)
                        >
                            "Same bodyweight range"
                        </button>
                        <button
                            type="button"
                            class="chip"
                            class:active=move || compare_mode.get() == CompareMode::SameWeightClass
                            on:click=move |_| set_compare_mode.set(CompareMode::SameWeightClass)
                        >
                            "Same weight class"
                        </button>
                        <button
                            type="button"
                            class="chip"
                            class:active=move || compare_mode.get() == CompareMode::SameAgeClass
                            on:click=move |_| set_compare_mode.set(CompareMode::SameAgeClass)
                        >
                            "Same age class"
                        </button>
                    </div>
                    <p class="muted">{move || compare_summary.get()}</p>
                </section>

                <section class="panel">
                    <h2>"Your percentile"</h2>
                    <div class="pct-track">
                        <div
                            class="pct-fill"
                            style:width=move || format!(
                                "{:.1}%",
                                percentile_percent.get().unwrap_or(0.0).clamp(0.0, 100.0)
                            )
                        ></div>
                    </div>
                    <p class="muted">
                        {move || match percentile_percent.get() {
                            Some(value) => format!("You marker: {:.1} / 100", value),
                            None => "Your marker appears after a matching slice loads.".to_string(),
                        }}
                    </p>
                    <button
                        type="button"
                        class="secondary"
                        on:click=move |_| set_show_main_charts.update(|open| *open = !*open)
                    >
                        {move || if show_main_charts.get() { "Hide distribution charts" } else { "View distribution charts" }}
                    </button>
                </section>

                <Show when=move || show_main_charts.get()>
                    <section class="panel">
                        <h2>"Histogram"</h2>
                        {move || match rebinned_hist.get() {
                            Some(h) => render_histogram_svg(&h, user_lift.get(), &hist_x_label.get()),
                            None => view! { <p>"No histogram available."</p> }.into_any(),
                        }}
                    </section>

                    <section class="panel">
                        <h2>{move || format!("Bodyweight vs {}", hist_x_label.get())}</h2>
                        <canvas node_ref=canvas_ref width="800" height="420"></canvas>
                    </section>
                </Show>

                <section class="panel">
                    <h2>"What if you got stronger?"</h2>
                    <p class="muted">
                        "Use sliders to project where small improvements could move your rank."
                    </p>
                    <div class="sim-grid">
                        <label class="sim-control">
                            <span>"Squat change"</span>
                            <input
                                type="range"
                                min="-50"
                                max="50"
                                step="0.5"
                                prop:value=move || squat_delta.get().to_string()
                                on:input=move |ev| set_squat_delta.set(parse_f32_input(&ev).clamp(-50.0, 50.0))
                            />
                            <strong>{move || format!("{:+.1} {}", kg_to_display(squat_delta.get(), use_lbs.get()), unit_label.get())}</strong>
                        </label>
                        <label class="sim-control">
                            <span>"Bench change"</span>
                            <input
                                type="range"
                                min="-50"
                                max="50"
                                step="0.5"
                                prop:value=move || bench_delta.get().to_string()
                                on:input=move |ev| set_bench_delta.set(parse_f32_input(&ev).clamp(-50.0, 50.0))
                            />
                            <strong>{move || format!("{:+.1} {}", kg_to_display(bench_delta.get(), use_lbs.get()), unit_label.get())}</strong>
                        </label>
                        <label class="sim-control">
                            <span>"Deadlift change"</span>
                            <input
                                type="range"
                                min="-50"
                                max="50"
                                step="0.5"
                                prop:value=move || deadlift_delta.get().to_string()
                                on:input=move |ev| set_deadlift_delta.set(parse_f32_input(&ev).clamp(-50.0, 50.0))
                            />
                            <strong>{move || format!("{:+.1} {}", kg_to_display(deadlift_delta.get(), use_lbs.get()), unit_label.get())}</strong>
                        </label>
                    </div>
                    <div class="preset-row">
                        <button type="button" class="chip" on:click=move |_| set_deadlift_delta.set((deadlift_delta.get_untracked() + 10.0).clamp(-50.0, 50.0))>"+10kg DL"</button>
                        <button type="button" class="chip" on:click=move |_| {
                            set_squat_delta.set((squat_delta.get_untracked() + 6.5).clamp(-50.0, 50.0));
                            set_bench_delta.set((bench_delta.get_untracked() + 6.5).clamp(-50.0, 50.0));
                            set_deadlift_delta.set((deadlift_delta.get_untracked() + 7.0).clamp(-50.0, 50.0));
                        }>"+20kg total"</button>
                        <button type="button" class="chip" on:click=move |_| {
                            set_squat_delta.set((squat_delta.get_untracked() + 5.0).clamp(-50.0, 50.0));
                            set_bench_delta.set((bench_delta.get_untracked() + 5.0).clamp(-50.0, 50.0));
                            set_deadlift_delta.set((deadlift_delta.get_untracked() + 5.0).clamp(-50.0, 50.0));
                        }>"Meet PRs"</button>
                        <button type="button" class="chip" on:click=move |_| {
                            set_squat_delta.set((squat_delta.get_untracked() + 10.0).clamp(-50.0, 50.0));
                            set_bench_delta.set((bench_delta.get_untracked() + 5.0).clamp(-50.0, 50.0));
                            set_deadlift_delta.set((deadlift_delta.get_untracked() + 12.5).clamp(-50.0, 50.0));
                        }>"1-year projection"</button>
                    </div>
                    <p class="sim-summary">
                        {move || format!(
                            "Projected total: {:.1} {} (S {:.1} / B {:.1} / D {:.1})",
                            kg_to_display(projected_total.get(), use_lbs.get()),
                            unit_label.get(),
                            kg_to_display(projected_squat.get(), use_lbs.get()),
                            kg_to_display(projected_bench.get(), use_lbs.get()),
                            kg_to_display(projected_deadlift.get(), use_lbs.get())
                        )}
                    </p>
                    <p class="muted">
                        {move || match projected_percentile.get() {
                            Some((pct, rank, total)) => format!(
                                "Projected: {:.1}% percentile, rank ~{} / {}, tier {}",
                                pct * 100.0,
                                rank,
                                total,
                                projected_rank_tier.get().unwrap_or("Unknown")
                            ),
                            None => "Projected percentile will appear after calculation.".to_string(),
                        }}
                    </p>
                    <p class="muted">
                        {move || match percentile_delta.get() {
                            Some(delta) => format!("Shift: {:+.2} percentile points", delta * 100.0),
                            None => "Shift: n/a".to_string(),
                        }}
                    </p>
                </section>

                <section class="panel faq">
                    <h2>"FAQ"</h2>
                    <details>
                        <summary>"What does percentile mean?"</summary>
                        <p>
                            "Percentile shows the share of comparable lifters you outperform. Higher percentile means stronger relative ranking."
                        </p>
                    </details>
                    <details>
                        <summary>"Where does the data come from?"</summary>
                        <p>
                            "Data is loaded from the bundled competition dataset version shown at the top of the page."
                        </p>
                    </details>
                    <details>
                        <summary>"Why does equipment matter?"</summary>
                        <p>
                            "Different equipment changes performance. Filtering by equipment gives fairer comparisons."
                        </p>
                    </details>
                </section>
        </div>
    }
}

#[cfg(debug_assertions)]
fn debug_log(message: &str) {
    web_sys::console::debug_1(&message.into());
}

#[cfg(not(debug_assertions))]
fn debug_log(_message: &str) {}
