#!/usr/bin/env python3
"""SEO/GEO crawl + benchmark for the Iron Insights static site.

Simulates what a non-JS crawler (Googlebot first pass, Bingbot) and AI answer
engines (GPTBot, ClaudeBot, PerplexityBot, Google-Extended) actually see. Iron
Insights ships as a client-side WASM app, so any content rendered by the app is
invisible to these agents -- this benchmark only scores the *served HTML*.

Usage:
    python scripts/seo_audit.py <site_dir> [--base-url URL] [--json]

<site_dir> is a directory of deployable files (the trunk `dist`/`docs` output,
or the `iron_insights_web` source tree, whose static SEO files are copied
verbatim into the build).

Exit code is non-zero when any CRITICAL gap remains, so it can gate CI.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from html.parser import HTMLParser
from pathlib import Path

DEFAULT_BASE = "https://gregorycarnegie.github.io/iron_insights/"

# Content that is purely the app boot skeleton -- not real answer content.
SKELETON_MARKERS = ("boot-shell", "skeleton", "LOADING")


class PageParser(HTMLParser):
    """Extracts SEO-relevant signals from a single HTML document."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.title_parts: list[str] = []
        self._in_title = False
        self._in_body = False
        self._in_script_or_style = False
        self.h1: list[str] = []
        self._in_h1 = False
        self.meta: dict[str, str] = {}
        self.og: dict[str, str] = {}
        self.twitter: dict[str, str] = {}
        self.canonical: str | None = None
        self.jsonld: list[dict] = []
        self._in_jsonld = False
        self._jsonld_buf: list[str] = []
        self.internal_links = 0
        self.external_links = 0
        self.body_text: list[str] = []
        self.lang: str | None = None

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "html":
            self.lang = a.get("lang")
        elif tag == "title":
            self._in_title = True
        elif tag == "body":
            self._in_body = True
        elif tag in ("script", "style"):
            self._in_script_or_style = True
            if tag == "script" and a.get("type") == "application/ld+json":
                self._in_jsonld = True
                self._jsonld_buf = []
        elif tag == "h1":
            self._in_h1 = True
        elif tag == "meta":
            name = (a.get("name") or "").lower()
            prop = (a.get("property") or "").lower()
            content = a.get("content", "")
            if name:
                self.meta[name] = content
            if prop.startswith("og:"):
                self.og[prop] = content
            if name.startswith("twitter:"):
                self.twitter[name] = content
        elif tag == "link":
            if (a.get("rel") or "").lower() == "canonical":
                self.canonical = a.get("href")
        elif tag == "a":
            href = a.get("href", "")
            if href.startswith("http") and "gregorycarnegie.github.io" not in href:
                self.external_links += 1
            elif href and not href.startswith("#"):
                self.internal_links += 1

    def handle_endtag(self, tag):
        if tag == "title":
            self._in_title = False
        elif tag in ("script", "style"):
            self._in_script_or_style = False
            if self._in_jsonld:
                self._in_jsonld = False
                raw = "".join(self._jsonld_buf).strip()
                try:
                    parsed = json.loads(raw)
                    if isinstance(parsed, list):
                        self.jsonld.extend(x for x in parsed if isinstance(x, dict))
                    elif isinstance(parsed, dict):
                        self.jsonld.append(parsed)
                except json.JSONDecodeError:
                    self.jsonld.append({"@type": "INVALID_JSON"})
        elif tag == "h1":
            self._in_h1 = False

    def handle_data(self, data):
        if self._in_title:
            self.title_parts.append(data)
        if self._in_jsonld:
            self._jsonld_buf.append(data)
        if self._in_h1:
            self.h1.append(data.strip())
        if self._in_body and not self._in_script_or_style:
            text = data.strip()
            if text:
                self.body_text.append(text)

    @property
    def title(self) -> str:
        return "".join(self.title_parts).strip()

    @property
    def jsonld_types(self) -> list[str]:
        types: list[str] = []
        for block in self.jsonld:
            t = block.get("@type")
            if isinstance(t, list):
                types.extend(t)
            elif t:
                types.append(t)
        return types

    @property
    def visible_word_count(self) -> int:
        return sum(len(t.split()) for t in self.body_text)

    @property
    def is_skeleton_only(self) -> bool:
        joined = " ".join(self.body_text)
        # Real content pages have prose; the boot shell has near-zero words.
        return self.visible_word_count < 40


