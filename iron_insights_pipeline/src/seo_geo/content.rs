use super::{
    snippets::{comparison_table_html, percentile_examples_html, standards_tables_html},
    stats::Stats,
};

mod calculators;
mod method;
mod standards;

use calculators::{barbell_plate_page, body_fat_page, one_rep_max_page, strength_percentile_page};
use method::method_page;
use standards::{sex_comparison_page, strength_standards_page};

pub(super) struct Faq {
    pub(super) q: &'static str,
    pub(super) a: &'static str,
}

pub(super) struct HowTo {
    pub(super) title: &'static str,
    pub(super) steps: Vec<&'static str>,
}

pub(super) struct Page {
    pub(super) slug: &'static str,
    pub(super) hash: &'static str,
    pub(super) title: &'static str,
    pub(super) desc: String,
    pub(super) h1: &'static str,
    pub(super) lead: String,
    pub(super) sections: Vec<(&'static str, String)>,
    pub(super) faqs: Vec<Faq>,
    pub(super) howto: Option<HowTo>,
}

const TIER_TABLE: &str = "<table><thead><tr><th>Percentile</th><th>Tier</th>\
<th>What it means</th></tr></thead><tbody>\
<tr><td>Top 1% (99th+)</td><td>Legend</td><td>National / world-class numbers</td></tr>\
<tr><td>95th-99th</td><td>Elite</td><td>Competitive at a high level</td></tr>\
<tr><td>80th-95th</td><td>Advanced</td><td>Strong, years of focused training</td></tr>\
<tr><td>60th-80th</td><td>Intermediate</td><td>Above the typical lifter</td></tr>\
<tr><td>Below 60th</td><td>Novice</td><td>Early in the strength journey</td></tr>\
</tbody></table>";

pub(super) fn build_pages(stats: Option<&Stats>) -> Vec<Page> {
    let standards = stats.map(standards_tables_html).unwrap_or_default();
    let comparison = stats.map(comparison_table_html).unwrap_or_default();
    let examples = stats.map(percentile_examples_html).unwrap_or_default();

    vec![
        strength_percentile_page(&examples),
        strength_standards_page(&standards),
        sex_comparison_page(&comparison),
        one_rep_max_page(),
        barbell_plate_page(),
        body_fat_page(),
        method_page(),
    ]
}
