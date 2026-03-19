use super::OnboardingSections;
use crate::webapp::helpers::{display_to_kg, kg_to_display};
use crate::webapp::models::SavedUiState;
use crate::webapp::ui::{age_label, lift_label, metric_label};
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn OnboardingPanel(form: OnboardingSections) -> impl IntoView {
    let OnboardingSections {
        identity,
        lifts,
        filters,
        actions,
    } = form;
    view! {
        <section class="panel onboarding">
            <div class="panel-titlebar">
                <div>
                    <h2>"Your Numbers"</h2>
                    <p class="muted">
                        "Set the cohort and the lifts you want scored against the dataset."
                    </p>
                </div>
            </div>
            <div class="grid simple">
                <label>"Sex"
                    <select on:change=move |ev| identity.set_sex.set(event_target_value(&ev))>
                        <For each=move || identity.sex_options.get() key=|v| v.clone() let:value>
                            <option
                                selected={
                                    let selected_value = value.clone();
                                    move || identity.sex.get() == selected_value
                                }
                                value={value.clone()}
                            >
                                {value.clone()}
                            </option>
                        </For>
                    </select>
                </label>
                <label>"Equipment"
                    <select on:change=move |ev| identity.set_equip.set(event_target_value(&ev))>
                        <For each=move || identity.equip_options.get() key=|v| v.clone() let:value>
                            <option
                                selected={
                                    let selected_value = value.clone();
                                    move || identity.equip.get() == selected_value
                                }
                                value={value.clone()}
                            >
                                {value.clone()}
                            </option>
                        </For>
                    </select>
                </label>
            </div>
            <div class="grid numbers">
                <label>{move || format!("Squat ({})", identity.unit_label.get())}
                    <input
                        type="number"
                        min="0"
                        max="600"
                        step="0.5"
                        prop:value=move || kg_to_display(lifts.squat.get(), identity.use_lbs.get()).to_string()
                        on:change=move |ev| {
                            let raw = event_target_value(&ev);
                            match raw.parse::<f32>() {
                                Ok(value) => {
                                    let value_kg = display_to_kg(value, identity.use_lbs.get());
                                    if (0.0..=600.0).contains(&value_kg) {
                                        lifts.set_squat.set(value_kg);
                                        lifts.set_squat_error.set(None);
                                    } else {
                                        lifts.set_squat_error.set(Some(format!(
                                            "Squat must be {:.0}-{:.0}{}.",
                                            kg_to_display(0.0, identity.use_lbs.get()),
                                            kg_to_display(600.0, identity.use_lbs.get()),
                                            identity.unit_label.get()
                                        )));
                                    }
                                }
                                Err(_) => lifts.set_squat_error.set(Some("Enter a valid squat number.".to_string())),
                            }
                        }
                    />
                </label>
                <label>{move || format!("Bench ({})", identity.unit_label.get())}
                    <input
                        type="number"
                        min="0"
                        max="600"
                        step="0.5"
                        prop:value=move || kg_to_display(lifts.bench.get(), identity.use_lbs.get()).to_string()
                        on:change=move |ev| {
                            let raw = event_target_value(&ev);
                            match raw.parse::<f32>() {
                                Ok(value) => {
                                    let value_kg = display_to_kg(value, identity.use_lbs.get());
                                    if (0.0..=600.0).contains(&value_kg) {
                                        lifts.set_bench.set(value_kg);
                                        lifts.set_bench_error.set(None);
                                    } else {
                                        lifts.set_bench_error.set(Some(format!(
                                            "Bench must be {:.0}-{:.0}{}.",
                                            kg_to_display(0.0, identity.use_lbs.get()),
                                            kg_to_display(600.0, identity.use_lbs.get()),
                                            identity.unit_label.get()
                                        )));
                                    }
                                }
                                Err(_) => lifts.set_bench_error.set(Some("Enter a valid bench number.".to_string())),
                            }
                        }
                    />
                </label>
                <label>{move || format!("Deadlift ({})", identity.unit_label.get())}
                    <input
                        type="number"
                        min="0"
                        max="600"
                        step="0.5"
                        prop:value=move || kg_to_display(lifts.deadlift.get(), identity.use_lbs.get()).to_string()
                        on:change=move |ev| {
                            let raw = event_target_value(&ev);
                            match raw.parse::<f32>() {
                                Ok(value) => {
                                    let value_kg = display_to_kg(value, identity.use_lbs.get());
                                    if (0.0..=600.0).contains(&value_kg) {
                                        lifts.set_deadlift.set(value_kg);
                                        lifts.set_deadlift_error.set(None);
                                    } else {
                                        lifts.set_deadlift_error.set(Some(format!(
                                            "Deadlift must be {:.0}-{:.0}{}.",
                                            kg_to_display(0.0, identity.use_lbs.get()),
                                            kg_to_display(600.0, identity.use_lbs.get()),
                                            identity.unit_label.get()
                                        )));
                                    }
                                }
                                Err(_) => lifts.set_deadlift_error.set(Some("Enter a valid deadlift number.".to_string())),
                            }
                        }
                    />
                </label>
                <label>{move || format!("Bodyweight ({})", identity.unit_label.get())}
                    <input
                        type="number"
                        min="35"
                        max="300"
                        step="0.5"
                        prop:value=move || kg_to_display(lifts.bodyweight.get(), identity.use_lbs.get()).to_string()
                        on:change=move |ev| {
                            let raw = event_target_value(&ev);
                            match raw.parse::<f32>() {
                                Ok(value) => {
                                    let value_kg = display_to_kg(value, identity.use_lbs.get());
                                    if (35.0..=300.0).contains(&value_kg) {
                                        lifts.set_bodyweight.set(value_kg);
                                        lifts.set_bodyweight_error.set(None);
                                    } else {
                                        lifts.set_bodyweight_error.set(Some(format!(
                                            "Bodyweight must be {:.0}-{:.0}{}.",
                                            kg_to_display(35.0, identity.use_lbs.get()),
                                            kg_to_display(300.0, identity.use_lbs.get()),
                                            identity.unit_label.get()
                                        )));
                                    }
                                }
                                Err(_) => lifts.set_bodyweight_error.set(Some("Enter a valid bodyweight number.".to_string())),
                            }
                        }
                    />
                </label>
            </div>
            <div class="input-errors">
                <Show when=move || lifts.squat_error.get().is_some()>
                    <p class="input-error">{move || lifts.squat_error.get().unwrap_or_default()}</p>
                </Show>
                <Show when=move || lifts.bench_error.get().is_some()>
                    <p class="input-error">{move || lifts.bench_error.get().unwrap_or_default()}</p>
                </Show>
                <Show when=move || lifts.deadlift_error.get().is_some()>
                    <p class="input-error">{move || lifts.deadlift_error.get().unwrap_or_default()}</p>
                </Show>
                <Show when=move || lifts.bodyweight_error.get().is_some()>
                    <p class="input-error">{move || lifts.bodyweight_error.get().unwrap_or_default()}</p>
                </Show>
            </div>
            <div class="control-row">
                <label class="units-toggle">
                    "Units"
                    <div class="toggle-buttons">
                        <button
                            type="button"
                            class="chip"
                            class:active=move || !identity.use_lbs.get()
                            on:click=move |_| identity.set_use_lbs.set(false)
                        >
                            "kg"
                        </button>
                        <button
                            type="button"
                            class="chip"
                            class:active=move || identity.use_lbs.get()
                            on:click=move |_| identity.set_use_lbs.set(true)
                        >
                            "lb"
                        </button>
                    </div>
                </label>
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| {
                        lifts.set_squat.set(0.0);
                        lifts.set_bench.set(0.0);
                        lifts.set_deadlift.set(0.0);
                        lifts.set_bodyweight.set(90.0);
                        actions.set_squat_delta.set(0.0);
                        actions.set_bench_delta.set(0.0);
                        actions.set_deadlift_delta.set(0.0);
                        actions.set_share_handle.set(String::new());
                        lifts.set_squat_error.set(None);
                        lifts.set_bench_error.set(None);
                        lifts.set_deadlift_error.set(None);
                        lifts.set_bodyweight_error.set(None);
                        actions.set_calculated.set(false);
                    }
                >
                    "Clear all"
                </button>
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| {
                        let Some(window) = web_sys::window() else {
                            actions.set_share_status.set(Some("No browser window.".to_string()));
                            return;
                        };
                        let Ok(Some(storage)) = window.local_storage() else {
                            actions.set_share_status.set(Some("Local storage unavailable.".to_string()));
                            return;
                        };
                        let Ok(Some(raw)) = storage.get_item("iron_insights_last_state") else {
                            actions.set_share_status.set(Some("No saved numbers found.".to_string()));
                            return;
                        };
                        let Ok(saved) = serde_json::from_str::<SavedUiState>(&raw) else {
                            actions.set_share_status.set(Some("Saved numbers are invalid.".to_string()));
                            return;
                        };
                        identity.set_sex.set(saved.sex);
                        identity.set_equip.set(saved.equip);
                        filters.set_wc.set(saved.wc);
                        filters.set_age.set(saved.age);
                        filters.set_tested.set(saved.tested);
                        filters.set_lift.set(saved.lift);
                        filters.set_metric.set(saved.metric);
                        lifts.set_squat.set(saved.squat.clamp(0.0, 600.0));
                        lifts.set_bench.set(saved.bench.clamp(0.0, 600.0));
                        lifts.set_deadlift.set(saved.deadlift.clamp(0.0, 600.0));
                        lifts.set_bodyweight.set(saved.bodyweight.clamp(35.0, 300.0));
                        actions.set_squat_delta.set(saved.squat_delta.clamp(-50.0, 50.0));
                        actions.set_bench_delta.set(saved.bench_delta.clamp(-50.0, 50.0));
                        actions.set_deadlift_delta.set(saved.deadlift_delta.clamp(-50.0, 50.0));
                        actions.set_lift_mult.set(saved.lift_mult.clamp(1, 4));
                        actions.set_bw_mult.set(saved.bw_mult.clamp(1, 5));
                        actions.set_share_handle.set(saved.share_handle);
                        actions.set_calculated.set(saved.calculated);
                        actions.set_share_status.set(Some("Loaded saved numbers.".to_string()));
                    }
                >
                    "Use my last numbers"
                </button>
            </div>
            <button
                type="button"
                class="primary"
                prop:disabled=move || actions.has_input_error.get() || actions.calculating.get()
                on:click=move |_| {
                    actions.set_calculating.set(true);
                    actions.set_share_status.set(None);
                    let set_calculating = actions.set_calculating;
                    let set_calculated = actions.set_calculated;
                    let set_reveal_tick = actions.set_reveal_tick;
                    gloo_timers::callback::Timeout::new(420, move || {
                        set_calculating.set(false);
                        set_calculated.set(true);
                        set_reveal_tick.update(|tick| *tick = tick.wrapping_add(1));
                    })
                    .forget();
                }
            >
                {move || if actions.calculating.get() { "Calculating..." } else { "Calculate my ranking" }}
            </button>
            <details class="advanced">
                <summary>"More Filters"</summary>
                <div class="grid">
                    <label title="Filter by drug-tested or untested meets.">
                        "Drug tested"
                        <select on:change=move |ev| filters.set_tested.set(event_target_value(&ev))>
                            <For each=move || filters.tested_options.get() key=|v| v.clone() let:value>
                                <option
                                    selected={
                                        let selected_value = value.clone();
                                        move || filters.tested.get() == selected_value
                                    }
                                    value={value.clone()}
                                >
                                    {value.clone()}
                                </option>
                            </For>
                        </select>
                    </label>
                    <label title="Compare only with this age class, or all ages.">
                        "Age class"
                        <select on:change=move |ev| filters.set_age.set(event_target_value(&ev))>
                            <For each=move || filters.age_options.get() key=|v| v.clone() let:value>
                                <option
                                    selected={
                                        let selected_value = value.clone();
                                        move || filters.age.get() == selected_value
                                    }
                                    value={value.clone()}
                                >
                                    {age_label(&value).to_string()}
                                </option>
                            </For>
                        </select>
                    </label>
                    <label title="Compare only to this bodyweight class, or choose All.">
                        "Weight class"
                        <select on:change=move |ev| filters.set_wc.set(event_target_value(&ev))>
                            <For each=move || filters.wc_options.get() key=|v| v.clone() let:value>
                                <option
                                    selected={
                                        let selected_value = value.clone();
                                        move || filters.wc.get() == selected_value
                                    }
                                    value={value.clone()}
                                >
                                    {value.clone()}
                                </option>
                            </For>
                        </select>
                    </label>
                    <label title="Pick squat, bench, deadlift, or total.">
                        "Lift focus"
                        <select on:change=move |ev| filters.set_lift.set(event_target_value(&ev))>
                            <For each=move || filters.lift_options.get() key=|v| v.clone() let:value>
                                <option
                                    selected={
                                        let selected_value = value.clone();
                                        move || filters.lift.get() == selected_value
                                    }
                                    value={value.clone()}
                                >
                                    {lift_label(&value).to_string()}
                                </option>
                            </For>
                        </select>
                    </label>
                    <label title="Use kilograms or points for comparison.">
                        "Compare by"
                        <select on:change=move |ev| filters.set_metric.set(event_target_value(&ev))>
                            <For each=move || filters.metric_options.get() key=|v| v.clone() let:value>
                                <option
                                    selected={
                                        let selected_value = value.clone();
                                        move || filters.metric.get() == selected_value
                                    }
                                    value={value.clone()}
                                >
                                    {metric_label(&value).to_string()}
                                </option>
                            </For>
                        </select>
                    </label>
                </div>
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| {
                        filters.set_tested.set("All".to_string());
                        filters.set_age.set("All Ages".to_string());
                        filters.set_wc.set("All".to_string());
                        filters.set_lift.set("T".to_string());
                        filters.set_metric.set("Kg".to_string());
                    }
                >
                    "Reset to defaults"
                </button>
            </details>
        </section>
    }
}
