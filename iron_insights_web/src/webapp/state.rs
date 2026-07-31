use super::{
    cross_sex::rows_from_slice_index,
    data::{fetch_binary_data_with_signal, fetch_json_data, fetch_json_data_with_signal},
    logging::debug_log,
    models::{LatestJson, RootIndex, SliceIndex, SliceRow, SliceSummary},
    ui::{pick_preferred, unique},
};
use crate::core::{HeatmapBin, HistogramBin, parse_combined_bin, parse_shard_key};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use leptos::{prelude::*, task::spawn_local};
use std::{cell::RefCell, collections::HashMap};
use web_sys::{AbortController, AbortSignal};

/// Grouped cohort selection signals + their option memos.
#[derive(Clone, Copy)]
pub(super) struct SelectionState {
    pub sex: ReadSignal<String>,
    pub set_sex: WriteSignal<String>,
    pub equip: ReadSignal<String>,
    pub set_equip: WriteSignal<String>,
    pub wc: ReadSignal<String>,
    pub set_wc: WriteSignal<String>,
    pub age: ReadSignal<String>,
    pub set_age: WriteSignal<String>,
    pub tested: ReadSignal<String>,
    pub set_tested: WriteSignal<String>,
    pub lift: ReadSignal<String>,
    pub set_lift: WriteSignal<String>,
    pub metric: ReadSignal<String>,
    pub set_metric: WriteSignal<String>,
    pub sex_opts: Memo<Vec<String>>,
    pub equip_opts: Memo<Vec<String>>,
    pub wc_opts: Memo<Vec<String>>,
    pub age_opts: Memo<Vec<String>>,
    pub tested_opts: Memo<Vec<String>>,
    pub lift_opts: Memo<Vec<String>>,
    pub metric_opts: Memo<Vec<String>>,
}

/// Grouped lifter inputs (squat/bench/deadlift/bw + units + errors + chart multipliers).
#[derive(Clone, Copy)]
pub(super) struct LifterInputState {
    pub squat: ReadSignal<f32>,
    pub set_squat: WriteSignal<f32>,
    pub squat_error: ReadSignal<Option<String>>,
    pub set_squat_error: WriteSignal<Option<String>>,
    pub bench: ReadSignal<f32>,
    pub set_bench: WriteSignal<f32>,
    pub bench_error: ReadSignal<Option<String>>,
    pub set_bench_error: WriteSignal<Option<String>>,
    pub deadlift: ReadSignal<f32>,
    pub set_deadlift: WriteSignal<f32>,
    pub deadlift_error: ReadSignal<Option<String>>,
    pub set_deadlift_error: WriteSignal<Option<String>>,
    pub bodyweight: ReadSignal<f32>,
    pub set_bodyweight: WriteSignal<f32>,
    pub bodyweight_error: ReadSignal<Option<String>>,
    pub set_bodyweight_error: WriteSignal<Option<String>>,
    pub use_lbs: ReadSignal<bool>,
    pub set_use_lbs: WriteSignal<bool>,
    pub set_lift_mult: WriteSignal<usize>,
    pub set_bw_mult: WriteSignal<usize>,
    pub has_input_error: Memo<bool>,
    pub unit_label: Memo<&'static str>,
}

/// Compute / derived state shared across pages (percentile, hist, blurbs).
#[derive(Clone, Copy)]
pub(super) struct ComputeState {
    pub calculated: ReadSignal<bool>,
    pub set_calculated: WriteSignal<bool>,
    pub reveal_tick: ReadSignal<u64>,
    pub set_reveal_tick: WriteSignal<u64>,
    pub user_lift: Memo<f32>,
    pub percentile: Memo<Option<(f32, usize, u32)>>,
    pub rank_tier: Memo<Option<&'static str>>,
    pub rebinned_hist: Memo<Option<HistogramBin>>,
    pub rebinned_heat: Memo<Option<HeatmapBin>>,
    pub hist_x_label: Memo<String>,
    pub chart_bodyweight: ReadSignal<f32>,
    pub load_error: ReadSignal<Option<String>>,
    pub dataset_blurb: Memo<String>,
    pub ranking_cohort_blurb: Memo<String>,
    pub slice_summary: Memo<Option<SliceSummary>>,
}

