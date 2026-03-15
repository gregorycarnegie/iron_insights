use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn DistributionControlsPanel(
    lift_mult: ReadSignal<usize>,
    set_lift_mult: WriteSignal<usize>,
    bw_mult: ReadSignal<usize>,
    set_bw_mult: WriteSignal<usize>,
) -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"Distribution Controls"</h2>
            <p class="muted">
                "Adjust histogram and heatmap grouping for the detailed comparison views."
            </p>
            <div class="grid simple">
                <label title="Grouping size used for lift distributions.">
                    "Lift grouping size"
                    <select
                        prop:value=move || lift_mult.get().to_string()
                        on:change=move |ev| {
                            set_lift_mult.set(event_target_value(&ev).parse::<usize>().unwrap_or(4))
                        }
                    >
                        <option value="1">"1x base"</option>
                        <option value="2">"2x base"</option>
                        <option value="4">"4x base"</option>
                    </select>
                </label>
                <label title="Bodyweight bucket size used in the heatmap.">
                    "Bodyweight grouping"
                    <select
                        prop:value=move || bw_mult.get().to_string()
                        on:change=move |ev| {
                            set_bw_mult.set(event_target_value(&ev).parse::<usize>().unwrap_or(5))
                        }
                    >
                        <option value="1">"1kg"</option>
                        <option value="2">"2kg"</option>
                        <option value="5">"5kg"</option>
                    </select>
                </label>
            </div>
            <button
                type="button"
                class="secondary"
                on:click=move |_| {
                    set_lift_mult.set(4);
                    set_bw_mult.set(5);
                }
            >
                "Reset chart grouping"
            </button>
        </section>
    }
}
