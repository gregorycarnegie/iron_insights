use super::{
    content::build_pages,
    formatting::{group_thousands, human_date},
    render::{Site, escape, render_page, render_robots, render_sitemap, strip_for_jsonld},
    snippets::{ordinal, wc_label},
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
