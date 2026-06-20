use super::{Faq, Page};

pub(super) fn strength_standards_page(standards: &str) -> Page {
    Page {
        slug: "powerlifting-strength-standards",
        hash: "nerds",
        title: "Powerlifting Strength Standards by Weight Class | Iron Insights",
        desc: "Median (p50) and elite (p90) squat, bench, deadlift and total thresholds by \
sex, weight class and equipment, from [[COUNT]] OpenPowerlifting results."
            .to_string(),
        h1: "Powerlifting strength standards by weight class and bodyweight",
        lead: "Strength standards answer 'how much should I lift for my bodyweight?'. Iron \
Insights derives them empirically: for every sex, weight class and equipment cohort it \
publishes the median (50th percentile) and a strong-lifter threshold (90th percentile) for \
squat, bench, deadlift and total, drawn from [[COUNT]] OpenPowerlifting competition results."
            .to_string(),
        sections: vec![
            (
                "Raw strength standards by weight class",
                format!(
                    "<p>These thresholds come straight from the distribution of competition \
lifters, so the median is literally the middle lifter in the cohort and the 90th-percentile \
line is what the top tenth pull.</p>{standards}"
                ),
            ),
            (
                "Bodyweight-adjusted scoring",
                "<p>To compare lifters of different bodyweights, totals are also expressed \
as DOTS, Wilks and Goodlift (GL) points. The Stats view shows the full distribution curve and \
a two-axis heatmap of lift versus bodyweight, plus year-over-year trend lines.</p>"
                    .to_string(),
            ),
        ],
        faqs: vec![
            Faq {
                q: "How much should I be able to squat for my weight?",
                a: "Look up your sex, weight class and equipment cohort: the median line is \
a typical competitive lifter and the 90th-percentile line marks a strong one.",
            },
            Faq {
                q: "What is DOTS / Wilks / GL?",
                a: "They are formulas that adjust a total for bodyweight so a 60kg and a \
120kg lifter can be compared on one scale.",
            },
        ],
        howto: None,
    }
}

pub(super) fn sex_comparison_page(comparison: &str) -> Page {
    Page {
        slug: "male-vs-female-strength-comparison",
        hash: "men-vs-women",
        title: "Men vs Women Strength Comparison | Iron Insights",
        desc: "How male and female powerlifting strength compares in matched weight-class \
and equipment cohorts, using DOTS-normalised distributions from [[COUNT]] results."
            .to_string(),
        h1: "How does male and female powerlifting strength compare?",
        lead: "Iron Insights compares men and women in <em>aligned</em> cohorts &mdash; the \
same weight class, equipment and lift &mdash; and overlays their full strength distributions. \
Because raw kilograms favour heavier lifters, the comparison is also shown in \
bodyweight-adjusted DOTS points so the gap reflects strength, not size."
            .to_string(),
        sections: vec![
            ("Median strength by lift", comparison.to_string()),
            (
                "Distributions overlap more than the medians suggest",
                "<p>A single ratio hides the overlap: the male and female distributions \
share a wide middle, so a strong woman out-lifts most men and vice versa. The live Sex \
Comparison view overlays both curves for the lift and cohort you pick so you can see where \
they meet and where they diverge.</p>"
                    .to_string(),
            ),
        ],
        faqs: vec![Faq {
            q: "How much stronger are men than women in powerlifting?",
            a: "Across raw lifters the median woman's bench is about 47% of the median \
man's, the squat about 58% and the deadlift about 59% &mdash; so the gap is largest in \
upper-body pressing. Once totals are adjusted for bodyweight with DOTS the gap narrows to \
roughly 88%, because the median female lifter is lighter.",
        }],
        howto: None,
    }
}
