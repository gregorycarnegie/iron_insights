use super::{
    content::build_pages,
    formatting::{group_thousands, human_date},
    render::{Site, escape, render_page, render_robots, render_sitemap, strip_for_jsonld},
    snippets::{
        comparison_table_html, ordinal, percentile_examples_html, standards_tables_html, wc_label,
    },
    stats::{percentile_value, value_percentile},
};
use iron_insights_core::HistogramBin;

fn test_site() -> Site<'static> {
    Site {
        base: "https://example.test/iron_insights/",
        lastmod: "2026-07-31",
        data_date: "31 July 2026",
        date_published: "2026-06-20",
        count_human: "2.8 million",
        count_exact: "2,844,167",
    }
}

#[test]
fn human_date_formats_iso() {
    assert_eq!(human_date("2026-06-19"), "19 June 2026");
    assert_eq!(human_date("2026-01-01"), "1 January 2026");
}

#[test]
fn human_date_passes_through_bad_input() {
    assert_eq!(human_date("nope"), "nope");
    assert_eq!(human_date("2026-13-01"), "2026-13-01");
}

#[test]
fn group_thousands_inserts_separators() {
    assert_eq!(group_thousands(0), "0");
    assert_eq!(group_thousands(999), "999");
    assert_eq!(group_thousands(1000), "1,000");
    assert_eq!(group_thousands(2844167), "2,844,167");
}

#[test]
fn ordinal_uses_correct_suffix() {
    assert_eq!(ordinal(1), "1st");
    assert_eq!(ordinal(2), "2nd");
    assert_eq!(ordinal(3), "3rd");
    assert_eq!(ordinal(11), "11th");
    assert_eq!(ordinal(12), "12th");
    assert_eq!(ordinal(13), "13th");
    assert_eq!(ordinal(21), "21st");
    assert_eq!(ordinal(45), "45th");
    assert_eq!(ordinal(90), "90th");
}

#[test]
fn wc_label_handles_plus_classes() {
    assert_eq!(wc_label("93"), "93 kg");
    assert_eq!(wc_label("120_"), "120 kg+");
    assert_eq!(wc_label("84_"), "84 kg+");
}

#[test]
fn strip_for_jsonld_drops_tags_and_unescapes() {
    assert_eq!(
        strip_for_jsonld("nothing to install &mdash; <em>runs</em> here"),
        "nothing to install \u{2014} runs here"
    );
}

#[test]
fn escape_encodes_html_specials() {
    assert_eq!(
        escape("a & b < c > d \" e"),
        "a &amp; b &lt; c &gt; d &quot; e"
    );
}

#[test]
fn percentile_value_reads_midpoint() {
    // Shares the app's mid-bin convention (iron_insights_core), so an SEO page
    // and the ranking view quote the same number for the same lift: a bin's
    // occupants count as sitting at its centre, not its edge.
    // 10 in [100,102.5), 10 in [102.5,105).
    let hist = HistogramBin::new(100.0, 105.0, 2.5, vec![10, 10]);
    assert_eq!(percentile_value(&hist, 0.5), 103.75);
    assert_eq!(value_percentile(&hist, 102.5), 75.0);
}

// ===== RENDERED PAGE INVARIANTS =====
//
// These run against every page `build_pages` produces, so a page added later is
// covered without touching the tests. `stats: None` is the degraded path taken
// when the published data is missing, which is also the path least likely to be
// exercised by hand.

#[test]
fn rendered_pages_leave_no_unsubstituted_placeholders() {
    let site = test_site();

    // Content carries `[[TOKEN]]` markers that `render::substitute` fills in.
    // A token added to copy but not to `substitute` would ship raw to readers,
    // and nothing else would notice.
    for pages in [build_pages(None), build_pages(None)] {
        for page in &pages {
            let html = render_page(&site, &pages, page);
            let leftover = html.find("[[").map(|i| {
                let end = (i + 40).min(html.len());
                html[i..end].to_string()
            });
            assert!(
                leftover.is_none(),
                "page `{}` shipped an unsubstituted placeholder: {:?}",
                page.slug,
                leftover
            );
        }
    }
}

#[test]
fn page_slugs_are_unique_and_url_safe() {
    let pages = build_pages(None);
    assert!(!pages.is_empty(), "there should be pages to publish");

    let mut seen = std::collections::BTreeSet::new();
    for page in &pages {
        // Output is written to `seo/<slug>/index.html`, so a duplicate slug
        // silently overwrites the earlier page on disk.
        assert!(seen.insert(page.slug), "duplicate page slug: {}", page.slug);
        assert!(!page.slug.is_empty(), "empty slug");
        assert!(
            page.slug
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "slug is not url-safe: {}",
            page.slug
        );
    }
}