/// Single shared application state distributed via `provide_context`.
#[derive(Clone, Copy)]
pub(super) struct AppState {
    pub selection: SelectionState,
    pub input: LifterInputState,
    pub compute: ComputeState,
}

#[derive(Clone, Copy)]
pub(super) struct RequestTracker {
    pub(super) current: ReadSignal<u64>,
    pub(super) set: WriteSignal<u64>,
    pub(super) label: &'static str,
}

pub(super) struct RequestStart {
    pub(super) id: u64,
    pub(super) signal: AbortSignal,
}

thread_local! {
    static REQUEST_ABORT_CONTROLLERS: RefCell<HashMap<&'static str, AbortController>> =
        RefCell::new(HashMap::new());
}

impl RequestTracker {
    pub(super) fn begin(self) -> RequestStart {
        let next_id = self.current.get_untracked().wrapping_add(1);
        self.set.set(next_id);

        let controller = AbortController::new().expect("AbortController should exist in browsers");
        let signal = controller.signal();
        REQUEST_ABORT_CONTROLLERS.with(|controllers| {
            let mut controllers = controllers.borrow_mut();
            if let Some(previous) = controllers.insert(self.label, controller) {
                previous.abort();
            }
        });

        RequestStart {
            id: next_id,
            signal,
        }
    }

