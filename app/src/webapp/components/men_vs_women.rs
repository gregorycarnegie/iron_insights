use super::shared::Corners;
use crate::core::{HeatmapBin, HistogramBin};
use crate::webapp::charts::{draw_cross_sex_heatmap_overlay, render_dual_histogram_svg};
use crate::webapp::helpers::kg_to_display;
use crate::webapp::models::{CrossSexComparison, SliceSummary};
use leptos::html::Canvas;
use leptos::prelude::*;

#[derive(Clone)]
pub struct MenVsWomenCtx {
    pub dataset_blurb: Memo<String>,
    pub calculated: ReadSignal<bool>,
    pub cross_sex_comparison: Memo<Result<CrossSexComparison, String>>,
    pub male_hist: ReadSignal<Option<HistogramBin>>,
    pub female_hist: ReadSignal<Option<HistogramBin>>,
    pub male_heat: ReadSignal<Option<HeatmapBin>>,
    pub female_heat: ReadSignal<Option<HeatmapBin>>,
    pub hist_loading: ReadSignal<bool>,
    pub hist_error: ReadSignal<Option<String>>,
    pub heat_loading: ReadSignal<bool>,
    pub heat_error: ReadSignal<Option<String>>,
    pub user_lift: Memo<f32>,
    pub bodyweight: ReadSignal<f32>,
    pub hist_x_label: Memo<String>,
    pub use_lbs: ReadSignal<bool>,
    pub unit_label: Memo<&'static str>,
    pub slice_summary: ReadSignal<Option<SliceSummary>>,
}

