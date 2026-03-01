use gloo_net::http::Request;
use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SliceKey {
    sex: String,
    equip: String,
    wc: String,
    age: String,
    tested: String,
    lift: String,
    metric: String,
}

#[derive(Debug, Clone, PartialEq)]
struct SliceRow {
    key: SliceKey,
    entry: SliceIndexEntry,
}

#[derive(Debug, Clone, PartialEq)]
struct HistogramBin {
    min: f32,
    max: f32,
    base_bin: f32,
    counts: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
struct HeatmapBin {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    base_x: f32,
    base_y: f32,
    width: usize,
    height: usize,
    grid: Vec<u32>,
}

pub fn run() {
    mount_to_body(|| view! { <App /> });
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

    let (lift_mult, set_lift_mult) = signal(4usize);
    let (bw_mult, set_bw_mult) = signal(5usize);

    let (hist, set_hist) = signal(None::<HistogramBin>);
    let (heat, set_heat) = signal(None::<HeatmapBin>);

    let canvas_ref: NodeRef<Canvas> = NodeRef::new();

    {
        let set_latest = set_latest;
        let set_root_index = set_root_index;
        let set_sex = set_sex;
        let set_equip = set_equip;
        let set_load_error = set_load_error;

        spawn_local(async move {
            let latest_json =
                fetch_json_first::<LatestJson>(&["./data/latest.json", "/data/latest.json"]).await;
            let Ok(latest_json) = latest_json else {
                set_load_error.set(Some(
                    "Failed to load latest dataset pointer (data/latest.json).".to_string(),
                ));
                return;
            };
            set_latest.set(Some(latest_json.clone()));

            let index_url = format!("./data/{}/index.json", latest_json.version);
            let index_url_abs = format!("/data/{}/index.json", latest_json.version);
            let index = fetch_json_first::<RootIndex>(&[&index_url, &index_url_abs]).await;
            let Ok(index) = index else {
                if let Err(err) = index {
                    set_load_error.set(Some(format!(
                        "Failed to load slice index for {}: {}",
                        latest_json.version, err
                    )));
                }
                return;
            };
            set_load_error.set(None);
            set_root_index.set(Some(index.clone()));

            let mut shard_keys: Vec<String> = index.shards.keys().cloned().collect();
            shard_keys.sort();
            if !shard_keys.is_empty() {
                let sexes = unique(
                    shard_keys
                        .iter()
                        .filter_map(|k| parse_shard_key(k).map(|(s, _)| s.to_string())),
                );
                let sex_default = pick_preferred(sexes, "M");
                let equips = unique(
                    shard_keys.iter().filter_map(|k| {
                        parse_shard_key(k).and_then(|(s, e)| {
                            if s == sex_default {
                                Some(e.to_string())
                            } else {
                                None
                            }
                        })
                    }),
                );
                let equip_default = pick_preferred(equips, "Raw");
                set_sex.set(sex_default);
                set_equip.set(equip_default);
            }
        });
    }

    {
        let set_slice_rows = set_slice_rows;
        let set_load_error = set_load_error;
        Effect::new(move |_| {
            let latest_v = latest.get();
            let root = root_index.get();
            let s = sex.get();
            let e = equip.get();

            let (Some(latest_v), Some(root)) = (latest_v, root) else {
                return;
            };

            let shard_key = format!("sex={s}|equip={e}");
            let Some(shard_rel) = root.shards.get(&shard_key).cloned() else {
                set_slice_rows.set(Vec::new());
                return;
            };

            let set_slice_rows = set_slice_rows;
            let set_load_error = set_load_error;
            spawn_local(async move {
                let url_rel = format!("./data/{}/{}", latest_v.version, shard_rel);
                let url_abs = format!("/data/{}/{}", latest_v.version, shard_rel);
                let shard = fetch_json_first::<SliceIndex>(&[&url_rel, &url_abs]).await;
                let Ok(shard) = shard else {
                    if let Err(err) = shard {
                        set_load_error.set(Some(format!("Failed to load shard {shard_key}: {err}")));
                    }
                    set_slice_rows.set(Vec::new());
                    return;
                };
                let mut rows = Vec::new();
                match shard.slices {
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
                set_load_error.set(None);
                set_slice_rows.set(rows);
            });
        });
    }

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
        let set_hist = set_hist;
        let set_heat = set_heat;
        Effect::new(move |_| {
            let row = current_row.get();
            let latest_v = latest.get();

            if let (Some(row), Some(latest_v)) = (row, latest_v) {
                let hist_url = format!("./data/{}/{}", latest_v.version, row.entry.hist);
                let heat_url = format!("./data/{}/{}", latest_v.version, row.entry.heat);

                let set_hist = set_hist;
                let set_heat = set_heat;
                spawn_local(async move {
                    if let Ok(resp) = Request::get(&hist_url).send().await
                        && let Ok(bytes) = resp.binary().await
                    {
                        set_hist.set(parse_hist_bin(&bytes));
                    }

                    if let Ok(resp) = Request::get(&heat_url).send().await
                        && let Ok(bytes) = resp.binary().await
                    {
                        set_heat.set(parse_heat_bin(&bytes));
                    }
                });
            }
        });
    }

    let sex_options = Memo::new(move |_| {
        root_index
            .get()
            .map(|root| {
                unique(
                    root.shards
                        .keys()
                        .filter_map(|k| parse_shard_key(k).map(|(s, _)| s.to_string())),
                )
            })
            .unwrap_or_default()
    });
    let equip_options = Memo::new(move |_| {
        let s = sex.get();
        root_index
            .get()
            .map(|root| {
                unique(root.shards.keys().filter_map(|k| {
                    parse_shard_key(k).and_then(|(sx, eq)| if sx == s { Some(eq.to_string()) } else { None })
                }))
            })
            .unwrap_or_default()
    });
    let tested_options = Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let w = wc.get();
        let a = age.get();
        unique(
            slice_rows
                .get()
                .iter()
                .filter(|r| r.key.sex == s && r.key.equip == e && r.key.wc == w && r.key.age == a)
                .map(|r| r.key.tested.clone()),
        )
    });
    let wc_options = Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let mut classes = unique(
            slice_rows
                .get()
                .iter()
                .filter(|r| r.key.sex == s && r.key.equip == e)
                .map(|r| r.key.wc.clone()),
        );
        classes.sort_by_key(|c| ipf_class_sort_key(c));
        classes
    });
    let age_options = Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let w = wc.get();
        let mut classes = unique(
            slice_rows
                .get()
                .iter()
                .filter(|r| r.key.sex == s && r.key.equip == e && r.key.wc == w)
                .map(|r| r.key.age.clone()),
        );
        classes.sort_by_key(|c| age_class_sort_key(c));
        classes
    });
    let lift_options = Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let w = wc.get();
        let a = age.get();
        let t = tested.get();
        unique(
            slice_rows
                .get()
                .iter()
                .filter(|r| {
                    r.key.sex == s
                        && r.key.equip == e
                        && r.key.wc == w
                        && r.key.age == a
                        && r.key.tested == t
                })
                .map(|r| r.key.lift.clone()),
        )
    });
    let metric_options = Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let w = wc.get();
        let a = age.get();
        let t = tested.get();
        let l = lift.get();
        unique(
            slice_rows
                .get()
                .iter()
                .filter(|r| {
                    r.key.sex == s
                        && r.key.equip == e
                        && r.key.wc == w
                        && r.key.age == a
                        && r.key.tested == t
                        && r.key.lift == l
                })
                .map(|r| r.key.metric.clone()),
        )
    });

    {
        let set_equip = set_equip;
        Effect::new(move |_| {
            let options = equip_options.get();
            let current = equip.get();
            if options.is_empty() {
                return;
            }
            if !options.iter().any(|v| v == &current) {
                set_equip.set(pick_preferred(options, "Raw"));
            }
        });
    }

    {
        let set_wc = set_wc;
        Effect::new(move |_| {
            let options = wc_options.get();
            let current = wc.get();
            if options.is_empty() {
                return;
            }
            if !options.iter().any(|v| v == &current) {
                set_wc.set(pick_preferred(options, "All"));
            }
        });
    }

    {
        let set_age = set_age;
        Effect::new(move |_| {
            let options = age_options.get();
            let current = age.get();
            if options.is_empty() {
                return;
            }
            if !options.iter().any(|v| v == &current) {
                set_age.set(pick_preferred(options, "All Ages"));
            }
        });
    }

    {
        let set_tested = set_tested;
        Effect::new(move |_| {
            let options = tested_options.get();
            let current = tested.get();
            if options.is_empty() {
                return;
            }
            if !options.iter().any(|v| v == &current) {
                set_tested.set(pick_preferred(options, "All"));
            }
        });
    }

    {
        let set_lift = set_lift;
        Effect::new(move |_| {
            let options = lift_options.get();
            let current = lift.get();
            if options.is_empty() {
                return;
            }
            if !options.iter().any(|v| v == &current) {
                set_lift.set(pick_preferred(options, "T"));
            }
        });
    }
    {
        let set_metric = set_metric;
        Effect::new(move |_| {
            let options = metric_options.get();
            let current = metric.get();
            if options.is_empty() {
                return;
            }
            if !options.iter().any(|v| v == &current) {
                let preferred = if lift.get() == "T" { "Kg" } else { "Kg" };
                set_metric.set(pick_preferred(options, preferred));
            }
        });
    }

    let user_lift = Memo::new(move |_| {
        let total = squat.get() + bench.get() + deadlift.get();
        match (lift.get(), metric.get()) {
            (l, _) if l == "S" => squat.get(),
            (l, _) if l == "B" => bench.get(),
            (l, _) if l == "D" => deadlift.get(),
            (l, m) if l == "T" && m == "Dots" => dots_points(&sex.get(), bodyweight.get(), total),
            (l, m) if l == "T" && m == "Wilks" => wilks_points(&sex.get(), bodyweight.get(), total),
            (l, m) if l == "T" && m == "GL" => {
                goodlift_points(&sex.get(), &equip.get(), bodyweight.get(), total)
            }
            (l, _) if l == "T" => total,
            _ => 0.0,
        }
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
                        <input type="number" prop:value=move || squat.get().to_string() on:input=move |ev| set_squat.set(parse_f32_input(&ev)) />
                    </label>
                    <label>"Bench (kg)"
                        <input type="number" prop:value=move || bench.get().to_string() on:input=move |ev| set_bench.set(parse_f32_input(&ev)) />
                    </label>
                    <label>"Deadlift (kg)"
                        <input type="number" prop:value=move || deadlift.get().to_string() on:input=move |ev| set_deadlift.set(parse_f32_input(&ev)) />
                    </label>
                    <label>"Bodyweight (kg)"
                        <input type="number" prop:value=move || bodyweight.get().to_string() on:input=move |ev| set_bodyweight.set(parse_f32_input(&ev)) />
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
            </section>

            <section class="panel stats">
                <h2>"Percentile"</h2>
                <p>
                    {move || match percentile.get() {
                        Some((pct, rank, total)) => format!("{:.1}% percentile | rank ~{} / {}", pct * 100.0, rank, total),
                        None => "No distribution loaded for this slice.".to_string(),
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

fn unique(items: impl Iterator<Item = String>) -> Vec<String> {
    let mut set = BTreeSet::new();
    for item in items {
        set.insert(item);
    }
    set.into_iter().collect()
}

fn pick_preferred(options: Vec<String>, preferred: &str) -> String {
    if options.is_empty() {
        return String::new();
    }
    if let Some(v) = options.iter().find(|v| v.as_str() == preferred) {
        return v.clone();
    }
    options[0].clone()
}

fn parse_slice_key(raw: &str) -> Option<SliceKey> {
    let mut sex = None;
    let mut equip = None;
    let mut wc = None;
    let mut age = None;
    let mut tested = None;
    let mut lift = None;
    let mut metric = None;

    for part in raw.split('|') {
        let (k, v) = part.split_once('=')?;
        match k {
            "sex" => sex = Some(v.to_string()),
            "equip" => equip = Some(v.to_string()),
            "wc" => wc = Some(v.to_string()),
            "age" => age = Some(v.to_string()),
            "tested" => tested = Some(v.to_string()),
            "lift" => lift = Some(v.to_string()),
            "metric" => metric = Some(v.to_string()),
            _ => {}
        }
    }

    Some(SliceKey {
        sex: sex?,
        equip: equip?,
        wc: wc?,
        age: age?,
        tested: tested?,
        lift: lift?,
        metric: metric.unwrap_or_else(|| "Kg".to_string()),
    })
}

fn parse_shard_key(raw: &str) -> Option<(&str, &str)> {
    let mut sex = None;
    let mut equip = None;
    for part in raw.split('|') {
        let (k, v) = part.split_once('=')?;
        match k {
            "sex" => sex = Some(v),
            "equip" => equip = Some(v),
            _ => {}
        }
    }
    Some((sex?, equip?))
}

fn entry_from_slice_key(raw: &str) -> Option<(SliceKey, SliceIndexEntry)> {
    let key = parse_slice_key(raw)?;
    let sex_slug = slug(&key.sex);
    let equip_slug = slug(&key.equip);
    let wc_slug = slug(&key.wc);
    let age_slug = slug(&key.age);
    let lift_name = lift_name_from_code(&key.lift)?;
    let tested_dir = tested_dir_from_bucket(&key.tested);
    let has_metric = raw.split('|').any(|part| part.starts_with("metric="));
    let base = if has_metric {
        let metric_dir = slug(&key.metric);
        format!("{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested_dir}/{metric_dir}/{lift_name}")
    } else {
        format!("{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested_dir}/{lift_name}")
    };
    Some((
        key,
        SliceIndexEntry {
            meta: format!("meta/{base}.json"),
            hist: format!("hist/{base}.bin"),
            heat: format!("heat/{base}.bin"),
        },
    ))
}

fn slug(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' => c.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' | '-' => c,
            _ => '_',
        })
        .collect()
}

fn lift_name_from_code(code: &str) -> Option<&'static str> {
    match code {
        "S" => Some("squat"),
        "B" => Some("bench"),
        "D" => Some("deadlift"),
        "T" => Some("total"),
        _ => None,
    }
}

fn tested_dir_from_bucket(bucket: &str) -> String {
    if bucket.eq_ignore_ascii_case("yes") {
        "tested".to_string()
    } else {
        slug(bucket)
    }
}

fn ipf_class_sort_key(class: &str) -> (u8, i32) {
    if let Some(prefix) = class.strip_suffix('+') {
        if let Ok(v) = prefix.parse::<i32>() {
            return (1, v);
        }
    }
    if let Ok(v) = class.parse::<i32>() {
        return (0, v);
    }
    (2, i32::MAX)
}

fn parse_hist_bin(bytes: &[u8]) -> Option<HistogramBin> {
    if bytes.len() < 22 || &bytes[0..4] != b"IIH1" {
        return None;
    }

    let base = f32::from_le_bytes(bytes[6..10].try_into().ok()?);
    let min = f32::from_le_bytes(bytes[10..14].try_into().ok()?);
    let max = f32::from_le_bytes(bytes[14..18].try_into().ok()?);
    let bins = u32::from_le_bytes(bytes[18..22].try_into().ok()?) as usize;

    let payload = bytes.get(22..)?;
    if payload.len() != bins * 4 {
        return None;
    }

    let mut counts = Vec::with_capacity(bins);
    for chunk in payload.chunks_exact(4) {
        counts.push(u32::from_le_bytes(chunk.try_into().ok()?));
    }

    Some(HistogramBin {
        min,
        max,
        base_bin: base,
        counts,
    })
}

fn parse_heat_bin(bytes: &[u8]) -> Option<HeatmapBin> {
    if bytes.len() < 38 || &bytes[0..4] != b"IIM1" {
        return None;
    }

    let base_x = f32::from_le_bytes(bytes[6..10].try_into().ok()?);
    let base_y = f32::from_le_bytes(bytes[10..14].try_into().ok()?);
    let min_x = f32::from_le_bytes(bytes[14..18].try_into().ok()?);
    let max_x = f32::from_le_bytes(bytes[18..22].try_into().ok()?);
    let min_y = f32::from_le_bytes(bytes[22..26].try_into().ok()?);
    let max_y = f32::from_le_bytes(bytes[26..30].try_into().ok()?);
    let width = u32::from_le_bytes(bytes[30..34].try_into().ok()?) as usize;
    let height = u32::from_le_bytes(bytes[34..38].try_into().ok()?) as usize;

    let payload = bytes.get(38..)?;
    if payload.len() != width * height * 4 {
        return None;
    }

    let mut grid = Vec::with_capacity(width * height);
    for chunk in payload.chunks_exact(4) {
        grid.push(u32::from_le_bytes(chunk.try_into().ok()?));
    }

    Some(HeatmapBin {
        min_x,
        max_x,
        min_y,
        max_y,
        base_x,
        base_y,
        width,
        height,
        grid,
    })
}

fn percentile_for_value(hist: Option<&HistogramBin>, value: f32) -> Option<(f32, usize, u32)> {
    let hist = hist?;
    if hist.counts.is_empty() {
        return None;
    }

    let total: u32 = hist.counts.iter().copied().sum();
    if total == 0 {
        return None;
    }

    let bin_idx = ((value - hist.min) / hist.base_bin)
        .floor()
        .clamp(0.0, (hist.counts.len() - 1) as f32) as usize;

    let below: u32 = hist.counts.iter().take(bin_idx).copied().sum();
    let current = hist.counts[bin_idx] as f32;
    let cdf = below as f32 + 0.5 * current;
    let pct = cdf / total as f32;
    let rank = ((1.0 - pct) * total as f32).round().max(1.0) as usize;

    Some((pct, rank, total))
}

fn dots_points(sex: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let bw = match sex {
        "F" => bodyweight_kg.clamp(40.0, 150.0),
        _ => bodyweight_kg.clamp(40.0, 210.0),
    };
    let denom = if sex == "F" {
        -57.96288
            + 13.6175032 * bw
            - 0.1126655495 * bw.powi(2)
            + 0.0005158568 * bw.powi(3)
            - 0.0000010706 * bw.powi(4)
    } else {
        -307.75076
            + 24.0900756 * bw
            - 0.1918759221 * bw.powi(2)
            + 0.0007391293 * bw.powi(3)
            - 0.0000010930 * bw.powi(4)
    };
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 500.0 / denom
    }
}

