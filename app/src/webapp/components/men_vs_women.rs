use super::shared::Corners;
use crate::core::{HeatmapBin, HistogramBin, percentile_for_value, value_for_percentile};
use crate::webapp::charts::{draw_cross_sex_heatmap_overlay, draw_dual_normal_curve_canvas};
use crate::webapp::models::{CrossSexComparison, CrossSexLiftComparison};
use crate::webapp::state::AppState;
use leptos::ev;
use leptos::html::Canvas;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct MenVsWomenCtx {
    pub cross_sex_comparison: Memo<Result<CrossSexComparison, String>>,
    pub male_hist: ReadSignal<Option<HistogramBin>>,
    pub female_hist: ReadSignal<Option<HistogramBin>>,
    pub male_heat: ReadSignal<Option<HeatmapBin>>,
    pub female_heat: ReadSignal<Option<HeatmapBin>>,
    pub hist_loading: ReadSignal<bool>,
    pub hist_error: ReadSignal<Option<String>>,
    pub heat_loading: ReadSignal<bool>,
    pub heat_error: ReadSignal<Option<String>>,
    pub lift_comparisons: ReadSignal<Vec<CrossSexLiftComparison>>,
    pub lift_comparison_loading: ReadSignal<bool>,
    pub lift_comparison_error: ReadSignal<Option<String>>,
}

fn format_kg(value: f32) -> String {
    if (value - value.round()).abs() < 0.05 {
        format!("{:.0}", value)
    } else {
        format!("{value:.1}")
    }
}

fn format_ratio(value: f32) -> String {
    format!("{value:.2}")
}

fn comparison_style(value: f32) -> String {
    format!("flex:{:.3}", value.max(0.01))
}

fn overlap_copy(
    male_hist: &HistogramBin,
    female_hist: &HistogramBin,
    metric_label: &str,
) -> Option<(String, String, String, String, String)> {
    let female_p95 = value_for_percentile(Some(female_hist), 0.95)?;
    let male_p95 = value_for_percentile(Some(male_hist), 0.95)?;
    let male_pct_at_female_p95 = percentile_for_value(Some(male_hist), female_p95)?.0 * 100.0;
    let female_as_male_p95 = if male_p95 > 0.0 {
        female_p95 / male_p95 * 100.0
    } else {
        0.0
    };
    let gap = (100.0 - female_as_male_p95).abs();
    Some((
        format_kg(female_p95),
        format!("{male_pct_at_female_p95:.0}%"),
        format!("{female_as_male_p95:.0}%"),
        format!("{gap:.0}%"),
        metric_label.to_string(),
    ))
}

