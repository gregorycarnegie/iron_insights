use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result};
use polars::prelude::*;

use super::{
    accumulation::{AccumulationRow, MetricPublisher},
    constants::VALID_BW_RANGE_KG,
    metric::{metric_value, metrics_for_lift},
    model::SliceIndexEntry,
    trends::parse_year_bucket,
};

pub(super) struct PublishRecordsJob<'a> {
    pub(super) records_path: &'a Path,
    pub(super) version_dir: &'a Path,
    pub(super) tested: &'a str,
    pub(super) lift: &'a str,
    pub(super) shard_indices: &'a mut BTreeMap<String, BTreeMap<String, SliceIndexEntry>>,
    pub(super) trends_acc: &'a mut BTreeMap<String, BTreeMap<i32, Vec<f32>>>,
}

pub(super) fn publish_records_for_lift(job: PublishRecordsJob<'_>) -> Result<()> {
    let PublishRecordsJob {
        records_path,
        version_dir,
        tested,
        lift,
        shard_indices,
        trends_acc,
    } = job;
    let df = collect_records_frame(records_path)?;

    let str_col = |name: &str| -> Result<&ChunkedArray<StringType>> {
        df.column(name)
            .with_context(|| format!("missing {name} column"))?
            .str()
            .with_context(|| format!("{name} column not string"))
    };

    let f32_col = |name: &str| -> Result<&ChunkedArray<Float32Type>> {
        df.column(name)
            .with_context(|| format!("missing {name} column"))?
            .f32()
            .with_context(|| format!("{name} column not f32"))
    };

    let sex_col = str_col("Sex")?;
    let equip_col = str_col("Equipment")?;
    let wc_col = str_col("IpfWeightClass")?;
    let age_col = str_col("AgeClassBucket")?;
    let lift_col = f32_col("best_lift")?;
    let bw_col = f32_col("bodyweight_at_best")?;
    let date_col = str_col("date_at_best")?;

    let mut publishers: Vec<MetricPublisher<'_>> = metrics_for_lift(lift)
        .iter()
        .copied()
        .map(|metric| MetricPublisher::new(metric, tested, lift))
        .collect();

    for i in 0..df.height() {
        let (Some(sex), Some(equipment), Some(weight_class), Some(age_class), Some(lift_value)) = (
            sex_col.get(i),
            equip_col.get(i),
            wc_col.get(i),
            age_col.get(i),
            lift_col.get(i),
        ) else {
            continue;
        };
        if lift_value <= 0.0 {
            continue;
        }

        // A bodyweight outside the plausible range is a typo, and the three
        // score metrics disagreed about what to do with it: DOTS and Wilks
        // clamp into range and publish a fabricated score, while Goodlift's
        // denominator goes non-positive and the row vanishes. Declining to
        // score it at all is the one answer that is right for each of them —
        // `metric_value` returns `None` without a bodyweight, so the row still
        // counts toward the kg histogram, which does not use one.
        let valid_bw = bw_col
            .get(i)
            .filter(|bw| bw.is_finite() && VALID_BW_RANGE_KG.contains(bw));
        let year = parse_year_bucket(date_col.get(i));

        for publisher in &mut publishers {
            let Some(x_value) =
                metric_value(publisher.metric, lift, sex, equipment, lift_value, valid_bw)
            else {
                continue;
            };
            publisher.accumulate_row(
                AccumulationRow {
                    sex,
                    equipment,
                    weight_class,
                    age_class,
                    year,
                    x_value,
                    valid_bw,
                },
                trends_acc,
            );
        }
    }

    for publisher in publishers {
        publisher.write_outputs(version_dir, tested, lift, shard_indices)?;
    }

    Ok(())
}

fn collect_records_frame(records_path: &Path) -> Result<DataFrame> {
    let parquet_path = records_path.to_string_lossy();
    LazyFrame::scan_parquet(parquet_path.as_ref().into(), ScanArgsParquet::default())
        .with_context(|| format!("failed scanning {}", records_path.display()))?
        .select([
            col("Sex"),
            col("Equipment"),
            col("IpfWeightClass"),
            col("AgeClassBucket"),
            col("best_lift"),
            col("bodyweight_at_best"),
            col("date_at_best"),
        ])
        .collect()
        .with_context(|| format!("failed collecting {}", records_path.display()))
}
