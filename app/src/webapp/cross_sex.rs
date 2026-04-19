use super::data::{fetch_binary_first, fetch_json_first};
use super::helpers::{ComparableLifter, comparable_lift_value};
use super::models::{
    CrossSexComparison, CrossSexLiftComparison, LatestJson, RootIndex, SliceIndex,
    SliceIndexEntries, SliceRow,
};
use super::slices::{entry_from_slice_key, parse_slice_key};
use super::state::RequestTracker;
use crate::core::{
    HeatmapBin, HistogramBin, equivalent_value_for_same_percentile, parse_combined_bin,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::cell::RefCell;
use std::collections::HashMap;

pub(super) const MIN_CROSS_SEX_COHORT_TOTAL: u32 = 50;
pub(super) const CROSS_SEX_LIFT_ROWS: [(&str, &str); 3] =
    [("S", "Squat"), ("B", "Bench"), ("D", "Deadlift")];

thread_local! {
    static ROWS_FROM_SLICE_INDEX_CACHE: RefCell<HashMap<String, Vec<SliceRow>>> =
        RefCell::new(HashMap::new());
}

#[derive(Clone, PartialEq)]
pub(super) struct CrossSexSliceChoice {
    pub(super) row: SliceRow,
    pub(super) weight_class_fallback: bool,
}

pub(super) fn rows_from_slice_index(index: SliceIndex) -> Vec<SliceRow> {
    let cache_key = slice_index_cache_key(&index);
    if let Some(rows) =
        ROWS_FROM_SLICE_INDEX_CACHE.with(|cache| cache.borrow().get(&cache_key).cloned())
    {
        return rows;
    }

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
    ROWS_FROM_SLICE_INDEX_CACHE.with(|cache| {
        cache.borrow_mut().insert(cache_key, rows.clone());
    });
    rows
}

fn slice_index_cache_key(index: &SliceIndex) -> String {
    match &index.slices {
        SliceIndexEntries::Map(entries) => {
            let mut key = String::from("map:");
            for (raw_key, entry) in entries {
                key.push_str(raw_key);
                key.push('\u{1e}');
                key.push_str(&entry.meta);
                key.push('\u{1e}');
                key.push_str(&entry.bin);
                key.push('\u{1e}');
                key.push_str(&entry.inline);
                if let Some(summary) = &entry.summary {
                    key.push('\u{1e}');
                    key.push_str(&summary.min_kg.to_string());
                    key.push('\u{1e}');
                    key.push_str(&summary.max_kg.to_string());
                    key.push('\u{1e}');
                    key.push_str(&summary.total.to_string());
                }
                key.push('\u{1f}');
            }
            key
        }
        SliceIndexEntries::Keys(keys) => {
            let mut key = String::from("keys:");
            for raw_key in keys {
                key.push_str(raw_key);
                key.push('\u{1f}');
            }
            key
        }
    }
}

pub(super) fn choose_cross_sex_slice(
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

fn histogram_weighted_mean(hist: &HistogramBin) -> Option<(f32, u32)> {
    if hist.counts.is_empty() || hist.base_bin <= 0.0 {
        return None;
    }
    let mut total = 0u32;
    let mut weighted_sum = 0.0f64;
    for (idx, count) in hist.counts.iter().copied().enumerate() {
        if count == 0 {
            continue;
        }
        let center = f64::from(hist.min) + (idx as f64 + 0.5) * f64::from(hist.base_bin);
        total = total.saturating_add(count);
        weighted_sum += center * f64::from(count);
    }
    (total > 0).then_some(((weighted_sum / f64::from(total)) as f32, total))
}

fn heatmap_mean_lift_bodyweight_ratio(heat: &HeatmapBin) -> Option<f32> {
    if heat.width == 0
        || heat.height == 0
        || heat.grid.len() != heat.width * heat.height
        || heat.base_x <= 0.0
        || heat.base_y <= 0.0
    {
        return None;
    }
    let mut total = 0u32;
    let mut weighted_ratio = 0.0f64;
    for y in 0..heat.height {
        let bodyweight = f64::from(heat.min_y) + (y as f64 + 0.5) * f64::from(heat.base_y);
        if bodyweight <= 0.0 {
            continue;
        }
        for x in 0..heat.width {
            let count = heat.grid[y * heat.width + x];
            if count == 0 {
                continue;
            }
            let lift = f64::from(heat.min_x) + (x as f64 + 0.5) * f64::from(heat.base_x);
            total = total.saturating_add(count);
            weighted_ratio += (lift / bodyweight) * f64::from(count);
        }
    }
    (total > 0).then_some((weighted_ratio / f64::from(total)) as f32)
}

fn dataset_file_url(version: &str, path: &str) -> String {
    let trimmed = path.trim_start_matches('/');
    format!("data/{version}/{trimmed}")
}

// ── Cross-sex rows loading ────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(super) struct CrossSexRowsCtx {
    pub(super) page_active: Memo<bool>,
    pub(super) latest: ReadSignal<Option<LatestJson>>,
    pub(super) root_index: ReadSignal<Option<RootIndex>>,
    pub(super) equip: ReadSignal<String>,
    pub(super) set_male_rows: WriteSignal<Vec<SliceRow>>,
    pub(super) set_female_rows: WriteSignal<Vec<SliceRow>>,
    pub(super) set_error: WriteSignal<Option<String>>,
    pub(super) request: RequestTracker,
}

pub(super) fn setup_cross_sex_rows_effect(ctx: CrossSexRowsCtx) {
    let CrossSexRowsCtx {
        page_active,
        latest,
        root_index,
        equip,
        set_male_rows,
        set_female_rows,
        set_error,
        request,
    } = ctx;

    Effect::new(move |_| {
        let next_id = request.current.get_untracked().wrapping_add(1);
        request.set.set(next_id);

        if !page_active.get() {
            set_male_rows.set(Vec::new());
            set_female_rows.set(Vec::new());
            set_error.set(None);
            return;
        }

        let (Some(latest_v), Some(root)) = (latest.get(), root_index.get()) else {
            set_male_rows.set(Vec::new());
            set_female_rows.set(Vec::new());
            set_error.set(None);
            return;
        };

        let equip_value = equip.get();
        if equip_value.is_empty() {
            set_male_rows.set(Vec::new());
            set_female_rows.set(Vec::new());
            set_error.set(None);
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
        set_error.set(None);

        spawn_local(async move {
            let mut male_rows = Vec::new();
            let mut female_rows = Vec::new();
            let mut issues = Vec::new();

            if let Some(rel) = male_shard_rel {
                let url = dataset_file_url(&latest_v.version, &rel);
                match fetch_json_first::<SliceIndex>(&[&url]).await {
                    Ok(index) => {
                        if request.current.get_untracked() != next_id {
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
                        if request.current.get_untracked() != next_id {
                            return;
                        }
                        female_rows = rows_from_slice_index(index);
                    }
                    Err(err) => issues.push(format!("Failed female shard: {err}")),
                }
            } else {
                issues.push("Missing female shard for selected equipment.".to_string());
            }

            if request.current.get_untracked() != next_id {
                return;
            }
            set_male_rows.set(male_rows);
            set_female_rows.set(female_rows);
            if !issues.is_empty() {
                set_error.set(Some(issues.join(" ")));
            }
        });
    });
}

// ── Cross-sex histogram loading ───────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(super) struct CrossSexHistCtx {
    pub(super) page_active: Memo<bool>,
    pub(super) calculated: ReadSignal<bool>,
    pub(super) latest: ReadSignal<Option<LatestJson>>,
    pub(super) male_choice: Memo<Option<CrossSexSliceChoice>>,
    pub(super) female_choice: Memo<Option<CrossSexSliceChoice>>,
    pub(super) current_bin: Memo<Option<String>>,
    pub(super) current_hist: ReadSignal<Option<HistogramBin>>,
    pub(super) set_male_hist: WriteSignal<Option<HistogramBin>>,
    pub(super) set_female_hist: WriteSignal<Option<HistogramBin>>,
    pub(super) set_loading: WriteSignal<bool>,
    pub(super) set_error: WriteSignal<Option<String>>,
    pub(super) request: RequestTracker,
}

pub(super) fn setup_cross_sex_hist_effect(ctx: CrossSexHistCtx) {
    let CrossSexHistCtx {
        page_active,
        calculated,
        latest,
        male_choice,
        female_choice,
        current_bin,
        current_hist,
        set_male_hist,
        set_female_hist,
        set_loading,
        set_error,
        request,
    } = ctx;

    Effect::new(move |_| {
        let next_id = request.current.get_untracked().wrapping_add(1);
        request.set.set(next_id);

        if !page_active.get() || !calculated.get() {
            set_male_hist.set(None);
            set_female_hist.set(None);
            set_loading.set(false);
            set_error.set(None);
            return;
        }

        let (Some(latest_v), Some(male_c), Some(female_c)) =
            (latest.get(), male_choice.get(), female_choice.get())
        else {
            set_male_hist.set(None);
            set_female_hist.set(None);
            set_loading.set(false);
            set_error.set(None);
            return;
        };

        let selected_bin = current_bin.get();
        let existing_hist = current_hist.get();
        let mut pre_male = None;
        let mut pre_female = None;
        if let (Some(bin), Some(h)) = (selected_bin, existing_hist) {
            if bin == male_c.row.entry.bin {
                pre_male = Some(h.clone());
            }
            if bin == female_c.row.entry.bin {
                pre_female = Some(h);
            }
        }
        set_male_hist.set(pre_male.clone());
        set_female_hist.set(pre_female.clone());

        if pre_male.is_some() && pre_female.is_some() {
            set_loading.set(false);
            set_error.set(None);
            return;
        }

        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            let mut issues = Vec::new();

            if pre_male.is_none() {
                let url = dataset_file_url(&latest_v.version, &male_c.row.entry.bin);
                match fetch_binary_first(&[&url]).await {
                    Ok(bytes) => match parse_combined_bin(&bytes).map(|(h, _)| h) {
                        Some(h) => {
                            if request.current.get_untracked() != next_id {
                                return;
                            }
                            set_male_hist.set(Some(h));
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
                            if request.current.get_untracked() != next_id {
                                return;
                            }
                            set_female_hist.set(Some(h));
                        }
                        None => issues.push("Invalid women's payload.".to_string()),
                    },
                    Err(e) => issues.push(format!("Women's bin error: {e}")),
                }
            }

            if request.current.get_untracked() != next_id {
                return;
            }
            set_loading.set(false);
            if !issues.is_empty() {
                set_error.set(Some(issues.join(" ")));
            }
        });
    });
}