fn wilks_points(sex: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let bw = match sex {
        "F" => bodyweight_kg.clamp(26.51, 154.53),
        _ => bodyweight_kg.clamp(40.0, 201.9),
    };
    let denom = if sex == "F" {
        594.31747775582
            - 27.23842536447 * bw
            + 0.82112226871 * bw.powi(2)
            - 0.00930733913 * bw.powi(3)
            + 0.00004731582 * bw.powi(4)
            - 0.00000009054 * bw.powi(5)
    } else {
        -216.0475144
            + 16.2606339 * bw
            - 0.002388645 * bw.powi(2)
            - 0.00113732 * bw.powi(3)
            + 0.00000701863 * bw.powi(4)
            - 0.00000001291 * bw.powi(5)
    };
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 500.0 / denom
    }
}

fn goodlift_points(sex: &str, equipment: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let classic = matches!(equipment, "Raw" | "Wraps" | "Straps");
    let (a, b, c) = match (sex, classic) {
        ("F", true) => (610.32796, 1045.59282, 0.03048),
        ("F", false) => (758.63878, 949.31382, 0.02435),
        ("M", true) => (1199.72839, 1025.18162, 0.00921),
        _ => (1236.25115, 1449.21864, 0.01644),
    };
    let denom = a - (b * (-c * bodyweight_kg).exp());
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 100.0 / denom
    }
}

