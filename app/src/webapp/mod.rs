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
use self::ui::{age_label, lift_label, metric_label, parse_f32_input, parse_f32_input_clamped};
use leptos::html::Canvas;
use leptos::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use crate::core::{
    dots_points, goodlift_points, percentile_for_value, rebin_1d, rebin_2d, wilks_points,
    HeatmapBin, HistogramBin,
};

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

#[component]
fn App() -> impl IntoView {
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

    view! {
        <div class="page">
            <header class="hero">
                <h1>"How Do I Stack Up?"</h1>
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
            </header>

            <section class="panel">
                <div class="grid">
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

                    <label>"IPF class"
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

                    <label>"Age class"
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

                    <label>"Tested"
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

                    <label>"Lift"
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

                    <label>"Compare by"
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
                </div>

                <div class="grid numbers">
                    <label>"Squat (kg)"
                        <input
                            type="number"
                            min="0"
                            max="600"
                            step="0.5"
                            prop:value=move || squat.get().to_string()
                            on:input=move |ev| {
                                set_squat.set(parse_f32_input_clamped(&ev, squat.get_untracked(), 0.0, 600.0))
                            }
                        />
                    </label>
                    <label>"Bench (kg)"
                        <input
                            type="number"
                            min="0"
                            max="600"
                            step="0.5"
                            prop:value=move || bench.get().to_string()
                            on:input=move |ev| {
                                set_bench.set(parse_f32_input_clamped(&ev, bench.get_untracked(), 0.0, 600.0))
                            }
                        />
                    </label>
                    <label>"Deadlift (kg)"
                        <input
                            type="number"
                            min="0"
                            max="600"
                            step="0.5"
                            prop:value=move || deadlift.get().to_string()
                            on:input=move |ev| {
                                set_deadlift.set(parse_f32_input_clamped(&ev, deadlift.get_untracked(), 0.0, 600.0))
                            }
                        />
                    </label>
                    <label>"Bodyweight (kg)"
                        <input
                            type="number"
                            min="35"
                            max="300"
                            step="0.5"
                            prop:value=move || bodyweight.get().to_string()
                            on:input=move |ev| {
                                set_bodyweight.set(parse_f32_input_clamped(
                                    &ev,
                                    bodyweight.get_untracked(),
                                    35.0,
                                    300.0,
                                ))
                            }
                        />
                    </label>
                </div>

                <div class="grid">
                    <label>"Lift bin"
                        <select
                            prop:value=move || lift_mult.get().to_string()
                            on:change=move |ev| set_lift_mult.set(event_target_value(&ev).parse::<usize>().unwrap_or(4))
                        >
                            <option value="1">"1x base"</option>
                            <option value="2">"2x base"</option>
                            <option value="4">"4x base"</option>
                        </select>
                    </label>
                    <label>"BW bin"
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

                <div class="simulator">
                    <h3>"Progression Simulator"</h3>
                    <p class="muted">
                        "Adjust projected lift changes to see percentile/rank shift using this slice's current distribution."
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
                            <strong>{move || format!("{:+.1} kg", squat_delta.get())}</strong>
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
                            <strong>{move || format!("{:+.1} kg", bench_delta.get())}</strong>
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
                            <strong>{move || format!("{:+.1} kg", deadlift_delta.get())}</strong>
                        </label>
                    </div>
                    <p class="sim-summary">
                        {move || format!(
                            "Projected total: {:.1} kg (S {:.1} / B {:.1} / D {:.1})",
                            projected_total.get(),
                            projected_squat.get(),
                            projected_bench.get(),
                            projected_deadlift.get()
                        )}
                    </p>
                    <p class="muted">
                        "Guardrails: lifts are clamped to 0-600kg, bodyweight to 35-300kg, and sliders to +/-50kg."
                    </p>
                </div>
            </section>

            <section class="panel stats">
                <h2>"Percentile"</h2>
                <p>
                    {move || match percentile.get() {
                        Some((pct, rank, total)) => format!("{:.1}% percentile | rank ~{} / {}", pct * 100.0, rank, total),
                        None => "No distribution loaded for this slice.".to_string(),
                    }}
                </p>
                <p>
                    {move || match projected_percentile.get() {
                        Some((pct, rank, total)) => {
                            format!("Projected: {:.1}% percentile | rank ~{} / {}", pct * 100.0, rank, total)
                        }
                        None => "Projected: unavailable until distribution is loaded.".to_string(),
                    }}
                </p>
                <p>
                    {move || match percentile_delta.get() {
                        Some(delta) => format!("Shift: {:+.2} percentile points", delta * 100.0),
                        None => "Shift: n/a".to_string(),
                    }}
                </p>
                <p class="muted">
                    {move || {
                        if lift.get() == "T" {
                            "For Total views, all three slider changes affect projection.".to_string()
                        } else {
                            format!(
                                "For {} views, only that lift directly affects projected percentile.",
                                lift_label(&lift.get())
                            )
                        }
                    }}
                </p>
            </section>

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
        </div>
    }
}

#[cfg(debug_assertions)]
fn debug_log(message: &str) {
    web_sys::console::debug_1(&message.into());
}

#[cfg(not(debug_assertions))]
fn debug_log(_message: &str) {}
