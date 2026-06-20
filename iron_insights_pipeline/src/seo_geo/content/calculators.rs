use super::{Faq, HowTo, Page, TIER_TABLE};

pub(super) fn strength_percentile_page(examples: &str) -> Page {
    Page {
        slug: "strength-percentile-calculator",
        hash: "ranking",
        title: "Powerlifting Strength Percentile Calculator | Iron Insights",
        desc: "Find the exact percentile of your squat, bench, deadlift or total against \
[[COUNT]] competition results by sex, bodyweight and equipment. Free, no signup."
            .to_string(),
        h1: "What percentile is my squat, bench, deadlift or total?",
        lead: "Iron Insights ranks your lift against more than [[COUNT]] competition \
results from the OpenPowerlifting database. Enter your bodyweight and lifts and it returns \
your exact percentile within your sex, weight-class and equipment cohort, then maps it to a \
strength tier from Novice to Legend."
            .to_string(),
        sections: vec![
            (
                "Strength tiers by percentile",
                format!(
                    "<p>Iron Insights labels every result with a tier so a raw percentile \
is easy to read:</p>{TIER_TABLE}"
                ),
            ),
            (
                "How the percentile is calculated",
                format!(
                    "<p>Your number is compared against the distribution of a \
<strong>matched cohort</strong> &mdash; same sex, weight class, equipment (raw, single-ply, \
etc.), tested status and age band &mdash; not against everyone at once. For the total you can \
rank by raw kilograms or by the DOTS, Wilks or Goodlift (GL) bodyweight-adjusted scores.</p>\
{examples}"
                ),
            ),
        ],
        faqs: vec![
            Faq {
                q: "What is a good powerlifting total?",
                a: "It depends on bodyweight, sex and equipment. Iron Insights treats \
anything above the 80th percentile of your cohort as Advanced and above the 95th as Elite. \
Compare against your own weight class rather than an absolute number.",
            },
            Faq {
                q: "How are the percentiles worked out?",
                a: "Each lift is placed in the distribution of lifters who share your sex, \
weight class, equipment, tested status and age band, using competition results from \
OpenPowerlifting.",
            },
            Faq {
                q: "Is it free?",
                a: "Yes. There is no signup and nothing to install &mdash; it runs entirely \
in your browser.",
            },
        ],
        howto: None,
    }
}

pub(super) fn one_rep_max_page() -> Page {
    Page {
        slug: "one-rep-max-calculator",
        hash: "1rm",
        title: "1RM Calculator (Epley, Brzycki, Mayhew, Lombardi) | Iron Insights",
        desc: "Estimate your one-rep max from any submaximal set across four validated \
formulas, with a percentage-based training table. Free, kg and lb."
            .to_string(),
        h1: "One-rep max (1RM) calculator",
        lead: "A one-rep max calculator estimates the heaviest single you could lift from a \
set you have already done. Enter the weight and reps and Iron Insights estimates your 1RM \
across four validated formulas, then builds a percentage table for programming your training \
weights."
            .to_string(),
        sections: vec![
            (
                "The four formulas",
                "<p>Estimates diverge as reps climb, so it helps to compare them:</p>\
<table><thead><tr><th>Formula</th><th>Equation</th><th>Best for</th></tr></thead><tbody>\
<tr><td>Epley</td><td>w &times; (1 + r/30)</td><td>Simple strength-room default</td></tr>\
<tr><td>Brzycki</td><td>w / (1.0278 &minus; 0.0278r)</td><td>Conservative at low reps</td></tr>\
<tr><td>Mayhew</td><td>100w / (52.2 + 41.9e^&minus;0.055r)</td><td>Non-linear across reps</td></tr>\
<tr><td>Lombardi</td><td>w &times; r^0.1</td><td>Smooth power relationship</td></tr>\
</tbody></table><p>where <em>w</em> is the weight lifted and <em>r</em> is the number of reps \
completed.</p>"
                    .to_string(),
            ),
            (
                "Training percentages",
                "<p>Once you have a 1RM estimate, common training intensities are roughly: \
95% for max singles, 85% for heavy triples, 75% for strength-hypertrophy sets of 6-8, and 65% \
for higher-rep volume work.</p>"
                    .to_string(),
            ),
        ],
        faqs: vec![
            Faq {
                q: "Which 1RM formula is most accurate?",
                a: "All are estimates. Epley and Brzycki are reliable up to about 10 reps; \
accuracy drops as reps rise. Comparing several formulas gives a sensible range rather than a \
single false-precision number.",
            },
            Faq {
                q: "How many reps should I use?",
                a: "Use a set of 10 reps or fewer for the best accuracy. The further from a \
true max effort, the larger the estimation error.",
            },
        ],
        howto: Some(HowTo {
            title: "How to estimate your one-rep max",
            steps: vec![
                "Pick a recent set you took close to failure, ideally 10 reps or fewer.",
                "Enter the weight lifted and the number of reps completed.",
                "Read the estimate and compare the four formulas for a sensible range \
rather than a single number.",
            ],
        }),
    }
}

