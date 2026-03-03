use crate::core::HistogramBin;
use crate::webapp::charts::render_histogram_svg;
use leptos::html::Canvas;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn ChartsPanel(
    show_main_charts: ReadSignal<bool>,
    rebinned_hist: Memo<Option<HistogramBin>>,
    user_lift: Memo<f32>,
    hist_x_label: Memo<String>,
    canvas_ref: NodeRef<Canvas>,
) -> impl IntoView {
    view! {
        <Show when=move || show_main_charts.get()>
            <section class="panel">
                <h2>"Histogram"</h2>
                {move || match rebinned_hist.get() {
                    Some(h) => render_histogram_svg(&h, user_lift.get(), &hist_x_label.get()),
                    None => view! { <p>"No histogram available."</p> }.into_any(),
                }}
            </section>

            <section class="panel">
                <h2>{move || format!("Bodyweight vs {}", hist_x_label.get())}</h2>
                <canvas node_ref=canvas_ref width="800" height="420"></canvas>
            </section>
        </Show>
    }
}
