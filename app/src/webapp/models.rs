use super::slices::SliceKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SavedUiState {
    pub(super) sex: String,
    pub(super) equip: String,
    pub(super) wc: String,
    pub(super) age: String,
    pub(super) tested: String,
    pub(super) lift: String,
    pub(super) metric: String,
    pub(super) squat: f32,
    pub(super) bench: f32,
    pub(super) deadlift: f32,
    pub(super) bodyweight: f32,
    pub(super) squat_delta: f32,
    pub(super) bench_delta: f32,
    pub(super) deadlift_delta: f32,
    pub(super) lift_mult: usize,
    pub(super) bw_mult: usize,
    pub(super) share_handle: String,
    pub(super) calculated: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LatestJson {
    pub(super) version: String,
    pub(super) revision: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct RootIndex {
    pub(super) shards: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct SliceIndex {
    pub(super) slices: SliceIndexEntries,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(super) enum SliceIndexEntries {
    Map(BTreeMap<String, SliceIndexEntry>),
    Keys(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct SliceIndexEntry {
    #[serde(default)]
    pub(super) meta: String,
    #[serde(default)]
    pub(super) bin: String,
    /// Base64-encoded IIC1 payload for sparse cohorts. When non-empty the app
    /// decodes and parses directly without a network fetch.
    #[serde(default)]
    pub(super) inline: String,
    #[serde(default)]
    pub(super) summary: Option<SliceSummary>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SliceRow {
    pub(super) key: SliceKey,
    pub(super) entry: SliceIndexEntry,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct SliceMetaJson {
    pub(super) hist: SliceMetaHist,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct SliceMetaHist {
    pub(super) min_kg: f32,
    pub(super) max_kg: f32,
    pub(super) total: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct SliceSummary {
    pub(super) min_kg: f32,
    pub(super) max_kg: f32,
    pub(super) total: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CrossSexLiftComparison {
    pub(super) lift: String,
    pub(super) label: String,
    pub(super) male_mean_kg: f32,
    pub(super) female_mean_kg: f32,
    pub(super) male_mean_bodyweight_ratio: Option<f32>,
    pub(super) female_mean_bodyweight_ratio: Option<f32>,
    pub(super) male_total: u32,
    pub(super) female_total: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CrossSexComparison {
    pub(super) male_percentile: f32,
    pub(super) female_percentile: f32,
    pub(super) male_total: u32,
    pub(super) female_total: u32,
    pub(super) male_input_value: f32,
    pub(super) female_input_value: f32,
    pub(super) female_value_at_male_percentile: f32,
    pub(super) male_value_at_female_percentile: f32,
    pub(super) metric: String,
    pub(super) male_weight_class: String,
    pub(super) female_weight_class: String,
    pub(super) male_wc_fallback: bool,
    pub(super) female_wc_fallback: bool,
    pub(super) caveat: Option<String>,
}
