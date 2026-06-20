use crate::webapp::helpers::{display_to_kg, format_input_bound};
use crate::webapp::state::AppState;
use leptos::prelude::*;
use web_sys::HtmlInputElement;

/// Corner decoration for panel
#[component]
pub(super) fn Corners() -> impl IntoView {
    view! {
        <span class="corner tl"></span>
        <span class="corner tr"></span>
        <span class="corner bl"></span>
        <span class="corner br"></span>
    }
}

/// Decimal-friendly numeric input.
///
/// Backed by a raw text buffer so partially-typed values (e.g. `200.`) survive
/// keystrokes instead of being clobbered by reformatting. The buffer holds the
/// value in display units; the canonical signal stays in kg. An effect re-syncs
/// the buffer only when the kg value or unit changes from outside (unit toggle,
/// sample load), never while the user is mid-edit.
#[component]
pub(super) fn DecimalInput(
    id: &'static str,
    value_kg: ReadSignal<f32>,
    set_value_kg: WriteSignal<f32>,
    set_error: WriteSignal<Option<String>>,
    use_lbs: ReadSignal<bool>,
    min_kg: f32,
    max_kg: f32,
    error_msg: &'static str,
) -> impl IntoView {
    let raw = RwSignal::new(format_input_bound(
        value_kg.get_untracked(),
        use_lbs.get_untracked(),
    ));

    Effect::new(move |_| {
        let kg = value_kg.get();
        let lbs = use_lbs.get();
        let in_sync = raw
            .get_untracked()
            .trim()
            .parse::<f32>()
            .ok()
            .is_some_and(|v| (display_to_kg(v, lbs) - kg).abs() <= 0.05);
        if !in_sync {
            raw.set(format_input_bound(kg, lbs));
        }
    });

    view! {
        <input
            id=id
            type="text"
            inputmode="decimal"
            prop:value=move || raw.get()
            on:input=move |ev| {
                let typed = event_target::<HtmlInputElement>(&ev).value();
                raw.set(typed.clone());
                match typed.trim().parse::<f32>() {
                    Ok(v) => {
                        let kg = display_to_kg(v, use_lbs.get());
                        if (min_kg..=max_kg).contains(&kg) {
                            set_error.set(None);
                            set_value_kg.set(kg);
                        } else {
                            set_error.set(Some(error_msg.to_string()));
                        }
                    }
                    Err(_) => {
                        if typed.trim().is_empty() {
                            set_error.set(None);
                        }
                    }
                }
            }
        />
    }
}

