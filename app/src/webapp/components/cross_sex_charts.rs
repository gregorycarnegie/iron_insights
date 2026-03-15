use crate::core::{HeatmapBin, HistogramBin, rebin_1d, rebin_2d};
use crate::webapp::charts::{draw_cross_sex_heatmap_overlay, render_dual_histogram_svg};
use crate::webapp::models::CrossSexComparison;
use leptos::ev;
use leptos::html::Canvas;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn CrossSexChartsPanel(
    hist_loading: ReadSignal<bool>,
    comparison: Memo<Result<CrossSexComparison, String>>,
    male_hist: ReadSignal<Option<HistogramBin>>,
    female_hist: ReadSignal<Option<HistogramBin>>,
    heat_loading: ReadSignal<bool>,
    heat_error: ReadSignal<Option<String>>,
    male_heat: ReadSignal<Option<HeatmapBin>>,
    female_heat: ReadSignal<Option<HeatmapBin>>,
    hist_x_label: Memo<String>,
    bodyweight: ReadSignal<f32>,
    lift_mult: ReadSignal<usize>,
    bw_mult: ReadSignal<usize>,
) -> impl IntoView {
    let canvas_ref: NodeRef<Canvas> = NodeRef::new();
    let (resize_tick, set_resize_tick) = signal(0u32);
    let resize_handle = window_event_listener(ev::resize, move |_| {
        set_resize_tick.update(|tick| *tick = tick.wrapping_add(1));
    });
    on_cleanup(move || resize_handle.remove());

    let marker_values = Memo::new(move |_| {
        comparison
            .get()
            .ok()
            .map(|data| (data.male_input_value, data.female_input_value))
    });
    let rebinned_male_hist = Memo::new(move |_| {
        male_hist.get().map(|hist| {
            let lift_grouping = lift_mult.get();
            HistogramBin::new(
                hist.min,
                hist.max,
                hist.base_bin * lift_grouping as f32,
                rebin_1d(hist.counts, lift_grouping),
            )
        })
    });
    let rebinned_female_hist = Memo::new(move |_| {
        female_hist.get().map(|hist| {
            let lift_grouping = lift_mult.get();
            HistogramBin::new(
                hist.min,
                hist.max,
                hist.base_bin * lift_grouping as f32,
                rebin_1d(hist.counts, lift_grouping),
            )
        })
    });
    let rebinned_male_heat = Memo::new(move |_| {
        male_heat.get().map(|heat| {
            let lift_grouping = lift_mult.get();
            let bodyweight_grouping = bw_mult.get();
            let (grid, width, height) = rebin_2d(
                heat.grid,
                heat.width,
                heat.height,
                lift_grouping,
                bodyweight_grouping,
            );
            HeatmapBin {
                min_x: heat.min_x,
                max_x: heat.max_x,
                min_y: heat.min_y,
                max_y: heat.max_y,
                base_x: heat.base_x * lift_grouping as f32,
                base_y: heat.base_y * bodyweight_grouping as f32,
                width,
                height,
                grid,
            }
        })
    });
    let rebinned_female_heat = Memo::new(move |_| {
        female_heat.get().map(|heat| {
            let lift_grouping = lift_mult.get();
            let bodyweight_grouping = bw_mult.get();
            let (grid, width, height) = rebin_2d(
                heat.grid,
                heat.width,
                heat.height,
                lift_grouping,
                bodyweight_grouping,
            );
            HeatmapBin {
                min_x: heat.min_x,
                max_x: heat.max_x,
                min_y: heat.min_y,
                max_y: heat.max_y,
                base_x: heat.base_x * lift_grouping as f32,
                base_y: heat.base_y * bodyweight_grouping as f32,
                width,
                height,
                grid,
            }
        })
    });

    Effect::new(move |_| {
        let _ = resize_tick.get();
        let Some(canvas) = canvas_ref.get() else {
            return;
        };
        let Some(male_heat) = rebinned_male_heat.get() else {
            return;
        };
        let Some(female_heat) = rebinned_female_heat.get() else {
            return;
        };
        let Some((male_value, female_value)) = marker_values.get() else {
            return;
        };
        draw_cross_sex_heatmap_overlay(
            &canvas,
            &male_heat,
            &female_heat,
            male_value,
            female_value,
            bodyweight.get(),
            &hist_x_label.get(),
        );
    });

    view! {
        <section class="panel">
            <h3>"Overlayed Histograms"</h3>
            <p class="muted">
                "Men and women share one x-axis here, with translucent fills showing where each cohort clusters."
            </p>
            <Show
                when=move || !hist_loading.get()
                fallback=move || view! { <p class="muted">"Loading male/female histograms..."</p> }
            >
                {move || match (
                    comparison.get(),
                    rebinned_male_hist.get(),
                    rebinned_female_hist.get(),
                ) {
                    (Ok(data), Some(male_hist), Some(female_hist)) => render_dual_histogram_svg(
                        &male_hist,
                        &female_hist,
                        data.male_input_value,
                        data.female_input_value,
                        &hist_x_label.get(),
                    ),
                    (Err(message), _, _) => view! { <p class="muted">{message}</p> }.into_any(),
                    _ => {
                        view! { <p class="muted">"Both cohorts need histogram data before the overlay can render."</p> }
                            .into_any()
                    }
                }}
            </Show>
        </section>

        <section class="panel">
            <h3>{move || format!("Overlayed Bodyweight vs {}", hist_x_label.get())}</h3>
            <p class="muted">
                "Each density map is drawn in its own color so shared hotspots and cohort-specific pockets stand out."
            </p>
            <Show
                when=move || !heat_loading.get()
                fallback=move || view! { <p class="muted">"Loading male/female heatmaps..."</p> }
            >
                {move || {
                    if let Some(error) = heat_error.get() {
                        return view! { <p class="muted">{error}</p> }.into_any();
                    }

                    match (
                        comparison.get(),
                        rebinned_male_heat.get(),
                        rebinned_female_heat.get(),
                    ) {
                        (Ok(_), Some(_), Some(_)) => view! {
                            <canvas class="heatmap-canvas" node_ref=canvas_ref width="800" height="420"></canvas>
                        }
                        .into_any(),
                        (Err(message), _, _) => {
                            view! { <p class="muted">{message}</p> }.into_any()
                        }
                        _ => view! {
                            <p class="muted">
                                "Both cohorts need heatmap data before the overlay can render."
                            </p>
                        }
                        .into_any(),
                    }
                }}
            </Show>
        </section>
    }
}
