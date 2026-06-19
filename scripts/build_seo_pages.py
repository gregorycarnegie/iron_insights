#!/usr/bin/env python3
"""Generate the static, crawlable SEO/GEO landing pages for Iron Insights.

Iron Insights is a client-side WASM app: search engines and AI answer engines
see only a loading skeleton at `/`. These hand-authored static pages give every
priority query a real, answer-first HTML document that (a) is indexable, (b)
carries structured data + source citations, and (c) links into the matching app
view (`/#ranking`, `/#1rm`, ...).

Source of truth for the *content* is the PAGES table below. Running this script
re-emits the committed HTML under `iron_insights_web/seo/<slug>/index.html`
plus `robots.txt` and `sitemap.xml`. Output is plain static files (no build-time
dependency); trunk copies them into the deploy dir via the copy-dir/copy-file
links in `index.html`.

Usage:  python scripts/build_seo_pages.py
"""

from __future__ import annotations

import html
import json
from pathlib import Path

BASE = "https://gregorycarnegie.github.io/iron_insights/"
WEB = Path(__file__).resolve().parent.parent / "iron_insights_web"
LASTMOD = "2026-06-19"
SOURCE = ("OpenPowerlifting", "https://www.openpowerlifting.org/")

# Shared, self-contained styling so pages render correctly regardless of the
# hashed app stylesheet name produced by release builds.
STYLE = """
:root{--bg:#0b0b0d;--panel:#141416;--ink:#e8e3d6;--ink-mute:#9a958a;
--line:#2a2926;--accent:#e8472b}
*{box-sizing:border-box}
html{scroll-behavior:smooth}
body{margin:0;background:var(--bg);color:var(--ink);
font-family:"JetBrains Mono",ui-monospace,monospace;font-size:15px;
line-height:1.65;-webkit-font-smoothing:antialiased}
a{color:var(--accent);text-decoration:none}
a:hover{text-decoration:underline}
.wrap{max-width:820px;margin:0 auto;padding:32px 20px 80px}
header.site{display:flex;justify-content:space-between;align-items:center;
border-bottom:1px solid var(--line);padding-bottom:16px;margin-bottom:8px}
.brand{font-family:"Archivo Black",system-ui,sans-serif;letter-spacing:.04em}
.brand .bar{display:inline-block;width:22px;height:8px;background:var(--accent);
margin-right:8px;vertical-align:middle}
nav.crumb{font-size:12px;color:var(--ink-mute);letter-spacing:.12em;
text-transform:uppercase;margin:18px 0}
h1{font-family:"Fraunces",Georgia,serif;font-weight:500;font-size:2.1rem;
line-height:1.15;margin:.2em 0 .4em}
h2{font-family:"Fraunces",Georgia,serif;font-weight:500;font-size:1.4rem;
margin:2em 0 .5em;border-top:1px solid var(--line);padding-top:1.2em}
.lead{font-size:1.12rem;color:var(--ink);background:var(--panel);
border-left:3px solid var(--accent);padding:16px 18px;margin:1em 0}
.cta{display:inline-block;background:var(--accent);color:#fff;font-weight:700;
letter-spacing:.06em;text-transform:uppercase;padding:12px 22px;margin:18px 0;
border-radius:4px}
.cta:hover{text-decoration:none;filter:brightness(1.08)}
table{width:100%;border-collapse:collapse;margin:1em 0;font-size:14px}
th,td{text-align:left;padding:8px 10px;border-bottom:1px solid var(--line)}
th{color:var(--ink-mute);text-transform:uppercase;font-size:11px;
letter-spacing:.1em}
ul{padding-left:1.2em}
li{margin:.3em 0}
details{border:1px solid var(--line);border-radius:4px;padding:10px 14px;
margin:.5em 0;background:var(--panel)}
summary{cursor:pointer;font-weight:500}
.faq h2{border:0;padding:0;margin-top:1.6em}
footer.site{border-top:1px solid var(--line);margin-top:48px;padding-top:20px;
font-size:13px;color:var(--ink-mute)}
footer.site a{color:var(--ink-mute)}
.related{display:flex;flex-wrap:wrap;gap:10px;margin:10px 0}
.related a{border:1px solid var(--line);padding:6px 12px;border-radius:4px;
color:var(--ink)}
.src{font-size:12px;color:var(--ink-mute);margin-top:8px}
""".strip()

