//! The one test that runs all four stages back to back.
//!
//! Every other stage test drives one stage against a fixture shaped like its
//! input. That catches schema drift between neighbours, but not a stage dropped
//! from the workflow, nor a value that is silently transformed on the way
//! through. This runs a synthetic CSV in at stage 1 and asserts on the rendered
//! SEO page that comes out of stage 4 — the same path the weekly refresh takes,
//! minus the HTTP GET.

use std::{io::Write, path::Path};

use tempfile::TempDir;

use crate::{
    DEFAULT_ZIP_URL,
    aggregate::{AggregateArgs, build_aggregates},
    download::{DownloadArgs, convert_downloaded_zip},
    publish_data::{PublishArgs, publish},
    seo_geo::{SeoArgs, generate},
};

const HEADER: &str = "Name,Sex,Event,Equipment,Tested,Sanctioned,Place,Date,Age,BodyweightKg,\
Squat1Kg,Squat2Kg,Squat3Kg,Squat4Kg,Best3SquatKg,Bench1Kg,Bench2Kg,Bench3Kg,Bench4Kg,Best3BenchKg,\
Deadlift1Kg,Deadlift2Kg,Deadlift3Kg,Deadlift4Kg,Best3DeadliftKg,TotalKg,Wilks,McCulloch,\
Glossbrenner,IPFPoints,Dots,Goodlift";

/// Lifters per cohort, matching `test_support::wide_cohort`.
const COHORT: u16 = 160;

/// One cohort of lifters, spread as wide as a single IPF weight class allows.
///
/// Both axes are stretched deliberately: the published payload is a histogram
/// crossed with a bodyweight grid, so a narrow fixture encodes small enough to
/// be inlined into the shard index, leaving stage 4 no `.bin` to read. But the
/// bodyweights cannot spread past the class, because stage 2 *derives* the class
/// from them — that would split the cohort four ways and leave each part too
/// sparse to clear the threshold on its own.
fn cohort_rows(
    sex: &str,
    name_prefix: &str,
    bw_base: f32,
    bw_step: f32,
    squat_base: f32,
) -> String {
    let mut out = String::new();
    for i in 0..COHORT {
        let bw = bw_base + f32::from(i % 25) * bw_step;
        let step = f32::from(i % 40) * 5.0;
        out.push_str(&lifter_row(
            &format!("{name_prefix}{i}"),
            sex,
            "1",
            bw,
            squat_base + step,
            squat_base * 0.6 + step,
            squat_base * 1.2 + step,
        ));
    }
    out
}

fn lifter_row(
    name: &str,
    sex: &str,
    place: &str,
    bw: f32,
    squat: f32,
    bench: f32,
    deadlift: f32,
) -> String {
    let total = squat + bench + deadlift;
    // Empty attempt columns on purpose: they are exactly the all-null numeric
    // columns stage 1's schema override exists to keep Float32.
    format!(
        "{name},{sex},SBD,Raw,Yes,Yes,{place},2026-03-07,30,{bw},\
,,,,{squat},,,,,{bench},,,,,{deadlift},{total},,,,,\n"
    )
}

fn synthetic_csv() -> String {
    let mut csv = String::from(HEADER);
    csv.push('\n');
    // Each cohort fills one IPF weight class: 74 < bw <= 83 for the men,
    // 57 < bw <= 63 for the women. Both classes are listed in
    // `stats::MEN_CLASSES`/`WOMEN_CLASSES`, so both reach the standards table
    // stage 4 renders.
    csv.push_str(&cohort_rows("M", "Man", 74.5, 0.34, 100.0));
    csv.push_str(&cohort_rows("F", "Woman", 57.2, 0.23, 60.0));

    // Rows stage 2 must throw away. They also keep `Place` a string column: the
    // real dump carries these markers, and a fixture of pure numeric places
    // would infer as i64 and make stage 2's own filters fail to compile a plan.
    for (name, place) in [("Dq", "DQ"), ("Dd", "DD"), ("Ns", "NS")] {
        csv.push_str(&lifter_row(name, "M", place, 77.0, 400.0, 300.0, 400.0));
    }
    csv
}

fn zip_with_csv(path: &Path, csv: &str) {
    let file = std::fs::File::create(path).expect("create zip");
    let mut writer = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    writer
        .start_file("openpowerlifting-2026-07-31.csv", options)
        .expect("start zip entry");
    writer.write_all(csv.as_bytes()).expect("write zip entry");
    writer.finish().expect("finish zip");
}

