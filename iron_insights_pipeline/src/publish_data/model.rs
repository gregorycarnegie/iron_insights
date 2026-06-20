use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct LatestJson {
    pub(super) version: String,
    pub(super) revision: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SliceMeta {
    pub(super) version: String,
    pub(super) sex: String,
    pub(super) equipment: String,
    pub(super) ipf_weight_class: String,
    pub(super) age_class: String,
    pub(super) tested: String,
    pub(super) lift: String,
    pub(super) metric: String,
    pub(super) hist: HistMeta,
    pub(super) heat: HeatMeta,
}

#[derive(Debug, Serialize)]
pub(super) struct HistMeta {
    pub(super) file: String,
    pub(super) base_bin_size_kg: f32,
    pub(super) min_kg: f32,
    pub(super) max_kg: f32,
    pub(super) bins: usize,
    pub(super) total: u64,
}

#[derive(Debug, Serialize)]
pub(super) struct HeatMeta {
    pub(super) file: String,
    pub(super) x_base_bin_size_kg: f32,
    pub(super) y_base_bin_size_kg: f32,
    pub(super) min_x_kg: f32,
    pub(super) max_x_kg: f32,
    pub(super) min_y_kg: f32,
    pub(super) max_y_kg: f32,
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) total: u64,
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
    pub(super) meta: String,
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