// ── Cross-sex heatmap loading ─────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(super) struct CrossSexHeatCtx {
    pub(super) page_active: Memo<bool>,
    pub(super) calculated: ReadSignal<bool>,
    pub(super) latest: ReadSignal<Option<LatestJson>>,
    pub(super) male_choice: Memo<Option<CrossSexSliceChoice>>,
    pub(super) female_choice: Memo<Option<CrossSexSliceChoice>>,
    pub(super) set_male_heat: WriteSignal<Option<HeatmapBin>>,
    pub(super) set_female_heat: WriteSignal<Option<HeatmapBin>>,
    pub(super) set_loading: WriteSignal<bool>,
    pub(super) set_error: WriteSignal<Option<String>>,
    pub(super) request: RequestTracker,
}

pub(super) fn setup_cross_sex_heat_effect(ctx: CrossSexHeatCtx) {
    let CrossSexHeatCtx {
        page_active,
        calculated,
        latest,
        male_choice,
        female_choice,
        set_male_heat,
        set_female_heat,
        set_loading,
        set_error,
        request,
    } = ctx;

    Effect::new(move |_| {
        let next_id = request.current.get_untracked().wrapping_add(1);
        request.set.set(next_id);

        if !page_active.get() || !calculated.get() {
            set_male_heat.set(None);
            set_female_heat.set(None);
            set_loading.set(false);
            set_error.set(None);
            return;
        }

        let (Some(latest_v), Some(male_c), Some(female_c)) =
            (latest.get(), male_choice.get(), female_choice.get())
        else {
            set_male_heat.set(None);
            set_female_heat.set(None);
            set_loading.set(false);
            return;
        };

        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            let mut issues = Vec::new();

            let male_url = dataset_file_url(&latest_v.version, &male_c.row.entry.bin);
            match fetch_binary_first(&[&male_url]).await {
                Ok(bytes) => match parse_combined_bin(&bytes).map(|(_, h)| h) {
                    Some(h) => {
                        if request.current.get_untracked() != next_id {
                            return;
                        }
                        set_male_heat.set(Some(h));
                    }
                    None => issues.push("Invalid men's payload.".to_string()),
                },
                Err(e) => issues.push(e),
            }

            let female_url = dataset_file_url(&latest_v.version, &female_c.row.entry.bin);
            match fetch_binary_first(&[&female_url]).await {
                Ok(bytes) => match parse_combined_bin(&bytes).map(|(_, h)| h) {
                    Some(h) => {
                        if request.current.get_untracked() != next_id {
                            return;
                        }
                        set_female_heat.set(Some(h));
                    }
                    None => issues.push("Invalid women's payload.".to_string()),
                },
                Err(e) => issues.push(e),
            }

            if request.current.get_untracked() != next_id {
                return;
            }
            set_loading.set(false);
            if !issues.is_empty() {
                set_error.set(Some(issues.join(" ")));
            }
        });
    });
}

