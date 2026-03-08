use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::thread;

use anyhow::{Context, Result};
use clap::Parser;
use polars::prelude::*;

#[derive(Debug, Parser)]
struct Args {
    #[arg(
        long,
        default_value = "pipeline/output/openpowerlifting-latest.parquet"
    )]
    input_parquet: PathBuf,

    #[arg(long, default_value = "pipeline/output/records")]
    output_dir: PathBuf,
}

#[derive(Clone, Copy)]
struct LiftSpec {
    name: &'static str,
    column: &'static str,
    events: &'static [&'static str],
}

const LIFT_SPECS: &[LiftSpec] = &[
    LiftSpec {
        name: "squat",
        column: "Best3SquatKg",
        events: &["SBD", "SD", "SB", "S"],
    },
    LiftSpec {
        name: "bench",
        column: "Best3BenchKg",
        events: &["SBD", "BD", "SB", "B"],
    },
    LiftSpec {
        name: "deadlift",
        column: "Best3DeadliftKg",
        events: &["SBD", "BD", "SD", "D"],
    },
    LiftSpec {
        name: "total",
        column: "TotalKg",
        events: &["SBD"],
    },
];

fn main() -> Result<()> {
    // Some Windows runs hit STATUS_STACK_OVERFLOW in deep Polars execution paths.
    // Run the workload on a larger stack to keep the pipeline stable.
    let handle = thread::Builder::new()
        .name("build-aggregates".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .context("failed to spawn build-aggregates worker thread")?;

    match handle.join() {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("failed to create {}", args.output_dir.display()))?;

    for tested_only in [false, true] {
        let tested_label = if tested_only { "tested" } else { "all" };
        let tested_dir = args.output_dir.join(tested_label);
        fs::create_dir_all(&tested_dir)
            .with_context(|| format!("failed to create {}", tested_dir.display()))?;

        for spec in LIFT_SPECS {
            let mut df = build_records(&args.input_parquet, *spec, tested_only)?;
            let out_path = tested_dir.join(format!("{}.parquet", spec.name));
            let mut out = File::create(&out_path)
                .with_context(|| format!("failed creating {}", out_path.display()))?;
            ParquetWriter::new(&mut out)
                .finish(&mut df)
                .with_context(|| format!("failed writing {}", out_path.display()))?;
            println!("Wrote: {}", out_path.display());
        }
    }

    Ok(())
}

fn build_records(input_parquet: &Path, spec: LiftSpec, tested_only: bool) -> Result<DataFrame> {
    let parquet_path = input_parquet.to_string_lossy();
    let source = LazyFrame::scan_parquet(parquet_path.as_ref().into(), ScanArgsParquet::default())
        .with_context(|| format!("failed scanning {}", input_parquet.display()))?;

    let tested_expr = when(col("Tested").eq(lit("Yes")))
        .then(lit("Yes"))
        .otherwise(lit("No"))
        .alias("TestedBucket");
    let ipf_wc_expr = derive_ipf_weight_class_expr();
    let age_class_expr = derive_age_class_expr();

    let mut filtered = source
        .filter(col("Sanctioned").eq(lit("Yes")))
        .filter(col("Sex").eq(lit("M")).or(col("Sex").eq(lit("F"))))
        .filter(event_filter(spec.events))
        .filter(col(spec.column).is_not_null())
        .filter(col(spec.column).gt(lit(0.0f32)))
        .filter(col("BodyweightKg").is_not_null())
        .filter(col("BodyweightKg").cast(DataType::Float32).gt(lit(0.0f32)))
        .filter(col("Age").is_not_null())
        .filter(col("Age").cast(DataType::Float32).gt(lit(0.0f32)))
        .filter(col("Place").neq(lit("DQ")))
        .filter(col("Place").neq(lit("DD")))
        .filter(col("Place").neq(lit("NS")))
        .with_column(tested_expr)
        .with_column(ipf_wc_expr)
        .with_column(age_class_expr)
        .select([
            col("Name"),
            col("Sex"),
            col("Equipment"),
            col("TestedBucket"),
            col("IpfWeightClass"),
            col("AgeClassBucket"),
            col(spec.column).cast(DataType::Float32).alias("lift_value"),
            col("BodyweightKg")
                .cast(DataType::Float32)
                .alias("BodyweightKg"),
            col("Date"),
            col("Federation"),
            col("MeetName"),
        ]);

    if tested_only {
        filtered = filtered.filter(col("TestedBucket").eq(lit("Yes")));
    }

    // MVP: build per-lifter best-lift table. We keep context columns with simple reducers.
    // NOTE: using sort_by(...).last() for context columns caused stack overflow on Windows
    // in this Polars query path, so we keep the stable reducers here.
    let result = filtered
        .group_by([
            col("Name"),
            col("Sex"),
            col("Equipment"),
            col("TestedBucket"),
            col("IpfWeightClass"),
            col("AgeClassBucket"),
        ])
        .agg([
            col("lift_value").max().alias("best_lift"),
            col("BodyweightKg").max().alias("bodyweight_at_best"),
            col("Date").max().alias("date_at_best"),
            col("Federation").first().alias("federation_at_best"),
            col("MeetName").first().alias("meet_name_at_best"),
        ])
        .collect()
        .context("failed collecting grouped records")?;

    Ok(result)
}

fn event_filter(events: &[&str]) -> Expr {
    events
        .iter()
        .fold(lit(false), |expr, ev| expr.or(col("Event").eq(lit(*ev))))
}

fn derive_ipf_weight_class_expr() -> Expr {
    let bw = col("BodyweightKg").cast(DataType::Float32);
    let men = when(bw.clone().lt_eq(lit(53.0f32)))
        .then(lit("53"))
        .when(bw.clone().lt_eq(lit(59.0f32)))
        .then(lit("59"))
        .when(bw.clone().lt_eq(lit(66.0f32)))
        .then(lit("66"))
        .when(bw.clone().lt_eq(lit(74.0f32)))
        .then(lit("74"))
        .when(bw.clone().lt_eq(lit(83.0f32)))
        .then(lit("83"))
        .when(bw.clone().lt_eq(lit(93.0f32)))
        .then(lit("93"))
        .when(bw.clone().lt_eq(lit(105.0f32)))
        .then(lit("105"))
        .when(bw.clone().lt_eq(lit(120.0f32)))
        .then(lit("120"))
        .otherwise(lit("120+"));

    let women = when(bw.clone().lt_eq(lit(43.0f32)))
        .then(lit("43"))
        .when(bw.clone().lt_eq(lit(47.0f32)))
        .then(lit("47"))
        .when(bw.clone().lt_eq(lit(52.0f32)))
        .then(lit("52"))
        .when(bw.clone().lt_eq(lit(57.0f32)))
        .then(lit("57"))
        .when(bw.clone().lt_eq(lit(63.0f32)))
        .then(lit("63"))
        .when(bw.clone().lt_eq(lit(69.0f32)))
        .then(lit("69"))
        .when(bw.clone().lt_eq(lit(76.0f32)))
        .then(lit("76"))
        .when(bw.clone().lt_eq(lit(84.0f32)))
        .then(lit("84"))
        .otherwise(lit("84+"));

    when(col("Sex").eq(lit("M")))
        .then(men)
        .when(col("Sex").eq(lit("F")))
        .then(women)
        .otherwise(lit("Unknown"))
        .alias("IpfWeightClass")
}

fn derive_age_class_expr() -> Expr {
    let age = col("Age").cast(DataType::Float32);
    when(age.clone().lt_eq(lit(12.0f32)))
        .then(lit("5-12"))
        .when(age.clone().lt_eq(lit(15.0f32)))
        .then(lit("13-15"))
        .when(age.clone().lt_eq(lit(17.0f32)))
        .then(lit("16-17"))
        .when(age.clone().lt_eq(lit(19.0f32)))
        .then(lit("18-19"))
        .when(age.clone().lt_eq(lit(23.0f32)))
        .then(lit("20-23"))
        .when(age.clone().lt_eq(lit(34.0f32)))
        .then(lit("24-34"))
        .when(age.clone().lt_eq(lit(39.0f32)))
        .then(lit("35-39"))
        .when(age.clone().lt_eq(lit(44.0f32)))
        .then(lit("40-44"))
        .when(age.clone().lt_eq(lit(49.0f32)))
        .then(lit("45-49"))
        .when(age.clone().lt_eq(lit(54.0f32)))
        .then(lit("50-54"))
        .when(age.clone().lt_eq(lit(59.0f32)))
        .then(lit("55-59"))
        .when(age.clone().lt_eq(lit(64.0f32)))
        .then(lit("60-64"))
        .when(age.clone().lt_eq(lit(69.0f32)))
        .then(lit("65-69"))
        .when(age.clone().lt_eq(lit(74.0f32)))
        .then(lit("70-74"))
        .when(age.clone().lt_eq(lit(79.0f32)))
        .then(lit("75-79"))
        .otherwise(lit("80+"))
        .alias("AgeClassBucket")
}

#[cfg(test)]
mod tests {
    use super::{derive_age_class_expr, derive_ipf_weight_class_expr, event_filter};
    use polars::prelude::*;

    #[test]
    fn event_filter_includes_expected_events() {
        let df = df!("Event" => &["SBD", "B", "X"]).expect("df");
        let out = df
            .lazy()
            .filter(event_filter(&["SBD", "B"]))
            .collect()
            .expect("collect");
        assert_eq!(out.height(), 2);
    }

    #[test]
    fn derive_age_class_expr_maps_boundaries() {
        let df = df!("Age" => &[12.0f32, 13.0, 34.0, 80.0]).expect("df");
        let out = df
            .lazy()
            .select([derive_age_class_expr()])
            .collect()
            .expect("collect");
        let col = out
            .column("AgeClassBucket")
            .expect("AgeClassBucket")
            .str()
            .expect("string");
        assert_eq!(col.get(0), Some("5-12"));
        assert_eq!(col.get(1), Some("13-15"));
        assert_eq!(col.get(2), Some("24-34"));
        assert_eq!(col.get(3), Some("80+"));
    }

    #[test]
    fn derive_ipf_weight_class_expr_maps_male_and_female() {
        let df = df!(
            "Sex" => &["M", "M", "F", "F"],
            "BodyweightKg" => &[83.0f32, 130.0, 57.0, 90.0]
        )
        .expect("df");
        let out = df
            .lazy()
            .select([derive_ipf_weight_class_expr()])
            .collect()
            .expect("collect");
        let col = out
            .column("IpfWeightClass")
            .expect("IpfWeightClass")
            .str()
            .expect("string");
        assert_eq!(col.get(0), Some("83"));
        assert_eq!(col.get(1), Some("120+"));
        assert_eq!(col.get(2), Some("57"));
        assert_eq!(col.get(3), Some("84+"));
    }
}