SEV_CRIT = "CRITICAL"
SEV_HIGH = "HIGH"
SEV_MED = "MEDIUM"
SEV_LOW = "LOW"


@dataclass
class Finding:
    dimension: str
    severity: str
    message: str
    page: str


@dataclass
class Report:
    findings: list[Finding] = field(default_factory=list)

    def add(self, dim, sev, msg, page):
        self.findings.append(Finding(dim, sev, msg, page))

    def by_severity(self, sev):
        return [f for f in self.findings if f.severity == sev]


def audit_page(path: Path, rel: str, base_url: str, rep: Report) -> PageParser:
    p = PageParser()
    p.feed(path.read_text(encoding="utf-8", errors="replace"))
    is_app_shell = "app-shell" in path.read_text(encoding="utf-8", errors="replace")

    # --- Indexation / crawlability ---
    robots = p.meta.get("robots", "")
    if "noindex" in robots.lower():
        rep.add("Indexation", SEV_CRIT, "Page is noindex", rel)

    # --- Titles ---
    if not p.title:
        rep.add("Titles", SEV_CRIT, "Missing <title>", rel)
    elif len(p.title) > 65:
        rep.add("Titles", SEV_LOW, f"Title {len(p.title)} chars (>65)", rel)

    # --- Meta description ---
    desc = p.meta.get("description", "")
    if not desc:
        rep.add("Meta", SEV_HIGH, "Missing meta description", rel)
    elif len(desc) > 165:
        rep.add("Meta", SEV_LOW, f"Description {len(desc)} chars (>165)", rel)

    # --- Canonical ---
    if not p.canonical:
        rep.add("Canonical", SEV_HIGH, "Missing rel=canonical", rel)

    # --- Open Graph / social ---
    for need in ("og:title", "og:description", "og:url", "og:image", "og:type"):
        if need not in p.og:
            rep.add("Social", SEV_MED, f"Missing {need}", rel)
    if "twitter:card" not in p.twitter:
        rep.add("Social", SEV_LOW, "Missing twitter:card", rel)

    # --- Structured data ---
    if "INVALID_JSON" in p.jsonld_types:
        rep.add("StructuredData", SEV_HIGH, "Invalid JSON-LD", rel)
    if not p.jsonld:
        rep.add("StructuredData", SEV_MED, "No JSON-LD structured data", rel)

    # --- Page intent / answer-first content ---
    if is_app_shell:
        # The SPA shell is expected to be thin; it must at least point crawlers
        # to real content via <noscript> and link to the static answer pages.
        raw = path.read_text(encoding="utf-8", errors="replace")
        if "<noscript" not in raw:
            rep.add("AnswerContent", SEV_HIGH,
                    "App shell has no <noscript> fallback content", rel)
    else:
        if not p.h1:
            rep.add("PageIntent", SEV_HIGH, "No <h1>", rel)
        if p.is_skeleton_only:
            rep.add("AnswerContent", SEV_CRIT,
                    f"Only {p.visible_word_count} visible words "
                    "(no answer-first content)", rel)
        if p.internal_links < 2:
            rep.add("InternalLinks", SEV_MED,
                    f"Only {p.internal_links} internal links", rel)
        # GEO: AI answer engines weight cited sources.
        body = " ".join(p.body_text).lower()
        if "openpowerlifting" not in body and not is_app_shell:
            rep.add("Citations", SEV_MED, "No source citation in body", rel)

    return p


