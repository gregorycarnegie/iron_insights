use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use pipeline::BuildMetadata;
use polars::prelude::*;
use serde::Serialize;

const HIST_MAGIC: [u8; 4] = *b"IIH1";
const HEAT_MAGIC: [u8; 4] = *b"IIM1";
const FORMAT_VERSION: u16 = 1;
const LIFT_BIN_BASE_KG: f32 = 2.5;
const BW_BIN_BASE_KG: f32 = 1.0;
const SCORE_BIN_BASE_POINTS: f32 = 2.5;
const ALL: &str = "All";
const ALL_AGES: &str = "All Ages";

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "pipeline/output/records")]
    records_dir: PathBuf,

    #[arg(long, default_value = "pipeline/output/build_metadata.json")]
    build_metadata_path: PathBuf,

    #[arg(long, default_value = "data")]
    data_dir: PathBuf,

    #[arg(long)]
    version: Option<String>,

    #[arg(long, default_value_t = 4)]
    keep_versions: usize,

    #[arg(long, default_value_t = false)]
    write_meta_files: bool,
}

#[derive(Debug, Serialize)]
struct LatestJson {
    version: String,
    revision: Option<String>,
}

#[derive(Debug, Serialize)]
struct SliceMeta {
    version: String,
    sex: String,
    equipment: String,
    ipf_weight_class: String,
    age_class: String,
    tested: String,
    lift: String,
    metric: String,
    hist: HistMeta,
    heat: HeatMeta,
}

#[derive(Debug, Serialize)]
struct HistMeta {
    file: String,
    base_bin_size_kg: f32,
    min_kg: f32,
    max_kg: f32,
    bins: usize,
    total: u64,
}

#[derive(Debug, Serialize)]
struct HeatMeta {
    file: String,
    x_base_bin_size_kg: f32,
    y_base_bin_size_kg: f32,
    min_x_kg: f32,
    max_x_kg: f32,
    min_y_kg: f32,
    max_y_kg: f32,
    width: usize,
    height: usize,
    total: u64,
}

#[derive(Debug)]
struct HistogramData {
    min: f32,
    max: f32,
    counts: Vec<u32>,
    total: u64,
}

#[derive(Debug)]
struct HeatmapData {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    width: usize,
    height: usize,
    grid: Vec<u32>,
    total: u64,
}

#[derive(Debug, Default)]
struct SliceAccumulator {
    lift_values: Vec<f32>,
    heat_points: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, Copy)]
enum Metric {
    Kg,
    Dots,
    Wilks,
    Gl,
}

#[derive(Debug, Serialize)]
struct RootIndex {
    version: String,
    shards: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct TrendsJson {
    version: String,
    bucket: String,
    series: Vec<TrendSeries>,
}

#[derive(Debug, Serialize)]
struct TrendSeries {
    key: String,
    points: Vec<TrendPoint>,
}

#[derive(Debug, Serialize)]
struct TrendPoint {
    year: i32,
    total: u32,
    p50: f32,
    p90: f32,
}

#[derive(Debug, Serialize)]
struct SliceIndex {
    version: String,
    shard_key: String,
    slices: BTreeMap<String, SliceIndexEntry>,
}

#[derive(Debug, Serialize)]
struct SliceIndexEntry {
    meta: String,
    hist: String,
    heat: String,
    summary: SliceSummary,
}

#[derive(Debug, Serialize)]
struct SliceSummary {
    min_kg: f32,
    max_kg: f32,
    total: u32,
}

struct PublishRecordsJob<'a> {
    records_path: &'a Path,
    version_dir: &'a Path,
    version: &'a str,
    tested: &'a str,
    lift: &'a str,
    write_meta_files: bool,
    shard_indices: &'a mut BTreeMap<String, BTreeMap<String, SliceIndexEntry>>,
    trends_acc: &'a mut BTreeMap<String, BTreeMap<i32, Vec<f32>>>,
}

type SliceKey<'a> = (&'a str, &'a str, &'a str, &'a str);

#[derive(Debug, Clone, Copy)]
struct AccumulationRow<'a> {
    sex: &'a str,
    equipment: &'a str,
    weight_class: &'a str,
    age_class: &'a str,
    year: Option<i32>,
    x_value: f32,
    valid_bw: Option<f32>,
}

