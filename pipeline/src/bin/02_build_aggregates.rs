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

    let mut filtered = source
        .filter(col("Sanctioned").eq(lit("Yes")))
        .filter(event_filter(spec.events))
        .filter(col(spec.column).is_not_null())
        .filter(col(spec.column).gt(lit(0.0)))
        .filter(col("Place").neq(lit("DQ")))
        .filter(col("Place").neq(lit("DD")))
        .filter(col("Place").neq(lit("NS")))
        .with_column(tested_expr)
        .select([
            col("Name"),
            col("Sex"),
            col("Equipment"),
            col("TestedBucket"),
            col(spec.column).alias("lift_value"),
            col("BodyweightKg"),
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
