use crate::webapp::helpers::kg_to_display;
use crate::webapp::models::CrossSexComparison;
use crate::webapp::ui::metric_label;
use leptos::prelude::*;

fn format_metric_value(value: f32, metric: &str, use_lbs: bool, unit_label: &str) -> String {
    if metric == "Kg" {
        format!("{:.1} {}", kg_to_display(value, use_lbs), unit_label)
    } else {
        format!("{:.2} {}", value, metric_label(metric))
    }
}

#[component]
pub(in crate::webapp) fn CrossSexPanel(
    loading: ReadSignal<bool>,
    comparison: Memo<Result<CrossSexComparison, String>>,
    use_lbs: ReadSignal<bool>,
    unit_label: Memo<&'static str>,
) -> impl IntoView {
    view! {
        <section class="panel">
            <h3>"Men vs Women"</h3>
            <p class="muted">
                "Cross-sex comparison keeps equipment, tested status, age class, lift, and metric aligned."
            </p>
            <Show
                when=move || !loading.get()
                fallback=move || view! { <p class="muted">"Loading male/female cohorts and distributions..."</p> }
            >
                {move || match comparison.get() {
                    Ok(data) => {
                        let male_wc_note = if data.male_wc_fallback {
                            format!("{} (fallback to All)", data.male_weight_class)
                        } else {
                            data.male_weight_class.clone()
                        };
                        let female_wc_note = if data.female_wc_fallback {
                            format!("{} (fallback to All)", data.female_weight_class)
                        } else {
                            data.female_weight_class.clone()
                        };
                        let metric_is_normalized =
                            matches!(data.metric.as_str(), "Dots" | "Wilks" | "GL");
                        let caveat_text = data.caveat.clone().unwrap_or_default();
                        let has_caveat = data.caveat.is_some();
                        view! {
                            <p class="muted">
                                {format!(
                                    "Same input, side-by-side: Men {:.1}% vs Women {:.1}%.",
                                    data.male_percentile * 100.0,
                                    data.female_percentile * 100.0
                                )}
                            </p>
                            {metric_grid! {
                                "Men percentile" => format!(" {:.1}%", data.male_percentile * 100.0),
                                "Women percentile" => format!(" {:.1}%", data.female_percentile * 100.0),
                                "Men cohort size" => format!(" {}", data.male_total),
                                "Women cohort size" => format!(" {}", data.female_total),
                                "Men weight class" => format!(" {}", male_wc_note),
                                "Women weight class" => format!(" {}", female_wc_note),
                                "Input score in men's cohort" => format!(
                                    " {}",
                                    format_metric_value(
                                        data.male_input_value,
                                        &data.metric,
                                        use_lbs.get(),
                                        unit_label.get()
                                    )
                                ),
                                "Input score in women's cohort" => format!(
                                    " {}",
                                    format_metric_value(
                                        data.female_input_value,
                                        &data.metric,
                                        use_lbs.get(),
                                        unit_label.get()
                                    )
                                ),
                                "Women equivalent at men's percentile" => format!(
                                    " {}",
                                    format_metric_value(
                                        data.female_value_at_male_percentile,
                                        &data.metric,
                                        use_lbs.get(),
                                        unit_label.get()
                                    )
                                ),
                                "Men equivalent at women's percentile" => format!(
                                    " {}",
                                    format_metric_value(
                                        data.male_value_at_female_percentile,
                                        &data.metric,
                                        use_lbs.get(),
                                        unit_label.get()
                                    )
                                ),
                            }}
                            <Show when=move || metric_is_normalized>
                                <p class="muted">
                                    "Normalized metric selected, which is usually the fairest cross-sex view."
                                </p>
                            </Show>
                            <Show when=move || has_caveat>
                                <p class="muted">{caveat_text.clone()}</p>
                            </Show>
                        }.into_any()
                    }
                    Err(message) => view! { <p class="muted">{message}</p> }.into_any(),
                }}
            </Show>
        </section>
    }
}