#[derive(Debug)]
struct MetricPublisher<'a> {
    metric: Metric,
    trend_key_suffix: String,
    slices: HashMap<SliceKey<'a>, SliceAccumulator>,
}

impl<'a> MetricPublisher<'a> {
    fn new(metric: Metric, tested: &str, lift: &str) -> Self {
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

    fn accumulate_row(
        &mut self,
        row: AccumulationRow<'a>,
        trends_acc: &mut BTreeMap<String, BTreeMap<i32, Vec<f32>>>,
    ) {
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

    fn write_outputs(
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

            let hist_rel = format!(
                "hist/{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested}/{metric_slug}/{lift}.bin"
            );
            let heat_rel = format!(
                "heat/{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested}/{metric_slug}/{lift}.bin"
            );
            let meta_rel = format!(
                "meta/{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested}/{metric_slug}/{lift}.json"
            );

            let hist_path = version_dir.join(&hist_rel);
            let heat_path = version_dir.join(&heat_rel);
            let meta_path = version_dir.join(&meta_rel);

            write_hist_bin(&hist_path, &hist_data, x_base)?;
            write_heat_bin(&heat_path, &heat_data, x_base, BW_BIN_BASE_KG)?;

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
                    file: hist_rel.clone(),
                    base_bin_size_kg: x_base,
                    min_kg: hist_data.min,
                    max_kg: hist_data.max,
                    bins: hist_data.counts.len(),
                    total: hist_data.total,
                },
                heat: HeatMeta {
                    file: heat_rel.clone(),
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
                    hist: hist_rel,
                    heat: heat_rel,
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

fn main() -> Result<()> {
    let args = Args::parse();
    fs::create_dir_all(&args.data_dir)
        .with_context(|| format!("failed to create {}", args.data_dir.display()))?;

    let build_meta = read_optional_build_metadata(&args.build_metadata_path)?;
    let version = resolve_version(args.version, build_meta.as_ref());
    let version_dir = args.data_dir.join(&version);

    fs::create_dir_all(&version_dir)
        .with_context(|| format!("failed to create {}", version_dir.display()))?;

    let mut shard_indices = BTreeMap::<String, BTreeMap<String, SliceIndexEntry>>::new();
    let mut trends_acc = BTreeMap::<String, BTreeMap<i32, Vec<f32>>>::new();

    for tested in ["all", "tested"] {
        for lift in ["squat", "bench", "deadlift", "total"] {
            let records_path = args
                .records_dir
                .join(tested)
                .join(format!("{lift}.parquet"));
            if !records_path.exists() {
                continue;
            }

            publish_records_for_lift(PublishRecordsJob {
                records_path: &records_path,
                version_dir: &version_dir,
                version: &version,
                tested,
                lift,
                write_meta_files: args.write_meta_files,
                shard_indices: &mut shard_indices,
                trends_acc: &mut trends_acc,
            })?;
        }
    }

    let mut shard_paths = BTreeMap::<String, String>::new();
    for (shard_key, slice_keys) in shard_indices {
        let Some((sex, equipment)) = parse_shard_key(&shard_key) else {
            continue;
        };
        let shard_rel = format!("index_shards/{}/{}/index.json", slug(sex), slug(equipment));
        let shard_path = version_dir.join(&shard_rel);
        if let Some(parent) = shard_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed creating {}", parent.display()))?;
        }
        let shard_index = SliceIndex {
            version: version.clone(),
            shard_key: shard_key.clone(),
            slices: slice_keys.into_iter().collect(),
        };
        fs::write(&shard_path, serde_json::to_vec(&shard_index)?)
            .with_context(|| format!("failed writing {}", shard_path.display()))?;
        shard_paths.insert(shard_key, shard_rel);
    }

    let index = RootIndex {
        version: version.clone(),
        shards: shard_paths,
    };
    let index_path = version_dir.join("index.json");
    fs::write(&index_path, serde_json::to_vec(&index)?)
        .with_context(|| format!("failed writing {}", index_path.display()))?;

    let trends = build_trends_json(&version, trends_acc);
    let trends_path = version_dir.join("trends.json");
    fs::write(&trends_path, serde_json::to_vec(&trends)?)
        .with_context(|| format!("failed writing {}", trends_path.display()))?;

    let latest = LatestJson {
        version: version.clone(),
        revision: build_meta.and_then(|m| m.dataset_revision),
    };

    let latest_path = args.data_dir.join("latest.json");
    fs::write(&latest_path, serde_json::to_vec_pretty(&latest)?)
        .with_context(|| format!("failed writing {}", latest_path.display()))?;

    prune_old_versions(&args.data_dir, args.keep_versions)?;

    println!("Published data version: {version}");
    println!("Updated latest pointer: {}", latest_path.display());
    Ok(())
}

fn publish_records_for_lift(job: PublishRecordsJob<'_>) -> Result<()> {
    let PublishRecordsJob {
        records_path,
        version_dir,
        version,
        tested,
        lift,
        write_meta_files,
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

        let valid_bw = bw_col
            .get(i)
            .and_then(|bw| if bw > 0.0 { Some(bw) } else { None });
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
        publisher.write_outputs(
            version_dir,
            version,
            tested,
            lift,
            write_meta_files,
            shard_indices,
        )?;
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

fn build_trends_json(
    version: &str,
    trends_acc: BTreeMap<String, BTreeMap<i32, Vec<f32>>>,
) -> TrendsJson {
    let mut series = Vec::new();
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
        if !points.is_empty() {
            series.push(TrendSeries { key, points });
        }
    }
    series.sort_by(|a, b| a.key.cmp(&b.key));
    TrendsJson {
        version: version.to_string(),
        bucket: "year".to_string(),
        series,
    }
}

fn parse_year_bucket(value: Option<&str>) -> Option<i32> {
    let raw = value?;
    let year = raw.get(0..4)?.parse::<i32>().ok()?;
    if (1900..=2100).contains(&year) {
        Some(year)
    } else {
        None
    }
}

fn quantile_sorted(values: &[f32], q: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let q = q.clamp(0.0, 1.0);
    let idx = ((values.len() - 1) as f32 * q).round() as usize;
    values[idx]
}

fn build_histogram(values: &[f32], base: f32) -> Result<HistogramData> {
    let min_val = values
        .iter()
        .copied()
        .reduce(f32::min)
        .ok_or_else(|| anyhow::anyhow!("cannot build histogram for empty input"))?;
    let max_val = values
        .iter()
        .copied()
        .reduce(f32::max)
        .ok_or_else(|| anyhow::anyhow!("cannot build histogram for empty input"))?;

    let min_edge = (min_val / base).floor() * base;
    let max_edge = ((max_val / base).floor() + 1.0f32) * base;
    let bins = (((max_edge - min_edge) / base).round() as usize).max(1);

    let mut counts = vec![0u32; bins];
    for value in values {
        let raw = ((value - min_edge) / base).floor();
        let idx = raw.clamp(0.0f32, (bins - 1) as f32) as usize;
        counts[idx] = counts[idx].saturating_add(1);
    }

    Ok(HistogramData {
        min: min_edge,
        max: max_edge,
        total: values.len() as u64,
        counts,
    })
}

fn build_heatmap(points: &[(f32, f32)], x_base: f32, y_base: f32) -> Result<HeatmapData> {
    if points.is_empty() {
        return Ok(HeatmapData {
            min_x: 0.0f32,
            max_x: 0.0f32,
            min_y: 0.0f32,
            max_y: 0.0f32,
            width: 0,
            height: 0,
            grid: Vec::new(),
            total: 0,
        });
    }

    let (mut min_x, mut max_x) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);

    for (x, y) in points {
        min_x = min_x.min(*x);
        max_x = max_x.max(*x);
        min_y = min_y.min(*y);
        max_y = max_y.max(*y);
    }

    let min_x_edge = (min_x / x_base).floor() * x_base;
    let max_x_edge = ((max_x / x_base).floor() + 1.0f32) * x_base;
    let min_y_edge = (min_y / y_base).floor() * y_base;
    let max_y_edge = ((max_y / y_base).floor() + 1.0f32) * y_base;

    let width = (((max_x_edge - min_x_edge) / x_base).round() as usize).max(1);
    let height = (((max_y_edge - min_y_edge) / y_base).round() as usize).max(1);

    let mut grid = vec![0u32; width * height];
    for (x, y) in points {
        let ix = (((x - min_x_edge) / x_base).floor()).clamp(0.0f32, (width - 1) as f32) as usize;
        let iy = (((y - min_y_edge) / y_base).floor()).clamp(0.0f32, (height - 1) as f32) as usize;
        let idx = iy * width + ix;
        grid[idx] = grid[idx].saturating_add(1);
    }

    Ok(HeatmapData {
        min_x: min_x_edge,
        max_x: max_x_edge,
        min_y: min_y_edge,
        max_y: max_y_edge,
        width,
        height,
        total: points.len() as u64,
        grid,
    })
}

fn write_hist_bin(path: &Path, hist: &HistogramData, x_base: f32) -> Result<()> {
    let mut bytes = Vec::with_capacity(4 + 2 + (3 * 4) + 4 + hist.counts.len() * 4);
    bytes.extend_from_slice(&HIST_MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&x_base.to_le_bytes());
    bytes.extend_from_slice(&hist.min.to_le_bytes());
    bytes.extend_from_slice(&hist.max.to_le_bytes());
    bytes.extend_from_slice(&(hist.counts.len() as u32).to_le_bytes());
    for value in &hist.counts {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    write_bytes(path, &bytes)
}

fn write_heat_bin(path: &Path, heat: &HeatmapData, x_base: f32, y_base: f32) -> Result<()> {
    let mut bytes = Vec::with_capacity(4 + 2 + (6 * 4) + (2 * 4) + heat.grid.len() * 4);
    bytes.extend_from_slice(&HEAT_MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&x_base.to_le_bytes());
    bytes.extend_from_slice(&y_base.to_le_bytes());
    bytes.extend_from_slice(&heat.min_x.to_le_bytes());
    bytes.extend_from_slice(&heat.max_x.to_le_bytes());
    bytes.extend_from_slice(&heat.min_y.to_le_bytes());
    bytes.extend_from_slice(&heat.max_y.to_le_bytes());
    bytes.extend_from_slice(&(heat.width as u32).to_le_bytes());
    bytes.extend_from_slice(&(heat.height as u32).to_le_bytes());
    for value in &heat.grid {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    write_bytes(path, &bytes)
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating {}", parent.display()))?;
    }
    fs::write(path, bytes).with_context(|| format!("failed writing {}", path.display()))?;
    Ok(())
}

fn read_optional_build_metadata(path: &Path) -> Result<Option<BuildMetadata>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path).with_context(|| format!("failed reading {}", path.display()))?;
    let metadata: BuildMetadata = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed parsing {}", path.display()))?;
    Ok(Some(metadata))
}

fn resolve_version(cli: Option<String>, metadata: Option<&BuildMetadata>) -> String {
    if let Some(version) = cli {
        return normalize_version(&version);
    }

    if let Some(meta) = metadata {
        let normalized = normalize_version(&meta.dataset_version);
        if is_valid_effective_version(&normalized) {
            return normalized;
        }
    }

    let today = Utc::now().format("%Y-%m-%d").to_string();
    format!("v{today}")
}

fn normalize_version(version: &str) -> String {
    if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    }
}

fn is_valid_effective_version(version: &str) -> bool {
    if version == "vYYYY-MM-DD" || version == "v0000-00-00" {
        return false;
    }
    is_version_dir_name(version)
}

fn prune_old_versions(data_dir: &Path, keep_versions: usize) -> Result<()> {
    let mut versions: Vec<PathBuf> = fs::read_dir(data_dir)
        .with_context(|| format!("failed reading {}", data_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_version_dir_name)
        })
        .collect();

