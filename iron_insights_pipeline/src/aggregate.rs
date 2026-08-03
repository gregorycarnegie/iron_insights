//! Stage 2: build the per-lifter best-lift aggregate tables (one Parquet per
//! lift, split into `all`/`tested` cohorts) consumed by stage 3.

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    thread,
};

use anyhow::{Context, Result};
use clap::Parser;
use iron_insights_core::{IPF_FEMALE_WEIGHT_CLASSES, IPF_MALE_WEIGHT_CLASSES};
use polars::prelude::*;

#[cfg(test)]
pub(crate) use Args as AggregateArgs;

/// Crate-visible so the stage-chain test can drive a build; the binary still
/// reaches it only through [`run`].
#[derive(Debug, Parser)]
pub(crate) struct Args {
    #[arg(
        long,
        default_value = "iron_insights_pipeline/output/openpowerlifting-latest.parquet"
    )]
    pub(crate) input_parquet: PathBuf,

    #[arg(long, default_value = "iron_insights_pipeline/output/records")]
    pub(crate) output_dir: PathBuf,
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

pub fn run() -> Result<()> {
    // Some Windows runs hit STATUS_STACK_OVERFLOW in deep Polars execution paths.
    // Run the workload on a larger stack to keep the pipeline stable.
    let handle = thread::Builder::new()
        .name("build-aggregates".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(build_all)
        .context("failed to spawn build-aggregates worker thread")?;

    match handle.join() {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

fn build_all() -> Result<()> {
    build_aggregates(&Args::parse())
}

/// Split from [`build_all`] so tests can drive a build without going through argv.
pub(crate) fn build_aggregates(args: &Args) -> Result<()> {
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
        ]);

    if tested_only {
        filtered = filtered.filter(col("TestedBucket").eq(lit("Yes")));
    }

    // MVP: build per-lifter best-lift table. We keep context columns with simple reducers.
    // NOTE: using sort_by(...).last() for context columns caused stack overflow on Windows
    // in this Polars query path, so we keep the stable reducers here.
    //
    // Name and TestedBucket are grouping keys only: they dedupe to one row per
    // lifter, then get dropped. Stage 3 reads neither (the tested cohort is the
    // output directory), and Name alone is 45% of the written file.
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
        ])
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
        .context("failed collecting grouped records")?;

    Ok(result)
}

fn event_filter(events: &[&str]) -> Expr {
    events
        .iter()
        .fold(lit(false), |expr, ev| expr.or(col("Event").eq(lit(*ev))))
}

/// Upper age bound (inclusive) and label per class; the last entry is the open-ended top.
const AGE_CLASSES: &[(f32, &str)] = &[
    (12.0, "5-12"),
    (15.0, "13-15"),
    (17.0, "16-17"),
    (19.0, "18-19"),
    (23.0, "20-23"),
    (34.0, "24-34"),
    (39.0, "35-39"),
    (44.0, "40-44"),
    (49.0, "45-49"),
    (54.0, "50-54"),
    (59.0, "55-59"),
    (64.0, "60-64"),
    (69.0, "65-69"),
    (74.0, "70-74"),
    (79.0, "75-79"),
    (f32::INFINITY, "80+"),
];