// ── Cross-sex lift comparison loading ────────────────────────────────────────

#[derive(Clone, Copy)]
pub(super) struct CrossSexLiftComparisonCtx {
    pub(super) page_active: Memo<bool>,
    pub(super) calculated: ReadSignal<bool>,
    pub(super) latest: ReadSignal<Option<LatestJson>>,
    pub(super) male_rows: ReadSignal<Vec<SliceRow>>,
    pub(super) female_rows: ReadSignal<Vec<SliceRow>>,
    pub(super) equip: ReadSignal<String>,
    pub(super) wc: ReadSignal<String>,
    pub(super) age: ReadSignal<String>,
    pub(super) tested: ReadSignal<String>,
    pub(super) set_comparisons: WriteSignal<Vec<CrossSexLiftComparison>>,
    pub(super) set_loading: WriteSignal<bool>,
    pub(super) set_error: WriteSignal<Option<String>>,
    pub(super) request: RequestTracker,
}

pub(super) fn setup_cross_sex_lift_comparison_effect(ctx: CrossSexLiftComparisonCtx) {
    let CrossSexLiftComparisonCtx {
        page_active,
        calculated,
        latest,
        male_rows,
        female_rows,
        equip,
        wc,
        age,
        tested,
        set_comparisons,
        set_loading,
        set_error,
        request,
    } = ctx;

    Effect::new(move |_| {
        let next_id = request.current.get_untracked().wrapping_add(1);
        request.set.set(next_id);

        if !page_active.get() || !calculated.get() {
            set_comparisons.set(Vec::new());
            set_loading.set(false);
            set_error.set(None);
            return;
        }

        let Some(latest_v) = latest.get() else {
            set_comparisons.set(Vec::new());
            set_loading.set(false);
            set_error.set(None);
            return;
        };

        let m_rows = male_rows.get();
        let f_rows = female_rows.get();
        let e = equip.get();
        let w = wc.get();
        let a = age.get();
        let t = tested.get();

        let choices = CROSS_SEX_LIFT_ROWS
            .iter()
            .filter_map(|(lift_code, label)| {
                let male = choose_cross_sex_slice(&m_rows, &e, &w, &a, &t, lift_code, "Kg")?;
                let female = choose_cross_sex_slice(&f_rows, &e, &w, &a, &t, lift_code, "Kg")?;
                Some(((*lift_code).to_string(), (*label).to_string(), male, female))
            })
            .collect::<Vec<_>>();

        if choices.is_empty() {
            set_comparisons.set(Vec::new());
            set_loading.set(false);
            set_error.set(Some(
                "No kg lift comparison slices for this cohort.".to_string(),
            ));
            return;
        }

        set_comparisons.set(Vec::new());
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            let mut comparisons = Vec::new();
            let mut issues = Vec::new();

            for (lift_code, label, male_c, female_c) in choices {
                let male_url = dataset_file_url(&latest_v.version, &male_c.row.entry.bin);
                let female_url = dataset_file_url(&latest_v.version, &female_c.row.entry.bin);

                let male_payload = fetch_binary_first(&[&male_url]).await;
                let female_payload = fetch_binary_first(&[&female_url]).await;
                if request.current.get_untracked() != next_id {
                    return;
                }

                let Ok(male_bytes) = male_payload else {
                    if let Err(err) = male_payload {
                        issues.push(format!("{label} men's bin error: {err}"));
                    }
                    continue;
                };
                let Ok(female_bytes) = female_payload else {
                    if let Err(err) = female_payload {
                        issues.push(format!("{label} women's bin error: {err}"));
                    }
                    continue;
                };

                let Some((male_hist, male_heat)) = parse_combined_bin(&male_bytes) else {
                    issues.push(format!("{label} men's payload was invalid."));
                    continue;
                };
                let Some((female_hist, female_heat)) = parse_combined_bin(&female_bytes) else {
                    issues.push(format!("{label} women's payload was invalid."));
                    continue;
                };

                let Some((male_mean_kg, male_total)) = histogram_weighted_mean(&male_hist) else {
                    issues.push(format!("{label} men's histogram was empty."));
                    continue;
                };
                let Some((female_mean_kg, female_total)) = histogram_weighted_mean(&female_hist)
                else {
                    issues.push(format!("{label} women's histogram was empty."));
                    continue;
                };

                comparisons.push(CrossSexLiftComparison {
                    lift: lift_code,
                    label,
                    male_mean_kg,
                    female_mean_kg,
                    male_mean_bodyweight_ratio: heatmap_mean_lift_bodyweight_ratio(&male_heat),
                    female_mean_bodyweight_ratio: heatmap_mean_lift_bodyweight_ratio(&female_heat),
                    male_total,
                    female_total,
                });
            }

            if request.current.get_untracked() != next_id {
                return;
            }
            set_comparisons.set(comparisons);
            set_loading.set(false);
            set_error.set(if issues.is_empty() {
                None
            } else {
                Some(issues.join(" "))
            });
        });
    });
}