    versions.sort();

    if versions.len() <= keep_versions {
        return Ok(());
    }

    let to_remove = versions.len() - keep_versions;
    for old in versions.into_iter().take(to_remove) {
        fs::remove_dir_all(&old)
            .with_context(|| format!("failed removing old version {}", old.display()))?;
    }

    Ok(())
}

fn is_version_dir_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() == 11
        && bytes[0] == b'v'
        && bytes[5] == b'-'
        && bytes[8] == b'-'
        && bytes[1..5].iter().all(u8::is_ascii_digit)
        && bytes[6..8].iter().all(u8::is_ascii_digit)
        && bytes[9..11].iter().all(u8::is_ascii_digit)
}

fn slug(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' => c.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' | '-' => c,
            _ => '_',
        })
        .collect()
}

fn parse_shard_key(raw: &str) -> Option<(&str, &str)> {
    let mut sex = None;
    let mut equip = None;
    for part in raw.split('|') {
        let (k, v) = part.split_once('=')?;
        match k {
            "sex" => sex = Some(v),
            "equip" => equip = Some(v),
            _ => {}
        }
    }
    Some((sex?, equip?))
}

fn tested_bucket(tested: &str) -> &'static str {
    if tested == "tested" { "Yes" } else { "All" }
}

fn lift_code(lift: &str) -> &'static str {
    match lift {
        "squat" => "S",
        "bench" => "B",
        "deadlift" => "D",
        "total" => "T",
        _ => "U",
    }
}

