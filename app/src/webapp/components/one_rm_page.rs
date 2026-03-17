use super::OneRepMaxPanel;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn OneRmPage() -> impl IntoView {
    view! {
        <header class="hero">
            <p class="hero-tag">"// strength estimation"</p>
            <h1>
                "Estimate Your"
                <br />
                <span>"1-Rep Max"</span>
            </h1>
            <p class="intro">"Use a hard set to estimate your max, map out likely rep targets, and sketch training weights."</p>
            <div class="hero-badges">
                <p class="data-badge">
                    <strong>"Epley + Brzycki"</strong>
                    " blended estimate"
                </p>
                <p class="data-badge">"Recent hard sets produce the most useful training numbers."</p>
            </div>
        </header>
        <OneRepMaxPanel />
    }
}