#[test]
fn a_csv_dropped_in_at_stage_1_comes_out_as_a_rendered_page_at_stage_4() {
    let temp = TempDir::new().expect("temp dir");
    let tmp_dir = temp.path().join("tmp");
    let output_dir = temp.path().join("output");
    let records_dir = output_dir.join("records");
    let data_dir = temp.path().join("data");
    let web_dir = temp.path().join("web");
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    std::fs::create_dir_all(&output_dir).expect("create output dir");

    // --- 01: zip -> canonical parquet + build_metadata.json ---
    let zip_path = tmp_dir.join("openpowerlifting-latest.zip");
    zip_with_csv(&zip_path, &synthetic_csv());
    convert_downloaded_zip(
        &DownloadArgs {
            zip_url: DEFAULT_ZIP_URL.to_string(),
            temp_dir: tmp_dir,
            output_dir: output_dir.clone(),
            dataset_version: "2026-07-31".to_string(),
            dataset_revision: Some("chain-test".to_string()),
        },
        &zip_path,
    )
    .expect("stage 1");

    // --- 02: canonical parquet -> per-lifter records ---
    build_aggregates(&AggregateArgs {
        input_parquet: output_dir.join("openpowerlifting-latest.parquet"),
        output_dir: records_dir.clone(),
    })
    .expect("stage 2");

    // --- 03: records -> published tree. No `version` override: the version has
    // to come from the metadata stage 1 stamped, which is the only link between
    // those two stages and is otherwise untested. ---
    publish(&PublishArgs {
        records_dir,
        build_metadata_path: output_dir.join("build_metadata.json"),
        data_dir: data_dir.clone(),
        version: None,
        keep_versions: 4,
    })
    .expect("stage 3");

    let latest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(data_dir.join("latest.json")).expect("read latest"))
            .expect("latest.json is valid json");
    assert_eq!(
        latest["version"], "v2026-07-31",
        "stage 1's dataset_version did not reach stage 3"
    );
    assert_eq!(latest["revision"], "chain-test");

    // Every lifter that went in must be counted in the rolled-up cohort. This is
    // the arithmetic that survives four stages of filtering, grouping, binning
    // and encoding, so it is the single number worth pinning end to end.
    let bin = data_dir
        .join("v2026-07-31")
        .join("bin/m/raw/all/all_ages/all/kg/squat.bin");
    let bytes = std::fs::read(&bin).unwrap_or_else(|e| {
        panic!("read {}: {e} — if the file is simply absent, the fixture has drifted back under INLINE_THRESHOLD", bin.display())
    });
    let (hist, _heat) =
        iron_insights_core::parse_combined_bin(&bytes).expect("parse published bin");
    assert_eq!(
        hist.total,
        u32::from(COHORT),
        "published squat histogram lost lifters between the CSV and the bin"
    );

    // --- 04: published tree -> SEO pages ---
    generate(&SeoArgs {
        data_dir,
        web_dir: web_dir.clone(),
        base_url: "https://example.test/iron_insights/".to_string(),
        date_published: "2026-06-20".to_string(),
    })
    .expect("stage 4");

    let page = |slug: &str| {
        std::fs::read_to_string(web_dir.join("seo").join(slug).join("index.html"))
            .unwrap_or_else(|e| panic!("read {slug}: {e}"))
    };

    let standards = page("powerlifting-strength-standards");
    assert!(
        standards.contains("<table>"),
        "no data table, so stage 4 read no payloads"
    );
    // Both cohorts must appear as their own row. Matching the cell markup rather
    // than the bare label keeps this from passing on a "83 kg" that happens to
    // sit in the static copy.
    assert!(
        standards.contains("<tr><td>83 kg</td>"),
        "men's 83 kg class missing from the standards table"
    );
    assert!(
        standards.contains("<tr><td>63 kg</td>"),
        "women's 63 kg class missing from the standards table"
    );

    // Stage 4 counts squat/bench/deadlift for both sexes: 6 histograms of COHORT
    // lifters each. Quoting the whole sentence rather than the bare number is
    // deliberate — 960 on its own also occurs among the rendered figures.
    let sentence = format!(
        "{} individual squat, bench and deadlift results",
        6 * u32::from(COHORT)
    );
    let method = page("how-iron-insights-works");
    assert!(
        method.contains(&sentence),
        "expected `{sentence}`:\n{method}"
    );
}