fn metrics_for_lift(lift: &str) -> &'static [Metric] {
    if lift == "total" {
        &[Metric::Kg, Metric::Dots, Metric::Wilks, Metric::Gl]
    } else {
        &[Metric::Kg]
    }
}

fn metric_base_bin(metric: Metric) -> f32 {
    match metric {
        Metric::Kg => LIFT_BIN_BASE_KG,
        Metric::Dots | Metric::Wilks | Metric::Gl => SCORE_BIN_BASE_POINTS,
    }
}

fn metric_slug(metric: Metric) -> &'static str {
    match metric {
        Metric::Kg => "kg",
        Metric::Dots => "dots",
        Metric::Wilks => "wilks",
        Metric::Gl => "gl",
    }
}

fn metric_code(metric: Metric) -> &'static str {
    match metric {
        Metric::Kg => "Kg",
        Metric::Dots => "Dots",
        Metric::Wilks => "Wilks",
        Metric::Gl => "GL",
    }
}

fn metric_value(
    metric: Metric,
    lift: &str,
    sex: &str,
    equipment: &str,
    lift_value: f32,
    bodyweight_kg: Option<f32>,
) -> Option<f32> {
    match metric {
        Metric::Kg => Some(lift_value),
        Metric::Dots => {
            if lift != "total" {
                return None;
            }
            Some(dots_points(sex, bodyweight_kg?, lift_value))
        }
        Metric::Wilks => {
            if lift != "total" {
                return None;
            }
            Some(wilks_points(sex, bodyweight_kg?, lift_value))
        }
        Metric::Gl => {
            if lift != "total" {
                return None;
            }
            Some(goodlift_points(sex, equipment, bodyweight_kg?, lift_value))
        }
    }
}