def audit_site(site_dir: Path, base_url: str) -> tuple[Report, dict]:
    rep = Report()
    html_files = sorted(site_dir.rglob("index.html")) + sorted(
        f for f in site_dir.rglob("*.html") if f.name != "index.html"
    )
    # Ignore generated/vendored output trees.
    html_files = [
        f for f in html_files
        if "dist" not in f.parts and "target" not in f.parts
    ]

    pages: dict[str, PageParser] = {}
    titles: dict[str, list[str]] = {}
    for f in html_files:
        rel = "/" + str(f.relative_to(site_dir)).replace("\\", "/")
        p = audit_page(f, rel, base_url, rep)
        pages[rel] = p
        titles.setdefault(p.title, []).append(rel)

    # --- Duplicate titles across pages ---
    for title, where in titles.items():
        if title and len(where) > 1:
            rep.add("Titles", SEV_HIGH,
                    f"Duplicate title on {len(where)} pages: {where}", "(site)")

    # --- robots.txt ---
    robots = site_dir / "robots.txt"
    if not robots.exists():
        rep.add("Crawlability", SEV_CRIT, "Missing robots.txt", "/robots.txt")
    else:
        txt = robots.read_text(encoding="utf-8", errors="replace")
        if "Sitemap:" not in txt:
            rep.add("Crawlability", SEV_HIGH,
                    "robots.txt has no Sitemap: directive", "/robots.txt")

    # --- sitemap.xml ---
    sitemap = site_dir / "sitemap.xml"
    sitemap_urls = 0
    if not sitemap.exists():
        rep.add("Crawlability", SEV_CRIT, "Missing sitemap.xml", "/sitemap.xml")
    else:
        sm = sitemap.read_text(encoding="utf-8", errors="replace")
        sitemap_urls = len(re.findall(r"<loc>", sm))
        if sitemap_urls == 0:
            rep.add("Crawlability", SEV_HIGH, "sitemap.xml has no <loc> URLs",
                    "/sitemap.xml")

    stats = {
        "html_pages": len(html_files),
        "content_pages": sum(1 for p in pages.values()
                             if not p.is_skeleton_only),
        "sitemap_urls": sitemap_urls,
        "pages": {
            rel: {
                "title": p.title,
                "words": p.visible_word_count,
                "h1": p.h1[:1],
                "jsonld": p.jsonld_types,
                "internal_links": p.internal_links,
                "canonical": bool(p.canonical),
            }
            for rel, p in pages.items()
        },
    }
    return rep, stats


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("site_dir", type=Path)
    ap.add_argument("--base-url", default=DEFAULT_BASE)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    rep, stats = audit_site(args.site_dir, args.base_url)

    if args.json:
        print(json.dumps({
            "stats": stats,
            "findings": [f.__dict__ for f in rep.findings],
        }, indent=2))
    else:
        print(f"\n=== SEO/GEO AUDIT: {args.site_dir} ===")
        print(f"HTML pages crawled : {stats['html_pages']}")
        print(f"Pages with real content (>=40 words): {stats['content_pages']}")
        print(f"Sitemap URLs       : {stats['sitemap_urls']}\n")
        for sev in (SEV_CRIT, SEV_HIGH, SEV_MED, SEV_LOW):
            items = rep.by_severity(sev)
            print(f"--- {sev} ({len(items)}) ---")
            for f in items:
                print(f"  [{f.dimension:<14}] {f.page:<48} {f.message}")
            print()

    n_crit = len(rep.by_severity(SEV_CRIT))
    n_high = len(rep.by_severity(SEV_HIGH))
    print(f"SUMMARY: {n_crit} critical, {n_high} high, "
          f"{len(rep.by_severity(SEV_MED))} medium, "
          f"{len(rep.by_severity(SEV_LOW))} low")
    return 1 if n_crit > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