pub(super) fn barbell_plate_page() -> Page {
    Page {
        slug: "barbell-plate-calculator",
        hash: "plate-calc",
        title: "Barbell Plate Calculator (kg & lb) | Iron Insights",
        desc: "Work out exactly which plates to load per side for any target weight, with \
IPF 20kg/15kg bars and collar options. Free, kg and lb."
            .to_string(),
        h1: "Barbell plate calculator: what plates do I load?",
        lead: "A plate calculator tells you which discs to put on each side of the bar to \
hit a target weight. Enter the target and Iron Insights subtracts the bar and collars, then \
picks the fewest plates per side &mdash; in kilograms or pounds."
            .to_string(),
        sections: vec![
            (
                "How loading is worked out",
                "<p>The maths is simple: <strong>(target &minus; bar &minus; collars) \
&divide; 2</strong> is the load per side, which is then filled with the largest plates first. \
A standard men's bar is 20&nbsp;kg, a women's bar is 15&nbsp;kg, and competition collars add \
2.5&nbsp;kg each.</p>"
                    .to_string(),
            ),
            (
                "Worked example",
                "<p>For a 180&nbsp;kg target on a 20&nbsp;kg bar with no collars, you need \
80&nbsp;kg per side: 2&times;20&nbsp;kg + 2&times;15&nbsp;kg + 1&times;10&nbsp;kg. The visual \
loader shows the colour-coded plates and flags if a target cannot be made exactly.</p>"
                    .to_string(),
            ),
        ],
        faqs: vec![
            Faq {
                q: "How much does a barbell weigh?",
                a: "A standard Olympic men's bar is 20&nbsp;kg (44&nbsp;lb); a women's bar \
is 15&nbsp;kg (33&nbsp;lb). The calculator lets you pick the bar and whether you are using \
collars.",
            },
            Faq {
                q: "What plates do I need for 100kg?",
                a: "On a 20&nbsp;kg bar that is 40&nbsp;kg per side: 2&times;20&nbsp;kg, or \
1&times;20 + 1&times;15 + 1&times;5. The tool shows the optimal set for any target.",
            },
        ],
        howto: Some(HowTo {
            title: "How to work out which plates to load",
            steps: vec![
                "Subtract the bar weight (20 kg men's, 15 kg women's) and any collars from \
your target weight.",
                "Halve the remainder to get the load required on each side.",
                "Fill each side with the largest plates first until you reach the per-side \
load.",
            ],
        }),
    }
}

pub(super) fn body_fat_page() -> Page {
    Page {
        slug: "body-fat-percentage-calculator",
        hash: "bodyfat",
        title: "Body Fat % Calculator (Navy, YMCA, Skinfold) | Iron Insights",
        desc: "Estimate body fat percentage with the US Navy tape method, YMCA, or \
Jackson-Pollock 3- and 7-site skinfolds, with lean and fat mass. Free."
            .to_string(),
        h1: "Body fat percentage calculator",
        lead: "Iron Insights estimates body fat percentage four ways &mdash; the US Navy \
tape method, the YMCA waist method, and the Jackson-Pollock 3- and 7-site skinfold equations \
&mdash; and reports the resulting fat mass and lean mass. Each method trades convenience for \
accuracy."
            .to_string(),
        sections: vec![
            (
                "The methods",
                "<ul><li><strong>US Navy</strong> &mdash; tape measurements of neck, waist \
(and hips for women) plus height. Quick and equipment-free.</li><li><strong>YMCA</strong> \
&mdash; waist circumference and bodyweight. The simplest estimate.</li>\
<li><strong>Jackson-Pollock 3-site</strong> &mdash; skinfold calipers at three sites plus \
age. More accurate with good technique.</li><li><strong>Jackson-Pollock 7-site</strong> \
&mdash; seven skinfold sites for the tightest estimate.</li></ul>"
                    .to_string(),
            ),
            (
                "A note on accuracy",
                "<p>Every field method is an estimate with several percentage points of \
error. Use one method consistently and track the <em>trend</em> rather than treating any \
single reading as exact. DEXA and hydrostatic weighing remain the laboratory references.</p>"
                    .to_string(),
            ),
        ],
        faqs: vec![
            Faq {
                q: "What is the most accurate way to measure body fat?",
                a: "In a field setting, multi-site skinfolds (Jackson-Pollock) with good \
caliper technique tend to beat tape methods. DEXA scans are the practical gold standard but \
require a clinic.",
            },
            Faq {
                q: "What is a healthy body fat percentage?",
                a: "Broadly, fitness ranges sit around 14-20% for men and 21-27% for women, \
with athletes often lower. Ranges shift with age and the estimation method.",
            },
        ],
        howto: None,
    }
}
