use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use super::constants::{ALL, ALL_AGES, BW_BIN_BASE_KG, VALID_BW_RANGE_KG};
use super::histogram::{INLINE_THRESHOLD, build_combined_bytes, build_heatmap, build_histogram};
use super::metric::{
    Metric, lift_code, metric_base_bin, metric_code, metric_max_valid, metric_slug, tested_bucket,
};
use super::model::{HeatMeta, HistMeta, SliceIndexEntry, SliceMeta, SliceSummary};
use super::util::{slug, write_bytes};

#[derive(Debug, Default)]
pub(super) struct SliceAccumulator {
    pub(super) lift_values: Vec<f32>,
    pub(super) heat_points: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AccumulationRow<'a> {
    pub(super) sex: &'a str,
    pub(super) equipment: &'a str,
    pub(super) weight_class: &'a str,
    pub(super) age_class: &'a str,
    pub(super) year: Option<i32>,
    pub(super) x_value: f32,
    pub(super) valid_bw: Option<f32>,
}

pub(super) type SliceKey<'a> = (&'a str, &'a str, &'a str, &'a str);

#[derive(Debug)]
pub(super) struct MetricPublisher<'a> {
    pub(super) metric: Metric,
    trend_key_suffix: String,
    pub(super) slices: HashMap<SliceKey<'a>, SliceAccumulator>,
}

impl<'a> MetricPublisher<'a> {
    pub(super) fn new(metric: Metric, tested: &str, lift: &str) -> Self {
        Self {
            metric,
            trend_key_suffix: format!(
                "|tested={}|lift={}|metric={}",
                tested_bucket(tested),
                lift_code(lift),
                metric_code(metric),
            ),
            slices: HashMap::new(),
        }
    }

    fn trend_key(&self, sex: &str, equipment: &str) -> String {
        let mut key = String::with_capacity(
            "sex=".len()
                + sex.len()
                + "|equip=".len()
                + equipment.len()
                + self.trend_key_suffix.len(),
        );
        key.push_str("sex=");
        key.push_str(sex);
        key.push_str("|equip=");
        key.push_str(equipment);
        key.push_str(&self.trend_key_suffix);
        key
    }

    pub(super) fn accumulate_row(
        &mut self,
        row: AccumulationRow<'a>,
        trends_acc: &mut BTreeMap<String, BTreeMap<i32, Vec<f32>>>,
    ) {
        // Drop implausible rows before they reach any accumulator: histogram and
        // heatmap axes are sized from observed min/max, so one corrupt value
        // stretches every grid built from this slice.
        if !row.x_value.is_finite()
            || row.x_value <= 0.0
            || row.x_value > metric_max_valid(self.metric)
        {
            return;
        }
        let row = AccumulationRow {
            valid_bw: row
                .valid_bw
                .filter(|bw| bw.is_finite() && VALID_BW_RANGE_KG.contains(bw)),
            ..row
        };

        if let Some(year) = row.year {
            for trend_equip in [row.equipment, ALL] {
                let trend_key = self.trend_key(row.sex, trend_equip);
                trends_acc
                    .entry(trend_key)
                    .or_default()
                    .entry(year)
                    .or_default()
                    .push(row.x_value);
            }
        }

        for key in [
            (row.sex, row.equipment, row.weight_class, row.age_class),
            (row.sex, ALL, row.weight_class, row.age_class),
            (row.sex, row.equipment, ALL, row.age_class),
            (row.sex, ALL, ALL, row.age_class),
            (row.sex, row.equipment, row.weight_class, ALL_AGES),
            (row.sex, ALL, row.weight_class, ALL_AGES),
            (row.sex, row.equipment, ALL, ALL_AGES),
            (row.sex, ALL, ALL, ALL_AGES),
        ] {
            let entry = self.slices.entry(key).or_default();
            entry.lift_values.push(row.x_value);
            if let Some(bw_value) = row.valid_bw {
                entry.heat_points.push((row.x_value, bw_value));
            }
        }
    }

    pub(super) fn write_outputs(
        self,
        version_dir: &Path,
        version: &str,
        tested: &str,
        lift: &str,
        write_meta_files: bool,
        shard_indices: &mut BTreeMap<String, BTreeMap<String, SliceIndexEntry>>,
    ) -> Result<()> {
        let metric = self.metric;
        let metric_slug = metric_slug(metric);
        let metric_code = metric_code(metric);
        let x_base = metric_base_bin(metric);

        for ((sex, equipment, weight_class, age_class), acc) in self.slices {
            if acc.lift_values.is_empty() {
                continue;
            }

            let hist_data = build_histogram(&acc.lift_values, x_base)?;
            let heat_data = build_heatmap(&acc.heat_points, x_base, BW_BIN_BASE_KG)?;

            let sex_slug = slug(sex);
            let equip_slug = slug(equipment);
            let wc_slug = slug(weight_class);
            let age_slug = slug(age_class);

            let meta_rel = format!(
                "meta/{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested}/{metric_slug}/{lift}.json"
            );
            let meta_path = version_dir.join(&meta_rel);

            let combined = build_combined_bytes(&hist_data, &heat_data, x_base, BW_BIN_BASE_KG);
            let (bin_rel, inline) = if combined.len() <= INLINE_THRESHOLD {
                (String::new(), BASE64.encode(&combined))
            } else {
                let rel = format!(
                    "bin/{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested}/{metric_slug}/{lift}.bin"
                );
                let path = version_dir.join(&rel);
                write_bytes(&path, &combined)?;
                (rel, String::new())
            };

            let meta = SliceMeta {
                version: version.to_string(),
                sex: sex.to_string(),
                equipment: equipment.to_string(),
                ipf_weight_class: weight_class.to_string(),
                age_class: age_class.to_string(),
                tested: tested.to_string(),
                lift: lift.to_string(),
                metric: metric_code.to_string(),
                hist: HistMeta {
                    file: bin_rel.clone(),
                    base_bin_size_kg: x_base,
                    min_kg: hist_data.min,
                    max_kg: hist_data.max,
                    bins: hist_data.counts.len(),
                    total: hist_data.total,
                },
                heat: HeatMeta {
                    file: bin_rel.clone(),
                    x_base_bin_size_kg: x_base,
                    y_base_bin_size_kg: BW_BIN_BASE_KG,
                    min_x_kg: heat_data.min_x,
                    max_x_kg: heat_data.max_x,
                    min_y_kg: heat_data.min_y,
                    max_y_kg: heat_data.max_y,
                    width: heat_data.width,
                    height: heat_data.height,
                    total: heat_data.total,
                },
            };

            if write_meta_files {
                if let Some(parent) = meta_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("failed creating {}", parent.display()))?;
                }
                fs::write(&meta_path, serde_json::to_vec(&meta)?)
                    .with_context(|| format!("failed writing {}", meta_path.display()))?;
            }

            let key = format!(
                "sex={}|equip={}|wc={}|age={}|tested={}|lift={}|metric={}",
                sex,
                equipment,
                weight_class,
                age_class,
                tested_bucket(tested),
                lift_code(lift),
                metric_code,
            );
            let shard_key = format!("sex={}|equip={}", sex, equipment);
            shard_indices.entry(shard_key).or_default().insert(
                key,
                SliceIndexEntry {
                    meta: if write_meta_files {
                        meta_rel
                    } else {
                        String::new()
                    },
                    bin: bin_rel,
                    inline,
                    summary: SliceSummary {
                        min_kg: hist_data.min,
                        max_kg: hist_data.max,
                        total: hist_data.total.min(u32::MAX as u64) as u32,
                    },
                },
            );
        }

        Ok(())
    }
}