#[component]
pub fn MenVsWomenPage(ctx: MenVsWomenCtx) -> impl IntoView {
    let MenVsWomenCtx {
        dataset_blurb,
        calculated,
        cross_sex_comparison,
        male_hist,
        female_hist,
        male_heat,
        female_heat,
        hist_loading,
        hist_error,
        heat_loading,
        heat_error,
        user_lift,
        bodyweight,
        hist_x_label,
        use_lbs,
        unit_label,
        slice_summary,
    } = ctx;

    let cross_canvas: NodeRef<Canvas> = NodeRef::new();

    Effect::new(move |_| {
        let Some(canvas) = cross_canvas.get() else {
            return;
        };
        let (Some(mh), Some(fh)) = (male_heat.get(), female_heat.get()) else {
            return;
        };
        draw_cross_sex_heatmap_overlay(
            &canvas,
            &mh,
            &fh,
            calculated.get().then(|| user_lift.get()),
            bodyweight.get(),
            &hist_x_label.get(),
        );
    });

    view! {
        <section class="page active" id="page-vs">
            <div class="page-head">
                <h1 class="page-title">
                    "Men " <span class="accent">"vs"</span> " women."
                </h1>
                <p class="page-lede">
                    <span class="serif">"Absolute strength diverges"</span>
                    " — relative strength converges. See where the curves overlap and where they don't."
                    <br />
                    <span style="font-size:11px;color:var(--ink-mute)">"Compute on Ranking page first to load cross-sex data."</span>
                </p>
            </div>

            // Cross-sex heads
            {move || match cross_sex_comparison.get() {
                Ok(cmp) => view! {
                    <div class="vs-head">
                        <div class="vs-col m">
                            <div class="sym">"♂"</div>
                            <div class="lbl">"MALE · " {cmp.male_weight_class.clone()}</div>
                            <div class="n">{cmp.male_total.to_string()}</div>
                            <div style="color:var(--ink-dim);font-size:11px;margin-top:4px">
                                {format!("{:.1}ᵗʰ percentile", cmp.male_percentile * 100.0)}
                            </div>
                        </div>
                        <div class="vs-vs">"//"</div>
                        <div class="vs-col w">
                            <div class="sym">"♀"</div>
                            <div class="lbl">"FEMALE · " {cmp.female_weight_class.clone()}</div>
                            <div class="n">{cmp.female_total.to_string()}</div>
                            <div style="color:var(--ink-dim);font-size:11px;margin-top:4px">
                                {format!("{:.1}ᵗʰ percentile", cmp.female_percentile * 100.0)}
                            </div>
                        </div>
                    </div>

                    // Equivalence info
                    <div class="panel" style="margin-bottom:24px">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"≡"</span>" CROSS-SEX EQUIVALENCE"</span>
                            <span>{cmp.metric.clone()}</span>
                        </div>
                        <div class="panel-body" style="font-size:13px;line-height:1.9;color:var(--ink-dim)">
                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:24px">
                                <div>
                                    <div style="font-family:'Archivo Black',sans-serif;color:var(--iron);font-size:11px;letter-spacing:0.2em;margin-bottom:8px">
                                        "AT YOUR MALE PERCENTILE"
                                    </div>
                                    <div>
                                        "A woman would need "
                                        <span style="color:var(--chalk);font-family:'Archivo Black',sans-serif">
                                            {format!("{:.1}", cmp.female_value_at_male_percentile)}
                                        </span>
                                        " " {cmp.metric.clone()}
                                    </div>
                                </div>
                                <div>
                                    <div style="font-family:'Archivo Black',sans-serif;color:var(--women);font-size:11px;letter-spacing:0.2em;margin-bottom:8px">
                                        "AT YOUR FEMALE PERCENTILE"
                                    </div>
                                    <div>
                                        "A man would need "
                                        <span style="color:var(--chalk);font-family:'Archivo Black',sans-serif">
                                            {format!("{:.1}", cmp.male_value_at_female_percentile)}
                                        </span>
                                        " " {cmp.metric.clone()}
                                    </div>
                                </div>
                            </div>
                            {cmp.caveat.as_ref().map(|c| view! {
                                <p style="margin-top:16px;font-size:11px;color:var(--ink-mute)">{c.clone()}</p>
                            })}
                        </div>
                    </div>
                }.into_any(),
                Err(msg) => view! {
                    <div class="notice" style="margin-bottom:24px">{msg}</div>
                }.into_any(),
            }}

            // Overlapping histogram
            <div class="panel" style="margin-bottom:24px">
                <Corners />
                <div class="panel-head">
                    <span><span class="tag">"Δ"</span>" OVERLAPPING DISTRIBUTIONS"</span>
                    <span>"LIFTER COUNT"</span>
                </div>
                <div class="panel-body">
                    <div class="hist-wrap">
                        {move || match (male_hist.get(), female_hist.get()) {
                            (Some(mh), Some(fh)) => render_dual_histogram_svg(
                                &mh, &fh,
                                calculated.get().then(|| user_lift.get()),
                                &hist_x_label.get(),
                            ),
                            _ => view! {
                                <div class="notice">
                                    {if let Some(err) = hist_error.get() { err }
                                    else if hist_loading.get() { "Loading distributions...".to_string() }
                                    else if calculated.get() { "Awaiting cross-sex data...".to_string() }
                                    else { "Compute on Ranking page first.".to_string() }}
                                </div>
                            }.into_any(),
                        }}
                    </div>
                </div>
            </div>

            // Overlay heatmap
            <div class="panel">
                <Corners />
                <div class="panel-head">
                    <span><span class="tag">"∩"</span>" DENSITY OVERLAY"</span>
                    <span>"LIFT × BODYWEIGHT"</span>
                </div>
                <div class="panel-body">
                    <div class="heat-wrap">
                        {move || {
                            let has_both = male_heat.get().is_some() && female_heat.get().is_some();
                            if has_both {
                                view! {
                                    <canvas
                                        node_ref=cross_canvas.clone()
                                        style="width:100%;height:100%;display:block"
                                    ></canvas>
                                }.into_any()
                            } else {
                                let msg = if let Some(err) = heat_error.get() { err }
                                    else if heat_loading.get() { "Loading heatmaps...".to_string() }
                                    else if calculated.get() { "Awaiting heatmap data...".to_string() }
                                    else { "Compute on Ranking page first.".to_string() };
                                view! { <div class="notice">{msg}</div> }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </section>
    }
}
