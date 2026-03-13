use crate::core::HistogramDiagnostics;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn DistributionDiagnosticsPanel(
    diagnostics: Memo<Option<HistogramDiagnostics>>,
    hist_x_label: Memo<String>,
) -> impl IntoView {
    view! {
        <section class="panel">
            <h3>"Distribution Diagnostics"</h3>
            <Show
                when=move || diagnostics.get().is_some()
                fallback=move || view! { <p class="muted">"Run a calculation to load diagnostics."</p> }
            >
                {move || {
                    let Some(d) = diagnostics.get() else {
                        return view! { <p class="muted">"Distribution diagnostics unavailable."</p> }.into_any();
                    };
                    let sparsity_pct = d.sparsity_score * 100.0;
                    let tiny_sample_warning = d.tiny_sample_warning;
                    let total_lifters = d.total_lifters;
                    view! {
                        <p class="muted">
                            {format!("Computed from current histogram bins for {}.", hist_x_label.get())}
                        </p>
                        {metric_grid! {
                            "p1" => format!(" {:.1}", d.p01),
                            "p5" => format!(" {:.1}", d.p05),
                            "p10" => format!(" {:.1}", d.p10),
                            "p25" => format!(" {:.1}", d.p25),
                            "p50" => format!(" {:.1}", d.p50),
                            "p75" => format!(" {:.1}", d.p75),
                            "p90" => format!(" {:.1}", d.p90),
                            "p95" => format!(" {:.1}", d.p95),
                            "p99" => format!(" {:.1}", d.p99),
                            "IQR" => format!(" {:.1}", d.iqr),
                            "Central 80%" => {
                                format!(" {:.1} to {:.1}", d.central_80_low, d.central_80_high)
                            },
                            "Mode bin" => {
                                format!(
                                    " {:.1} to {:.1} ({} lifters)",
                                    d.mode_bin_start,
                                    d.mode_bin_end,
                                    d.mode_bin_count
                                )
                            },
                            "Occupied bins" => format!(" {} / {}", d.occupied_bins, d.total_bins),
                            "Sparsity score" => format!(" {:.1}%", sparsity_pct),
                        }}
                        <Show when=move || tiny_sample_warning>
                            <p class="muted">
                                {format!(
                                    "Sample quality warning: only {} lifters in this slice, so tail estimates are noisy.",
                                    total_lifters
                                )}
                            </p>
                        </Show>
                    }.into_any()
                }}
            </Show>
        </section>
    }
}
