use super::data::{fetch_binary_first, fetch_json_first};
use super::debug_log;
use super::models::{
    LatestJson, RootIndex, SliceIndex, SliceIndexEntries, SliceMetaJson, SliceRow, SliceSummary,
    TrendSeries, TrendsJson,
};
use super::slices::{entry_from_slice_key, parse_shard_key, parse_slice_key};
use super::ui::{pick_preferred, unique};
use crate::core::{HeatmapBin, HistogramBin, parse_combined_bin};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use leptos::prelude::*;
use leptos::task::spawn_local;

pub(super) struct DefaultSelectionOptions {
    pub(super) equip: Memo<Vec<String>>,
    pub(super) wc: Memo<Vec<String>>,
    pub(super) age: Memo<Vec<String>>,
    pub(super) tested: Memo<Vec<String>>,
    pub(super) lift: Memo<Vec<String>>,
    pub(super) metric: Memo<Vec<String>>,
}

pub(super) struct DefaultSelectionSignals {
    pub(super) equip: ReadSignal<String>,
    pub(super) wc: ReadSignal<String>,
    pub(super) age: ReadSignal<String>,
    pub(super) tested: ReadSignal<String>,
    pub(super) lift: ReadSignal<String>,
    pub(super) metric: ReadSignal<String>,
}

pub(super) struct DefaultSelectionSetters {
    pub(super) equip: WriteSignal<String>,
    pub(super) wc: WriteSignal<String>,
    pub(super) age: WriteSignal<String>,
    pub(super) tested: WriteSignal<String>,
    pub(super) lift: WriteSignal<String>,
    pub(super) metric: WriteSignal<String>,
}

#[derive(Clone, Copy)]
pub(super) struct RequestTracker {
    pub(super) current: ReadSignal<u64>,
    pub(super) set: WriteSignal<u64>,
}

pub(super) struct SliceRowsSelection {
    pub(super) sex: ReadSignal<String>,
    pub(super) equip: ReadSignal<String>,
}

pub(super) struct SliceRowsOutputs {
    pub(super) set_slice_rows: WriteSignal<Vec<SliceRow>>,
    pub(super) set_load_error: WriteSignal<Option<String>>,
}

pub(super) struct SliceRowsEffectContext {
    pub(super) latest: ReadSignal<Option<LatestJson>>,
    pub(super) root_index: ReadSignal<Option<RootIndex>>,
    pub(super) selection: SliceRowsSelection,
    pub(super) outputs: SliceRowsOutputs,
    pub(super) request: RequestTracker,
}

pub(super) struct DistributionOutputs {
    pub(super) set_hist: WriteSignal<Option<HistogramBin>>,
    pub(super) set_heat: WriteSignal<Option<HeatmapBin>>,
    pub(super) set_hist_load_ms: WriteSignal<Option<u32>>,
    pub(super) set_heat_load_ms: WriteSignal<Option<u32>>,
    pub(super) set_load_error: WriteSignal<Option<String>>,
}

pub(super) struct DistributionEffectContext {
    pub(super) current_row: Memo<Option<SliceRow>>,
    pub(super) latest: ReadSignal<Option<LatestJson>>,
    pub(super) should_load_hist: ReadSignal<bool>,
    pub(super) should_load_heat: Memo<bool>,
    pub(super) outputs: DistributionOutputs,
    pub(super) request: RequestTracker,
}

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

pub(super) fn setup_slice_rows_effect(context: SliceRowsEffectContext) {
    let SliceRowsEffectContext {
        latest,
        root_index,
        selection,
        outputs,
        request,
    } = context;
    Effect::new(move |_| {
        let next_request_id = request.current.get_untracked().wrapping_add(1);
        request.set.set(next_request_id);

        let latest_v = latest.get();
        let root = root_index.get();
        let s = selection.sex.get();
        let e = selection.equip.get();

        let (Some(latest_v), Some(root)) = (latest_v, root) else {
            outputs.set_slice_rows.set(Vec::new());
            return;
        };

        let shard_key = format!("sex={s}|equip={e}");
        let Some(shard_rel) = root.shards.get(&shard_key).cloned() else {
            outputs.set_slice_rows.set(Vec::new());
            return;
        };

        let set_slice_rows = outputs.set_slice_rows;
        let set_load_error = outputs.set_load_error;
        let slice_request_id = request.current;
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
    root_index: ReadSignal<Option<RootIndex>>,
    sex: ReadSignal<String>,
    equip: ReadSignal<String>,
    should_load_trends: Memo<bool>,
    set_trends: WriteSignal<Vec<TrendSeries>>,
    trends_request_id: ReadSignal<u64>,
    set_trends_request_id: WriteSignal<u64>,
) {
    Effect::new(move |_| {
        let next_request_id = trends_request_id.get_untracked().wrapping_add(1);
        set_trends_request_id.set(next_request_id);

        let latest_v = latest.get();
        let root = root_index.get();
        let load_trends = should_load_trends.get();
        let shard_key = format!("sex={}|equip={}", sex.get(), equip.get());

        if !load_trends {
            set_trends.set(Vec::new());
            return;
        }

        let (Some(latest_v), Some(root)) = (latest_v, root) else {
            set_trends.set(Vec::new());
            return;
        };

        let Some(shard_rel) = root.trends_shards.get(&shard_key).cloned() else {
            set_trends.set(Vec::new());
            return;
        };

        let set_trends = set_trends;
        let trends_request_id = trends_request_id;
        spawn_local(async move {
            let trends_url = data_url(&format!("{}/{}", latest_v.version, shard_rel));
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
                    set_trends.set(Vec::new());
                }
            }
        });
    });
}

