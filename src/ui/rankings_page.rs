// src/ui/rankings_page.rs - Rankings page with server-side rendering and filters
use maud::{html, Markup, DOCTYPE, PreEscaped};
use crate::models::{RankingsParams, RankingsResponse};
// Remove unused import

pub fn render_rankings_page(
    rankings: Option<&RankingsResponse>,
    params: &RankingsParams
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head_with_title("Rankings - Iron Insights"))
            body {
                div.container {
                    (render_header_with_active("rankings"))
                    main.rankings-main {
                        div.rankings-header {
                            h1 { "🏆 Powerlifting Rankings" }
                            p.subtitle {
                                "Top performers from the complete OpenPowerlifting dataset "
                                @if let Some(r) = rankings {
                                    span.total-count { "(" (r.total_count) " total records)" }
                                }
                            }
                        }

                        (render_rankings_filters(params))

                        @if let Some(rankings_data) = rankings {
                            (render_rankings_content(rankings_data, params))
                        } @else {
                            div.loading-state {
                                p { "Loading rankings..." }
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

fn render_head_with_title(title: &str) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            title { (title) }
            link rel="stylesheet" href="/static/css/style.css";
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet";
        }
    }
}

fn render_header_with_active(active_page: &str) -> Markup {
    html! {
        header.site-header {
            div.header-content {
                div.logo {
                    a href="/" { "💪 Iron Insights" }
                }
                nav.main-nav {
                    a href="/" class=(if active_page == "home" { "active" } else { "" }) { "Home" }
                    a href="/analytics" class=(if active_page == "analytics" { "active" } else { "" }) { "Analytics" }
                    a href="/rankings" class=(if active_page == "rankings" { "active" } else { "" }) { "Rankings" }
                    a href="/1rm" class=(if active_page == "1rm" { "active" } else { "" }) { "1RM Calc" }
                    a href="/about" class=(if active_page == "about" { "active" } else { "" }) { "About" }
                }
            }
        }
    }
}

fn render_rankings_filters(params: &RankingsParams) -> Markup {
    html! {
        div.rankings-filters {
            form #rankings-form method="get" action="/rankings" {
                div.filter-grid {
                    div.filter-group {
                        label for="sex" { "Sex" }
                        select #sex name="sex" {
                            option value="" selected=(params.sex.is_none()) { "All" }
                            option value="M" selected=(params.sex.as_deref() == Some("M")) { "Male" }
                            option value="F" selected=(params.sex.as_deref() == Some("F")) { "Female" }
                        }
                    }

                    div.filter-group {
                        label for="equipment" { "Equipment" }
                        select #equipment name="equipment" {
                            option value="" selected=(params.equipment.is_none()) { "All" }
                            option value="Raw" selected=(params.equipment.as_deref() == Some("Raw")) { "Raw" }
                            option value="Wraps" selected=(params.equipment.as_deref() == Some("Wraps")) { "Wraps" }
                            option value="Single-ply" selected=(params.equipment.as_deref() == Some("Single-ply")) { "Single-ply" }
                            option value="Multi-ply" selected=(params.equipment.as_deref() == Some("Multi-ply")) { "Multi-ply" }
                        }
                    }

                    div.filter-group {
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

                    div.filter-group {
                        label for="federation" { "Federation" }
                        select #federation name="federation" {
                            option value="" selected=(params.federation.is_none()) { "All" }
                            option value="ipf" selected=(params.federation.as_deref() == Some("ipf")) { "IPF" }
                            option value="usapl" selected=(params.federation.as_deref() == Some("usapl")) { "USAPL" }
                            option value="uspa" selected=(params.federation.as_deref() == Some("uspa")) { "USPA" }
                            option value="wrpf" selected=(params.federation.as_deref() == Some("wrpf")) { "WRPF" }
                        }
                    }

                    div.filter-group {
                        label for="sort_by" { "Sort By" }
                        select #sort_by name="sort_by" {
                            option value="dots" selected=(params.sort_by.as_deref() == Some("dots") || params.sort_by.is_none()) { "DOTS Score" }
                            option value="total" selected=(params.sort_by.as_deref() == Some("total")) { "Total" }
                            option value="squat" selected=(params.sort_by.as_deref() == Some("squat")) { "Squat" }
                            option value="bench" selected=(params.sort_by.as_deref() == Some("bench")) { "Bench" }
                            option value="deadlift" selected=(params.sort_by.as_deref() == Some("deadlift")) { "Deadlift" }
                        }
                    }

                    div.filter-group {
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
                    button type="submit" class="btn btn-primary" { "Apply Filters" }
                    a href="/rankings" class="btn btn-secondary" { "Clear" }
                }
            }
        }
    }
}

