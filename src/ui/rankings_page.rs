// src/ui/rankings_page.rs - Rankings page with server-side rendering and filters
use crate::{
    models::{RankingsParams, RankingsResponse},
    ui::components::*,
};
use maud::{DOCTYPE, Markup, PreEscaped, html};

pub fn render_rankings_page(
    rankings: Option<&RankingsResponse>,
    params: &RankingsParams,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head_minimal())
            body.no-js {
                div.container {
                    (render_header())
                    main #main-content.page-transition role="main" {
                        div.main-content.rankings-page {
                            (render_rankings_hero(rankings))
                            (render_rankings_filters(params))

                            @if let Some(rankings_data) = rankings {
                                (render_rankings_content(rankings_data, params))
                            } @else {
                                div.loading-state.glass-card {
                                    p { "Loading rankings..." }
                                }
                            }
                        }
                    }
                }
                (render_rankings_scripts())
                (render_rankings_styles())
            }
        }
    }
}

fn render_rankings_hero(rankings: Option<&RankingsResponse>) -> Markup {
    html! {
        section.rankings-hero aria-labelledby="rankings-heading" {
            article.feature-card.glass-card.card-hover.rankings-feature {
                div.icon-wrap aria-hidden="true" {
                    svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" {
                        path d="M8 21h8" {}
                        path d="M12 17v4" {}
                        path d="M7 4h10" {}
                        path d="M17 4v2a3 3 0 0 1-3 3h-4a3 3 0 0 1-3-3V4" {}
                        path d="M7 4v2a3 3 0 0 1-3 3H3a3 3 0 0 1-3-3V4z" {}
                        path d="M17 4v2a3 3 0 0 0 3 3h1a3 3 0 0 0 3-3V4z" {}
                    }
                }
                h1.feature-title #rankings-heading { "Powerlifting Rankings" }
                p.feature-desc {
                    "Explore the strongest totals from the open powerlifting community with rich filters and competition-ready context."
                }
                @if let Some(response) = rankings {
                    div.rankings-meta {
                        span.meta-label { "Records Tracked" }
                        span.meta-value { (response.total_count) }
                    }
                } @else {
                    div.rankings-meta.skeleton {
                        span.meta-label { "Records Tracked" }
                        span.meta-value { "—" }
                    }
                }
            }
        }
    }
}