#[component]
pub(super) fn InputForm() -> impl IntoView {
    let app = use_context::<AppState>().expect("AppState must be provided by App");
    let sel = app.selection;
    let inp = app.input;
    let cmp = app.compute;

    let on_compute = move |_| {
        inp.set_squat_delta.set(0.0);
        inp.set_bench_delta.set(0.0);
        inp.set_deadlift_delta.set(0.0);
        inp.set_lift_mult.set(4);
        inp.set_bw_mult.set(5);
        cmp.set_calculating.set(true);
        let tick = cmp.reveal_tick.get_untracked();
        cmp.set_reveal_tick.set(tick.wrapping_add(1));
        cmp.set_calculated.set(true);
        cmp.set_calculating.set(false);
    };

    view! {
        <div class="input-stack">
            // Sex
            <div>
                <label>"Sex"</label>
                <div class="toggle-group">
                    {move || {
                        sel.sex_opts.get().into_iter().map(|opt| {
                            let opt_clone = opt.clone();
                            view! {
                                <button
                                    class:on=move || sel.sex.get() == opt
                                    on:click=move |_| sel.set_sex.set(opt_clone.clone())
                                >
                                    {if opt_clone == "M" { "Male" } else { "Female" }}
                                </button>
                            }
                        }).collect_view()
                    }}
                </div>
            </div>

            // Units
            <div>
                <label>"Units"</label>
                <div class="toggle-group">
                    <button class:on=move || !inp.use_lbs.get() on:click=move |_| inp.set_use_lbs.set(false)>"Kilograms"</button>
                    <button class:on=move || inp.use_lbs.get() on:click=move |_| inp.set_use_lbs.set(true)>"Pounds"</button>
                </div>
            </div>

            // Bodyweight
            <div>
                <label for="lifter-bodyweight">"Bodyweight"</label>
                <div class="lift-row">
                    <DecimalInput
                        id="lifter-bodyweight"
                        value_kg=inp.bodyweight
                        set_value_kg=inp.set_bodyweight
                        set_error=inp.set_bodyweight_error
                        use_lbs=inp.use_lbs
                        min_kg=35.0
                        max_kg=300.0
                        error_msg="Enter 35–300 kg."
                    />
                    <div class="hint">{move || inp.unit_label.get().to_uppercase()}</div>
                </div>
                {move || inp.bodyweight_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            // Squat
            <div>
                <label for="lifter-squat">"Squat"</label>
                <div class="lift-row">
                    <DecimalInput
                        id="lifter-squat"
                        value_kg=inp.squat
                        set_value_kg=inp.set_squat
                        set_error=inp.set_squat_error
                        use_lbs=inp.use_lbs
                        min_kg=0.0
                        max_kg=600.0
                        error_msg="Enter 0–600 kg."
                    />
                    <div class="hint">{move || inp.unit_label.get().to_uppercase()}</div>
                </div>
                {move || inp.squat_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            // Bench
            <div>
                <label for="lifter-bench">"Bench Press"</label>
                <div class="lift-row">
                    <DecimalInput
                        id="lifter-bench"
                        value_kg=inp.bench
                        set_value_kg=inp.set_bench
                        set_error=inp.set_bench_error
                        use_lbs=inp.use_lbs
                        min_kg=0.0
                        max_kg=600.0
                        error_msg="Enter 0–600 kg."
                    />
                    <div class="hint">{move || inp.unit_label.get().to_uppercase()}</div>
                </div>
                {move || inp.bench_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            // Deadlift
            <div>
                <label for="lifter-deadlift">"Deadlift"</label>
                <div class="lift-row">
                    <DecimalInput
                        id="lifter-deadlift"
                        value_kg=inp.deadlift
                        set_value_kg=inp.set_deadlift
                        set_error=inp.set_deadlift_error
                        use_lbs=inp.use_lbs
                        min_kg=0.0
                        max_kg=600.0
                        error_msg="Enter 0–600 kg."
                    />
                    <div class="hint">{move || inp.unit_label.get().to_uppercase()}</div>
                </div>
                {move || inp.deadlift_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            <details class="advanced-fields">
                <summary>
                    <span><span class="adv-chevron">"›"</span>" Advanced"</span>
                    <small>{move || format!("{} / {} / {}", sel.equip.get(), sel.age.get(), sel.metric.get())}</small>
                </summary>

                <div class="advanced-grid">
                    <div>
                        <label>"Equipment"</label>
                        <select
                            on:change=move |ev| {
                                sel.set_equip.set(event_target_value(&ev));
                            }
                            prop:value=move || sel.equip.get()
                        >
                            {move || sel.equip_opts.get().into_iter().map(|opt| {
                                let opt_clone = opt.clone();
                                view! { <option value={opt_clone.clone()} prop:selected=move || sel.equip.get() == opt_clone>{opt}</option> }
                            }).collect_view()}
                        </select>
                    </div>

                    <div>
                        <label>"Weight Class"</label>
                        <select on:change=move |ev| sel.set_wc.set(event_target_value(&ev)) prop:value=move || sel.wc.get()>
                            {move || sel.wc_opts.get().into_iter().map(|opt| {
                                let opt_c = opt.clone();
                                view! { <option value={opt_c.clone()} prop:selected=move || sel.wc.get() == opt_c>{opt}</option> }
                            }).collect_view()}
                        </select>
                    </div>

                    <div>
                        <label>"Age Class"</label>
                        <select on:change=move |ev| sel.set_age.set(event_target_value(&ev)) prop:value=move || sel.age.get()>
                            {move || sel.age_opts.get().into_iter().map(|opt| {
                                let opt_c = opt.clone();
                                view! { <option value={opt_c.clone()} prop:selected=move || sel.age.get() == opt_c>{opt}</option> }
                            }).collect_view()}
                        </select>
                    </div>

                    <div>
                        <label>"Tested Status"</label>
                        <select on:change=move |ev| sel.set_tested.set(event_target_value(&ev)) prop:value=move || sel.tested.get()>
                            {move || sel.tested_opts.get().into_iter().map(|opt| {
                                let opt_c = opt.clone();
                                view! { <option value={opt_c.clone()} prop:selected=move || sel.tested.get() == opt_c>{opt}</option> }
                            }).collect_view()}
                        </select>
                    </div>

                    <div>
                        <label>"Lift"</label>
                        <select on:change=move |ev| sel.set_lift.set(event_target_value(&ev)) prop:value=move || sel.lift.get()>
                            {move || sel.lift_opts.get().into_iter().map(|opt| {
                                let label = match opt.as_str() {
                                    "S" => "Squat", "B" => "Bench", "D" => "Deadlift",
                                    "T" => "Total", _ => "Unknown",
                                };
                                let opt_c = opt.clone();
                                view! { <option value={opt_c.clone()} prop:selected=move || sel.lift.get() == opt_c>{label}</option> }
                            }).collect_view()}
                        </select>
                    </div>

                    <div>
                        <label>"Metric"</label>
                        <select on:change=move |ev| sel.set_metric.set(event_target_value(&ev)) prop:value=move || sel.metric.get()>
                            {move || sel.metric_opts.get().into_iter().map(|opt| {
                                let opt_c = opt.clone();
                                view! { <option value={opt_c.clone()} prop:selected=move || sel.metric.get() == opt_c>{opt}</option> }
                            }).collect_view()}
                        </select>
                    </div>
                </div>
            </details>

            <button
                class="btn"
                disabled=move || inp.has_input_error.get() || cmp.calculating.get()
                on:click=on_compute
            >
                {move || if cmp.calculating.get() { "COMPUTING..." } else { "COMPUTE PERCENTILE →" }}
            </button>
        </div>
    }
}