// ── Cross-sex comparison derivation ──────────────────────────────────────────

#[derive(Clone, Copy)]
pub(super) struct CrossSexComparisonCtx {
    pub(super) calculated: ReadSignal<bool>,
    pub(super) rows_error: ReadSignal<Option<String>>,
    pub(super) male_choice: Memo<Option<CrossSexSliceChoice>>,
    pub(super) female_choice: Memo<Option<CrossSexSliceChoice>>,
    pub(super) hist_error: ReadSignal<Option<String>>,
    pub(super) male_hist: ReadSignal<Option<HistogramBin>>,
    pub(super) female_hist: ReadSignal<Option<HistogramBin>>,
    pub(super) equip: ReadSignal<String>,
    pub(super) lift: ReadSignal<String>,
    pub(super) metric: ReadSignal<String>,
    pub(super) bodyweight: ReadSignal<f32>,
    pub(super) squat: ReadSignal<f32>,
    pub(super) bench: ReadSignal<f32>,
    pub(super) deadlift: ReadSignal<f32>,
}

pub(super) fn make_cross_sex_comparison(
    ctx: CrossSexComparisonCtx,
) -> Memo<Result<CrossSexComparison, String>> {
    let CrossSexComparisonCtx {
        calculated,
        rows_error,
        male_choice,
        female_choice,
        hist_error,
        male_hist,
        female_hist,
        equip,
        lift,
        metric,
        bodyweight,
        squat,
        bench,
        deadlift,
    } = ctx;

    Memo::new(move |_| -> Result<CrossSexComparison, String> {
        if !calculated.get() {
            return Err("Calculate first.".to_string());
        }
        if let Some(err) = rows_error.get() {
            return Err(err);
        }
        let mc = male_choice
            .get()
            .ok_or("No matching men's cohort.".to_string())?;
        let fc = female_choice
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
                "Cohort too small (<{MIN_CROSS_SEX_COHORT_TOTAL} lifters)."
            ));
        }
        if let Some(err) = hist_error.get() {
            return Err(err);
        }
        let mh = male_hist
            .get()
            .ok_or("Men's distribution not loaded.".to_string())?;
        let fh = female_hist
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
            equivalent_value_for_same_percentile(Some(&mh), Some(&fh), male_val)
                .ok_or("Could not compute women's equivalent.".to_string())?;
        let (female_pct, male_at_female_pct) =
            equivalent_value_for_same_percentile(Some(&fh), Some(&mh), female_val)
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
    })
}