fn rebin_1d(counts: Vec<u32>, k: usize) -> Vec<u32> {
    if k <= 1 {
        return counts;
    }
    counts
        .chunks(k)
        .map(|chunk| chunk.iter().copied().sum())
        .collect()
}

fn rebin_2d(
    grid: Vec<u32>,
    width: usize,
    height: usize,
    kx: usize,
    ky: usize,
) -> (Vec<u32>, usize, usize) {
    if kx <= 1 && ky <= 1 {
        return (grid, width, height);
    }

    let w2 = width.div_ceil(kx.max(1));
    let h2 = height.div_ceil(ky.max(1));
    let mut out = vec![0u32; w2 * h2];

    for y in 0..height {
        for x in 0..width {
            let src = y * width + x;
            let dst = (y / ky.max(1)) * w2 + (x / kx.max(1));
            out[dst] = out[dst].saturating_add(grid[src]);
        }
    }

    (out, w2, h2)
}

fn parse_f32_input(ev: &web_sys::Event) -> f32 {
    event_target::<HtmlInputElement>(ev)
        .value()
        .parse::<f32>()
        .unwrap_or(0.0)
}

fn lift_label(code: &str) -> &'static str {
    match code {
        "S" => "Squat",
        "B" => "Bench",
        "D" => "Deadlift",
        "T" => "Total",
        _ => "Unknown",
    }
}