FONTS = (
    '<link rel="preconnect" href="https://fonts.googleapis.com" />'
    '<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />'
    '<link href="https://fonts.googleapis.com/css2?family=Archivo+Black&'
    "family=JetBrains+Mono:wght@300;400;500;700&"
    "family=Fraunces:ital,opsz,wght@0,9..144,400;0,9..144,500;1,9..144,500&"
    'display=swap" rel="stylesheet" />'
)

# Each page: slug, hash route, title, description, h1, lead (the direct GEO
# answer), body (list of (h2, html) sections), and faqs (q, a).
TIER_TABLE = """<table><thead><tr><th>Percentile</th><th>Tier</th>
<th>What it means</th></tr></thead><tbody>
<tr><td>Top 1% (99th+)</td><td>Legend</td><td>National / world-class numbers</td></tr>
<tr><td>95th-99th</td><td>Elite</td><td>Competitive at a high level</td></tr>
<tr><td>80th-95th</td><td>Advanced</td><td>Strong, years of focused training</td></tr>
<tr><td>60th-80th</td><td>Intermediate</td><td>Above the typical lifter</td></tr>
<tr><td>Below 60th</td><td>Novice</td><td>Early in the strength journey</td></tr>
</tbody></table>"""

PAGES = [
    {
        "slug": "strength-percentile-calculator",
        "hash": "ranking",
        "title": "Powerlifting Strength Percentile Calculator | Iron Insights",
        "desc": "Find the exact percentile of your squat, bench, deadlift or "
                "total against 2M+ federated lifters by sex, bodyweight and "
                "equipment. Free, no signup.",
        "h1": "What percentile is my squat, bench, deadlift or total?",
        "lead": "Iron Insights ranks your lift against more than 2,000,000 "
                "competition results from the OpenPowerlifting database. Enter "
                "your bodyweight and lifts and it returns your exact percentile "
                "within your sex, weight-class and equipment cohort, then maps "
                "it to a strength tier from Novice to Legend.",
        "sections": [
            ("Strength tiers by percentile",
             "<p>Iron Insights labels every result with a tier so a raw "
             "percentile is easy to read:</p>" + TIER_TABLE),
            ("How the percentile is calculated",
             "<p>Your number is compared against the distribution of a "
             "<strong>matched cohort</strong> &mdash; same sex, weight class, "
             "equipment (raw, single-ply, etc.), tested status and age band "
             "&mdash; not against everyone at once. For the total you can rank "
             "by raw kilograms or by the DOTS, Wilks or Goodlift (GL) "
             "bodyweight-adjusted scores.</p>"),
        ],
        "faqs": [
            ("What is a good powerlifting total?",
             "It depends on bodyweight, sex and equipment. Iron Insights "
             "treats anything above the 80th percentile of your cohort as "
             "Advanced and above the 95th as Elite. Compare against your own "
             "weight class rather than an absolute number."),
            ("How are the percentiles worked out?",
             "Each lift is placed in the distribution of lifters who share "
             "your sex, weight class, equipment, tested status and age band, "
             "using competition results from OpenPowerlifting."),
            ("Is it free?",
             "Yes. There is no signup and nothing to install &mdash; it runs "
             "entirely in your browser."),
        ],
    },
    {
        "slug": "powerlifting-strength-standards",
        "hash": "nerds",
        "title": "Powerlifting Strength Standards by Weight Class | Iron Insights",
        "desc": "Median (p50) and elite (p90) squat, bench, deadlift and total "
                "thresholds by sex, weight class and equipment, from 2M+ "
                "OpenPowerlifting results.",
        "h1": "Powerlifting strength standards by weight class and bodyweight",
        "lead": "Strength standards answer 'how much should I lift for my "
                "bodyweight?'. Iron Insights derives them empirically: for "
                "every sex, weight class and equipment cohort it publishes the "
                "median (50th percentile) and a strong-lifter threshold (90th "
                "percentile) for squat, bench, deadlift and total, drawn from "
                "2M+ OpenPowerlifting competition results.",
        "sections": [
            ("Empirical, not made up",
             "<p>Most 'strength standards' charts are someone's opinion. These "
             "come from the actual distribution of competition lifters, so the "
             "median is literally the middle lifter in your cohort and the "
             "90th-percentile line is what the top tenth pull.</p>"),
            ("Bodyweight-adjusted scoring",
             "<p>To compare lifters of different bodyweights, totals are also "
             "expressed as DOTS, Wilks and Goodlift (GL) points. The Stats view "
             "shows the full distribution curve and a two-axis heatmap of lift "
             "versus bodyweight, plus year-over-year trend lines.</p>"),
        ],
        "faqs": [
            ("How much should I be able to squat for my weight?",
             "Look up your sex, weight class and equipment cohort: the median "
             "line is a typical competitive lifter and the 90th-percentile "
             "line marks a strong one."),
            ("What is DOTS / Wilks / GL?",
             "They are formulas that adjust a total for bodyweight so a 60kg "
             "and a 120kg lifter can be compared on one scale."),
        ],
    },
    {
        "slug": "male-vs-female-strength-comparison",
        "hash": "men-vs-women",
        "title": "Men vs Women Strength Comparison | Iron Insights",
        "desc": "How male and female powerlifting strength compares in matched "
                "weight-class and equipment cohorts, using DOTS-normalised "
                "distributions from 2M+ results.",
        "h1": "How does male and female powerlifting strength compare?",
        "lead": "Iron Insights compares men and women in <em>aligned</em> "
                "cohorts &mdash; the same weight class, equipment and lift "
                "&mdash; and overlays their full strength distributions. "
                "Because raw kilograms favour heavier lifters, the comparison "
                "is also shown in bodyweight-adjusted DOTS points so the gap "
                "reflects strength, not size.",
        "sections": [
            ("Distribution overlap, not single numbers",
             "<p>Rather than quoting one 'men are X% stronger' figure, the Sex "
             "Comparison view overlays the male and female distribution curves "
             "for the lift you pick, so you can see where the two populations "
             "overlap and where they diverge.</p>"),
            ("Per-lift breakdown",
             "<p>Squat, bench and deadlift are compared separately. The "
             "relative gap is typically largest in the bench press and "
             "smallest in the lower-body lifts &mdash; the tool shows the "
             "actual cohort numbers rather than a rule of thumb.</p>"),
        ],
        "faqs": [
            ("How much stronger are men than women in powerlifting?",
             "It varies by lift and bodyweight. The gap is largest in upper-"
             "body pressing and narrows once totals are adjusted for "
             "bodyweight with DOTS. The comparison view shows the real "
             "distributions for your chosen cohort."),
        ],
    },
    {
        "slug": "one-rep-max-calculator",
        "hash": "1rm",
        "title": "1RM Calculator (Epley, Brzycki, Mayhew, Lombardi) | Iron Insights",
        "desc": "Estimate your one-rep max from any submaximal set across four "
                "validated formulas, with a percentage-based training table. "
                "Free, kg and lb.",
        "h1": "One-rep max (1RM) calculator",
        "lead": "A one-rep max calculator estimates the heaviest single you "
                "could lift from a set you have already done. Enter the weight "
                "and reps and Iron Insights estimates your 1RM across four "
                "validated formulas, then builds a percentage table for "
                "programming your training weights.",
        "sections": [
            ("The four formulas",
             "<p>Estimates diverge as reps climb, so it helps to compare "
             "them:</p><table><thead><tr><th>Formula</th><th>Equation</th>"
             "<th>Best for</th></tr></thead><tbody>"
             "<tr><td>Epley</td><td>w &times; (1 + r/30)</td>"
             "<td>Simple strength-room default</td></tr>"
             "<tr><td>Brzycki</td><td>w / (1.0278 &minus; 0.0278r)</td>"
             "<td>Conservative at low reps</td></tr>"
             "<tr><td>Mayhew</td><td>100w / (52.2 + 41.9e^&minus;0.055r)</td>"
             "<td>Non-linear across reps</td></tr>"
             "<tr><td>Lombardi</td><td>w &times; r^0.1</td>"
             "<td>Smooth power relationship</td></tr>"
             "</tbody></table><p>where <em>w</em> is the weight lifted and "
             "<em>r</em> is the number of reps completed.</p>"),
            ("Training percentages",
             "<p>Once you have a 1RM estimate, common training intensities are "
             "roughly: 95% for max singles, 85% for heavy triples, 75% for "
             "strength-hypertrophy sets of 6-8, and 65% for higher-rep volume "
             "work.</p>"),
        ],
        "faqs": [
            ("Which 1RM formula is most accurate?",
             "All are estimates. Epley and Brzycki are reliable up to about "
             "10 reps; accuracy drops as reps rise. Comparing several formulas "
             "gives a sensible range rather than a single false-precision "
             "number."),
            ("How many reps should I use?",
             "Use a set of 10 reps or fewer for the best accuracy. The further "
             "from a true max effort, the larger the estimation error."),
        ],
    },
    {
        "slug": "barbell-plate-calculator",
        "hash": "plate-calc",
        "title": "Barbell Plate Calculator (kg & lb) | Iron Insights",
        "desc": "Work out exactly which plates to load per side for any target "
                "weight, with IPF 20kg/15kg bars and collar options. Free, kg "
                "and lb.",
        "h1": "Barbell plate calculator: what plates do I load?",
        "lead": "A plate calculator tells you which discs to put on each side "
                "of the bar to hit a target weight. Enter the target and Iron "
                "Insights subtracts the bar and collars, then picks the fewest "
                "plates per side &mdash; in kilograms or pounds.",
        "sections": [
            ("How loading is worked out",
             "<p>The maths is simple: <strong>(target &minus; bar &minus; "
             "collars) &divide; 2</strong> is the load per side, which is then "
             "filled with the largest plates first. A standard men's bar is "
             "20&nbsp;kg, a women's bar is 15&nbsp;kg, and competition collars "
             "add 2.5&nbsp;kg each.</p>"),
            ("Worked example",
             "<p>For a 180&nbsp;kg target on a 20&nbsp;kg bar with no collars, "
             "you need 80&nbsp;kg per side: 2&times;20&nbsp;kg + 2&times;15&nbsp;"
             "kg + 1&times;10&nbsp;kg. The visual loader shows the colour-coded "
             "plates and flags if a target cannot be made exactly.</p>"),
        ],
        "faqs": [
            ("How much does a barbell weigh?",
             "A standard Olympic men's bar is 20&nbsp;kg (44&nbsp;lb); a "
             "women's bar is 15&nbsp;kg (33&nbsp;lb). The calculator lets you "
             "pick the bar and whether you are using collars."),
            ("What plates do I need for 100kg?",
             "On a 20&nbsp;kg bar that is 40&nbsp;kg per side: 2&times;20&nbsp;"
             "kg, or 1&times;20 + 1&times;15 + 1&times;5. The tool shows the "
             "optimal set for any target."),
        ],
    },
    {
        "slug": "body-fat-percentage-calculator",
        "hash": "bodyfat",
        "title": "Body Fat % Calculator (Navy, YMCA, Skinfold) | Iron Insights",
        "desc": "Estimate body fat percentage with the US Navy tape method, "
                "YMCA, or Jackson-Pollock 3- and 7-site skinfolds, with lean "
                "and fat mass. Free.",
        "h1": "Body fat percentage calculator",
        "lead": "Iron Insights estimates body fat percentage four ways &mdash; "
                "the US Navy tape method, the YMCA waist method, and the "
                "Jackson-Pollock 3- and 7-site skinfold equations &mdash; and "
                "reports the resulting fat mass and lean mass. Each method "
                "trades convenience for accuracy.",
        "sections": [
            ("The methods",
             "<ul>"
             "<li><strong>US Navy</strong> &mdash; tape measurements of neck, "
             "waist (and hips for women) plus height. Quick and equipment-free."
             "</li>"
             "<li><strong>YMCA</strong> &mdash; waist circumference and "
             "bodyweight. The simplest estimate.</li>"
             "<li><strong>Jackson-Pollock 3-site</strong> &mdash; skinfold "
             "calipers at three sites plus age. More accurate with good "
             "technique.</li>"
             "<li><strong>Jackson-Pollock 7-site</strong> &mdash; seven "
             "skinfold sites for the tightest estimate.</li>"
             "</ul>"),
            ("A note on accuracy",
             "<p>Every field method is an estimate with several percentage "
             "points of error. Use one method consistently and track the "
             "<em>trend</em> rather than treating any single reading as exact. "
             "DEXA and hydrostatic weighing remain the laboratory references.</p>"),
        ],
        "faqs": [
            ("What is the most accurate way to measure body fat?",
             "In a field setting, multi-site skinfolds (Jackson-Pollock) with "
             "good caliper technique tend to beat tape methods. DEXA scans are "
             "the practical gold standard but require a clinic."),
            ("What is a healthy body fat percentage?",
             "Broadly, fitness ranges sit around 14-20% for men and 21-27% for "
             "women, with athletes often lower. Ranges shift with age and the "
             "estimation method."),
        ],
    },
]