#[test]
fn every_page_is_listed_in_the_sitemap() {
    let site = test_site();
    let pages = build_pages(None);
    let sitemap = render_sitemap(site.base, site.lastmod, &pages);

    for page in &pages {
        // Trunk's copy-dir lands each page directory at the deploy root, so the
        // canonical URL carries no `seo/` segment even though the source does.
        let url = format!("{}{}/", site.base, page.slug);
        assert!(
            sitemap.contains(&url),
            "page `{}` is published but missing from sitemap.xml",
            page.slug
        );
    }
    assert!(sitemap.contains(site.lastmod), "sitemap lacks lastmod");
}

#[test]
fn every_page_is_wired_up_for_deployment() {
    // `iron_insights_web/index.html` names each SEO directory in a `copy-dir`
    // link, one per page. Trunk copies only what is listed, so a page added to
    // `build_pages` without a matching link is generated on every refresh and
    // then silently dropped from the deploy — it would 404 while sitting in the
    // sitemap. Stage 4 writes into this file's directory by default, so the two
    // are already coupled; this keeps them honest.
    let index_html = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("iron_insights_web")
        .join("index.html");
    let markup = std::fs::read_to_string(&index_html)
        .unwrap_or_else(|e| panic!("read {}: {e}", index_html.display()));

    for page in build_pages(None) {
        let link = format!("href=\"seo/{}\"", page.slug);
        assert!(
            markup.contains(&link),
            "page `{}` has no copy-dir link in index.html, so it would never deploy",
            page.slug
        );
    }
}

#[test]
fn robots_points_crawlers_at_the_sitemap() {
    let site = test_site();
    let robots = render_robots(site.base);

    assert!(
        robots.contains(&format!("{}sitemap.xml", site.base)),
        "robots.txt must advertise the sitemap:\n{robots}"
    );
}

// ===== END-TO-END GENERATION =====
//
// Stage 4 reads the tree stage 3 publishes, so these run a real publish first
// rather than hand-building a `bin/` directory. That makes them a test of the
// seam as much as of stage 4: a change to the published layout that stage 4 is
// not taught about fails here.

use crate::seo_geo::{SeoArgs, generate};
use crate::test_support::{publish_tree, wide_cohort};

/// Publishes a cohort wide enough to survive as real `.bin` files, then
/// generates the site into a temp web dir.
fn generate_site() -> (tempfile::TempDir, std::path::PathBuf) {
    let (temp, data_dir) = publish_tree(
        "v2026-07-31",
        &[
            ("squat", wide_cohort("M", "83", 100.0)),
            ("bench", wide_cohort("M", "83", 60.0)),
            ("deadlift", wide_cohort("F", "63", 80.0)),
            ("total", wide_cohort("M", "83", 300.0)),
        ],
    );
    let web_dir = temp.path().join("web");

    generate(&SeoArgs {
        data_dir,
        web_dir: web_dir.clone(),
        base_url: "https://example.test/iron_insights/".to_string(),
        date_published: "2026-06-20".to_string(),
    })
    .expect("stage 4 should succeed");

    (temp, web_dir)
}

#[test]
fn writes_every_page_plus_robots_and_sitemap() {
    let (_temp, web_dir) = generate_site();

    for page in build_pages(None) {
        let out = web_dir.join("seo").join(page.slug).join("index.html");
        assert!(out.is_file(), "stage 4 did not write {}", out.display());

        let html = std::fs::read_to_string(&out).expect("read page");
        assert!(html.starts_with("<!doctype html>"), "{} lacks a doctype", page.slug);
        assert!(
            html.contains("application/ld+json"),
            "{} lost its structured data",
            page.slug
        );
    }

    assert!(web_dir.join("robots.txt").is_file());
    assert!(web_dir.join("sitemap.xml").is_file());
}

#[test]
fn pages_carry_figures_read_back_from_the_published_bins() {
    let (_temp, web_dir) = generate_site();
    let html = std::fs::read_to_string(
        web_dir
            .join("seo")
            .join("powerlifting-strength-standards")
            .join("index.html"),
    )
    .expect("read standards page");

    // The whole point of stage 4 is injecting real figures. If the fixture were
    // small enough to be inlined into the index, `load_stats` would find no
    // `.bin` files, silently fall back, and this page would ship the generic
    // copy — so assert the data actually made the round trip.
    assert!(
        html.contains("<table>"),
        "standards page has no data table, so stats did not load"
    );
    assert!(
        !html.contains("over 2 million"),
        "page fell back to the placeholder count instead of the real one"
    );
    assert!(
        html.contains("83"),
        "the published 83 kg cohort is missing from the standards table"
    );
}

