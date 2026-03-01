use std::fs::{self, File};
use std::path::PathBuf;

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

fn build_records(input_parquet: &PathBuf, spec: LiftSpec, tested_only: bool) -> Result<DataFrame> {
    let parquet_path = input_parquet.to_string_lossy();
    let source = LazyFrame::scan_parquet(parquet_path.as_ref().into(), ScanArgsParquet::default())
        .with_context(|| format!("failed scanning {}", input_parquet.display()))?;

    let tested_expr = when(col("Tested").eq(lit("Yes")))
        .then(lit("Yes"))
        .otherwise(lit("No"))
        .alias("TestedBucket");
    let ipf_wc_expr = derive_ipf_weight_class_expr();

    let mut filtered = source
        .filter(col("Sanctioned").eq(lit("Yes")))
        .filter(col("Sex").eq(lit("M")).or(col("Sex").eq(lit("F"))))
        .filter(event_filter(spec.events))
        .filter(col(spec.column).is_not_null())
        .filter(col(spec.column).gt(lit(0.0f32)))
        .filter(col("BodyweightKg").is_not_null())
        .filter(col("BodyweightKg").cast(DataType::Float32).gt(lit(0.0f32)))
        .filter(col("Place").neq(lit("DQ")))
        .filter(col("Place").neq(lit("DD")))
        .filter(col("Place").neq(lit("NS")))
        .with_column(tested_expr)
        .with_column(ipf_wc_expr)
        .select([
            col("Name"),
            col("Sex"),
            col("Equipment"),
            col("TestedBucket"),
            col("IpfWeightClass"),
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

    // MVP: build per-lifter best-lift table. We keep context columns with simple reducers;
    // selecting context exactly from the max-lift row can be improved in a follow-up pass.
    let result = filtered
        .group_by([
            col("Name"),
            col("Sex"),
            col("Equipment"),
            col("TestedBucket"),
            col("IpfWeightClass"),
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