#[component]
pub fn MenVsWomenPage(ctx: MenVsWomenCtx) -> impl IntoView {
    let app = use_context::<AppState>().expect("AppState must be provided by App");
    let calculated = app.compute.calculated;
    let user_lift = app.compute.user_lift;
    let hist_x_label = app.compute.hist_x_label;
    let bodyweight = app.input.bodyweight;
    let MenVsWomenCtx {
        cross_sex_comparison,
        male_hist,
        female_hist,
        male_heat,
        female_heat,
        hist_loading,
        hist_error,
        heat_loading,
        heat_error,
        lift_comparisons,
        lift_comparison_loading,
        lift_comparison_error,
    } = ctx;

    let cross_canvas: NodeRef<Canvas> = NodeRef::new();
    let curve_canvas: NodeRef<Canvas> = NodeRef::new();
    let (chart_resize_tick, set_chart_resize_tick) = signal(0u32);
    let resize_handle = window_event_listener(ev::resize, move |_| {
        set_chart_resize_tick.update(|tick| *tick = tick.wrapping_add(1));
    });
    on_cleanup(move || resize_handle.remove());

    Effect::new(move |_| {
        let _ = chart_resize_tick.get();
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

    Effect::new(move |_| {
        let _ = chart_resize_tick.get();
        let Some(canvas) = curve_canvas.get() else {
            return;
        };
        let (Some(mh), Some(fh)) = (male_hist.get(), female_hist.get()) else {
            return;
        };
        draw_dual_normal_curve_canvas(
            &canvas,
            &mh,
            &fh,
            calculated.get().then(|| user_lift.get()),
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
                    <span>"KDE / SCALED"</span>
                </div>
                <div class="panel-body">
                    <p class="chart-summary">
                        {move || match (male_hist.get(), female_hist.get()) {
                            (Some(mh), Some(fh)) => {
                                overlap_copy(&mh, &fh, &hist_x_label.get())
                                    .map(|(female_p95, male_covered, _, _, metric)| {
                                        format!(
                                            "The female P95 mark is {female_p95} {metric}, which clears the bottom {male_covered} of male lifters in this cohort."
                                        )
                                    })
                                    .unwrap_or_else(|| {
                                        "The curves show how male and female lift distributions overlap in this cohort.".to_string()
                                    })
                            }
                            _ => "Compute on Ranking first to compare the male and female distribution curves.".to_string(),
                        }}
                    </p>
                    <div class="hist-wrap">
                        {move || match (male_hist.get(), female_hist.get()) {
                            (Some(_), Some(_)) => view! {
                                <canvas
                                    node_ref=curve_canvas.clone()
                                    class="hist"
                                    role="img"
                                    aria-label="Scaled normal distribution curves comparing men and women"
                                ></canvas>
                            }.into_any(),
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

            {move || {
                let rows = lift_comparisons.get();
                if rows.is_empty() {
                    let msg = if let Some(err) = lift_comparison_error.get() { err }
                        else if lift_comparison_loading.get() { "Loading lift comparisons...".to_string() }
                        else if calculated.get() { "Awaiting lift comparison data...".to_string() }
                        else { "Compute on Ranking page first.".to_string() };
                    view! {
                        <div class="panel" style="margin-bottom:24px">
                            <Corners />
                            <div class="panel-head">
                                <span><span class="tag">"▲"</span>" LIFT COMPARISONS"</span>
                                <span>"MEAN · KG / × BW"</span>
                            </div>
                            <div class="panel-body"><div class="notice">{msg}</div></div>
                        </div>
                    }.into_any()
                } else {
                    let absolute_rows = rows.clone();
                    let relative_rows = rows
                        .iter()
                        .filter(|row| {
                            row.male_mean_bodyweight_ratio.is_some()
                                && row.female_mean_bodyweight_ratio.is_some()
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    view! {
                        <div class="vs-charts">
                            <div class="panel">
                                <Corners />
                                <div class="panel-head">
                                    <span><span class="tag">"▲"</span>" ABSOLUTE LIFTS"</span>
                                    <span>"MEAN · KG"</span>
                                </div>
                                <div class="panel-body">
                                    <p class="chart-summary">
                                        "Male and female average lifts are compared directly in kilograms for squat, bench, and deadlift."
                                    </p>
                                    {absolute_rows.into_iter().map(|row| {
                                        let male = format_kg(row.male_mean_kg);
                                        let female = format_kg(row.female_mean_kg);
                                        let male_style = comparison_style(row.male_mean_kg);
                                        let female_style = comparison_style(row.female_mean_kg);
                                        view! {
                                            <div class="vs-bar-row">
                                                <div class="vs-bar-label">
                                                    <span>{row.label.to_uppercase()}</span>
                                                    <span>{format!("{male} / {female} KG")}</span>
                                                </div>
                                                <div class="ratio-strip">
                                                    <div class="s m" style=male_style>{format!("♂ {male}")}</div>
                                                    <div class="s w" style=female_style>{format!("♀ {female}")}</div>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>

                            <div class="panel">
                                <Corners />
                                <div class="panel-head">
                                    <span><span class="tag">"÷"</span>" RELATIVE TO BODYWEIGHT"</span>
                                    <span>"MEAN · × BW"</span>
                                </div>
                                <div class="panel-body">
                                    <p class="chart-summary">
                                        "Relative strength compares each average lift to average bodyweight for the same sex and cohort."
                                    </p>
                                    {if relative_rows.is_empty() {
                                        view! { <div class="notice">"Relative comparisons unavailable for this cohort."</div> }.into_any()
                                    } else {
                                        view! {
                                            <>
                                                {relative_rows.into_iter().map(|row| {
                                                    let male_value = row.male_mean_bodyweight_ratio.unwrap_or_default();
                                                    let female_value = row.female_mean_bodyweight_ratio.unwrap_or_default();
                                                    let male = format_ratio(male_value);
                                                    let female = format_ratio(female_value);
                                                    let male_style = comparison_style(male_value);
                                                    let female_style = comparison_style(female_value);
                                                    view! {
                                                        <div class="vs-bar-row">
                                                            <div class="vs-bar-label">
                                                                <span>{format!("{}/BW", row.label.to_uppercase())}</span>
                                                                <span>{format!("{male}× / {female}×")}</span>
                                                            </div>
                                                            <div class="ratio-strip">
                                                                <div class="s m" style=male_style>{format!("♂ {male}")}</div>
                                                                <div class="s w" style=female_style>{format!("♀ {female}")}</div>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </>
                                        }.into_any()
                                    }}
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }
            }}

            {move || match (male_hist.get(), female_hist.get()) {
                (Some(mh), Some(fh)) => {
                    let metric = hist_x_label.get();
                    if let Some((female_p95, male_covered, female_ratio, gap, metric)) =
                        overlap_copy(&mh, &fh, &metric)
                    {
                        view! {
                            <div class="panel" style="margin-bottom:24px">
                                <Corners />
                                <div class="panel-head">
                                    <span><span class="tag">"∩"</span>" OVERLAP ZONES"</span>
                                    <span>"WHERE THE STRONG MEET"</span>
                                </div>
                                <div class="panel-body vs-overlap-copy">
                                    "Top "
                                    <span class="women-highlight">"5%"</span>
                                    " of female lifters in this cohort clear "
                                    <span class="women-highlight">{female_p95}</span>
                                    " "
                                    <span class="chalk-highlight">{metric.clone()}</span>
                                    ", beating the bottom "
                                    <span class="men-highlight">{male_covered}</span>
                                    " of male lifters. At the 95th percentile, the female mark is "
                                    <span class="chalk-highlight">{female_ratio}</span>
                                    " of the male mark, a "
                                    <span class="chalk-highlight">{gap}</span>
                                    " gap."
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="panel" style="margin-bottom:24px">
                                <Corners />
                                <div class="panel-head">
                                    <span><span class="tag">"∩"</span>" OVERLAP ZONES"</span>
                                    <span>"WHERE THE STRONG MEET"</span>
                                </div>
                                <div class="panel-body"><div class="notice">"Overlap statistics unavailable for this cohort."</div></div>
                            </div>
                        }.into_any()
                    }
                }
                _ => view! {
                    <div class="panel" style="margin-bottom:24px">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"∩"</span>" OVERLAP ZONES"</span>
                            <span>"WHERE THE STRONG MEET"</span>
                        </div>
                        <div class="panel-body"><div class="notice">"Compute on Ranking page first."</div></div>
                    </div>
                }.into_any(),
            }}

            // Overlay heatmap
            <div class="panel">
                <Corners />
                <div class="panel-head">
                    <span><span class="tag">"∩"</span>" DENSITY OVERLAY"</span>
                    <span>"LIFT × BODYWEIGHT"</span>
                </div>
                <div class="panel-body">
                    <p class="chart-summary">
                        {move || {
                            if male_heat.get().is_some() && female_heat.get().is_some() {
                                format!(
                                    "The overlay shows where male and female lifters cluster by {} and bodyweight; your marker uses your current inputs.",
                                    hist_x_label.get(),
                                )
                            } else {
                                "Compute on Ranking first to load the male and female lift-by-bodyweight density overlay.".to_string()
                            }
                        }}
                    </p>
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
