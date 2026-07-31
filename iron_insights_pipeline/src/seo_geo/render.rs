use serde_json::json;

use super::{
    assets::{FONTS, STYLE},
    constants::{SOURCE_NAME, SOURCE_URL},
    content::Page,
};

/// Author/publisher entity for E-E-A-T signalling (GEO authority).
fn publisher() -> serde_json::Value {
    json!({
        "@type": "Person",
        "name": "Gregory Carnegie",
        "url": "https://github.com/gregorycarnegie"
    })
}

pub(super) fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub(super) fn strip_for_jsonld(s: &str) -> String {
    // Remove tags.
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&mdash;", "\u{2014}")
        .replace("&nbsp;", " ")
        .replace("&times;", "\u{00d7}")
        .replace("&divide;", "\u{00f7}")
        .replace("&minus;", "\u{2212}")
        .replace("&rarr;", "\u{2192}")
        .replace("&amp;", "&")
}

pub(super) struct Site<'a> {
    pub(super) base: &'a str,
    pub(super) lastmod: &'a str,
    pub(super) data_date: &'a str,
    pub(super) date_published: &'a str,
    pub(super) count_human: &'a str,
    pub(super) count_exact: &'a str,
}

pub(super) fn page_url(base: &str, slug: &str) -> String {
    format!("{base}{slug}/")
}

