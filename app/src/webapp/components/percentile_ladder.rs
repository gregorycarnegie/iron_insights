use super::PercentileLadderData;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn PercentileLadderPanel(ladder: PercentileLadderData) -> impl IntoView {
    let PercentileLadderData {
        calculated,
        metric_is_kg_comparable,
        estimates,
    } = ladder;
    view! {
        <section class="panel">
            <h3>"Kg Per Percentile"</h3>
            <Show
                when=move || calculated.get()
                fallback=move || view! { <p class="muted">"Calculate first to estimate percentile ladders."</p> }
            >
                <Show
                    when=move || metric_is_kg_comparable.get()
                    fallback=move || {
                        view! {
                            <p class="muted">
                                "This panel is available when your selected lift/metric is kg-comparable."
                            </p>
                        }
                    }
                >
                    {metric_grid! {
                        "Next +1 percentile point" => move || match estimates.kg_for_next_1pct.get() {
                            Some(kg) => format!(" +{:.1} kg", kg.max(0.0)),
                            None => " n/a".to_string(),
                        },
                        "Next +5 percentile points" => move || match estimates.kg_for_next_5pct.get() {
                            Some(kg) => format!(" +{:.1} kg", kg.max(0.0)),
                            None => " n/a".to_string(),
                        },
                        "Next +10 percentile points" => move || match estimates.kg_for_next_10pct.get() {
                            Some(kg) => format!(" +{:.1} kg", kg.max(0.0)),
                            None => " n/a".to_string(),
                        },
                        "+2.5 kg likely buys" => move || match estimates.pct_gain_plus_2_5kg.get() {
                            Some(gain) => format!(" +{:.2} percentile points", gain.max(0.0)),
                            None => " n/a".to_string(),
                        },
                        "+5 kg likely buys" => move || match estimates.pct_gain_plus_5kg.get() {
                            Some(gain) => format!(" +{:.2} percentile points", gain.max(0.0)),
                            None => " n/a".to_string(),
                        },
                        "+10 kg likely buys" => move || match estimates.pct_gain_plus_10kg.get() {
                            Some(gain) => format!(" +{:.2} percentile points", gain.max(0.0)),
                            None => " n/a".to_string(),
                        },
                    }}
                </Show>
            </Show>
        </section>
    }
}
