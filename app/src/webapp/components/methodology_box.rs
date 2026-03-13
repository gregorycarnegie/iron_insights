use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn MethodologyBoxPanel(
    exact_slice_key: Memo<String>,
    shard_key: Memo<String>,
    dataset_version: Memo<String>,
    dataset_revision: Memo<String>,
    histogram_bin_width: Memo<Option<f32>>,
    heatmap_dims: Memo<Option<(usize, usize, f32, f32)>>,
    summary_stats: Memo<Option<(u32, f32, f32)>>,
    load_timing_blurb: Memo<String>,
) -> impl IntoView {
    let (show_load_timing, set_show_load_timing) = signal(false);

    view! {
        <section class="panel">
            <h3>"Methodology and Debug"</h3>
            <p class="muted">
                "Percentile is computed from histogram bins using a mid-bin CDF approximation."
            </p>
            {metric_grid! {
                "Exact slice key" => move || format!(" {}", exact_slice_key.get()),
                "Shard key" => move || format!(" {}", shard_key.get()),
                "Dataset version" => move || format!(" {}", dataset_version.get()),
                "Dataset revision" => move || format!(" {}", dataset_revision.get()),
                "Histogram bin width" => move || match histogram_bin_width.get() {
                    Some(width) => format!(" {:.2}", width),
                    None => " n/a".to_string(),
                },
                "Heatmap dimensions" => move || match heatmap_dims.get() {
                    Some((w, h, bx, by)) => format!(" {}x{} (bin x {:.2}, y {:.2})", w, h, bx, by),
                    None => " n/a".to_string(),
                },
                "Summary total" => move || match summary_stats.get() {
                    Some((total, _, _)) => format!(" {}", total),
                    None => " n/a".to_string(),
                },
                "Summary min/max (kg)" => move || match summary_stats.get() {
                    Some((_, min, max)) => format!(" {:.1} / {:.1}", min, max),
                    None => " n/a".to_string(),
                },
            }}
            <label class="methodology-toggle">
                <input
                    type="checkbox"
                    prop:checked=move || show_load_timing.get()
                    on:change=move |ev| set_show_load_timing.set(event_target_checked(&ev))
                />
                "Show load timings"
            </label>
            <Show when=move || show_load_timing.get()>
                <p class="muted">{move || load_timing_blurb.get()}</p>
            </Show>
        </section>
    }
}
