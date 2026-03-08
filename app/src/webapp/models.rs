use super::slices::SliceKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CompareMode {
    AllLifters,
    SameBodyweightRange,
    SameWeightClass,
    SameAgeClass,
    SameTestedStatus,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) struct SavedSnapshot {
    pub(super) saved_at_secs: u64,
    pub(super) percentile: f32,
    pub(super) rank: usize,
    pub(super) total_lifters: u32,
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
pub(super) struct TrendsJson {
    pub(super) bucket: String,
    pub(super) series: Vec<TrendSeries>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct TrendSeries {
    pub(super) key: String,
    pub(super) points: Vec<TrendPoint>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct TrendPoint {
    pub(super) year: i32,
    pub(super) total: u32,
    pub(super) p50: f32,
    pub(super) p90: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct SliceIndex {
    pub(super) shard_key: String,
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
    pub(super) hist: String,
    pub(super) heat: String,
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
