use std::collections::BTreeMap;

use super::model::{TrendPoint, TrendSeries, TrendsJson};

pub(super) fn build_trends_shards(
    version: &str,
    trends_acc: BTreeMap<String, BTreeMap<i32, Vec<f32>>>,
) -> BTreeMap<String, TrendsJson> {
    // Accumulate series into per-shard buckets.
    let mut by_shard: BTreeMap<String, Vec<TrendSeries>> = BTreeMap::new();
    for (key, by_year) in trends_acc {
        let mut points = Vec::new();
        for (year, mut values) in by_year {
            if values.is_empty() {
                continue;
            }
            values.sort_by(f32::total_cmp);
            let p50 = quantile_sorted(&values, 0.50);
            let p90 = quantile_sorted(&values, 0.90);
            points.push(TrendPoint {
                year,
                total: values.len().min(u32::MAX as usize) as u32,
                p50,
                p90,
            });
        }
        points.sort_by_key(|p| p.year);
        if points.is_empty() {
            continue;
        }
        // Extract the `sex=X|equip=Y` shard prefix from the key.
        let shard_key = key.split('|').take(2).collect::<Vec<_>>().join("|");
        by_shard
            .entry(shard_key)
            .or_default()
            .push(TrendSeries { key, points });
    }

    by_shard
        .into_iter()
        .map(|(shard_key, mut series)| {
            series.sort_by(|a, b| a.key.cmp(&b.key));
            let payload = TrendsJson {
                version: version.to_string(),
                bucket: "year".to_string(),
                series,
            };
            (shard_key, payload)
        })
        .collect()
}

pub(super) fn parse_year_bucket(value: Option<&str>) -> Option<i32> {
    let raw = value?;
    let year = raw.get(0..4)?.parse::<i32>().ok()?;
    if (1900..=2100).contains(&year) {
        Some(year)
    } else {
        None
    }
}

pub(super) fn quantile_sorted(values: &[f32], q: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let q = q.clamp(0.0, 1.0);
    let idx = ((values.len() - 1) as f32 * q).round() as usize;
    values[idx]
}