def page_url(slug: str) -> str:
    return f"{BASE}{slug}/"


def jsonld_blocks(page: dict) -> str:
    crumb = {
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        "itemListElement": [
            {"@type": "ListItem", "position": 1, "name": "Iron Insights",
             "item": BASE},
            {"@type": "ListItem", "position": 2,
             "name": page["h1"], "item": page_url(page["slug"])},
        ],
    }
    faq = {
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": [
            {"@type": "Question", "name": _strip(q),
             "acceptedAnswer": {"@type": "Answer", "text": _strip(a)}}
            for q, a in page["faqs"]
        ],
    }
    webapp = {
        "@context": "https://schema.org",
        "@type": "WebApplication",
        "name": "Iron Insights",
        "url": page_url(page["slug"]),
        "applicationCategory": "SportsApplication",
        "operatingSystem": "Any",
        "isPartOf": {"@type": "WebSite", "name": "Iron Insights", "url": BASE},
        "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
        "creditText": SOURCE[0],
        "isBasedOn": SOURCE[1],
    }
    out = []
    for block in (webapp, crumb, faq):
        out.append('<script type="application/ld+json">\n'
                   + json.dumps(block, indent=2) + "\n</script>")
    return "\n".join(out)


def _strip(s: str) -> str:
    """Plain text for JSON-LD: unescape entities, drop tags."""
    import re
    s = re.sub(r"<[^>]+>", "", s)
    return html.unescape(s)


