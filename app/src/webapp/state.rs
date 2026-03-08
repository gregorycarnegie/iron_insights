use super::data::{fetch_binary_first, fetch_json_first};
use super::debug_log;
use super::models::{
    LatestJson, RootIndex, SliceIndex, SliceIndexEntries, SliceMetaJson, SliceRow, SliceSummary,
    TrendSeries, TrendsJson,
};
use super::slices::{entry_from_slice_key, parse_shard_key, parse_slice_key};
use super::ui::{pick_preferred, unique};
use crate::core::{HeatmapBin, HistogramBin, parse_heat_bin, parse_hist_bin};
use leptos::prelude::*;
use leptos::task::spawn_local;

fn data_url(path_suffix: &str) -> String {
    let trimmed = path_suffix.trim_start_matches('/');
    // Use a relative path so GitHub Pages project sites resolve to
    // /<repo>/data/... instead of the domain root /data/...
    format!("data/{trimmed}")
}

pub(super) fn init_dataset_load(
    set_latest: WriteSignal<Option<LatestJson>>,
    set_root_index: WriteSignal<Option<RootIndex>>,
    set_sex: WriteSignal<String>,
    set_equip: WriteSignal<String>,
    set_load_error: WriteSignal<Option<String>>,
) {
    spawn_local(async move {
        let latest_url = data_url("latest.json");
        let latest_json = fetch_json_first::<LatestJson>(&[&latest_url]).await;
        let Ok(latest_json) = latest_json else {
            set_load_error.set(Some(
                "Failed to load latest dataset pointer (data/latest.json).".to_string(),
            ));
            return;
        };
        set_latest.set(Some(latest_json.clone()));

        let index_url = data_url(&format!("{}/index.json", latest_json.version));
        let index = fetch_json_first::<RootIndex>(&[&index_url]).await;
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
            let equips = unique(shard_keys.iter().filter_map(|k| {
                parse_shard_key(k).and_then(|(s, e)| {
                    if s == sex_default {
                        Some(e.to_string())
                    } else {
                        None
                    }
                })
            }));
            let equip_default = pick_preferred(equips, "Raw");
            set_sex.set(sex_default);
            set_equip.set(equip_default);
        }
    });
}

