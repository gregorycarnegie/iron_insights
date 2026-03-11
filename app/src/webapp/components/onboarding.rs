use crate::webapp::helpers::{display_to_kg, kg_to_display};
use crate::webapp::models::SavedUiState;
use crate::webapp::ui::{age_label, lift_label, metric_label};
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
#[component]
pub(in crate::webapp) fn OnboardingPanel(
    sex_options: Memo<Vec<String>>,
    sex: ReadSignal<String>,
    set_sex: WriteSignal<String>,
    equip_options: Memo<Vec<String>>,
    equip: ReadSignal<String>,
    set_equip: WriteSignal<String>,
    unit_label: Memo<&'static str>,
    use_lbs: ReadSignal<bool>,
    set_use_lbs: WriteSignal<bool>,
    squat: ReadSignal<f32>,
    set_squat: WriteSignal<f32>,
    squat_error: ReadSignal<Option<String>>,
    set_squat_error: WriteSignal<Option<String>>,
    bench: ReadSignal<f32>,
    set_bench: WriteSignal<f32>,
    bench_error: ReadSignal<Option<String>>,
    set_bench_error: WriteSignal<Option<String>>,
    deadlift: ReadSignal<f32>,
    set_deadlift: WriteSignal<f32>,
    deadlift_error: ReadSignal<Option<String>>,
    set_deadlift_error: WriteSignal<Option<String>>,
    bodyweight: ReadSignal<f32>,
    set_bodyweight: WriteSignal<f32>,
    bodyweight_error: ReadSignal<Option<String>>,
    set_bodyweight_error: WriteSignal<Option<String>>,
    set_squat_delta: WriteSignal<f32>,
    set_bench_delta: WriteSignal<f32>,
    set_deadlift_delta: WriteSignal<f32>,
    set_share_handle: WriteSignal<String>,
    set_calculated: WriteSignal<bool>,
    has_input_error: Memo<bool>,
    calculating: ReadSignal<bool>,
    set_calculating: WriteSignal<bool>,
    set_share_status: WriteSignal<Option<String>>,
    tested_options: Memo<Vec<String>>,
    tested: ReadSignal<String>,
    set_tested: WriteSignal<String>,
    age_options: Memo<Vec<String>>,
    age: ReadSignal<String>,
    set_age: WriteSignal<String>,
    wc_options: Memo<Vec<String>>,
    wc: ReadSignal<String>,
    set_wc: WriteSignal<String>,
    lift_options: Memo<Vec<String>>,
    lift: ReadSignal<String>,
    set_lift: WriteSignal<String>,
    metric_options: Memo<Vec<String>>,
    metric: ReadSignal<String>,
    set_metric: WriteSignal<String>,
    set_lift_mult: WriteSignal<usize>,
    set_bw_mult: WriteSignal<usize>,
) -> impl IntoView {
    view! {
        <section class="panel onboarding">
            <h2>"Your Numbers"</h2>
            <div class="grid simple">
                <label>"Sex"
                    <select on:change=move |ev| set_sex.set(event_target_value(&ev))>
                        <For each=move || sex_options.get() key=|v| v.clone() let:value>
                            <option
                                selected={
                                    let selected_value = value.clone();
                                    move || sex.get() == selected_value
                                }
                                value={value.clone()}
                            >
                                {value.clone()}
                            </option>
                        </For>
                    </select>
                </label>
                <label>"Equipment"
                    <select on:change=move |ev| set_equip.set(event_target_value(&ev))>
                        <For each=move || equip_options.get() key=|v| v.clone() let:value>
                            <option
                                selected={
                                    let selected_value = value.clone();
                                    move || equip.get() == selected_value
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
                <label>{move || format!("Squat ({})", unit_label.get())}
                    <input
                        type="number"
                        min="0"
                        max="600"
                        step="0.5"
                        prop:value=move || kg_to_display(squat.get(), use_lbs.get()).to_string()
                        on:change=move |ev| {
                            let raw = event_target_value(&ev);
                            match raw.parse::<f32>() {
                                Ok(value) => {
                                    let value_kg = display_to_kg(value, use_lbs.get());
                                    if (0.0..=600.0).contains(&value_kg) {
                                        set_squat.set(value_kg);
                                        set_squat_error.set(None);
                                    } else {
                                        set_squat_error.set(Some(format!(
                                            "Squat must be {:.0}-{:.0}{}.",
                                            kg_to_display(0.0, use_lbs.get()),
                                            kg_to_display(600.0, use_lbs.get()),
                                            unit_label.get()
                                        )));
                                    }
                                }
                                Err(_) => set_squat_error.set(Some("Enter a valid squat number.".to_string())),
                            }
                        }
                    />
                </label>
                <label>{move || format!("Bench ({})", unit_label.get())}
                    <input
                        type="number"
                        min="0"
                        max="600"
                        step="0.5"
                        prop:value=move || kg_to_display(bench.get(), use_lbs.get()).to_string()
                        on:change=move |ev| {
                            let raw = event_target_value(&ev);
                            match raw.parse::<f32>() {
                                Ok(value) => {
                                    let value_kg = display_to_kg(value, use_lbs.get());
                                    if (0.0..=600.0).contains(&value_kg) {
                                        set_bench.set(value_kg);
                                        set_bench_error.set(None);
                                    } else {
                                        set_bench_error.set(Some(format!(
                                            "Bench must be {:.0}-{:.0}{}.",
                                            kg_to_display(0.0, use_lbs.get()),
                                            kg_to_display(600.0, use_lbs.get()),
                                            unit_label.get()
                                        )));
                                    }
                                }
                                Err(_) => set_bench_error.set(Some("Enter a valid bench number.".to_string())),
                            }
                        }
                    />
                </label>
                <label>{move || format!("Deadlift ({})", unit_label.get())}
                    <input
                        type="number"
                        min="0"
                        max="600"
                        step="0.5"
                        prop:value=move || kg_to_display(deadlift.get(), use_lbs.get()).to_string()
                        on:change=move |ev| {
                            let raw = event_target_value(&ev);
                            match raw.parse::<f32>() {
                                Ok(value) => {
                                    let value_kg = display_to_kg(value, use_lbs.get());
                                    if (0.0..=600.0).contains(&value_kg) {
                                        set_deadlift.set(value_kg);
                                        set_deadlift_error.set(None);
                                    } else {
                                        set_deadlift_error.set(Some(format!(
                                            "Deadlift must be {:.0}-{:.0}{}.",
                                            kg_to_display(0.0, use_lbs.get()),
                                            kg_to_display(600.0, use_lbs.get()),
                                            unit_label.get()
                                        )));
                                    }
                                }
                                Err(_) => set_deadlift_error.set(Some("Enter a valid deadlift number.".to_string())),
                            }
                        }
                    />
                </label>
                <label>{move || format!("Bodyweight ({})", unit_label.get())}
                    <input
                        type="number"
                        min="35"
                        max="300"
                        step="0.5"
                        prop:value=move || kg_to_display(bodyweight.get(), use_lbs.get()).to_string()
                        on:change=move |ev| {
                            let raw = event_target_value(&ev);
                            match raw.parse::<f32>() {
                                Ok(value) => {
                                    let value_kg = display_to_kg(value, use_lbs.get());
                                    if (35.0..=300.0).contains(&value_kg) {
                                        set_bodyweight.set(value_kg);
                                        set_bodyweight_error.set(None);
                                    } else {
                                        set_bodyweight_error.set(Some(format!(
                                            "Bodyweight must be {:.0}-{:.0}{}.",
                                            kg_to_display(35.0, use_lbs.get()),
                                            kg_to_display(300.0, use_lbs.get()),
                                            unit_label.get()
                                        )));
                                    }
                                }
                                Err(_) => set_bodyweight_error.set(Some("Enter a valid bodyweight number.".to_string())),
                            }
                        }
                    />
                </label>
            </div>
            <div class="input-errors">
                <Show when=move || squat_error.get().is_some()>
                    <p class="input-error">{move || squat_error.get().unwrap_or_default()}</p>
                </Show>
                <Show when=move || bench_error.get().is_some()>
                    <p class="input-error">{move || bench_error.get().unwrap_or_default()}</p>
                </Show>
                <Show when=move || deadlift_error.get().is_some()>
                    <p class="input-error">{move || deadlift_error.get().unwrap_or_default()}</p>
                </Show>
                <Show when=move || bodyweight_error.get().is_some()>
                    <p class="input-error">{move || bodyweight_error.get().unwrap_or_default()}</p>
                </Show>
            </div>
            <div class="control-row">
                <label class="units-toggle">
                    "Units"
                    <div class="toggle-buttons">
                        <button
                            type="button"
                            class="chip"
                            class:active=move || !use_lbs.get()
                            on:click=move |_| set_use_lbs.set(false)
                        >
                            "kg"
                        </button>
                        <button
                            type="button"
                            class="chip"
                            class:active=move || use_lbs.get()
                            on:click=move |_| set_use_lbs.set(true)
                        >
                            "lb"
                        </button>
                    </div>
                </label>
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| {
                        set_squat.set(0.0);
                        set_bench.set(0.0);
                        set_deadlift.set(0.0);
                        set_bodyweight.set(90.0);
                        set_squat_delta.set(0.0);
                        set_bench_delta.set(0.0);
                        set_deadlift_delta.set(0.0);
                        set_share_handle.set(String::new());
                        set_squat_error.set(None);
                        set_bench_error.set(None);
                        set_deadlift_error.set(None);
                        set_bodyweight_error.set(None);
                        set_calculated.set(false);
                    }
                >
                    "Clear all"
                </button>
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| {
                        let Some(window) = web_sys::window() else {
                            set_share_status.set(Some("No browser window.".to_string()));
                            return;
                        };
                        let Ok(Some(storage)) = window.local_storage() else {
                            set_share_status.set(Some("Local storage unavailable.".to_string()));
                            return;
                        };
                        let Ok(Some(raw)) = storage.get_item("iron_insights_last_state") else {
                            set_share_status.set(Some("No saved numbers found.".to_string()));
                            return;
                        };
                        let Ok(saved) = serde_json::from_str::<SavedUiState>(&raw) else {
                            set_share_status.set(Some("Saved numbers are invalid.".to_string()));
                            return;
                        };
                        set_sex.set(saved.sex);
                        set_equip.set(saved.equip);
                        set_wc.set(saved.wc);
                        set_age.set(saved.age);
                        set_tested.set(saved.tested);
                        set_lift.set(saved.lift);
                        set_metric.set(saved.metric);
                        set_squat.set(saved.squat.clamp(0.0, 600.0));
                        set_bench.set(saved.bench.clamp(0.0, 600.0));
                        set_deadlift.set(saved.deadlift.clamp(0.0, 600.0));
                        set_bodyweight.set(saved.bodyweight.clamp(35.0, 300.0));
                        set_squat_delta.set(saved.squat_delta.clamp(-50.0, 50.0));
                        set_bench_delta.set(saved.bench_delta.clamp(-50.0, 50.0));
                        set_deadlift_delta.set(saved.deadlift_delta.clamp(-50.0, 50.0));
                        set_lift_mult.set(saved.lift_mult.clamp(1, 4));
                        set_bw_mult.set(saved.bw_mult.clamp(1, 5));
                        set_share_handle.set(saved.share_handle);
                        set_calculated.set(saved.calculated);
                        set_share_status.set(Some("Loaded saved numbers.".to_string()));
                    }
                >
                    "Use my last numbers"
                </button>
            </div>
            <button
                type="button"
                class="primary"
                prop:disabled=move || has_input_error.get() || calculating.get()
                on:click=move |_| {
                    set_calculating.set(true);
                    set_share_status.set(None);
                    let set_calculating = set_calculating;
                    let set_calculated = set_calculated;
                    gloo_timers::callback::Timeout::new(420, move || {
                        set_calculating.set(false);
                        set_calculated.set(true);
                    })
                    .forget();
                }
            >
                {move || if calculating.get() { "Calculating..." } else { "Calculate my ranking" }}
            </button>
            <details class="advanced">
                <summary>"More Filters"</summary>
                <div class="grid">
                    <label title="Filter by drug-tested or untested meets.">
                        "Drug tested"
                        <select on:change=move |ev| set_tested.set(event_target_value(&ev))>
                            <For each=move || tested_options.get() key=|v| v.clone() let:value>
                                <option
                                    selected={
                                        let selected_value = value.clone();
                                        move || tested.get() == selected_value
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
                        <select on:change=move |ev| set_age.set(event_target_value(&ev))>
                            <For each=move || age_options.get() key=|v| v.clone() let:value>
                                <option
                                    selected={
                                        let selected_value = value.clone();
                                        move || age.get() == selected_value
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
                        <select on:change=move |ev| set_wc.set(event_target_value(&ev))>
                            <For each=move || wc_options.get() key=|v| v.clone() let:value>
                                <option
                                    selected={
                                        let selected_value = value.clone();
                                        move || wc.get() == selected_value
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
                        <select on:change=move |ev| set_lift.set(event_target_value(&ev))>
                            <For each=move || lift_options.get() key=|v| v.clone() let:value>
                                <option
                                    selected={
                                        let selected_value = value.clone();
                                        move || lift.get() == selected_value
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
                        <select on:change=move |ev| set_metric.set(event_target_value(&ev))>
                            <For each=move || metric_options.get() key=|v| v.clone() let:value>
                                <option
                                    selected={
                                        let selected_value = value.clone();
                                        move || metric.get() == selected_value
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
                        set_tested.set("All".to_string());
                        set_age.set("All Ages".to_string());
                        set_wc.set("All".to_string());
                        set_lift.set("T".to_string());
                        set_metric.set("Kg".to_string());
                    }
                >
                    "Reset to defaults"
                </button>
            </details>
        </section>
    }
}
