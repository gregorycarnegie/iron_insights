use super::{Faq, Page};

pub(super) fn method_page() -> Page {
    Page {
        slug: "how-iron-insights-works",
        hash: "nerds",
        title: "How Iron Insights Calculates Strength Percentiles | Methodology",
        desc: "How Iron Insights computes strength percentiles: matched cohorts, empirical \
p50/p90 standards, and DOTS/Wilks/GL scoring from OpenPowerlifting data."
            .to_string(),
        h1: "How Iron Insights calculates strength percentiles",
        lead: "Iron Insights is built on one idea: compare a lifter only against people \
like them. Every percentile, strength standard and sex comparison is derived directly from \
[[COUNT]] competition results in the OpenPowerlifting database &mdash; no hand-tuned charts or \
rules of thumb. This page documents the method."
            .to_string(),
        sections: vec![
            (
                "Matched cohorts",
                "<p>A lift is never ranked against everyone at once. Iron Insights groups \
results by <strong>sex, weight class, equipment</strong> (raw, single-ply, multi-ply, wraps \
and so on), <strong>tested status</strong> and <strong>age band</strong>, then ranks your \
number inside that cohort. A 75&nbsp;kg raw tested lifter is measured against other \
75&nbsp;kg raw tested lifters.</p>"
                    .to_string(),
            ),
            (
                "Percentiles and standards",
                "<p>Within a cohort, each lift sits in a histogram of competition results. \
Your percentile is the share of that cohort you exceed. The published strength standards are \
read off the same distribution: the <strong>median (p50)</strong> is the middle lifter and \
the <strong>90th percentile (p90)</strong> marks a strong one. Because they come from real \
results, the standards update automatically as the dataset grows.</p>"
                    .to_string(),
            ),
            (
                "Bodyweight-adjusted scoring",
                "<p>To compare lifters of different sizes, totals are also scored with \
<strong>DOTS</strong>, <strong>Wilks</strong> and <strong>Goodlift (GL)</strong> \
coefficients. These formulas weight a total against bodyweight so a 60&nbsp;kg and a \
120&nbsp;kg lifter land on one scale.</p>"
                    .to_string(),
            ),
            (
                "Data and refresh",
                "<p>The source is the public-domain OpenPowerlifting results database, \
currently [[COUNT_EXACT]] individual squat, bench and deadlift results (data current as of \
[[DATA_DATE]]). The dataset is refreshed on a roughly weekly cadence and every figure on the \
site is recomputed from it.</p>"
                    .to_string(),
            ),
        ],
        faqs: vec![
            Faq {
                q: "Where does the data come from?",
                a: "All figures are computed from OpenPowerlifting, the public-domain \
competition results database. Iron Insights is independent and not affiliated with \
OpenPowerlifting.",
            },
            Faq {
                q: "How often is it updated?",
                a: "The published dataset is refreshed on a roughly weekly cadence, and \
percentiles, standards and comparisons are recomputed each time.",
            },
            Faq {
                q: "Does my data leave my device?",
                a: "No. The calculators run entirely in your browser; your numbers are not \
uploaded or stored.",
            },
        ],
        howto: None,
    }
}