pub(super) fn setup_distribution_effect(context: DistributionEffectContext) {
    let DistributionEffectContext {
        current_row,
        latest,
        should_load_hist,
        should_load_heat,
        outputs,
        request,
    } = context;
    Effect::new(move |_| {
        let next_request_id = request.current.get_untracked().wrapping_add(1);
        request.set.set(next_request_id);

        let row = current_row.get();
        let latest_v = latest.get();
        let should_load_hist = should_load_hist.get();
        let should_load_heat = should_load_heat.get();

        if !should_load_hist {
            outputs.set_hist.set(None);
            outputs.set_heat.set(None);
            outputs.set_hist_load_ms.set(None);
            outputs.set_heat_load_ms.set(None);
            return;
        }

        if !should_load_heat {
            outputs.set_heat.set(None);
            outputs.set_heat_load_ms.set(None);
        }

        if let (Some(row), Some(latest_v)) = (row, latest_v) {
            let set_hist = outputs.set_hist;
            let set_heat = outputs.set_heat;
            let set_hist_load_ms = outputs.set_hist_load_ms;
            let set_heat_load_ms = outputs.set_heat_load_ms;
            let set_load_error = outputs.set_load_error;
            let dist_request_id = request.current;
            set_hist.set(None);
            set_hist_load_ms.set(None);
            if should_load_heat {
                set_heat.set(None);
                set_heat_load_ms.set(None);
            }

            // Fast path: payload is inlined as base64 — no network fetch needed.
            if !row.entry.inline.is_empty() {
                match BASE64.decode(&row.entry.inline) {
                    Ok(bytes) => match parse_combined_bin(&bytes) {
                        Some((hist, heat)) => {
                            set_hist.set(Some(hist));
                            set_hist_load_ms.set(Some(0));
                            if should_load_heat {
                                set_heat.set(Some(heat));
                                set_heat_load_ms.set(Some(0));
                            }
                        }
                        None => {
                            set_load_error.set(Some("Invalid inlined binary payload.".to_string()))
                        }
                    },
                    Err(_) => set_load_error
                        .set(Some("Failed to decode inlined binary payload.".to_string())),
                }
            } else {
                let bin_url = data_url(&format!("{}/{}", latest_v.version, row.entry.bin));
                spawn_local(async move {
                    if dist_request_id.get_untracked() != next_request_id {
                        debug_log(&format!(
                            "Ignored stale distribution response for request {next_request_id}"
                        ));
                        return;
                    }

                    match fetch_binary_first(&[&bin_url]).await {
                        Ok(bytes) => {
                            if dist_request_id.get_untracked() != next_request_id {
                                debug_log(&format!(
                                    "Ignored stale combined payload for request {next_request_id}"
                                ));
                                return;
                            }
                            match parse_combined_bin(&bytes) {
                                Some((hist, heat)) => {
                                    set_hist.set(Some(hist));
                                    set_hist_load_ms.set(Some(0));
                                    if should_load_heat {
                                        set_heat.set(Some(heat));
                                        set_heat_load_ms.set(Some(0));
                                    }
                                }
                                None => {
                                    set_load_error.set(Some(format!(
                                        "Invalid or unsupported combined binary format: {bin_url}"
                                    )));
                                }
                            }
                        }
                        Err(_) => {
                            if dist_request_id.get_untracked() != next_request_id {
                                debug_log(&format!(
                                    "Ignored stale combined error for request {next_request_id}"
                                ));
                                return;
                            }
                            set_hist.set(None);
                            set_load_error.set(Some(format!("Failed to fetch data: {bin_url}")));
                        }
                    }
                });
            }
        } else {
            outputs.set_hist.set(None);
            outputs.set_heat.set(None);
            outputs.set_hist_load_ms.set(None);
            outputs.set_heat_load_ms.set(None);
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

fn setup_preferred_selection_effect(
    options: Memo<Vec<String>>,
    current: ReadSignal<String>,
    set_current: WriteSignal<String>,
    preferred: &'static str,
) {
    Effect::new(move |_| {
        let values = options.get();
        if values.is_empty() {
            return;
        }

        let selected = current.get();
        if !values.iter().any(|value| value == &selected) {
            set_current.set(pick_preferred(values, preferred));
        }
    });
}

pub(super) fn setup_default_selection_effects(
    options: DefaultSelectionOptions,
    current: DefaultSelectionSignals,
    setters: DefaultSelectionSetters,
) {
    setup_preferred_selection_effect(options.equip, current.equip, setters.equip, "Raw");
    setup_preferred_selection_effect(options.wc, current.wc, setters.wc, "All");
    setup_preferred_selection_effect(options.age, current.age, setters.age, "All Ages");
    setup_preferred_selection_effect(options.tested, current.tested, setters.tested, "All");
    setup_preferred_selection_effect(options.lift, current.lift, setters.lift, "T");
    setup_preferred_selection_effect(options.metric, current.metric, setters.metric, "Kg");
}