fn jsonld_blocks(site: &Site, page: &Page) -> String {
    let url = page_url(site.base, page.slug);
    let webapp = json!({
        "@context": "https://schema.org",
        "@type": "WebApplication",
        "name": "Iron Insights",
        "url": url,
        "applicationCategory": "SportsApplication",
        "operatingSystem": "Any",
        "isPartOf": {"@type": "WebSite", "name": "Iron Insights", "url": site.base},
        "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
        "creditText": SOURCE_NAME,
        "isBasedOn": SOURCE_URL,
        "author": publisher(),
        "publisher": publisher(),
        "datePublished": site.date_published,
        "dateModified": site.lastmod,
    });
    let crumb = json!({
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        "itemListElement": [
            {"@type": "ListItem", "position": 1, "name": "Iron Insights", "item": site.base},
            {"@type": "ListItem", "position": 2, "name": page.h1, "item": url},
        ],
    });
    let faq = json!({
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": page.faqs.iter().map(|f| json!({
            "@type": "Question",
            "name": strip_for_jsonld(f.q),
            "acceptedAnswer": {"@type": "Answer", "text": strip_for_jsonld(f.a)},
        })).collect::<Vec<_>>(),
    });

    let mut blocks = vec![webapp, crumb, faq];
    if let Some(howto) = &page.howto {
        blocks.push(json!({
            "@context": "https://schema.org",
            "@type": "HowTo",
            "name": howto.title,
            "step": howto.steps.iter().enumerate().map(|(i, s)| json!({
                "@type": "HowToStep",
                "position": i + 1,
                "name": strip_for_jsonld(s),
                "text": strip_for_jsonld(s),
            })).collect::<Vec<_>>(),
        }));
    }

    blocks
        .iter()
        .map(|b| {
            format!(
                "<script type=\"application/ld+json\">\n{}\n</script>",
                serde_json::to_string_pretty(b).unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn substitute(text: &str, site: &Site) -> String {
    text.replace("[[COUNT]]", site.count_human)
        .replace("[[COUNT_EXACT]]", site.count_exact)
        .replace("[[DATA_DATE]]", site.data_date)
}

pub(super) fn render_page(site: &Site, pages: &[Page], page: &Page) -> String {
    let url = page_url(site.base, page.slug);
    let related = pages
        .iter()
        .filter(|p| p.slug != page.slug)
        .map(|p| format!("<a href=\"{}{}/\">{}</a>", site.base, p.slug, p.h1))
        .collect::<Vec<_>>()
        .join("\n");
    let sections = page
        .sections
        .iter()
        .map(|(h2, body)| format!("<h2>{}</h2>\n{body}", escape(h2)))
        .collect::<Vec<_>>()
        .join("\n");
    let faqs = page
        .faqs
        .iter()
        .map(|f| {
            format!(
                "<details><summary>{}</summary><p>{}</p></details>",
                escape(f.q),
                f.a
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let jsonld = jsonld_blocks(site, page);
    let base = site.base;
    let hash = page.hash;

    let rendered = format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{title}</title>
<meta name="description" content="{desc}" />
<link rel="canonical" href="{url}" />
<meta name="robots" content="index, follow" />
<meta name="theme-color" content="#0b0b0d" />
<link rel="icon" type="image/svg+xml" href="{base}assets/favicon.svg" />
<meta property="og:type" content="website" />
<meta property="og:site_name" content="Iron Insights" />
<meta property="og:title" content="{title}" />
<meta property="og:description" content="{desc}" />
<meta property="og:url" content="{url}" />
<meta property="og:image" content="{base}assets/favicon.svg" />
<meta name="twitter:card" content="summary" />
<meta name="twitter:title" content="{title}" />
<meta name="twitter:description" content="{desc}" />
{FONTS}
<style>{STYLE}</style>
{jsonld}
</head>
<body>
<div class="wrap">
<header class="site">
  <div class="brand"><span class="bar"></span>IRON INSIGHTS</div>
  <a href="{base}">Open the app &rarr;</a>
</header>
<nav class="crumb" aria-label="Breadcrumb">
  <a href="{base}">Iron Insights</a> / {h1}
</nav>
<main>
<h1>{h1}</h1>
<p class="lead">{lead}</p>
<a class="cta" href="{base}#{hash}">Try the live tool &rarr;</a>
{sections}
<section class="faq">
<h2>Frequently asked questions</h2>
{faqs}
</section>
<p class="src">Figures computed from [[COUNT_EXACT]] competition results;
data current as of [[DATA_DATE]]. Data source: lifter results from
<a href="{SOURCE_URL}" rel="nofollow">{SOURCE_NAME}</a>, the open powerlifting
results database (public domain). Iron Insights is an independent project and is
not affiliated with OpenPowerlifting.</p>
<a class="cta" href="{base}#{hash}">Open the live tool &rarr;</a>
</main>
<footer class="site">
<p>More Iron Insights tools:</p>
<div class="related">
{related}
</div>
<p style="margin-top:18px">&copy; 2026 Iron Insights &middot;
<a href="{base}">Home</a></p>
</footer>
</div>
</body>
</html>
"##,
        title = escape(page.title),
        desc = escape(&page.desc),
        h1 = escape(page.h1),
        lead = page.lead,
    );
    substitute(&rendered, site)
}

pub(super) fn render_robots(base: &str) -> String {
    format!(
        "# Iron Insights robots.txt\n\
User-agent: *\n\
Allow: /\n\n\
# AI answer engines are explicitly welcome to read and cite the site.\n\
User-agent: GPTBot\nAllow: /\n\n\
User-agent: ClaudeBot\nAllow: /\n\n\
User-agent: PerplexityBot\nAllow: /\n\n\
User-agent: Google-Extended\nAllow: /\n\n\
Sitemap: {base}sitemap.xml\n"
    )
}

pub(super) fn render_sitemap(base: &str, lastmod: &str, pages: &[Page]) -> String {
    let mut items = format!(
        "  <url>\n    <loc>{base}</loc>\n    <lastmod>{lastmod}</lastmod>\n    <priority>1.0</priority>\n  </url>"
    );
    for p in pages {
        items.push_str(&format!(
            "\n  <url>\n    <loc>{}</loc>\n    <lastmod>{lastmod}</lastmod>\n    <priority>0.8</priority>\n  </url>",
            page_url(base, p.slug)
        ));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{items}\n</urlset>\n"
    )
}