fn metric_label(code: &str) -> &'static str {
    match code {
        "Kg" => "Kg",
        "Dots" => "DOTS",
        "Wilks" => "Wilks",
        "GL" => "GL",
        _ => "Kg",
    }
}

fn age_label(code: &str) -> String {
    match code {
        "All Ages" => "All Ages".to_string(),
        "5-12" => "Youth 5-12".to_string(),
        "13-15" => "Teen 13-15".to_string(),
        "16-17" => "Teen 16-17".to_string(),
        "18-19" => "Teen 18-19".to_string(),
        "20-23" => "Juniors 20-23".to_string(),
        "24-34" => "Seniors 24-34".to_string(),
        "35-39" => "Submasters 35-39".to_string(),
        _ => {
            if let Some((a, b)) = code.split_once('-') {
                format!("Masters {a}-{b}")
            } else if let Some(a) = code.strip_suffix('+') {
                format!("Masters {a}+")
            } else {
                code.to_string()
            }
        }
    }
}

fn age_class_sort_key(class: &str) -> (u8, i32) {
    if class == "All Ages" {
        return (0, -1);
    }
    let start = class
        .split(['-', '+'])
        .next()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(i32::MAX);
    (1, start)
}

fn render_histogram_svg(hist: &HistogramBin, user_value: f32, x_label: &str) -> AnyView {
    let max_count = hist.counts.iter().copied().max().unwrap_or(1) as f32;
    let w = 760.0f32;
    let h = 240.0f32;
    let left = 52.0f32;
    let right = 16.0f32;
    let top = 12.0f32;
    let bottom = 34.0f32;
    let plot_w = (w - left - right).max(1.0);
    let plot_h = (h - top - bottom).max(1.0);
    let bar_w = (plot_w / hist.counts.len().max(1) as f32).max(1.0);

    let marker_x =
        left + ((user_value - hist.min) / (hist.max - hist.min).max(0.0001)).clamp(0.0, 1.0) * plot_w;
    let bars: Vec<(usize, u32)> = hist.counts.iter().copied().enumerate().collect();
    let x_mid = (hist.min + hist.max) * 0.5;
    let y_tick_mid = (max_count * 0.5).round() as u32;

    view! {
        <svg class="hist" viewBox="0 0 760 240" preserveAspectRatio="none">
            <rect x="0" y="0" width={w.to_string()} height={h.to_string()} fill="#f7f5ef" />
            <line x1={left.to_string()} y1={(top + plot_h).to_string()} x2={(left + plot_w).to_string()} y2={(top + plot_h).to_string()} stroke="#8a8a84" stroke-width="1" />
            <line x1={left.to_string()} y1={top.to_string()} x2={left.to_string()} y2={(top + plot_h).to_string()} stroke="#8a8a84" stroke-width="1" />
            {bars
                .into_iter()
                .map(|(i, c)| {
                    let bh = (c as f32 / max_count) * plot_h;
                    let x = left + i as f32 * bar_w;
                    let y = top + plot_h - bh;
                    view! {
                        <rect
                            x={x.to_string()}
                            y={y.to_string()}
                            width={(bar_w - 1.0).max(0.5).to_string()}
                            height={bh.to_string()}
                            fill="#154734"
                        />
                    }
                })
                .collect_view()}
            <line x1={marker_x.to_string()} y1={top.to_string()} x2={marker_x.to_string()} y2={(top + plot_h).to_string()} stroke="#d6452b" stroke-width="3" />

            <text x={left.to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill="#4b4b44" text-anchor="middle">{format!("{:.0}", hist.min)}</text>
            <text x={(left + plot_w * 0.5).to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill="#4b4b44" text-anchor="middle">{format!("{:.0}", x_mid)}</text>
            <text x={(left + plot_w).to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill="#4b4b44" text-anchor="middle">{format!("{:.0}", hist.max)}</text>

            <text x={(left - 8.0).to_string()} y={(top + plot_h).to_string()} font-size="11" fill="#4b4b44" text-anchor="end">{ "0" }</text>
            <text x={(left - 8.0).to_string()} y={(top + plot_h * 0.5 + 4.0).to_string()} font-size="11" fill="#4b4b44" text-anchor="end">{y_tick_mid.to_string()}</text>
            <text x={(left - 8.0).to_string()} y={(top + 4.0).to_string()} font-size="11" fill="#4b4b44" text-anchor="end">{(max_count.round() as u32).to_string()}</text>

            <text x={(left + plot_w * 0.5).to_string()} y={(h - 4.0).to_string()} font-size="12" fill="#20342c" text-anchor="middle">{x_label.to_string()}</text>
            <text x="14" y={(top + plot_h * 0.5).to_string()} font-size="12" fill="#20342c" text-anchor="middle" transform={format!("rotate(-90,14,{})", top + plot_h * 0.5)}>"Lifter count"</text>

            <rect x={(w - 142.0).to_string()} y="10" width="132" height="34" rx="6" fill="#ffffff" stroke="#d5d2c7" />
            <rect x={(w - 132.0).to_string()} y="19" width="14" height="6" fill="#154734" />
            <text x={(w - 112.0).to_string()} y="25" font-size="11" fill="#20342c">"Distribution"</text>
            <line x1={(w - 132.0).to_string()} y1="34" x2={(w - 118.0).to_string()} y2="34" stroke="#d6452b" stroke-width="3" />
            <text x={(w - 112.0).to_string()} y="37" font-size="11" fill="#20342c">"Your value"</text>
        </svg>
    }
    .into_any()
}

fn draw_heatmap(
    canvas: &HtmlCanvasElement,
    heat: &HeatmapBin,
    user_lift: f32,
    user_bw: f32,
    x_label: &str,
) {
    let Ok(Some(ctx)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(ctx) = ctx.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let cw = canvas.width() as f64;
    let ch = canvas.height() as f64;
    let left = 58.0f64;
    let right = 96.0f64;
    let top = 18.0f64;
    let bottom = 44.0f64;
    let plot_w = (cw - left - right).max(1.0);
    let plot_h = (ch - top - bottom).max(1.0);

    ctx.set_fill_style_str("#fcfaf4");
    ctx.fill_rect(0.0, 0.0, cw, ch);

    if heat.width == 0 || heat.height == 0 || heat.grid.is_empty() {
        return;
    }

    let max_cell = heat.grid.iter().copied().max().unwrap_or(1) as f64;
    let cell_w = plot_w / heat.width as f64;
    let cell_h = plot_h / heat.height as f64;

    for y in 0..heat.height {
        for x in 0..heat.width {
            let idx = y * heat.width + x;
            let v = heat.grid[idx] as f64;
            if v <= 0.0 {
                continue;
            }
            let a = (v / max_cell).clamp(0.05, 1.0);
            let color = format!("rgba(11, 89, 160, {a})");
            ctx.set_fill_style_str(&color);
            ctx.fill_rect(
                left + x as f64 * cell_w,
                top + plot_h - ((y + 1) as f64 * cell_h),
                cell_w,
                cell_h,
            );
        }
    }

    let x = left
        + ((user_lift - heat.min_x) / (heat.max_x - heat.min_x).max(0.0001)).clamp(0.0, 1.0) as f64
            * plot_w;
    let y = top + plot_h
        - (((user_bw - heat.min_y) / (heat.max_y - heat.min_y).max(0.0001)).clamp(0.0, 1.0) as f64
            * plot_h);

    ctx.begin_path();
    ctx.set_fill_style_str("#d6452b");
    let _ = ctx.arc(x, y, 5.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();

    ctx.set_stroke_style_str("#8a8a84");
    ctx.begin_path();
    ctx.move_to(left, top + plot_h);
    ctx.line_to(left + plot_w, top + plot_h);
    ctx.move_to(left, top);
    ctx.line_to(left, top + plot_h);
    ctx.stroke();

    ctx.set_fill_style_str("#4b4b44");
    ctx.set_font("11px Space Grotesk, sans-serif");
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");
    let _ = ctx.fill_text(&format!("{:.0}", heat.min_x), left, top + plot_h + 8.0);
    let _ = ctx.fill_text(
        &format!("{:.0}", (heat.min_x + heat.max_x) * 0.5),
        left + plot_w * 0.5,
        top + plot_h + 8.0,
    );
    let _ = ctx.fill_text(&format!("{:.0}", heat.max_x), left + plot_w, top + plot_h + 8.0);

    ctx.set_text_align("right");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(&format!("{:.0}", heat.min_y), left - 8.0, top + plot_h);
    let _ = ctx.fill_text(
        &format!("{:.0}", (heat.min_y + heat.max_y) * 0.5),
        left - 8.0,
        top + plot_h * 0.5,
    );
    let _ = ctx.fill_text(&format!("{:.0}", heat.max_y), left - 8.0, top);

    ctx.set_fill_style_str("#20342c");
    ctx.set_font("12px Space Grotesk, sans-serif");
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");
    let _ = ctx.fill_text(x_label, left + plot_w * 0.5, ch - 18.0);

    let _ = ctx.save();
    let _ = ctx.translate(16.0, top + plot_h * 0.5);
    let _ = ctx.rotate(-std::f64::consts::FRAC_PI_2);
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");
    let _ = ctx.fill_text("Bodyweight (kg)", 0.0, 0.0);
    let _ = ctx.restore();

    let legend_x = left + plot_w + 22.0;
    let legend_y = top + 30.0;
    let legend_h = (plot_h - 50.0).max(40.0);
    let steps = 24usize;
    for i in 0..steps {
        let t0 = i as f64 / steps as f64;
        let t1 = (i + 1) as f64 / steps as f64;
        let alpha = 0.08 + (1.0 - t0) * (1.0 - 0.08);
        ctx.set_fill_style_str(&format!("rgba(11, 89, 160, {alpha})"));
        let y0 = legend_y + t0 * legend_h;
        let h0 = (t1 - t0) * legend_h;
        ctx.fill_rect(legend_x, y0, 14.0, h0 + 0.5);
    }
    ctx.set_stroke_style_str("#bdb8a7");
    ctx.stroke_rect(legend_x, legend_y, 14.0, legend_h);

    ctx.set_fill_style_str("#20342c");
    ctx.set_font("11px Space Grotesk, sans-serif");
    ctx.set_text_align("left");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text("Density", legend_x - 2.0, legend_y - 10.0);
    let _ = ctx.fill_text("High", legend_x + 20.0, legend_y + 2.0);
    let _ = ctx.fill_text("Low", legend_x + 20.0, legend_y + legend_h - 2.0);

    ctx.begin_path();
    ctx.set_fill_style_str("#d6452b");
    let _ = ctx.arc(legend_x + 7.0, legend_y + legend_h + 16.0, 4.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();
    ctx.set_fill_style_str("#20342c");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text("You", legend_x + 20.0, legend_y + legend_h + 16.0);
}

async fn fetch_json_first<T: for<'de> Deserialize<'de>>(urls: &[&str]) -> Result<T, String> {
    let mut errors = Vec::new();
    for url in urls {
        match Request::get(url).send().await {
            Ok(resp) if resp.ok() => match resp.json::<T>().await {
                Ok(value) => return Ok(value),
                Err(err) => errors.push(format!("{url}: parse error: {err}")),
            },
            Ok(resp) => errors.push(format!("{url}: http {}", resp.status())),
            Err(err) => errors.push(format!("{url}: request error: {err}")),
        }
    }
    Err(errors.join(" | "))
}