    pub(super) fn finish(self, id: u64) {
        if self.current.get_untracked() != id {
            return;
        }
        REQUEST_ABORT_CONTROLLERS.with(|controllers| {
            controllers.borrow_mut().remove(self.label);
        });
    }
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

#[allow(clippy::struct_field_names)]
pub(super) struct DistributionOutputs {
    pub(super) set_hist: WriteSignal<Option<HistogramBin>>,
    pub(super) set_heat: WriteSignal<Option<HeatmapBin>>,
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

pub(super) fn init_dataset_load(
    set_latest: WriteSignal<Option<LatestJson>>,
    set_root_index: WriteSignal<Option<RootIndex>>,
    set_sex: WriteSignal<String>,
    set_equip: WriteSignal<String>,
    set_load_error: WriteSignal<Option<String>>,
) {
    spawn_local(async move {
        let latest_json = fetch_json_data::<LatestJson>("latest.json").await;
        let Ok(latest_json) = latest_json else {
            if let Err(err) = latest_json {
                set_load_error.set(Some(format!(
                    "Failed to load latest dataset pointer (data/latest.json): {err}"
                )));
            }
            return;
        };
        set_latest.set(Some(latest_json.clone()));

        let index_path = format!("{}/index.json", latest_json.version);
        let index = fetch_json_data::<RootIndex>(&index_path).await;
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
            let sex_default = pick_preferred(&sexes, "M");
            let equips = unique(shard_keys.iter().filter_map(|k| {
                parse_shard_key(k).and_then(|(s, e)| {
                    if s == sex_default {
                        Some(e.to_string())
                    } else {
                        None
                    }
                })
            }));
            let equip_default = pick_preferred(&equips, "Raw");
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
        let request_start = request.begin();
        let next_request_id = request_start.id;

        let latest_v = latest.get();
        let root = root_index.get();
        let s = selection.sex.get();
        let e = selection.equip.get();

        let (Some(latest_v), Some(root)) = (latest_v, root) else {
            outputs.set_slice_rows.set(Vec::new());
            request.finish(next_request_id);
            return;
        };

        let shard_key = format!("sex={s}|equip={e}");
        let Some(shard_rel) = root.shards.get(&shard_key).cloned() else {
            outputs.set_slice_rows.set(Vec::new());
            request.finish(next_request_id);
            return;
        };

        let set_slice_rows = outputs.set_slice_rows;
        let set_load_error = outputs.set_load_error;
        let slice_request_id = request.current;
        let signal = request_start.signal;
        spawn_local(async move {
            let shard_path = format!("{}/{}", latest_v.version, shard_rel);
            let shard = fetch_json_data_with_signal::<SliceIndex>(&shard_path, Some(&signal)).await;
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
                request.finish(next_request_id);
                return;
            };
            set_load_error.set(None);
            set_slice_rows.set(rows_from_slice_index(shard));
            request.finish(next_request_id);
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
        let request_start = request.begin();
        let next_request_id = request_start.id;

        let row = current_row.get();
        let latest_v = latest.get();
        let should_load_hist = should_load_hist.get();
        let should_load_heat = should_load_heat.get();

        if !should_load_hist {
            outputs.set_hist.set(None);
            outputs.set_heat.set(None);
            request.finish(next_request_id);
            return;
        }

        if !should_load_heat {
            outputs.set_heat.set(None);
        }

        if let (Some(row), Some(latest_v)) = (row, latest_v) {
            let set_hist = outputs.set_hist;
            let set_heat = outputs.set_heat;
            let set_load_error = outputs.set_load_error;
            let dist_request_id = request.current;
            let signal = request_start.signal;
            set_hist.set(None);
            if should_load_heat {
                set_heat.set(None);
            }

            // Fast path: payload is inlined as base64 — no network fetch needed.
            if row.entry.inline.is_empty() {
                let bin_path = format!("{}/{}", latest_v.version, row.entry.bin);
                spawn_local(async move {
                    if dist_request_id.get_untracked() != next_request_id {
                        debug_log(&format!(
                            "Ignored stale distribution response for request {next_request_id}"
                        ));
                        return;
                    }

                    if let Ok(bytes) = fetch_binary_data_with_signal(&bin_path, Some(&signal)).await
                    {
                        if dist_request_id.get_untracked() != next_request_id {
                            debug_log(&format!(
                                "Ignored stale combined payload for request {next_request_id}"
                            ));
                            return;
                        }
                        match parse_combined_bin(&bytes) {
                            Some((hist, heat)) => {
                                set_hist.set(Some(hist));
                                if should_load_heat {
                                    set_heat.set(Some(heat));
                                }
                            }
                            None => {
                                set_load_error.set(Some(format!(
                                    "Invalid or unsupported combined binary format: {bin_path}"
                                )));
                            }
                        }
                        request.finish(next_request_id);
                    } else {
                        if dist_request_id.get_untracked() != next_request_id {
                            debug_log(&format!(
                                "Ignored stale combined error for request {next_request_id}"
                            ));
                            return;
                        }
                        set_hist.set(None);
                        set_load_error.set(Some(format!("Failed to fetch data: {bin_path}")));
                        request.finish(next_request_id);
                    }
                });
            } else {
                match BASE64.decode(&row.entry.inline) {
                    Ok(bytes) => match parse_combined_bin(&bytes) {
                        Some((hist, heat)) => {
                            set_hist.set(Some(hist));
                            if should_load_heat {
                                set_heat.set(Some(heat));
                            }
                        }
                        None => {
                            set_load_error.set(Some("Invalid inlined binary payload.".to_string()));
                        }
                    },
                    Err(_) => set_load_error
                        .set(Some("Failed to decode inlined binary payload.".to_string())),
                }
                request.finish(next_request_id);
            }
        } else {
            outputs.set_hist.set(None);
            outputs.set_heat.set(None);
            request.finish(next_request_id);
        }
    });
}

/// The cohort summary ships inline in the slice index, so it needs no fetch.
pub(super) fn slice_summary(current_row: Memo<Option<SliceRow>>) -> Memo<Option<SliceSummary>> {
    Memo::new(move |_| current_row.get().and_then(|row| row.entry.summary))
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
            set_current.set(pick_preferred(&values, preferred));
        }
    });
}

/// Keeps every cohort dropdown on a valid value: when the options change and the
/// current pick is gone, fall back to the preferred default (or the first option).
pub(super) fn setup_default_selection_effects(sel: SelectionState) {
    for (options, current, set_current, preferred) in [
        (sel.equip_opts, sel.equip, sel.set_equip, "Raw"),
        (sel.wc_opts, sel.wc, sel.set_wc, "All"),
        (sel.age_opts, sel.age, sel.set_age, "All Ages"),
        (sel.tested_opts, sel.tested, sel.set_tested, "All"),
        (sel.lift_opts, sel.lift, sel.set_lift, "T"),
        (sel.metric_opts, sel.metric, sel.set_metric, "Kg"),
    ] {
        setup_preferred_selection_effect(options, current, set_current, preferred);
    }
}
