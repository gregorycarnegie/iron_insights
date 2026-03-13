use crate::core::HistogramDensity;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn RarityPanel(
    density: Memo<Option<HistogramDensity>>,
    hist_x_label: Memo<String>,
) -> impl IntoView {
    view! {
        <section class="panel">
            <h3>"Rarity and Crowding"</h3>
            <Show
                when=move || density.get().is_some()
                fallback=move || view! { <p class="muted">"Run a calculation to see local crowding."</p> }
            >
                {move || {
                    let Some(d) = density.get() else {
                        return view! { <p class="muted">"Density snapshot unavailable."</p> }.into_any();
                    };
                    let bucket = match d.label {
                        "dense middle" => "Dense middle",
                        "moderately common" => "Moderately common",
                        "rare air" => "Rare air",
                        _ => "Extreme tail",
                    };
                    let neighborhood_pct = d.neighborhood_share * 100.0;
                    let local_ratio_pct = d.local_density_ratio * 100.0;
                    view! {
                        <p class="muted">
                            {format!(
                                "{}: {} (bin {:.1} to {:.1} on {}).",
                                bucket, d.current_bin_count, d.bin_start, d.bin_end, hist_x_label.get()
                            )}
                        </p>
                        {metric_grid! {
                            "Current bin count" => format!(" {}", d.current_bin_count),
                            "Left bin count" => format!(" {}", d.left_bin_count),
                            "Right bin count" => format!(" {}", d.right_bin_count),
                            "Neighbor share" => format!(" {:.1}%", neighborhood_pct),
                            "Vs mode density" => format!(" {:.1}%", local_ratio_pct),
                            "Label" => format!(" {}", d.label),
                        }}
                    }.into_any()
                }}
            </Show>
        </section>
    }
}
