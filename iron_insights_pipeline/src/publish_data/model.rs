use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct LatestJson {
    pub(super) version: String,
    pub(super) revision: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct RootIndex {
    pub(super) version: String,
    pub(super) shards: BTreeMap<String, String>,
    pub(super) trends_shards: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub(super) struct TrendsJson {
    pub(super) version: String,
    pub(super) bucket: String,
    pub(super) series: Vec<TrendSeries>,
}

#[derive(Debug, Serialize)]
pub(super) struct TrendSeries {
    pub(super) key: String,
    pub(super) points: Vec<TrendPoint>,
}

#[derive(Debug, Serialize)]
pub(super) struct TrendPoint {
    pub(super) year: i32,
    pub(super) total: u32,
    pub(super) p50: f32,
    pub(super) p90: f32,
}

#[derive(Debug, Serialize)]
pub(super) struct SliceIndex {
    pub(super) version: String,
    pub(super) shard_key: String,
    pub(super) slices: BTreeMap<String, SliceIndexEntry>,
}

#[derive(Debug, Serialize)]
pub(super) struct SliceIndexEntry {
    /// Relative path to the combined IIC1 binary, or empty if the payload is inlined.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(super) bin: String,
    /// Base64-encoded IIC1 payload for sparse cohorts (≤ INLINE_THRESHOLD bytes).
    /// When present, the app decodes and parses directly without a network fetch.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(super) inline: String,
    pub(super) summary: SliceSummary,
}

#[derive(Debug, Serialize)]
pub(super) struct SliceSummary {
    pub(super) min_kg: f32,
    pub(super) max_kg: f32,
    pub(super) total: u32,
}
