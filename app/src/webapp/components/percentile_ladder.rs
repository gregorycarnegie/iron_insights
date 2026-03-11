use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
#[component]
pub(in crate::webapp) fn PercentileLadderPanel(
    calculated: ReadSignal<bool>,
    metric_is_kg_comparable: Memo<bool>,
    kg_for_next_1pct: Memo<Option<f32>>,
    kg_for_next_5pct: Memo<Option<f32>>,
    kg_for_next_10pct: Memo<Option<f32>>,
    pct_gain_plus_2_5kg: Memo<Option<f32>>,
    pct_gain_plus_5kg: Memo<Option<f32>>,
    pct_gain_plus_10kg: Memo<Option<f32>>,
) -> impl IntoView {
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
                    <div class="nerd-metrics-grid">
                        <p>
                            <strong>"Next +1 percentile point"</strong>
                            {move || match kg_for_next_1pct.get() {
                                Some(kg) => format!(" +{:.1} kg", kg.max(0.0)),
                                None => " n/a".to_string(),
                            }}
                        </p>
                        <p>
                            <strong>"Next +5 percentile points"</strong>
                            {move || match kg_for_next_5pct.get() {
                                Some(kg) => format!(" +{:.1} kg", kg.max(0.0)),
                                None => " n/a".to_string(),
                            }}
                        </p>
                        <p>
                            <strong>"Next +10 percentile points"</strong>
                            {move || match kg_for_next_10pct.get() {
                                Some(kg) => format!(" +{:.1} kg", kg.max(0.0)),
                                None => " n/a".to_string(),
                            }}
                        </p>
                        <p>
                            <strong>"+2.5 kg likely buys"</strong>
                            {move || match pct_gain_plus_2_5kg.get() {
                                Some(gain) => format!(" +{:.2} percentile points", gain.max(0.0)),
                                None => " n/a".to_string(),
                            }}
                        </p>
                        <p>
                            <strong>"+5 kg likely buys"</strong>
                            {move || match pct_gain_plus_5kg.get() {
                                Some(gain) => format!(" +{:.2} percentile points", gain.max(0.0)),
                                None => " n/a".to_string(),
                            }}
                        </p>
                        <p>
                            <strong>"+10 kg likely buys"</strong>
                            {move || match pct_gain_plus_10kg.get() {
                                Some(gain) => format!(" +{:.2} percentile points", gain.max(0.0)),
                                None => " n/a".to_string(),
                            }}
                        </p>
                    </div>
                </Show>
            </Show>
        </section>
    }
}
