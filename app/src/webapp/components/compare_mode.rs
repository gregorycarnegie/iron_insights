use crate::webapp::models::CompareMode;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn CompareModePanel(
    compare_mode: ReadSignal<CompareMode>,
    set_compare_mode: WriteSignal<CompareMode>,
    tested: ReadSignal<String>,
    set_tested: WriteSignal<String>,
    equip: ReadSignal<String>,
    set_equip: WriteSignal<String>,
    age: ReadSignal<String>,
    set_age: WriteSignal<String>,
    compare_summary: Memo<String>,
) -> impl IntoView {
    let set_tested_value = move |candidate: &str| {
        set_tested.set(candidate.to_string());
    };
    let set_equip_value = move |candidate: &str| {
        set_equip.set(candidate.to_string());
    };
    let set_age_value = move |candidate: &str| {
        set_age.set(candidate.to_string());
    };

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
                <button
                    type="button"
                    class="chip"
                    class:active=move || compare_mode.get() == CompareMode::SameTestedStatus
                    on:click=move |_| set_compare_mode.set(CompareMode::SameTestedStatus)
                >
                    "Same tested status"
                </button>
            </div>
            <div class="preset-row">
                <button
                    type="button"
                    class="chip"
                    on:click=move |_| {
                        set_compare_mode.set(CompareMode::SameTestedStatus);
                        set_equip_value("Raw");
                        set_tested_value("Yes");
                        set_age_value("All Ages");
                    }
                >
                    "Preset: Tested Raw"
                </button>
                <button
                    type="button"
                    class="chip"
                    on:click=move |_| {
                        set_compare_mode.set(CompareMode::SameTestedStatus);
                        set_equip_value("Raw");
                        set_tested_value("No");
                        set_age_value("All Ages");
                    }
                >
                    "Preset: Untested Raw"
                </button>
                <button
                    type="button"
                    class="chip"
                    on:click=move |_| {
                        set_compare_mode.set(CompareMode::SameAgeClass);
                        set_equip_value("Raw");
                        set_tested_value("Yes");
                        set_age_value("35-39");
                    }
                >
                    "Preset: Masters Tested"
                </button>
            </div>
            <p class="muted">{move || compare_summary.get()}</p>
            <p class="muted">
                {move || format!(
                    "Current filters: equip {}, tested {}, age {}",
                    equip.get(),
                    tested.get(),
                    age.get()
                )}
            </p>
        </section>
    }
}
