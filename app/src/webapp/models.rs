use super::slices::SliceKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CompareMode {
    AllLifters,
    SameBodyweightRange,
    SameWeightClass,
    SameAgeClass,
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
    pub(super) meta: String,
    pub(super) hist: String,
    pub(super) heat: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SliceRow {
    pub(super) key: SliceKey,
    pub(super) entry: SliceIndexEntry,
}
