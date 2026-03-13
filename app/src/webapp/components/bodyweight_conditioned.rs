use crate::core::BodyweightConditionedStats;
use crate::webapp::helpers::kg_to_display;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn BodyweightConditionedPanel(
    calculated: ReadSignal<bool>,
    conditioned: Memo<Option<BodyweightConditionedStats>>,
    use_lbs: ReadSignal<bool>,
    unit_label: Memo<&'static str>,
) -> impl IntoView {
    view! {
        <section class="panel">
            <h3>"Bodyweight-Conditioned Percentile"</h3>
            <Show
                when=move || calculated.get()
                fallback=move || {
                    view! {
                        <p class="muted">"Calculate first to estimate bodyweight-conditioned percentile."</p>
                    }
                }
            >
                <Show
                    when=move || conditioned.get().is_some()
                    fallback=move || {
                        view! {
                            <p class="muted">
                                "This metric appears after the nerd heatmap loads for the selected slice."
                            </p>
                        }
                    }
                >
                    {move || {
                        let Some(stats) = conditioned.get() else {
                            return view! { <p class="muted">"Bodyweight-conditioned metric unavailable."</p> }.into_any();
                        };
                        let bw_low = kg_to_display(stats.bw_window_low, use_lbs.get());
                        let bw_high = kg_to_display(stats.bw_window_high, use_lbs.get());
                        let neighborhood_pct = stats.neighborhood_share * 100.0;
                        view! {
                            <p class="muted">
                                {format!(
                                    "Among nearby bodyweights ({:.1}-{:.1} {}), you're stronger than {:.1}% (rank ~{} / {}).",
                                    bw_low,
                                    bw_high,
                                    unit_label.get(),
                                    stats.percentile * 100.0,
                                    stats.rank,
                                    stats.total_nearby
                                )}
                            </p>
                            {metric_grid! {
                                "Nearby lifters" => format!(" {}", stats.total_nearby),
                                "Current BW bin" => format!(
                                    " {:.1}-{:.1} {}",
                                    kg_to_display(stats.bw_bin_low, use_lbs.get()),
                                    kg_to_display(stats.bw_bin_high, use_lbs.get()),
                                    unit_label.get()
                                ),
                                "Current heat cell count" => format!(" {}", stats.local_cell_count),
                                "3x3 neighborhood count" => format!(" {}", stats.neighborhood_count),
                                "Neighborhood share" => format!(" {:.2}%", neighborhood_pct),
                                "Lift bin (heatmap x)" => format!(
                                    " {:.1}-{:.1}",
                                    stats.lift_bin_low,
                                    stats.lift_bin_high
                                ),
                            }}
                        }.into_any()
                    }}
                </Show>
            </Show>
        </section>
    }
}