def render_page(page: dict) -> str:
    url = page_url(page["slug"])
    related = "\n".join(
        f'<a href="{BASE}{p["slug"]}/">{p["h1"]}</a>'
        for p in PAGES if p["slug"] != page["slug"]
    )
    sections = "\n".join(
        f"<h2>{html.escape(h2)}</h2>\n{body}" for h2, body in page["sections"]
    )
    faqs = "\n".join(
        f"<details><summary>{html.escape(q)}</summary><p>{a}</p></details>"
        for q, a in page["faqs"]
    )
    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{html.escape(page["title"])}</title>
<meta name="description" content="{html.escape(page["desc"])}" />
<link rel="canonical" href="{url}" />
<meta name="robots" content="index, follow" />
<meta name="theme-color" content="#0b0b0d" />
<link rel="icon" type="image/svg+xml" href="{BASE}assets/favicon.svg" />
<meta property="og:type" content="website" />
<meta property="og:site_name" content="Iron Insights" />
<meta property="og:title" content="{html.escape(page["title"])}" />
<meta property="og:description" content="{html.escape(page["desc"])}" />
<meta property="og:url" content="{url}" />
<meta property="og:image" content="{BASE}assets/favicon.svg" />
<meta name="twitter:card" content="summary" />
<meta name="twitter:title" content="{html.escape(page["title"])}" />
<meta name="twitter:description" content="{html.escape(page["desc"])}" />
{FONTS}
<style>{STYLE}</style>
{jsonld_blocks(page)}
</head>
<body>
<div class="wrap">
<header class="site">
  <div class="brand"><span class="bar"></span>IRON INSIGHTS</div>
  <a href="{BASE}">Open the app &rarr;</a>
