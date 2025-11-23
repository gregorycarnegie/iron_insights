use crate::AssetManifest;
use crate::components::*;
use maud::{DOCTYPE, Markup, html};

pub fn render_donate_page(manifest: &AssetManifest) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head_minimal(manifest))
            body.no-js {
                div.container {
                    (render_header(Some("/donate")))
                    main #main-content.page-transition role="main" {
                        div.main-content.donate-page {
                            section.donate-hero aria-labelledby="donate-heading" {
                                h1 #donate-heading { "Support Iron Insights" }
                                p.lead {
                                    "Iron Insights is an independent project. Your support keeps the servers running, funds data updates, and helps us build new tools for the strength community."
                                }
                                div.hero-actions {
                                    a.btn.btn-primary href="https://www.paypal.com/donate" target="_blank" rel="noopener" { "Donate with PayPal" }
                                    a.btn.btn-secondary href="mailto:support@ironinsights.app" { "Contact the Team" }
                                }
                            }

                            section.donation-options aria-labelledby="options-heading" {
                                h2 #options-heading { "Choose the way you want to contribute" }
                                div.donation-grid role="list" {
                                    article.donation-card role="listitem" {
                                        h3 { "PayPal" }
                                        p { "Fast, secure, and familiar. Use PayPal for one-time or recurring support." }
                                        a.btn.btn-primary href="https://www.paypal.com/donate" target="_blank" rel="noopener" { "Open PayPal" }
                                        small.note { "Update this link with your hosted PayPal button when ready." }
                                    }
                                    article.donation-card role="listitem" {
                                        h3 { "Bitcoin (BTC)" }
                                        p { "Send BTC from any wallet using the address below." }
                                        div.wallet {
                                            span.label { "BTC address" }
                                            code { "bc1-your-bitcoin-address" }
                                        }
                                    }
                                    article.donation-card role="listitem" {
                                        h3 { "Ethereum (ETH)" }
                                        p { "Support the project with ETH or compatible ERC-20 tokens." }
                                        div.wallet {
                                            span.label { "ETH address" }
                                            code { "0xYourEthereumAddress" }
                                        }
                                    }
                                    article.donation-card role="listitem" {
                                        h3 { "Dogecoin (DOGE)" }
                                        p { "Fuel the gains with everyone's favorite meme coin." }
                                        div.wallet {
                                            span.label { "DOGE address" }
                                            code { "DYourDogecoinAddress" }
                                        }
                                    }
                                }
                            }

                            section.impact aria-labelledby="impact-heading" {
                                h2 #impact-heading { "Where your support goes" }
                                ul.impact-list {
                                    li { "Infrastructure: hosting, data storage, and background processing for new datasets." }
                                    li { "Research and development: experimentation with new analytics, viz types, and calculators." }
                                    li { "Community features: collaboration tools for teams, coaches, and meet organizers." }
                                }
                            }

                            section.donate-faq aria-labelledby="donate-faq-heading" {
                                h2 #donate-faq-heading { "Common questions" }
                                div.faq-grid {
                                    details {
                                        summary { "Is my contribution tax-deductible?" }
                                        p { "Iron Insights is currently a community-driven project and does not operate as a registered nonprofit. Contributions are not tax-deductible." }
                                    }
                                    details {
                                        summary { "Can I set up a recurring donation?" }
                                        p { "Yes. PayPal supports monthly contributions. For crypto, consider using an exchange or wallet that automates recurring transfers." }
                                    }
                                    details {
                                        summary { "Do you offer sponsorships or partnerships?" }
                                        p { "We are open to aligned collaborations that benefit lifters. Reach out so we can discuss availability and impact." }
                                    }
                                    details {
                                        summary { "How else can I help?" }
                                        p { "Spread the word, contribute code or documentation, and share feedback on features you want to see next." }
                                    }
                                }
                            }
                        }
                    }
                }
                style {
                    (maud::PreEscaped(r#"
                        .donate-page {
                            display: flex;
                            flex-direction: column;
                            gap: 3rem;
                            padding: 2rem 0 4rem;
                        }

                        .donate-hero {
                            background: linear-gradient(135deg, var(--secondary-light), var(--surface));
                            border-radius: 16px;
                            padding: 3rem;
                            box-shadow: 0 12px 32px rgba(14, 165, 233, 0.18);
                            text-align: left;
                        }

                        .donate-hero h1 {
                            font-size: 2.75rem;
                            margin-bottom: 1rem;
                            color: var(--text-primary);
                        }

                        .donate-hero .lead {
                            font-size: 1.2rem;
                            color: var(--text-secondary);
                            margin-bottom: 1.5rem;
                        }

                        .hero-actions {
                            display: flex;
                            flex-wrap: wrap;
                            gap: 1rem;
                        }

                        .donation-grid {
                            display: grid;
                            grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
                            gap: 1.5rem;
                        }

                        .donation-card {
                            background: var(--surface);
                            border: 1px solid var(--border);
                            border-radius: 12px;
                            padding: 1.75rem;
                            box-shadow: 0 6px 18px rgba(15, 23, 42, 0.08);
                            display: flex;
                            flex-direction: column;
                            gap: 1rem;
                        }

                        .donation-card h3 {
                            margin: 0;
                            font-size: 1.3rem;
                            color: var(--text-primary);
                        }

                        .donation-card p {
                            color: var(--text-secondary);
                            line-height: 1.6;
                        }

                        .donation-card .btn {
                            align-self: flex-start;
                        }

                        .donation-card .note {
                            color: var(--text-muted);
                            font-size: 0.85rem;
                        }

                        .wallet {
                            background: var(--surface-secondary);
                            border-radius: 8px;
                            padding: 0.85rem 1rem;
                            border: 1px dashed var(--border);
                        }

                        .wallet .label {
                            display: block;
                            font-size: 0.85rem;
                            color: var(--text-muted);
                            margin-bottom: 0.25rem;
                        }

                        .wallet code {
                            font-family: "Fira Code", "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
                            font-size: 0.95rem;
                            color: var(--text-primary);
                            word-break: break-all;
                        }

                        .impact-list {
                            list-style: none;
                            padding: 0;
                            margin: 0;
                            display: flex;
                            flex-direction: column;
                            gap: 1rem;
                        }

                        .impact-list li {
                            background: var(--surface);
                            border: 1px solid var(--border);
                            border-radius: 12px;
                            padding: 1.25rem 1.5rem;
                            box-shadow: 0 4px 14px rgba(15, 23, 42, 0.08);
                            line-height: 1.6;
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

                        .btn {
                            display: inline-flex;
                            align-items: center;
                            gap: 0.5rem;
                            padding: 0.75rem 1.5rem;
                            border-radius: 8px;
                            font-weight: 600;
                            text-decoration: none;
                            transition: all 0.2s ease;
                        }

                        .btn-primary {
                            background: var(--primary);
                            color: #fff;
                        }

                        .btn-primary:hover {
                            background: var(--primary-dark);
                            transform: translateY(-2px);
                        }

                        .btn-secondary {
                            background: var(--surface);
                            color: var(--text-primary);
                            border: 2px solid var(--border);
                        }

                        .btn-secondary:hover {
                            background: var(--border);
                            transform: translateY(-2px);
                        }

                        @media (max-width: 768px) {
                            .donate-hero {
                                padding: 2rem;
                            }

                            .donate-hero h1 {
                                font-size: 2.2rem;
                            }

                            .donate-hero .lead {
                                font-size: 1.05rem;
                            }

                            .hero-actions {
                                flex-direction: column;
                                align-items: stretch;
                            }

                            .btn {
                                justify-content: center;
                            }
                        }
                    "#))
                }
            }
        }
    }
}
