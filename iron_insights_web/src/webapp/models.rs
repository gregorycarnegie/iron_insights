use crate::core::SliceKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(super) const SAVED_UI_STATE_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SavedUiState {
    #[serde(default)]
    pub(super) version: u8,
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
    pub(super) lift_mult: usize,
    pub(super) bw_mult: usize,
    pub(super) calculated: bool,
}

impl SavedUiState {
    pub(super) fn from_storage_json(raw: &str) -> Option<Self> {
        let mut saved = serde_json::from_str::<Self>(raw).ok()?;
        match saved.version {
            0 => {
                saved.version = SAVED_UI_STATE_VERSION;
                Some(saved)
            }
            SAVED_UI_STATE_VERSION => Some(saved),
            _ => None,
        }
    }
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
    pub(super) slices: BTreeMap<String, SliceIndexEntry>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct SliceIndexEntry {
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
