use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn PercentilePanel(
    percentile_percent: Memo<Option<f32>>,
    show_main_charts: ReadSignal<bool>,
    set_show_main_charts: WriteSignal<bool>,
) -> impl IntoView {
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
                    Some(value) => format!("You marker: {:.1} / 100", value),
                    None => "Your marker appears after a matching slice loads.".to_string(),
                }}
            </p>
            <button
                type="button"
                class="secondary"
                on:click=move |_| set_show_main_charts.update(|open| *open = !*open)
            >
                {move || if show_main_charts.get() { "Hide distribution charts" } else { "View distribution charts" }}
            </button>
        </section>
    }
}
