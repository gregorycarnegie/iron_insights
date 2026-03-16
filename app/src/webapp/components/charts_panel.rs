use crate::core::{HistogramBin, histogram_mean_stddev};
use crate::webapp::charts::render_histogram_svg;
use leptos::html::Canvas;
use leptos::prelude::*;

fn format_hist_stat(value: f32, x_label: &str) -> String {
    if x_label == "Lift (kg)" {
        format!("{value:.1} kg")
    } else if x_label.ends_with("Points") {
        format!("{value:.2} pts")
    } else {
        format!("{value:.2}")
    }
}

#[component]
pub(in crate::webapp) fn ChartsPanel(
    rebinned_hist: Memo<Option<HistogramBin>>,
    user_lift: Memo<f32>,
    hist_x_label: Memo<String>,
    canvas_ref: NodeRef<Canvas>,
    show_heat_indicator: ReadSignal<bool>,
    set_show_heat_indicator: WriteSignal<bool>,
) -> impl IntoView {
    let (show_hist_indicator, set_show_hist_indicator) = signal(true);

    view! {
        <section class="panel">
            <div class="panel-titlebar">
                <h2>"Histogram"</h2>
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| set_show_hist_indicator.update(|value| *value = !*value)
                >
                    {move || {
                        if show_hist_indicator.get() {
                            "Hide input indicator"
                        } else {
                            "Show input indicator"
                        }
                    }}
                </button>
            </div>
            {move || match rebinned_hist.get() {
                Some(h) => render_histogram_svg(
                    &h,
                    show_hist_indicator.get().then(|| user_lift.get()),
                    &hist_x_label.get(),
                ),
                None => view! { <p>"No histogram available."</p> }.into_any(),
            }}
            <Show when=move || rebinned_hist.get().is_some()>
                {move || {
                    let Some(hist) = rebinned_hist.get() else {
                        return view! { <></> }.into_any();
                    };
                    let Some((mean, std_dev)) = histogram_mean_stddev(Some(&hist)) else {
                        return view! { <></> }.into_any();
                    };
                    let x_label = hist_x_label.get();
                    view! {
                        <>
                            {metric_grid! {
                                "Mean" => format!(" {}", format_hist_stat(mean, &x_label)),
                                "SD" => format!(" {}", format_hist_stat(std_dev, &x_label)),
                            }}
                            <p class="muted">
                                "Mean and standard deviation are estimated from the rebinned histogram bin centers."
                            </p>
                        </>
                    }
                    .into_any()
                }}
            </Show>
        </section>

        <section class="panel">
            <div class="panel-titlebar">
                <h2>{move || format!("Bodyweight vs {}", hist_x_label.get())}</h2>
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| set_show_heat_indicator.update(|value| *value = !*value)
                >
                    {move || {
                        if show_heat_indicator.get() {
                            "Hide input indicator"
                        } else {
                            "Show input indicator"
                        }
                    }}
                </button>
            </div>
            <canvas class="heatmap-canvas" node_ref=canvas_ref width="800" height="420"></canvas>
        </section>
    }
}