fn render_rankings_filters(params: &RankingsParams) -> Markup {
    html! {
        section.rankings-controls aria-label="Rankings filters" {
            h2.section-heading { "Refine the leaderboard" }
            form #rankings-form method="get" action="/rankings" {
                div.filter-card.glass-card {
                    div.filter-grid {
                        div.filter-group.control-group {
                            label for="sex" { "Sex" }
                            select #sex name="sex" {
                                option value="" selected=(params.sex.is_none()) { "All" }
                                option value="M" selected=(params.sex.as_deref() == Some("M")) { "Male" }
                                option value="F" selected=(params.sex.as_deref() == Some("F")) { "Female" }
                            }
                        }

                        div.filter-group.control-group {
                            label for="equipment" { "Equipment" }
                            select #equipment name="equipment" {
                                option value="" selected=(params.equipment.is_none()) { "All" }
                                option value="Raw" selected=(params.equipment.as_deref() == Some("Raw")) { "Raw" }
                                option value="Wraps" selected=(params.equipment.as_deref() == Some("Wraps")) { "Wraps" }
                                option value="Single-ply" selected=(params.equipment.as_deref() == Some("Single-ply")) { "Single-ply" }
                                option value="Multi-ply" selected=(params.equipment.as_deref() == Some("Multi-ply")) { "Multi-ply" }
                            }
                        }

                        div.filter-group.control-group {
                            label for="weight_class" { "Weight Class" }
                            select #weight_class name="weight_class" {
                                option value="" selected=(params.weight_class.is_none()) { "All" }
                                option value="59" selected=(params.weight_class.as_deref() == Some("59")) { "-59kg" }
                                option value="66" selected=(params.weight_class.as_deref() == Some("66")) { "-66kg" }
                                option value="74" selected=(params.weight_class.as_deref() == Some("74")) { "-74kg" }
                                option value="83" selected=(params.weight_class.as_deref() == Some("83")) { "-83kg" }
                                option value="93" selected=(params.weight_class.as_deref() == Some("93")) { "-93kg" }
                                option value="105" selected=(params.weight_class.as_deref() == Some("105")) { "-105kg" }
                                option value="120" selected=(params.weight_class.as_deref() == Some("120")) { "-120kg" }
                                option value="120+" selected=(params.weight_class.as_deref() == Some("120+")) { "120kg+" }
                            }
                        }

                        div.filter-group.control-group {
                            label for="federation" { "Federation" }
                            select #federation name="federation" {
                                option value="" selected=(params.federation.is_none()) { "All" }
                                option value="ipf" selected=(params.federation.as_deref() == Some("ipf")) { "IPF" }
                                option value="usapl" selected=(params.federation.as_deref() == Some("usapl")) { "USAPL" }
                                option value="uspa" selected=(params.federation.as_deref() == Some("uspa")) { "USPA" }
                                option value="wrpf" selected=(params.federation.as_deref() == Some("wrpf")) { "WRPF" }
                            }
                        }

                        div.filter-group.control-group {
                            label for="sort_by" { "Sort By" }
                            select #sort_by name="sort_by" {
                                option value="dots" selected=(params.sort_by.as_deref() == Some("dots") || params.sort_by.is_none()) { "DOTS Score" }
                                option value="total" selected=(params.sort_by.as_deref() == Some("total")) { "Total" }
                                option value="squat" selected=(params.sort_by.as_deref() == Some("squat")) { "Squat" }
                                option value="bench" selected=(params.sort_by.as_deref() == Some("bench")) { "Bench" }
                                option value="deadlift" selected=(params.sort_by.as_deref() == Some("deadlift")) { "Deadlift" }
                            }
                        }

                        div.filter-group.control-group {
                            label for="year" { "Year" }
                            select #year name="year" {
                                option value="" selected=(params.year.is_none()) { "All Years" }
                                @for year in (2020..=2024).rev() {
                                    option value=(year) selected=(params.year == Some(year)) { (year) }
                                }
                            }
                        }
                    }

                    div.filter-actions {
                        button.btn.btn-primary type="submit" { "Apply Filters" }
                        a.btn.btn-tertiary href="/rankings" { "Clear" }
                    }
                }
            }
        }
    }
}

fn render_rankings_content(rankings: &RankingsResponse, params: &RankingsParams) -> Markup {
    html! {
        section.rankings-body {
            div.rankings-summary {
                p.results-info {
                    "Showing " (rankings.entries.len()) " of " (rankings.total_count) " results "
                    "(Page " (rankings.page) " of " (rankings.total_pages) ")"
                }
            }

            div.rankings-table-container.glass-card {
                table.data-table.rankings-table {
                    thead {
                        tr {
                            th { "Rank" }
                            th { "Name" }
                            th { "Fed" }
                            th { "Date" }
                            th { "Sex" }
                            th { "Eq" }
                            th { "Class" }
                            th { "BW" }
                            th { "Squat" }
                            th { "Bench" }
                            th { "Deadlift" }
                            th { "Total" }
                            th { "DOTS" }
                        }
                    }
                    tbody {
                        @for entry in &rankings.entries {
                            tr {
                                td.rank { (entry.rank) }
                                td.name { (entry.name) }
                                td.federation { (entry.federation) }
                                td.date { (entry.date) }
                                td.sex { (entry.sex) }
                                td.equipment { (entry.equipment) }
                                td.weight-class { (entry.weight_class) }
                                td.bodyweight { (format!("{:.1}", entry.bodyweight)) }
                                td.squat { (format!("{:.1}", entry.squat)) }
                                td.bench { (format!("{:.1}", entry.bench)) }
                                td.deadlift { (format!("{:.1}", entry.deadlift)) }
                                td.total { (format!("{:.1}", entry.total)) }
                                td.dots { (format!("{:.1}", entry.dots)) }
                            }
                        }
                    }
                }
            }

            (render_pagination(rankings, params))
        }
    }
}

