use crate::webapp::models::CompareMode;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn CompareModePanel(
    compare_mode: ReadSignal<CompareMode>,
    set_compare_mode: WriteSignal<CompareMode>,
    compare_summary: Memo<String>,
) -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"Compare mode"</h2>
            <div class="compare-tabs">
                <button
                    type="button"
                    class="chip"
                    class:active=move || compare_mode.get() == CompareMode::AllLifters
                    on:click=move |_| set_compare_mode.set(CompareMode::AllLifters)
                >
                    "All lifters"
                </button>
                <button
                    type="button"
                    class="chip"
                    class:active=move || compare_mode.get() == CompareMode::SameBodyweightRange
                    on:click=move |_| set_compare_mode.set(CompareMode::SameBodyweightRange)
                >
                    "Same bodyweight range"
                </button>
                <button
                    type="button"
                    class="chip"
                    class:active=move || compare_mode.get() == CompareMode::SameWeightClass
                    on:click=move |_| set_compare_mode.set(CompareMode::SameWeightClass)
                >
                    "Same weight class"
                </button>
                <button
                    type="button"
                    class="chip"
                    class:active=move || compare_mode.get() == CompareMode::SameAgeClass
                    on:click=move |_| set_compare_mode.set(CompareMode::SameAgeClass)
                >
                    "Same age class"
                </button>
            </div>
            <p class="muted">{move || compare_summary.get()}</p>
        </section>
    }
}
