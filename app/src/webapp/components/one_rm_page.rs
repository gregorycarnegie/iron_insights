use super::OneRepMaxPanel;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn OneRmPage() -> impl IntoView {
    view! {
        <header class="hero">
            <h1>"Estimate Your 1-Rep Max"</h1>
            <p>"Use a hard set to estimate your max, map out likely rep targets, and sketch training weights."</p>
        </header>
        <OneRepMaxPanel />
    }
}
