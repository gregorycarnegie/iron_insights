use super::{
    formatting::{group_thousands, human_date},
    render::{escape, strip_for_jsonld},
    snippets::{ordinal, wc_label},
    stats::{percentile_value, value_percentile},
};
use iron_insights_core::HistogramBin;

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
