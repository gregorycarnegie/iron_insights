use super::PlateCalcPanel;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn PlateCalcPage() -> impl IntoView {
    view! {
        <header class="hero">
            <p class="hero-tag">"// barbell loading"</p>
            <h1>
                "Load Your"
                <br />
                <span>"Barbell"</span>
            </h1>
            <p class="intro">"Enter a target weight and see exactly which plates go on each side, with a visual breakdown."</p>
            <div class="hero-badges">
                <p class="data-badge">
                    <strong>"Standard competition plates"</strong>
                    " (25 / 20 / 15 / 10 / 5 / 2.5 / 1.25 kg)"
                </p>
                <p class="data-badge">"Supports kg and lb units."</p>
            </div>
        </header>
        <PlateCalcPanel />
    }
}