fn render_pagination(rankings: &RankingsResponse, params: &RankingsParams) -> Markup {
    let current_page = rankings.page;
    let total_pages = rankings.total_pages;

    html! {
        div.pagination {
            @if rankings.has_prev {
                a.page-link href=(build_page_url(params, current_page - 1)) { "← Previous" }
            } @else {
                span.page-link.disabled { "← Previous" }
            }

            div.page-numbers {
                @for page in pagination_range(current_page, total_pages) {
                    @if page == current_page {
                        span.page-number.current { (page) }
                    } @else if page == 0 {
                        span.page-ellipsis { "..." }
                    } @else {
                        a.page-number href=(build_page_url(params, page)) { (page) }
                    }
                }
            }

            @if rankings.has_next {
                a.page-link href=(build_page_url(params, current_page + 1)) { "Next →" }
            } @else {
                span.page-link.disabled { "Next →" }
            }
        }
    }
}

fn pagination_range(current: u32, total: u32) -> Vec<u32> {
    let mut pages = Vec::new();

    if total <= 7 {
        pages.extend(1..=total);
    } else if current <= 4 {
        pages.extend(1..=5);
        pages.push(0);
        pages.push(total);
    } else if current >= total - 3 {
        pages.push(1);
        pages.push(0);
        pages.extend((total - 4)..=total);
    } else {
        pages.push(1);
        pages.push(0);
        pages.extend((current - 1)..=(current + 1));
        pages.push(0);
        pages.push(total);
    }

    pages
}

fn build_page_url(params: &RankingsParams, page: u32) -> String {
    let mut url = format!("/rankings?page={}", page);

    if let Some(sex) = &params.sex {
        url.push_str(&format!("&sex={}", sex));
    }
    if let Some(equipment) = &params.equipment {
        url.push_str(&format!("&equipment={}", equipment));
    }
    if let Some(weight_class) = &params.weight_class {
        url.push_str(&format!("&weight_class={}", weight_class));
    }
    if let Some(federation) = &params.federation {
        url.push_str(&format!("&federation={}", federation));
    }
    if let Some(year) = params.year {
        url.push_str(&format!("&year={}", year));
    }
    if let Some(sort_by) = &params.sort_by {
        if sort_by != "dots" {
            url.push_str(&format!("&sort_by={}", sort_by));
        }
    }

    url
}

