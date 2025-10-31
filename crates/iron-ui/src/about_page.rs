use crate::components::*;
use maud::{DOCTYPE, Markup, html};

pub fn render_about_page() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head_minimal())
            body.no-js {
                div.container {
                    (render_header())
                    main #main-content.page-transition role="main" {
                        div.main-content.about-page {
                            section.about-hero aria-labelledby="about-heading" {
                                h1 #about-heading { "About Iron Insights" }
                                p.lead {
                                    "We built Iron Insights to make strength analytics approachable, actionable, and inspiring for lifters at every stage."
                                }
                                p.lead-secondary {
                                    "Our platform combines competition-grade data with modern visualization and accessibility-first design so you can focus on training smarter."
                                }
                            }

                            section.about-mission aria-labelledby="mission-heading" {
                                h2 #mission-heading { "Our focus" }
                                div.mission-grid role="list" {
                                    article.mission-card role="listitem" {
                                        h3 { "Data you can trust" }
                                        p {
                                            "We ingest and clean public meet results, then expose the metrics that coaches and athletes rely on: DOTS scoring, percentile benchmarks, and trend analysis."
                                        }
                                    }
                                    article.mission-card role="listitem" {
                                        h3 { "Tools built for clarity" }
                                        p {
                                            "From interactive charts to shareable highlight cards, each feature was designed to answer the questions lifters actually ask about progress, readiness, and competition strategy."
                                        }
                                    }
                                    article.mission-card role="listitem" {
                                        h3 { "Performance without the bloat" }
                                        p {
                                            "Iron Insights ships with pre-optimized assets, efficient caching, and no intrusive trackers, so you spend less time waiting and more time interpreting your training."
                                        }
                                    }
                                }
                            }

                            section.about-pillars aria-labelledby="pillars-heading" {
                                h2 #pillars-heading { "What sets Iron Insights apart" }
                                ul.pillars-list {
                                    li {
                                        strong { "Purpose-built visualizations." }
                                        span { " We highlight the lifts, classes, and time ranges that matter, with quick filters for equipment, sex, and bodyweight bands." }
                                    }
                                    li {
                                        strong { "Share-ready storytelling." }
                                        span { " Export beautiful cards that celebrate PRs, team finishes, or meet recaps in seconds." }
                                    }
                                    li {
                                        strong { "Real-world context." }
                                        span { " Percentiles, reference cohorts, and power-to-weight comparisons keep every data point grounded in reality." }
                                    }
                                }
                            }

                            section.about-faq aria-labelledby="faq-heading" {
                                h2 #faq-heading { "Frequently asked questions" }
                                div.faq-grid {
                                    details {
                                        summary { "Where does the data come from?" }
                                        p {
                                            "We process the latest OpenPowerlifting datasets, clean irregular entries, and cache derivative metrics so they remain fast to access during heavy traffic." }
                                    }
                                    details {
                                        summary { "Can I use Iron Insights during meet prep?" }
                                        p {
                                            "Yes. The analytics page lets you model attempts, compare historical meets, and spot trends that inform tapering and attempt selection." }
                                    }
                                    details {
                                        summary { "Do you support teams or coaches?" }
                                        p {
                                            "Team dashboards are on the roadmap. In the meantime, coaches can export charts, share cards, and use the 1RM calculator with their lifters." }
                                    }
                                    details {
                                        summary { "How can I contribute or request features?" }
                                        p {
                                            "We welcome feedback. Reach out through the contact links in the footer or open an issue on the project repository." }
                                    }
                                }
                            }
                        }
                    }
                }
                style {
                    (maud::PreEscaped(r#"
                        .about-page {
                            display: flex;
                            flex-direction: column;
                            gap: 3rem;
                            padding: 2rem 0 4rem;
                        }

                        .about-hero {
                            background: linear-gradient(135deg, var(--primary-light), var(--surface));
                            border-radius: 16px;
                            padding: 3rem;
                            text-align: left;
                            box-shadow: 0 12px 32px rgba(37, 99, 235, 0.12);
                        }

                        .about-hero h1 {
                            font-size: 2.75rem;
                            margin-bottom: 1rem;
                            color: var(--text-primary);
                        }

                        .lead {
                            font-size: 1.25rem;
                            color: var(--text-secondary);
                            margin-bottom: 1rem;
                        }

                        .lead-secondary {
                            font-size: 1.1rem;
                            color: var(--text-secondary);
                        }

                        .mission-grid {
                            display: grid;
                            grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
                            gap: 1.5rem;
                        }

                        .mission-card {
                            background: var(--surface);
                            border: 1px solid var(--border);
                            padding: 1.75rem;
                            border-radius: 12px;
                            box-shadow: 0 6px 18px rgba(15, 23, 42, 0.08);
                        }

                        .mission-card h3 {
                            margin-bottom: 0.75rem;
                            color: var(--text-primary);
                            font-size: 1.25rem;
                        }

                        .mission-card p {
                            color: var(--text-secondary);
                            line-height: 1.6;
                        }

                        .pillars-list {
                            list-style: none;
                            padding: 0;
                            margin: 0;
                            display: flex;
                            flex-direction: column;
                            gap: 1.25rem;
                        }

                        .pillars-list li {
                            background: var(--surface);
                            border: 1px solid var(--border);
                            padding: 1.5rem;
                            border-radius: 12px;
                            box-shadow: 0 4px 14px rgba(15, 23, 42, 0.08);
                            line-height: 1.7;
                        }

                        .pillars-list strong {
                            display: block;
                            color: var(--text-primary);
                        }

                        .pillars-list span {
                            color: var(--text-secondary);
                        }

                        .faq-grid {
                            display: grid;
                            grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
                            gap: 1.25rem;
                        }

                        .faq-grid details {
                            background: var(--surface);
                            border: 1px solid var(--border);
                            border-radius: 12px;
                            padding: 1.25rem 1.5rem;
                            box-shadow: 0 6px 16px rgba(15, 23, 42, 0.08);
                        }

                        .faq-grid summary {
                            cursor: pointer;
                            font-weight: 600;
                            color: var(--text-primary);
                            margin-bottom: 0.75rem;
                        }

                        .faq-grid p {
                            color: var(--text-secondary);
                            line-height: 1.6;
                        }

                        @media (max-width: 768px) {
                            .about-hero {
                                padding: 2rem;
                            }

                            .about-hero h1 {
                                font-size: 2.2rem;
                            }

                            .lead {
                                font-size: 1.1rem;
                            }
                        }
                    "#))
                }
            }
        }
    }
}
