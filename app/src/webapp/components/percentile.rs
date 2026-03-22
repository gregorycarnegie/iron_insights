use leptos::prelude::*;

#[component]
// False positive: `percentile_percent` is used in the view but the Leptos macro-generated
// props struct triggers a dead_code warning because rustc can't trace the field read.
pub(in crate::webapp) fn PercentilePanel(percentile_percent: Memo<Option<f32>>) -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"Your percentile"</h2>
            <div class="pct-track">
                <div
                    class="pct-fill"
                    style:width=move || format!(
                        "{:.1}%",
                        percentile_percent.get().unwrap_or(0.0).clamp(0.0, 100.0)
                    )
                ></div>
            </div>
            <p class="muted">
                {move || match percentile_percent.get() {
                    Some(value) => format!("Your marker: {:.1} / 100", value),
                    None => "Your marker appears after your matching group loads.".to_string(),
                }}
            </p>
        </section>
    }
}
