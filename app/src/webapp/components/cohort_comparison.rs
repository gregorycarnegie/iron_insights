use crate::webapp::models::CohortComparisonRow;
use leptos::prelude::*;
use std::collections::BTreeMap;

#[component]
pub(in crate::webapp) fn CohortComparisonPanel(
    rows: Memo<Vec<CohortComparisonRow>>,
    exact_deltas_enabled: ReadSignal<bool>,
    set_exact_deltas_enabled: WriteSignal<bool>,
    exact_percentiles: ReadSignal<BTreeMap<String, Option<f32>>>,
    exact_loading: ReadSignal<bool>,
    exact_error: ReadSignal<Option<String>>,
) -> impl IntoView {
    let current_exact_percentile = Memo::new(move |_| {
        let current_id = rows
            .get()
            .into_iter()
            .find(|row| row.is_current)
            .map(|row| row.id);
        let Some(current_id) = current_id else {
            return None;
        };
        exact_percentiles
            .get()
            .get(&current_id)
            .and_then(|value| *value)
    });

    view! {
        <section class="panel">
            <h3>"Cohort Comparison Table"</h3>
            <p class="muted">
                "Quick rows use embedded shard summaries only. Enable exact deltas to fetch extra histograms."
            </p>
            <label class="cohort-compare-toggle">
                <input
                    type="checkbox"
                    prop:checked=move || exact_deltas_enabled.get()
                    on:change=move |ev| {
                        set_exact_deltas_enabled.set(event_target_checked(&ev));
                    }
                />
                "Load exact percentile deltas"
            </label>
            <Show
                when=move || !rows.get().is_empty()
                fallback=move || view! { <p class="muted">"No comparison rows available for this selection."</p> }
            >
                <div class="comparison-table-wrap">
                    <table class="comparison-table">
                        <thead>
                            <tr>
                                <th>"Cohort"</th>
                                <th>"Filters"</th>
                                <th>"Lifters"</th>
                                <th>"Δ lifters"</th>
                                <th>"Observed range (kg)"</th>
                                <th>"Exact percentile"</th>
                                <th>"Δ percentile pts"</th>
                                <th>"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || rows.get()
                                key=|row| row.id.clone()
                                let:row
                            >
                                {{
                                let row_id = row.id.clone();
                                let row_id_for_percentile = row_id.clone();
                                let row_id_for_delta = row_id;
                                let status_class = if row.status_ok {
                                    "comparison-status-ok"
                                } else {
                                    "comparison-status-muted"
                                };
                                    view! {
                                        <tr class:comparison-current=row.is_current>
                                            <td>{row.label}</td>
                                            <td>
                                                {format!(
                                                    "wc {}, age {}, tested {}, metric {}",
                                                    row.wc, row.age, row.tested, row.metric
                                                )}
                                            </td>
                                            <td>{row.total.map(|value| value.to_string()).unwrap_or_else(|| "n/a".to_string())}</td>
                                            <td>
                                                {row.total_delta
                                                    .map(|delta| format!("{:+}", delta))
                                                    .unwrap_or_else(|| "n/a".to_string())}
                                            </td>
                                            <td>
                                                {match (row.min_kg, row.max_kg) {
                                                    (Some(min), Some(max)) => format!("{:.1} - {:.1}", min, max),
                                                    _ => "n/a".to_string(),
                                                }}
                                            </td>
                                            <td>
                                                {move || {
                                                    if !exact_deltas_enabled.get() {
                                                        return "off".to_string();
                                                    }
                                                    exact_percentiles
                                                        .get()
                                                        .get(&row_id_for_percentile)
                                                        .and_then(|value| *value)
                                                        .map(|pct| format!("{:.2}%", pct))
                                                        .unwrap_or_else(|| "n/a".to_string())
                                                }}
                                            </td>
                                            <td>
                                                {move || {
                                                    if !exact_deltas_enabled.get() {
                                                        return "off".to_string();
                                                    }
                                                    let row_pct = exact_percentiles
                                                        .get()
                                                        .get(&row_id_for_delta)
                                                        .and_then(|value| *value);
                                                    match (row_pct, current_exact_percentile.get()) {
                                                        (Some(row_pct), Some(base_pct)) => format!("{:+.2}", row_pct - base_pct),
                                                        _ => "n/a".to_string(),
                                                    }
                                                }}
                                            </td>
                                            <td class=status_class>{row.status}</td>
                                        </tr>
                                    }
                                }}
                            </For>
                        </tbody>
                    </table>
                </div>
            </Show>
            <Show when=move || exact_deltas_enabled.get() && exact_loading.get()>
                <p class="muted">"Loading exact percentile deltas..."</p>
            </Show>
            <Show when=move || exact_error.get().is_some()>
                <p class="muted">{move || exact_error.get().unwrap_or_default()}</p>
            </Show>
        </section>
    }
}