fn render_rankings_content(rankings: &RankingsResponse, params: &RankingsParams) -> Markup {
    html! {
        div.rankings-content {
            div.rankings-meta {
                p.results-info {
                    "Showing " (rankings.entries.len()) " of " (rankings.total_count) " results "
                    "(Page " (rankings.page) " of " (rankings.total_pages) ")"
                }
            }

            div.rankings-table-container {
                table.rankings-table {
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
        // Show all pages
        pages.extend(1..=total);
    } else if current <= 4 {
        // Show first 5 pages, ellipsis, last page
        pages.extend(1..=5);
        pages.push(0); // ellipsis
        pages.push(total);
    } else if current >= total - 3 {
        // Show first page, ellipsis, last 5 pages
        pages.push(1);
        pages.push(0); // ellipsis
        pages.extend((total - 4)..=total);
    } else {
        // Show first page, ellipsis, current-1, current, current+1, ellipsis, last page
        pages.push(1);
        pages.push(0); // ellipsis
        pages.extend((current - 1)..=(current + 1));
        pages.push(0); // ellipsis
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
        if sort_by != "dots" { // Only include if not default
            url.push_str(&format!("&sort_by={}", sort_by));
        }
    }

    url
}

fn render_rankings_scripts() -> Markup {
    html! {
        script {
            (PreEscaped(r#"
                // Progressive enhancement for rankings form
                document.addEventListener('DOMContentLoaded', function() {
                    const form = document.getElementById('rankings-form');
                    const selects = form.querySelectorAll('select');

                    // Auto-submit on filter change for progressive enhancement
                    selects.forEach(select => {
                        select.addEventListener('change', function() {
                            form.submit();
                        });
                    });

                    // Add loading state
                    form.addEventListener('submit', function() {
                        const submitBtn = form.querySelector('button[type="submit"]');
                        if (submitBtn) {
                            submitBtn.textContent = 'Loading...';
                            submitBtn.disabled = true;
                        }
                    });
                });
            "#))
        }
    }
}

fn render_rankings_styles() -> Markup {
    html! {
        style {
            r#"
            /* Rankings page specific styles */
            .rankings-main {
                max-width: 1200px;
                margin: 0 auto;
                padding: 2rem;
            }

            .rankings-header {
                text-align: center;
                margin-bottom: 2rem;
            }

            .rankings-header h1 {
                color: var(--text-primary, #2c3e50);
                margin-bottom: 0.5rem;
            }

            .subtitle {
                color: var(--text-secondary, #6c757d);
                font-size: 1.1rem;
            }

            .total-count {
                font-weight: 600;
                color: var(--primary, #e74c3c);
            }

            /* Filters */
            .rankings-filters {
                background: var(--surface, #f8f9fa);
                border-radius: 8px;
                padding: 1.5rem;
                margin-bottom: 2rem;
                border: 1px solid var(--border, #dee2e6);
            }

            .filter-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
                gap: 1rem;
                margin-bottom: 1rem;
            }

            .filter-group {
                display: flex;
                flex-direction: column;
            }

            .filter-group label {
                font-weight: 600;
                margin-bottom: 0.25rem;
                color: var(--text-primary, #495057);
            }

            .filter-group select {
                padding: 0.5rem;
                border: 1px solid var(--border, #ced4da);
                border-radius: 4px;
                background: white;
                font-size: 0.9rem;
                color: var(--text-primary);
            }

            .filter-actions {
                display: flex;
                gap: 1rem;
                justify-content: center;
            }

            .btn {
                padding: 0.75rem 1.5rem;
                border: none;
                border-radius: 4px;
                font-weight: 600;
                text-decoration: none;
                display: inline-block;
                text-align: center;
                cursor: pointer;
                transition: all 0.2s;
            }

            .btn-primary {
                background: var(--primary, #007bff);
                color: white;
            }

            .btn-primary:hover {
                background: var(--primary-dark, #0056b3);
            }

            .btn-secondary {
                background: var(--text-secondary, #6c757d);
                color: white;
            }

            .btn-secondary:hover {
                background: #545b62;
            }

            /* Rankings content */
            .rankings-content {
                margin-top: 2rem;
            }

            .rankings-meta {
                margin-bottom: 1rem;
            }

            .results-info {
                color: var(--text-secondary, #6c757d);
                font-size: 0.9rem;
                text-align: center;
            }

            /* Table */
            .rankings-table-container {
                overflow-x: auto;
                border-radius: 8px;
                box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                margin-bottom: 2rem;
                border: 1px solid var(--border, #dee2e6);
            }

            .rankings-table {
                width: 100%;
                border-collapse: collapse;
                background: var(--surface, white);
                font-size: 0.9rem;
            }

            .rankings-table th {
                background: var(--surface-dark, #343a40);
                color: white;
                padding: 0.75rem 0.5rem;
                text-align: left;
                font-weight: 600;
                white-space: nowrap;
            }

            .rankings-table td {
                padding: 0.75rem 0.5rem;
                border-bottom: 1px solid var(--border, #dee2e6);
                white-space: nowrap;
                color: var(--text-primary);
            }

            .rankings-table tbody tr:hover {
                background: var(--surface-secondary, #f8f9fa);
            }

            .rankings-table .rank {
                font-weight: 600;
                color: var(--primary, #e74c3c);
                text-align: center;
            }

            .rankings-table .name {
                font-weight: 600;
                max-width: 200px;
                overflow: hidden;
                text-overflow: ellipsis;
            }

            .rankings-table .dots {
                font-weight: 600;
                color: var(--success, #28a745);
            }

            /* Pagination */
            .pagination {
                display: flex;
                justify-content: center;
                align-items: center;
                gap: 0.5rem;
                margin-top: 2rem;
            }

            .page-link {
                padding: 0.5rem 1rem;
                text-decoration: none;
                color: var(--primary, #007bff);
                border: 1px solid var(--border, #dee2e6);
                border-radius: 4px;
                transition: all 0.2s;
            }

            .page-link:hover {
                background: var(--surface-secondary, #e9ecef);
            }

            .page-link.disabled {
                color: var(--text-secondary, #6c757d);
                cursor: not-allowed;
                pointer-events: none;
            }

            .page-numbers {
                display: flex;
                gap: 0.25rem;
            }

            .page-number {
                padding: 0.5rem 0.75rem;
                text-decoration: none;
                color: var(--primary, #007bff);
                border: 1px solid var(--border, #dee2e6);
                border-radius: 4px;
                transition: all 0.2s;
            }

            .page-number:hover {
                background: var(--surface-secondary, #e9ecef);
            }

            .page-number.current {
                background: var(--primary, #007bff);
                color: white;
                border-color: var(--primary, #007bff);
            }

            .page-ellipsis {
                padding: 0.5rem 0.75rem;
                color: var(--text-secondary, #6c757d);
            }

            /* Loading state */
            .loading-state {
                text-align: center;
                padding: 3rem;
                color: var(--text-secondary, #6c757d);
            }

            /* Responsive design */
            @media (max-width: 768px) {
                .rankings-main {
                    padding: 1rem;
                }

                .filter-grid {
                    grid-template-columns: 1fr;
                }

                .filter-actions {
                    flex-direction: column;
                }

                .rankings-table {
                    font-size: 0.8rem;
                }

                .rankings-table th,
                .rankings-table td {
                    padding: 0.5rem 0.25rem;
                }

                .pagination {
                    flex-wrap: wrap;
                }
            }
            "#
        }
    }
}