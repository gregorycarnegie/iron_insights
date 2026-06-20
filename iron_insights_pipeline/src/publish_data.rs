use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

mod accumulation;
mod constants;
mod histogram;
mod metric;
mod model;
mod records;
mod trends;
mod util;
mod versioning;

use model::{LatestJson, RootIndex, SliceIndex, SliceIndexEntry};
use records::{PublishRecordsJob, publish_records_for_lift};
use trends::build_trends_shards;
use util::{parse_shard_key, slug};
use versioning::{prune_old_versions, read_optional_build_metadata, resolve_version};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "iron_insights_pipeline/output/records")]
    records_dir: PathBuf,

    #[arg(
        long,
        default_value = "iron_insights_pipeline/output/build_metadata.json"
    )]
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

pub fn run() -> Result<()> {
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

    let trends_shards = build_trends_shards(&version, trends_acc);
    let mut trends_shard_paths = BTreeMap::<String, String>::new();
    for (shard_key, payload) in &trends_shards {
        let Some((sex, equip)) = parse_shard_key(shard_key) else {
            continue;
        };
        let rel = format!("trends_shards/{}/{}/trends.json", slug(sex), slug(equip));
        let path = version_dir.join(&rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed creating {}", parent.display()))?;
        }
        fs::write(&path, serde_json::to_vec(payload)?)
            .with_context(|| format!("failed writing {}", path.display()))?;
        trends_shard_paths.insert(shard_key.clone(), rel);
    }

    let index = RootIndex {
        version: version.clone(),
        shards: shard_paths,
        trends_shards: trends_shard_paths,
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

#[cfg(test)]
mod tests;
