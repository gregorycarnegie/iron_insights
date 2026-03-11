use super::OneRepMaxPanel;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn OneRmPage() -> impl IntoView {
    view! {
        <header class="hero">
            <h1>"Estimate Your 1-Rep Max"</h1>
            <p>"Use a training set to estimate your max with common formulas."</p>
        </header>
        <OneRepMaxPanel />
    }
}