fn render_rankings_scripts() -> Markup {
    html! {
        script {
            (PreEscaped(r#"
                document.addEventListener('DOMContentLoaded', function() {
                    const form = document.getElementById('rankings-form');
                    const selects = form ? form.querySelectorAll('select') : [];

                    selects.forEach(select => {
                        select.addEventListener('change', function() {
                            if (form) {
                                form.submit();
                            }
                        });
                    });

                    if (form) {
                        form.addEventListener('submit', function() {
                            const submitBtn = form.querySelector('button[type=\"submit\"]');
                            if (submitBtn) {
                                submitBtn.textContent = 'Loading...';
                                submitBtn.disabled = true;
                            }
                        });
                    }
                });
            "#))
        }
    }
}

fn render_rankings_styles() -> Markup {
    html! {
        style {
            r#"
            .rankings-page {
                display: flex;
                flex-direction: column;
                gap: 2.5rem;
                padding: 2rem 2.25rem 3rem;
            }

            .rankings-hero {
                max-width: 1100px;
                margin: 0 auto;
                width: 100%;
            }

            .rankings-feature {
                padding: 2.25rem;
                display: flex;
                flex-direction: column;
                gap: 1rem;
            }

            .rankings-feature .icon-wrap {
                width: 3rem;
                height: 3rem;
                border-radius: 0.75rem;
                background: rgba(var(--primary-rgb), 0.12);
                display: flex;
                align-items: center;
                justify-content: center;
                color: var(--primary);
            }

            .rankings-meta {
                display: inline-flex;
                align-items: baseline;
                gap: 0.45rem;
                padding: 0.45rem 0.85rem;
                border-radius: 999px;
                background: rgba(var(--primary-rgb), 0.08);
                color: var(--primary);
                font-weight: 600;
                width: fit-content;
            }

            .rankings-meta .meta-label {
                font-size: 0.85rem;
                color: var(--text-secondary);
                text-transform: uppercase;
                letter-spacing: 0.08em;
                font-weight: 600;
            }

            .rankings-meta .meta-value {
                font-size: 1.1rem;
            }

            .rankings-meta.skeleton {
                opacity: 0.6;
            }

            .section-heading {
                font-size: 1.35rem;
                font-weight: 700;
                color: var(--text-primary);
                margin-bottom: 1rem;
            }

            .filter-card {
                padding: 1.75rem;
                display: flex;
                flex-direction: column;
                gap: 1.25rem;
            }

            .filter-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
                gap: 1rem;
            }

            .filter-group label {
                font-weight: 600;
                color: var(--text-secondary);
                margin-bottom: 0.35rem;
            }

            .filter-group select {
                padding: 0.6rem 0.75rem;
                border-radius: 0.5rem;
                border: 1px solid var(--border);
                background: var(--surface);
                font-size: 0.95rem;
                color: var(--text-primary);
                transition: border-color 0.2s, box-shadow 0.2s;
            }

            .filter-group select:focus {
                border-color: var(--primary);
                box-shadow: 0 0 0 3px rgba(var(--primary-rgb), 0.16);
            }

            .filter-actions {
                display: flex;
                flex-wrap: wrap;
                gap: 0.75rem;
                justify-content: flex-end;
            }

            .rankings-body {
                display: flex;
                flex-direction: column;
                gap: 1.5rem;
            }

            .rankings-summary {
                text-align: center;
            }

            .results-info {
                color: var(--text-secondary);
                font-size: 0.95rem;
            }

            .rankings-table-container {
                overflow-x: auto;
            }

            .rankings-table {
                min-width: 900px;
            }

            .rankings-table thead {
                background: linear-gradient(135deg, rgba(var(--primary-rgb), 0.68), rgba(var(--primary-rgb), 0.52));
            }

            .rankings-table th {
                color: #fff;
                white-space: nowrap;
            }

            .rankings-table tbody tr {
                background: var(--surface);
            }

            .rankings-table tbody tr:nth-child(even) {
                background: var(--surface-secondary);
            }

            .rankings-table td {
                white-space: nowrap;
                color: var(--text-primary);
            }

            .rankings-table .rank {
                font-weight: 700;
                text-align: center;
                color: var(--primary);
            }

            .rankings-table .name {
                font-weight: 600;
                max-width: 220px;
                overflow: hidden;
                text-overflow: ellipsis;
            }

            .rankings-table .dots {
                font-weight: 700;
                color: var(--secondary);
            }

            .pagination {
                display: flex;
                justify-content: center;
                align-items: center;
                gap: 0.5rem;
                flex-wrap: wrap;
                margin-top: 1rem;
            }

            .page-link,
            .page-number {
                display: inline-flex;
                align-items: center;
                justify-content: center;
                padding: 0.45rem 0.9rem;
                border-radius: 999px;
                border: 1px solid var(--border);
                background: var(--surface);
                font-weight: 600;
                color: var(--text-secondary);
                text-decoration: none;
                transition: all 0.2s ease;
            }

            .page-link:hover,
            .page-number:hover {
                color: var(--primary);
                border-color: rgba(var(--primary-rgb), 0.35);
                box-shadow: 0 6px 16px rgba(var(--primary-rgb), 0.18);
            }

            .page-link.disabled {
                opacity: 0.45;
                pointer-events: none;
            }

            .page-number.current {
                background: var(--primary);
                color: #fff;
                border-color: var(--primary);
                box-shadow: 0 10px 24px rgba(var(--primary-rgb), 0.28);
            }

            .page-ellipsis {
                padding: 0.45rem 0.75rem;
                color: var(--text-tertiary);
            }

            .loading-state {
                padding: 2.5rem;
                text-align: center;
                color: var(--text-secondary);
            }

            @media (max-width: 1024px) {
                .rankings-page {
                    padding: 1.5rem 1.75rem 2.5rem;
                }
            }

            @media (max-width: 768px) {
                .rankings-page {
                    padding: 1.5rem;
                }

                .rankings-feature {
                    padding: 1.75rem;
                }

                .filter-actions {
                    justify-content: center;
                }

                .rankings-table {
                    min-width: 720px;
                    font-size: 0.85rem;
                }
            }

            @media (max-width: 540px) {
                .rankings-page {
                    padding: 1.25rem;
                }

                .filter-grid {
                    grid-template-columns: 1fr;
                }

                .rankings-table {
                    min-width: 640px;
                }
            }
            "#
        }
    }
}