pub(super) fn setup_slice_rows_effect(
    latest: ReadSignal<Option<LatestJson>>,
    root_index: ReadSignal<Option<RootIndex>>,
    sex: ReadSignal<String>,
    equip: ReadSignal<String>,
    set_slice_rows: WriteSignal<Vec<SliceRow>>,
    set_load_error: WriteSignal<Option<String>>,
    slice_request_id: ReadSignal<u64>,
    set_slice_request_id: WriteSignal<u64>,
) {
    Effect::new(move |_| {
        let next_request_id = slice_request_id.get_untracked().wrapping_add(1);
        set_slice_request_id.set(next_request_id);

        let latest_v = latest.get();
        let root = root_index.get();
        let s = sex.get();
        let e = equip.get();

        let (Some(latest_v), Some(root)) = (latest_v, root) else {
            set_slice_rows.set(Vec::new());
            return;
        };

        let shard_key = format!("sex={s}|equip={e}");
        let Some(shard_rel) = root.shards.get(&shard_key).cloned() else {
            set_slice_rows.set(Vec::new());
            return;
        };

        let set_slice_rows = set_slice_rows;
        let set_load_error = set_load_error;
        let slice_request_id = slice_request_id;
        spawn_local(async move {
            let shard_url = data_url(&format!("{}/{}", latest_v.version, shard_rel));
            let shard = fetch_json_first::<SliceIndex>(&[&shard_url]).await;
            if slice_request_id.get_untracked() != next_request_id {
                debug_log(&format!(
                    "Ignored stale shard response for request {next_request_id}"
                ));
                return;
            }
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

pub(super) fn setup_trends_effect(
    latest: ReadSignal<Option<LatestJson>>,
    set_trends: WriteSignal<Vec<TrendSeries>>,
    trends_request_id: ReadSignal<u64>,
    set_trends_request_id: WriteSignal<u64>,
) {
    Effect::new(move |_| {
        let next_request_id = trends_request_id.get_untracked().wrapping_add(1);
        set_trends_request_id.set(next_request_id);

        let latest_v = latest.get();
        let Some(latest_v) = latest_v else {
            set_trends.set(Vec::new());
            return;
        };

        let set_trends = set_trends;
        let trends_request_id = trends_request_id;
        spawn_local(async move {
            let trends_url = data_url(&format!("{}/trends.json", latest_v.version));
            match fetch_json_first::<TrendsJson>(&[&trends_url]).await {
                Ok(payload) => {
                    if trends_request_id.get_untracked() != next_request_id {
                        debug_log(&format!(
                            "Ignored stale trends response for request {next_request_id}"
                        ));
                        return;
                    }
                    if payload.bucket == "year" {
                        set_trends.set(payload.series);
                    } else {
                        set_trends.set(Vec::new());
                    }
                }
                Err(_err) => {
                    if trends_request_id.get_untracked() != next_request_id {
                        return;
                    }
                    // Older data versions may not include trends.json yet.
                    set_trends.set(Vec::new());
                }
            }
        });
    });
}

pub(super) fn setup_distribution_effect(
    current_row: Memo<Option<SliceRow>>,
    latest: ReadSignal<Option<LatestJson>>,
    calculated: ReadSignal<bool>,
    show_main_charts: ReadSignal<bool>,
    set_hist: WriteSignal<Option<HistogramBin>>,
    set_heat: WriteSignal<Option<HeatmapBin>>,
    set_hist_load_ms: WriteSignal<Option<u32>>,
    set_heat_load_ms: WriteSignal<Option<u32>>,
    set_load_error: WriteSignal<Option<String>>,
    dist_request_id: ReadSignal<u64>,
    set_dist_request_id: WriteSignal<u64>,
) {
    Effect::new(move |_| {
        let next_request_id = dist_request_id.get_untracked().wrapping_add(1);
        set_dist_request_id.set(next_request_id);

        let row = current_row.get();
        let latest_v = latest.get();
        let should_load_hist = calculated.get();
        let should_load_heat = show_main_charts.get();

        if !should_load_hist {
            set_hist.set(None);
            set_heat.set(None);
            set_hist_load_ms.set(None);
            set_heat_load_ms.set(None);
            return;
        }

        if let (Some(row), Some(latest_v)) = (row, latest_v) {
            let hist_url = data_url(&format!("{}/{}", latest_v.version, row.entry.hist));
            let heat_url = data_url(&format!("{}/{}", latest_v.version, row.entry.heat));

            let set_hist = set_hist;
            let set_heat = set_heat;
            let set_load_error = set_load_error;
            let dist_request_id = dist_request_id;
            let hist_err = hist_url.clone();
            let heat_err = heat_url.clone();
            set_hist.set(None);
            set_hist_load_ms.set(None);
            if should_load_heat {
                set_heat.set(None);
                set_heat_load_ms.set(None);
            }
            spawn_local(async move {
                if dist_request_id.get_untracked() != next_request_id {
                    debug_log(&format!(
                        "Ignored stale distribution response for request {next_request_id}"
                    ));
                    return;
                }

                if let Ok(bytes) = fetch_binary_first(&[&hist_url]).await {
                    if dist_request_id.get_untracked() != next_request_id {
                        debug_log(&format!(
                            "Ignored stale histogram payload for request {next_request_id}"
                        ));
                        return;
                    }
                    let parsed = parse_hist_bin(&bytes);
                    if parsed.is_none() {
                        set_load_error.set(Some(format!(
                            "Invalid or unsupported histogram binary format: {hist_err}"
                        )));
                    }
                    set_hist.set(parsed);
                    set_hist_load_ms.set(Some(0));
                } else {
                    if dist_request_id.get_untracked() != next_request_id {
                        debug_log(&format!(
                            "Ignored stale histogram error for request {next_request_id}"
                        ));
                        return;
                    }
                    set_hist.set(None);
                    set_load_error.set(Some(format!("Failed to fetch histogram data: {hist_err}")));
                }

                if should_load_heat {
                    if let Ok(bytes) = fetch_binary_first(&[&heat_url]).await {
                        if dist_request_id.get_untracked() != next_request_id {
                            debug_log(&format!(
                                "Ignored stale heatmap payload for request {next_request_id}"
                            ));
                            return;
                        }
                        let parsed = parse_heat_bin(&bytes);
                        if parsed.is_none() {
                            set_load_error.set(Some(format!(
                                "Invalid or unsupported heatmap binary format: {heat_err}"
                            )));
                        }
                        set_heat.set(parsed);
                        set_heat_load_ms.set(Some(0));
                    } else {
                        if dist_request_id.get_untracked() != next_request_id {
                            debug_log(&format!(
                                "Ignored stale heatmap error for request {next_request_id}"
                            ));
                            return;
                        }
                        set_heat.set(None);
                        set_load_error
                            .set(Some(format!("Failed to fetch heatmap data: {heat_err}")));
                    }
                }
            });
        } else {
            set_hist.set(None);
            set_heat.set(None);
            set_hist_load_ms.set(None);
            set_heat_load_ms.set(None);
        }
    });
}

pub(super) fn setup_slice_summary_effect(
    current_row: Memo<Option<SliceRow>>,
    latest: ReadSignal<Option<LatestJson>>,
    set_summary: WriteSignal<Option<SliceSummary>>,
    set_summary_load_ms: WriteSignal<Option<u32>>,
    set_load_error: WriteSignal<Option<String>>,
    summary_request_id: ReadSignal<u64>,
    set_summary_request_id: WriteSignal<u64>,
) {
    Effect::new(move |_| {
        let next_request_id = summary_request_id.get_untracked().wrapping_add(1);
        set_summary_request_id.set(next_request_id);

        let row = current_row.get();
        let latest_v = latest.get();
        let (Some(row), Some(latest_v)) = (row, latest_v) else {
            set_summary.set(None);
            set_summary_load_ms.set(None);
            return;
        };

        if let Some(summary) = row.entry.summary.clone() {
            set_summary.set(Some(summary));
            set_summary_load_ms.set(Some(0));
            return;
        }

        if row.entry.meta.trim().is_empty() {
            set_summary.set(None);
            set_summary_load_ms.set(None);
            return;
        }

        let meta_url = data_url(&format!("{}/{}", latest_v.version, row.entry.meta));
        let meta_err = meta_url.clone();
        let summary_request_id = summary_request_id;
        let set_summary = set_summary;
        let set_summary_load_ms = set_summary_load_ms;
        let set_load_error = set_load_error;
        set_summary.set(None);
        set_summary_load_ms.set(None);

        spawn_local(async move {
            let meta = fetch_json_first::<SliceMetaJson>(&[&meta_url]).await;
            if summary_request_id.get_untracked() != next_request_id {
                debug_log(&format!(
                    "Ignored stale summary response for request {next_request_id}"
                ));
                return;
            }
            match meta {
                Ok(meta) => {
                    set_summary.set(Some(SliceSummary {
                        min_kg: meta.hist.min_kg,
                        max_kg: meta.hist.max_kg,
                        total: meta.hist.total,
                    }));
                    set_summary_load_ms.set(Some(0));
                }
                Err(err) => {
                    set_summary.set(None);
                    set_load_error.set(Some(format!(
                        "Failed to load slice summary {meta_err}: {err}"
                    )));
                }
            }
        });
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn setup_default_selection_effects(
    equip_options: Memo<Vec<String>>,
    wc_options: Memo<Vec<String>>,
    age_options: Memo<Vec<String>>,
    tested_options: Memo<Vec<String>>,
    lift_options: Memo<Vec<String>>,
    metric_options: Memo<Vec<String>>,
    equip: ReadSignal<String>,
    wc: ReadSignal<String>,
    age: ReadSignal<String>,
    tested: ReadSignal<String>,
    lift: ReadSignal<String>,
    metric: ReadSignal<String>,
    set_equip: WriteSignal<String>,
    set_wc: WriteSignal<String>,
    set_age: WriteSignal<String>,
    set_tested: WriteSignal<String>,
    set_lift: WriteSignal<String>,
    set_metric: WriteSignal<String>,
) {
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
}
