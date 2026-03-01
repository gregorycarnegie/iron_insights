use std::collections::BTreeMap;
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

#[derive(Debug, Serialize)]
struct RootIndex {
    version: String,
    shards: BTreeMap<String, String>,
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
    hist_min_kg: f32,
    hist_max_kg: f32,
    heat_min_x_kg: f32,
    heat_max_x_kg: f32,
    heat_min_y_kg: f32,
    heat_max_y_kg: f32,
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

    for tested in ["all", "tested"] {
        for lift in ["squat", "bench", "deadlift", "total"] {
            let records_path = args
                .records_dir
                .join(tested)
                .join(format!("{lift}.parquet"));
            if !records_path.exists() {
                continue;
            }

            publish_records_for_lift(
                &records_path,
                &version_dir,
                &version,
                tested,
                lift,
                &mut shard_indices,
            )?;
        }
    }

    let mut shard_paths = BTreeMap::<String, String>::new();
    for (shard_key, slices) in shard_indices {
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
            slices,
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

fn publish_records_for_lift(
    records_path: &Path,
    version_dir: &Path,
    version: &str,
    tested: &str,
    lift: &str,
    shard_indices: &mut BTreeMap<String, BTreeMap<String, SliceIndexEntry>>,
) -> Result<()> {
    let parquet_path = records_path.to_string_lossy();
    let df = LazyFrame::scan_parquet(parquet_path.as_ref().into(), ScanArgsParquet::default())
        .with_context(|| format!("failed scanning {}", records_path.display()))?
        .collect()
        .with_context(|| format!("failed collecting {}", records_path.display()))?;

    let sex_col = df
        .column("Sex")
        .context("missing Sex column")?
        .str()
        .context("Sex column not string")?;
    let equip_col = df
        .column("Equipment")
        .context("missing Equipment column")?
        .str()
        .context("Equipment column not string")?;
    let wc_col = df
        .column("IpfWeightClass")
        .context("missing IpfWeightClass column")?
        .str()
        .context("IpfWeightClass column not string")?;
    let age_col = df
        .column("AgeClassBucket")
        .context("missing AgeClassBucket column")?
        .str()
        .context("AgeClassBucket column not string")?;
    let lift_col = df
        .column("best_lift")
        .context("missing best_lift column")?
        .f32()
        .context("best_lift column not f32")?;
    let bw_col = df
        .column("bodyweight_at_best")
        .context("missing bodyweight_at_best column")?
        .f32()
        .context("bodyweight_at_best column not f32")?;

    let mut slices = BTreeMap::<(String, String, String, String), SliceAccumulator>::new();
    for i in 0..df.height() {
        let (Some(sex), Some(equipment), Some(weight_class), Some(age_class), Some(lift_value)) =
            (
                sex_col.get(i),
                equip_col.get(i),
                wc_col.get(i),
                age_col.get(i),
                lift_col.get(i),
            )
        else {
            continue;
        };
        if lift_value <= 0.0 {
            continue;
        }

        let sex = sex.to_string();
        let equipment = equipment.to_string();
        let weight_class = weight_class.to_string();
        let age_class = age_class.to_string();
        let valid_bw = bw_col
            .get(i)
            .and_then(|bw| if bw > 0.0 { Some(bw) } else { None });

        // Publish specific and roll-up slices so UI can offer "All" for equipment/wc/age.
        let keys = [
            (
                sex.clone(),
                equipment.clone(),
                weight_class.clone(),
                age_class.clone(),
            ),
            (
                sex.clone(),
                "All".to_string(),
                weight_class.clone(),
                age_class.clone(),
            ),
            (
                sex.clone(),
                equipment.clone(),
                "All".to_string(),
                age_class.clone(),
            ),
            (
                sex.clone(),
                "All".to_string(),
                "All".to_string(),
                age_class.clone(),
            ),
            (
                sex.clone(),
                equipment.clone(),
                weight_class.clone(),
                "All Ages".to_string(),
            ),
            (
                sex.clone(),
                "All".to_string(),
                weight_class.clone(),
                "All Ages".to_string(),
            ),
            (
                sex.clone(),
                equipment.clone(),
                "All".to_string(),
                "All Ages".to_string(),
            ),
            (
                sex,
                "All".to_string(),
                "All".to_string(),
                "All Ages".to_string(),
            ),
        ];
        for key in keys {
            let entry = slices.entry(key).or_default();
            entry.lift_values.push(lift_value);
            if let Some(bw_value) = valid_bw {
                entry.heat_points.push((lift_value, bw_value));
            }
        }
    }

    for ((sex, equipment, weight_class, age_class), acc) in slices {
        if acc.lift_values.is_empty() {
            continue;
        }

        let hist_data = build_histogram(&acc.lift_values, LIFT_BIN_BASE_KG)?;
        let heat_data = build_heatmap(&acc.heat_points, LIFT_BIN_BASE_KG, BW_BIN_BASE_KG)?;

        let sex_slug = slug(&sex);
        let equip_slug = slug(&equipment);
        let wc_slug = slug(&weight_class);
        let age_slug = slug(&age_class);

        let hist_rel = format!("hist/{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested}/{lift}.bin");
        let heat_rel = format!("heat/{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested}/{lift}.bin");
        let meta_rel = format!("meta/{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested}/{lift}.json");

        let hist_path = version_dir.join(&hist_rel);
        let heat_path = version_dir.join(&heat_rel);
        let meta_path = version_dir.join(&meta_rel);

        write_hist_bin(&hist_path, &hist_data)?;
        write_heat_bin(&heat_path, &heat_data)?;

        let meta = SliceMeta {
            version: version.to_string(),
            sex: sex.clone(),
            equipment: equipment.clone(),
            ipf_weight_class: weight_class.clone(),
            age_class: age_class.clone(),
            tested: tested.to_string(),
            lift: lift.to_string(),
            hist: HistMeta {
                file: hist_rel,
                base_bin_size_kg: LIFT_BIN_BASE_KG,
                min_kg: hist_data.min,
                max_kg: hist_data.max,
                bins: hist_data.counts.len(),
                total: hist_data.total,
            },
            heat: HeatMeta {
                file: heat_rel,
                x_base_bin_size_kg: LIFT_BIN_BASE_KG,
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

        if let Some(parent) = meta_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed creating {}", parent.display()))?;
        }
        fs::write(&meta_path, serde_json::to_vec(&meta)?)
            .with_context(|| format!("failed writing {}", meta_path.display()))?;

        let key = format!(
            "sex={}|equip={}|wc={}|age={}|tested={}|lift={}",
            sex,
            equipment,
            weight_class,
            age_class,
            tested_bucket(tested),
            lift_code(lift)
        );
        let shard_key = format!("sex={}|equip={}", sex, equipment);
        shard_indices
            .entry(shard_key)
            .or_default()
            .insert(
            key,
            SliceIndexEntry {
                meta: meta_rel,
                hist: hist_path
                    .strip_prefix(version_dir)
                    .unwrap_or(&hist_path)
                    .to_string_lossy()
                    .replace('\\', "/"),
                heat: heat_path
                    .strip_prefix(version_dir)
                    .unwrap_or(&heat_path)
                    .to_string_lossy()
                    .replace('\\', "/"),
                hist_min_kg: hist_data.min,
                hist_max_kg: hist_data.max,
                heat_min_x_kg: heat_data.min_x,
                heat_max_x_kg: heat_data.max_x,
                heat_min_y_kg: heat_data.min_y,
                heat_max_y_kg: heat_data.max_y,
            },
        );
    }

    Ok(())
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

fn write_hist_bin(path: &Path, hist: &HistogramData) -> Result<()> {
    let mut bytes = Vec::with_capacity(4 + 2 + (3 * 4) + 4 + hist.counts.len() * 4);
    bytes.extend_from_slice(&HIST_MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&LIFT_BIN_BASE_KG.to_le_bytes());
    bytes.extend_from_slice(&hist.min.to_le_bytes());
    bytes.extend_from_slice(&hist.max.to_le_bytes());
    bytes.extend_from_slice(&(hist.counts.len() as u32).to_le_bytes());
    for value in &hist.counts {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    write_bytes(path, &bytes)
}

fn write_heat_bin(path: &Path, heat: &HeatmapData) -> Result<()> {
    let mut bytes = Vec::with_capacity(4 + 2 + (6 * 4) + (2 * 4) + heat.grid.len() * 4);
    bytes.extend_from_slice(&HEAT_MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&LIFT_BIN_BASE_KG.to_le_bytes());
    bytes.extend_from_slice(&BW_BIN_BASE_KG.to_le_bytes());
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
        return normalize_version(version);
    }

    if let Some(meta) = metadata {
        let normalized = normalize_version(meta.dataset_version.clone());
        if is_valid_effective_version(&normalized) {
            return normalized;
        }
    }

    let today = Utc::now().format("%Y-%m-%d").to_string();
    format!("v{today}")
}

fn normalize_version(version: String) -> String {
    if version.starts_with('v') {
        version
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
                .is_some_and(|n| is_version_dir_name(n))
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