#[allow(clippy::excessive_precision)]
fn dots_points(sex: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let bw = match sex {
        "F" => bodyweight_kg.clamp(40.0, 150.0),
        _ => bodyweight_kg.clamp(40.0, 210.0),
    };
    let denom = if sex == "F" {
        -57.96288 + 13.6175032 * bw - 0.1126655495 * bw.powi(2) + 0.0005158568 * bw.powi(3)
            - 0.0000010706 * bw.powi(4)
    } else {
        -307.75076 + 24.0900756 * bw - 0.1918759221 * bw.powi(2) + 0.0007391293 * bw.powi(3)
            - 0.0000010930 * bw.powi(4)
    };
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 500.0 / denom
    }
}

#[allow(clippy::excessive_precision)]
fn wilks_points(sex: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let bw = match sex {
        "F" => bodyweight_kg.clamp(26.51, 154.53),
        _ => bodyweight_kg.clamp(40.0, 201.9),
    };
    let denom = if sex == "F" {
        594.31747775582 - 27.23842536447 * bw + 0.82112226871 * bw.powi(2)
            - 0.00930733913 * bw.powi(3)
            + 0.00004731582 * bw.powi(4)
            - 0.00000009054 * bw.powi(5)
    } else {
        -216.0475144 + 16.2606339 * bw - 0.002388645 * bw.powi(2) - 0.00113732 * bw.powi(3)
            + 0.00000701863 * bw.powi(4)
            - 0.00000001291 * bw.powi(5)
    };
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 500.0 / denom
    }
}