</header>
<nav class="crumb" aria-label="Breadcrumb">
  <a href="{BASE}">Iron Insights</a> / {html.escape(page["h1"])}
</nav>
<main>
<h1>{html.escape(page["h1"])}</h1>
<p class="lead">{page["lead"]}</p>
<a class="cta" href="{BASE}#{page["hash"]}">Try the live tool &rarr;</a>
{sections}
<section class="faq">
<h2>Frequently asked questions</h2>
{faqs}
</section>
<p class="src">Data source: lifter results from
<a href="{SOURCE[1]}" rel="nofollow">{SOURCE[0]}</a>, the open powerlifting
results database (public domain). Iron Insights is an independent project and is
not affiliated with OpenPowerlifting.</p>
<a class="cta" href="{BASE}#{page["hash"]}">Open the live tool &rarr;</a>
</main>
<footer class="site">
<p>More Iron Insights tools:</p>
<div class="related">
{related}
</div>
<p style="margin-top:18px">&copy; 2026 Iron Insights &middot;
<a href="{BASE}">Home</a></p>
</footer>
</div>
</body>
</html>
"""


def render_robots() -> str:
    return (
        "# Iron Insights robots.txt\n"
        "User-agent: *\n"
        "Allow: /\n\n"
        "# AI answer engines are explicitly welcome to read and cite the site.\n"
        "User-agent: GPTBot\nAllow: /\n\n"
        "User-agent: ClaudeBot\nAllow: /\n\n"
        "User-agent: PerplexityBot\nAllow: /\n\n"
        "User-agent: Google-Extended\nAllow: /\n\n"
        f"Sitemap: {BASE}sitemap.xml\n"
    )


def render_sitemap() -> str:
    urls = [(BASE, "1.0")] + [(page_url(p["slug"]), "0.8") for p in PAGES]
    items = "\n".join(
        f"  <url>\n    <loc>{u}</loc>\n    <lastmod>{LASTMOD}</lastmod>\n"
        f"    <priority>{pr}</priority>\n  </url>"
        for u, pr in urls
    )
    return ('<?xml version="1.0" encoding="UTF-8"?>\n'
            '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
            f"{items}\n</urlset>\n")


def main() -> None:
    for page in PAGES:
        out = WEB / "seo" / page["slug"] / "index.html"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(render_page(page), encoding="utf-8")
        print(f"wrote {out.relative_to(WEB)}")
    (WEB / "robots.txt").write_text(render_robots(), encoding="utf-8")
    print("wrote robots.txt")
    (WEB / "sitemap.xml").write_text(render_sitemap(), encoding="utf-8")
    print("wrote sitemap.xml")


if __name__ == "__main__":
    main()
