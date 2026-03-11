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
                        <div class="nerd-metrics-grid">
                            <p><strong>"p1"</strong>{format!(" {:.1}", d.p01)}</p>
                            <p><strong>"p5"</strong>{format!(" {:.1}", d.p05)}</p>
                            <p><strong>"p10"</strong>{format!(" {:.1}", d.p10)}</p>
                            <p><strong>"p25"</strong>{format!(" {:.1}", d.p25)}</p>
                            <p><strong>"p50"</strong>{format!(" {:.1}", d.p50)}</p>
                            <p><strong>"p75"</strong>{format!(" {:.1}", d.p75)}</p>
                            <p><strong>"p90"</strong>{format!(" {:.1}", d.p90)}</p>
                            <p><strong>"p95"</strong>{format!(" {:.1}", d.p95)}</p>
                            <p><strong>"p99"</strong>{format!(" {:.1}", d.p99)}</p>
                            <p><strong>"IQR"</strong>{format!(" {:.1}", d.iqr)}</p>
                            <p>
                                <strong>"Central 80%"</strong>
                                {format!(" {:.1} to {:.1}", d.central_80_low, d.central_80_high)}
                            </p>
                            <p>
                                <strong>"Mode bin"</strong>
                                {format!(" {:.1} to {:.1} ({} lifters)", d.mode_bin_start, d.mode_bin_end, d.mode_bin_count)}
                            </p>
                            <p>
                                <strong>"Occupied bins"</strong>
                                {format!(" {} / {}", d.occupied_bins, d.total_bins)}
                            </p>
                            <p><strong>"Sparsity score"</strong>{format!(" {:.1}%", sparsity_pct)}</p>
                        </div>
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