#[allow(clippy::excessive_precision)]
fn goodlift_points(sex: &str, equipment: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let classic = matches!(equipment, "Raw" | "Wraps" | "Straps");
    let (a, b, c) = match (sex, classic) {
        ("F", true) => (610.32796, 1045.59282, 0.03048),
        ("F", false) => (758.63878, 949.31382, 0.02435),
        ("M", true) => (1199.72839, 1025.18162, 0.00921),
        _ => (1236.25115, 1449.21864, 0.01644),
    };
    let denom = a - (b * (-c * bodyweight_kg).exp());
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 100.0 / denom
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BW_BIN_BASE_KG, LIFT_BIN_BASE_KG, build_heatmap, build_histogram, dots_points,
        goodlift_points, parse_year_bucket, quantile_sorted, wilks_points,
    };

    #[test]
    fn dots_points_increase_with_total() {
        let low = dots_points("M", 90.0, 500.0);
        let high = dots_points("M", 90.0, 600.0);
        assert!(high > low);
        assert!(low > 0.0);
    }

    #[test]
    fn wilks_points_increase_with_total() {
        let low = wilks_points("F", 63.0, 350.0);
        let high = wilks_points("F", 63.0, 420.0);
        assert!(high > low);
        assert!(low > 0.0);
    }

    #[test]
    fn goodlift_points_differs_by_equipment() {
        let raw = goodlift_points("M", "Raw", 90.0, 700.0);
        let equipped = goodlift_points("M", "Single-ply", 90.0, 700.0);
        assert!(raw.is_finite());
        assert!(equipped.is_finite());
        assert_ne!(raw, equipped);
    }

    #[test]
    fn build_histogram_uses_expected_edges_and_total() {
        let values = vec![100.0, 101.0, 102.4, 104.9, 105.0];
        let hist = build_histogram(&values, LIFT_BIN_BASE_KG).expect("histogram should build");

        assert_eq!(hist.min, 100.0);
        assert_eq!(hist.max, 107.5);
        assert_eq!(hist.counts, vec![3, 1, 1]);
        assert_eq!(hist.total, 5);
        assert_eq!(
            hist.counts.iter().copied().map(u64::from).sum::<u64>(),
            hist.total
        );
    }

    #[test]
    fn build_heatmap_empty_is_zero_shape() {
        let heat =
            build_heatmap(&[], LIFT_BIN_BASE_KG, BW_BIN_BASE_KG).expect("heatmap should build");
        assert_eq!(heat.width, 0);
        assert_eq!(heat.height, 0);
        assert_eq!(heat.total, 0);
        assert!(heat.grid.is_empty());
    }

    #[test]
    fn build_heatmap_bins_points_and_preserves_total() {
        let points = vec![
            (100.0, 80.0),
            (101.0, 80.2),
            (102.4, 80.9),
            (104.9, 81.1),
            (105.0, 81.9),
        ];
        let heat =
            build_heatmap(&points, LIFT_BIN_BASE_KG, BW_BIN_BASE_KG).expect("heatmap should build");

        assert_eq!(heat.min_x, 100.0);
        assert_eq!(heat.max_x, 107.5);
        assert_eq!(heat.min_y, 80.0);
        assert_eq!(heat.max_y, 82.0);
        assert_eq!(heat.width, 3);
        assert_eq!(heat.height, 2);
        assert_eq!(heat.total, points.len() as u64);
        assert_eq!(
            heat.grid.iter().copied().map(u64::from).sum::<u64>(),
            heat.total
        );
    }

    #[test]
    fn parse_year_bucket_accepts_valid_dates() {
        assert_eq!(parse_year_bucket(Some("2026-03-07")), Some(2026));
        assert_eq!(parse_year_bucket(Some("1999-12-31")), Some(1999));
    }

    #[test]
    fn parse_year_bucket_rejects_invalid_dates() {
        assert_eq!(parse_year_bucket(Some("bad")), None);
        assert_eq!(parse_year_bucket(Some("1800-01-01")), None);
        assert_eq!(parse_year_bucket(None), None);
    }

    #[test]
    fn quantile_sorted_uses_nearest_rank_on_sorted_data() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(quantile_sorted(&values, 0.0), 10.0);
        assert_eq!(quantile_sorted(&values, 0.5), 30.0);
        assert_eq!(quantile_sorted(&values, 0.9), 50.0);
    }
}
