//! Stage 4: generate the static, crawlable SEO/GEO landing pages.
//!
//! Iron Insights ships as a client-side WASM app, so search engines and AI
//! answer engines see only a loading skeleton at `/`. These hand-authored
//! static pages give every priority query a real, answer-first HTML document
//! that (a) is indexable, (b) carries structured data + source citations, and
//! (c) links into the matching app view (`/#ranking`, `/#1rm`, ...).
//!
//! The concrete figures (medians, elite thresholds, sex gaps, dataset size)
//! are read straight from the published `.bin` cohort histograms produced by
//! `03_publish_data`, so they regenerate with every weekly refresh instead of
//! going stale. This is a Rust port of the former `scripts/build_seo_pages.py`.
//!
//! Output is plain static files under `<web-dir>/seo/<slug>/index.html` plus
//! `robots.txt` and `sitemap.xml`; trunk copies them into the deploy dir via
//! the copy-dir/copy-file links in `index.html`.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;

mod assets;
mod constants;
mod content;
mod formatting;
mod render;
mod snippets;
mod stats;

use constants::DATEPUBLISHED_DEFAULT;
use content::build_pages;
use formatting::{group_thousands, human_date};
use render::{Site, render_page, render_robots, render_sitemap};
use stats::load_stats;

#[derive(Debug, Parser)]
#[command(about = "Generate the static SEO/GEO landing pages from published data")]
struct Args {
    /// Published data dir (source of truth, read for concrete figures).
    #[arg(long, default_value = "data")]
    data_dir: PathBuf,

    /// Web crate dir: receives seo/, robots.txt and sitemap.xml.
    #[arg(long, default_value = "iron_insights_web")]
    web_dir: PathBuf,

    /// Canonical site origin (with trailing slash).
    #[arg(
        long,
        default_value = "https://gregorycarnegie.github.io/iron_insights/"
    )]
    base_url: String,

    /// schema.org datePublished for the pages.
    #[arg(long, default_value = DATEPUBLISHED_DEFAULT)]
    date_published: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct LatestJson {
    pub(super) version: String,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    let latest_path = args.data_dir.join("latest.json");
    let latest: LatestJson = serde_json::from_slice(
        &fs::read(&latest_path)
            .with_context(|| format!("failed reading {}", latest_path.display()))?,
    )
    .with_context(|| format!("failed parsing {}", latest_path.display()))?;

    let lastmod = latest.version.trim_start_matches('v').to_string();
    let data_date = human_date(&lastmod);
    let bins = args.data_dir.join(&latest.version).join("bin");

    let stats = load_stats(&bins);
    match &stats {
        Some(s) => println!(
            "loaded stats from {}: {} results",
            bins.display(),
            group_thousands(s.result_count)
        ),
        None => println!(
            "WARNING: published data not found at {}; data tables will be omitted",
            bins.display()
        ),
    }

    let (count_human, count_exact) = match &stats {
        Some(s) if s.result_count > 0 => (
            format!("{:.1} million", s.result_count as f64 / 1e6),
            group_thousands(s.result_count),
        ),
        _ => ("2 million".to_string(), "over 2 million".to_string()),
    };

    let site = Site {
        base: &args.base_url,
        lastmod: &lastmod,
        data_date: &data_date,
        date_published: &args.date_published,
        count_human: &count_human,
        count_exact: &count_exact,
    };

    let pages = build_pages(stats.as_ref());

    let seo_dir = args.web_dir.join("seo");
    for page in &pages {
        let out = seo_dir.join(page.slug).join("index.html");
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed creating {}", parent.display()))?;
        }
        fs::write(&out, render_page(&site, &pages, page))
            .with_context(|| format!("failed writing {}", out.display()))?;
        println!("wrote {}", out.display());
    }

    let robots_path = args.web_dir.join("robots.txt");
    fs::write(&robots_path, render_robots(&args.base_url))
        .with_context(|| format!("failed writing {}", robots_path.display()))?;
    println!("wrote {}", robots_path.display());

    let sitemap_path = args.web_dir.join("sitemap.xml");
    fs::write(
        &sitemap_path,
        render_sitemap(&args.base_url, &lastmod, &pages),
    )
    .with_context(|| format!("failed writing {}", sitemap_path.display()))?;
    println!("wrote {}", sitemap_path.display());

    Ok(())
}

#[cfg(test)]
mod tests;