fn bounded_class_chain(value: Expr, classes: &[(f32, &'static str)]) -> Expr {
    // Build a when/then chain from a shared boundary table.
    // Bootstrap with two entries to produce a ChainedThen — Then and ChainedThen are distinct
    // Polars types so both can't be held in the same mut variable.
    let mut iter = classes.iter();
    let (u0, l0) = iter.next().expect("classes must have at least 2 entries");
    let (u1, l1) = iter.next().expect("classes must have at least 2 entries");
    let mut expr = when(value.clone().lt_eq(lit(*u0)))
        .then(lit(*l0))
        .when(value.clone().lt_eq(lit(*u1)))
        .then(lit(*l1));
    for (upper, label) in iter {
        if upper.is_finite() {
            expr = expr
                .when(value.clone().lt_eq(lit(*upper)))
                .then(lit(*label));
        } else {
            return expr.otherwise(lit(*label));
        }
    }
    expr.otherwise(lit("Unknown"))
}

fn derive_ipf_weight_class_expr() -> Expr {
    let bw = col("BodyweightKg").cast(DataType::Float32);
    let men = bounded_class_chain(bw.clone(), IPF_MALE_WEIGHT_CLASSES);
    let women = bounded_class_chain(bw, IPF_FEMALE_WEIGHT_CLASSES);

    when(col("Sex").eq(lit("M")))
        .then(men)
        .when(col("Sex").eq(lit("F")))
        .then(women)
        .otherwise(lit("Unknown"))
        .alias("IpfWeightClass")
}

fn derive_age_class_expr() -> Expr {
    bounded_class_chain(col("Age").cast(DataType::Float32), AGE_CLASSES).alias("AgeClassBucket")
}

#[cfg(test)]
mod tests {
    use super::{
        Args, build_aggregates, derive_age_class_expr, derive_ipf_weight_class_expr, event_filter,
    };
    use polars::prelude::*;
    use std::path::Path;
    use tempfile::TempDir;

    /// One competition result, in the shape stage 1 writes.
    struct SourceRow {
        name: &'static str,
        sex: &'static str,
        equipment: &'static str,
        event: &'static str,
        tested: &'static str,
        sanctioned: &'static str,
        place: &'static str,
        age: f32,
        bodyweight: f32,
        squat: f32,
        bench: f32,
        deadlift: f32,
        total: f32,
        date: &'static str,
    }

    impl SourceRow {
        /// A clean, sanctioned, raw male SBD result that survives every filter.
        fn valid(name: &'static str) -> Self {
            Self {
                name,
                sex: "M",
                equipment: "Raw",
                event: "SBD",
                tested: "Yes",
                sanctioned: "Yes",
                place: "1",
                age: 30.0,
                bodyweight: 92.0,
                squat: 200.0,
                bench: 140.0,
                deadlift: 240.0,
                total: 580.0,
                date: "2026-03-07",
            }
        }
    }

    fn write_source_parquet(path: &Path, rows: &[SourceRow]) {
        std::fs::create_dir_all(path.parent().expect("source path has a parent"))
            .expect("create source dir");

        let mut df = df!(
            "Name" => rows.iter().map(|r| r.name).collect::<Vec<_>>(),
            "Sex" => rows.iter().map(|r| r.sex).collect::<Vec<_>>(),
            "Equipment" => rows.iter().map(|r| r.equipment).collect::<Vec<_>>(),
            "Event" => rows.iter().map(|r| r.event).collect::<Vec<_>>(),
            "Tested" => rows.iter().map(|r| r.tested).collect::<Vec<_>>(),
            "Sanctioned" => rows.iter().map(|r| r.sanctioned).collect::<Vec<_>>(),
            "Place" => rows.iter().map(|r| r.place).collect::<Vec<_>>(),
            "Age" => rows.iter().map(|r| r.age).collect::<Vec<_>>(),
            "BodyweightKg" => rows.iter().map(|r| r.bodyweight).collect::<Vec<_>>(),
            "Best3SquatKg" => rows.iter().map(|r| r.squat).collect::<Vec<_>>(),
            "Best3BenchKg" => rows.iter().map(|r| r.bench).collect::<Vec<_>>(),
            "Best3DeadliftKg" => rows.iter().map(|r| r.deadlift).collect::<Vec<_>>(),
            "TotalKg" => rows.iter().map(|r| r.total).collect::<Vec<_>>(),
            "Date" => rows.iter().map(|r| r.date).collect::<Vec<_>>(),
        )
        .expect("source frame should build");

        let mut file = std::fs::File::create(path).expect("create source parquet");
        ParquetWriter::new(&mut file)
            .finish(&mut df)
            .expect("write source parquet");
    }

    /// Runs stage 2 over `rows` and returns the temp dir plus the records dir.
    fn run_stage_2(rows: &[SourceRow]) -> (TempDir, std::path::PathBuf) {
        let temp = TempDir::new().expect("temp dir");
        let input_parquet = temp.path().join("source.parquet");
        let output_dir = temp.path().join("records");
        write_source_parquet(&input_parquet, rows);

        build_aggregates(&Args {
            input_parquet,
            output_dir: output_dir.clone(),
        })
        .expect("stage 2 should succeed");

        (temp, output_dir)
    }

    fn read_records(records_dir: &Path, tested: &str, lift: &str) -> DataFrame {
        let path = records_dir.join(tested).join(format!("{lift}.parquet"));
        LazyFrame::scan_parquet(
            path.to_string_lossy().as_ref().into(),
            ScanArgsParquet::default(),
        )
        .unwrap_or_else(|e| panic!("scan {}: {e}", path.display()))
        .collect()
        .expect("collect records")
    }

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

    // ===== END-TO-END AGGREGATE =====

    #[test]
    fn writes_a_parquet_per_lift_and_cohort() {
        let (_temp, records) = run_stage_2(&[SourceRow::valid("Ada")]);

        // Stage 3 walks exactly this grid and skips what is missing, so a lift
        // silently dropped here becomes a lift missing from the whole site.
        for tested in ["all", "tested"] {
            for lift in ["squat", "bench", "deadlift", "total"] {
                let path = records.join(tested).join(format!("{lift}.parquet"));
                assert!(path.is_file(), "stage 2 did not write {}", path.display());
            }
        }
    }

    #[test]
    fn output_columns_match_what_stage_3_reads() {
        let (_temp, records) = run_stage_2(&[SourceRow::valid("Ada")]);
        let df = read_records(&records, "all", "squat");

        // `publish_data::collect_records_frame` selects these by name. Name and
        // TestedBucket are grouping keys only and must not survive: Name alone
        // was 45% of the written file.
        let mut columns: Vec<_> = df
            .get_column_names()
            .iter()
            .map(|c| c.to_string())
            .collect();
        columns.sort();
        assert_eq!(
            columns,
            [
                "AgeClassBucket",
                "Equipment",
                "IpfWeightClass",
                "Sex",
                "best_lift",
                "bodyweight_at_best",
                "date_at_best",
            ]
        );
    }

    #[test]
    fn keeps_only_each_lifters_best_result() {
        // Same lifter, same cohort, three meets: one row out, holding the best.
        let mut light = SourceRow::valid("Ada");
        light.squat = 180.0;
        light.date = "2026-01-01";
        let mut heavy = SourceRow::valid("Ada");
        heavy.squat = 215.0;
        heavy.date = "2026-06-01";
        let mut middling = SourceRow::valid("Ada");
        middling.squat = 200.0;

        let (_temp, records) = run_stage_2(&[light, heavy, middling]);
        let df = read_records(&records, "all", "squat");

        assert_eq!(df.height(), 1, "one row per lifter per cohort");
        let best = df
            .column("best_lift")
            .expect("best_lift")
            .f32()
            .expect("f32");
        assert_eq!(best.get(0), Some(215.0));
    }

    #[test]
    fn drops_rows_that_are_not_real_results() {
        let mut dq = SourceRow::valid("Disqualified");
        dq.place = "DQ";
        let mut unsanctioned = SourceRow::valid("Unsanctioned");
        unsanctioned.sanctioned = "No";
        let mut no_bodyweight = SourceRow::valid("NoBodyweight");
        no_bodyweight.bodyweight = 0.0;
        let mut bench_only = SourceRow::valid("BenchOnly");
        bench_only.event = "B";

        let (_temp, records) = run_stage_2(&[
            SourceRow::valid("Ada"),
            dq,
            unsanctioned,
            no_bodyweight,
            bench_only,
        ]);

        // Only Ada has a squat that counts; the bench-only lifter has no squat
        // event, and the rest fail a validity filter.
        let squats = read_records(&records, "all", "squat");
        assert_eq!(squats.height(), 1);

        // ...but the bench-only lifter does belong in the bench table.
        let benches = read_records(&records, "all", "bench");
        assert_eq!(benches.height(), 2);
    }

    #[test]
    fn tested_cohort_excludes_untested_lifters() {
        let mut untested = SourceRow::valid("Untested");
        untested.tested = "";

        let (_temp, records) = run_stage_2(&[SourceRow::valid("Ada"), untested]);

        assert_eq!(read_records(&records, "all", "squat").height(), 2);
        assert_eq!(read_records(&records, "tested", "squat").height(), 1);
    }
}