#[test]
fn degrades_to_generic_copy_when_the_published_bins_are_missing() {
    let (temp, data_dir) = publish_tree("v2026-07-31", &[("squat", wide_cohort("M", "83", 100.0))]);

    // latest.json still points at the version, but the payloads are gone. This
    // is what a partial or interrupted publish looks like, and stage 4 must
    // still produce a site rather than panicking or writing nothing.
    std::fs::remove_dir_all(data_dir.join("v2026-07-31").join("bin")).expect("drop bins");

    let web_dir = temp.path().join("web-degraded");
    generate(&SeoArgs {
        data_dir,
        web_dir: web_dir.clone(),
        base_url: "https://example.test/iron_insights/".to_string(),
        date_published: "2026-06-20".to_string(),
    })
    .expect("stage 4 should still succeed without payloads");

    let html = std::fs::read_to_string(
        web_dir
            .join("seo")
            .join("strength-percentile-calculator")
            .join("index.html"),
    )
    .expect("read percentile page");
    assert!(html.contains("over 2 million"), "expected the fallback count");
}

// ===== SNIPPET RENDERING =====
//
// These build the figure tables from a `Stats` value. Driving them with a
// synthetic one pins the arithmetic that turns two medians into a percentage,
// which the page-level invariants above deliberately do not look at.

fn synthetic_stats() -> crate::seo_geo::stats::Stats {
    use crate::seo_geo::stats::{ClassRow, Example, Stats};

    Stats {
        men: vec![ClassRow {
            wc: "83",
            cells: [Some((200.0, 260.0)), None, None, None],
        }],
        women: vec![ClassRow {
            wc: "63",
            cells: [Some((100.0, 140.0)), None, None, None],
        }],
        // Women's median is 60% of men's: 100 * 120 / 200.
        comparison: vec![("squat", 200.0, 120.0, 60.0)],
        dots_m: Some(400.0),
        dots_f: Some(360.0),
        result_count: 2_844_167,
        // Both sexes, so the male/female wording branch is exercised.
        examples: vec![
            Example {
                sex: "m",
                wc: "83",
                lift: "squat",
                value: 180.0,
                pct: 42.0,
            },
            Example {
                sex: "f",
                wc: "63",
                lift: "deadlift",
                value: 150.0,
                pct: 71.0,
            },
        ],
    }
}

#[test]
fn comparison_table_reports_the_female_to_male_ratio() {
    let html = comparison_table_html(&synthetic_stats());

    assert!(html.contains("Squat"), "lift label missing:\n{html}");
    assert!(html.contains("200 kg") && html.contains("120 kg"), "{html}");
    // The ratio column and the DOTS note are both `100 * f / m`; a swapped
    // operator here would publish a plausible but wrong headline number.
    assert!(html.contains("60%"), "expected the 60% ratio:\n{html}");
    assert!(html.contains("about 90%"), "expected the 90% DOTS gap:\n{html}");
}

#[test]
fn comparison_table_is_empty_without_figures() {
    let mut stats = synthetic_stats();
    stats.comparison.clear();
    assert!(comparison_table_html(&stats).is_empty());
}

#[test]
fn percentile_examples_render_each_worked_case() {
    let html = percentile_examples_html(&synthetic_stats());

    assert!(html.contains("83 kg"), "weight class label missing:\n{html}");
    assert!(html.contains("180 kg squat"), "{html}");
    assert!(html.contains("42nd percentile"), "{html}");

    // The sex word is chosen by a string compare; both branches must render.
    assert!(html.contains("83 kg male"), "male wording missing:\n{html}");
    assert!(html.contains("63 kg female"), "female wording missing:\n{html}");
    assert!(html.contains("71st percentile"), "{html}");

    let mut empty = synthetic_stats();
    empty.examples.clear();
    assert!(percentile_examples_html(&empty).is_empty());
}

#[test]
fn standards_tables_label_both_sexes() {
    let html = standards_tables_html(&synthetic_stats());
    assert!(html.contains("83"), "men's class missing:\n{html}");
    assert!(html.contains("63"), "women's class missing:\n{html}");
}

#[test]
fn fails_loudly_when_there_is_nothing_published() {
    let temp = tempfile::TempDir::new().expect("temp dir");

    let result = generate(&SeoArgs {
        data_dir: temp.path().join("empty"),
        web_dir: temp.path().join("web"),
        base_url: "https://example.test/iron_insights/".to_string(),
        date_published: "2026-06-20".to_string(),
    });

    // Publishing a site built from no data at all would quietly replace good
    // pages with empty ones on the next deploy.
    assert!(result.is_err(), "missing latest.json must be an error");
}
