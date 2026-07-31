use crate::webapp::{
    helpers::{display_to_kg, format_input_bound},
    state::AppState,
};
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

/// One labelled lift/bodyweight field: input, unit hint, and its validation message.
#[component]
fn LiftField(
    id: &'static str,
    label: &'static str,
    value_kg: ReadSignal<f32>,
    set_value_kg: WriteSignal<f32>,
    error: ReadSignal<Option<String>>,
    set_error: WriteSignal<Option<String>>,
    use_lbs: ReadSignal<bool>,
    unit_label: Memo<&'static str>,
    min_kg: f32,
    max_kg: f32,
    error_msg: &'static str,
) -> impl IntoView {
    view! {
        <div>
            <label for=id>{label}</label>
            <div class="lift-row">
                <DecimalInput
                    id=id
                    value_kg=value_kg
                    set_value_kg=set_value_kg
                    set_error=set_error
                    use_lbs=use_lbs
                    min_kg=min_kg
                    max_kg=max_kg
                    error_msg=error_msg
                />
                <div class="hint">{move || unit_label.get().to_uppercase()}</div>
            </div>
            {move || error.get().map(|e| view! { <p class="notice error">{e}</p> })}
        </div>
    }
}

/// One labelled cohort dropdown. `label_for` renders each option's display text,
/// which is the only thing that differs between the advanced selects.
#[component]
fn SelectField(
    label: &'static str,
    options: Memo<Vec<String>>,
    current: ReadSignal<String>,
    set_current: WriteSignal<String>,
    #[prop(default = |opt: &str| opt.to_string())] label_for: fn(&str) -> String,
) -> impl IntoView {
    view! {
        <div>
            <label>{label}</label>
            <select
                on:change=move |ev| set_current.set(event_target_value(&ev))
                prop:value=move || current.get()
            >
                {move || options.get().into_iter().map(|opt| {
                    let text = label_for(&opt);
                    let selected_when = opt.clone();
                    view! {
                        <option value=opt.clone() prop:selected=move || current.get() == selected_when>
                            {text}
                        </option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}

fn lift_code_label(code: &str) -> String {
    match code {
        "S" => "Squat",
        "B" => "Bench",
        "D" => "Deadlift",
        "T" => "Total",
        _ => "Unknown",
    }
    .to_string()
}

#[component]
pub(super) fn InputForm() -> impl IntoView {
    let app = use_context::<AppState>().expect("AppState must be provided by App");
    let sel = app.selection;
    let inp = app.input;
    let cmp = app.compute;

    let on_compute = move |_| {
        inp.set_lift_mult.set(4);
        inp.set_bw_mult.set(5);
        let tick = cmp.reveal_tick.get_untracked();
        cmp.set_reveal_tick.set(tick.wrapping_add(1));
        cmp.set_calculated.set(true);
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

            <LiftField
                id="lifter-bodyweight" label="Bodyweight"
                value_kg=inp.bodyweight set_value_kg=inp.set_bodyweight
                error=inp.bodyweight_error set_error=inp.set_bodyweight_error
                use_lbs=inp.use_lbs unit_label=inp.unit_label
                min_kg=35.0 max_kg=300.0 error_msg="Enter 35–300 kg."
            />
            <LiftField
                id="lifter-squat" label="Squat"
                value_kg=inp.squat set_value_kg=inp.set_squat
                error=inp.squat_error set_error=inp.set_squat_error
                use_lbs=inp.use_lbs unit_label=inp.unit_label
                min_kg=0.0 max_kg=600.0 error_msg="Enter 0–600 kg."
            />
            <LiftField
                id="lifter-bench" label="Bench Press"
                value_kg=inp.bench set_value_kg=inp.set_bench
                error=inp.bench_error set_error=inp.set_bench_error
                use_lbs=inp.use_lbs unit_label=inp.unit_label
                min_kg=0.0 max_kg=600.0 error_msg="Enter 0–600 kg."
            />
            <LiftField
                id="lifter-deadlift" label="Deadlift"
                value_kg=inp.deadlift set_value_kg=inp.set_deadlift
                error=inp.deadlift_error set_error=inp.set_deadlift_error
                use_lbs=inp.use_lbs unit_label=inp.unit_label
                min_kg=0.0 max_kg=600.0 error_msg="Enter 0–600 kg."
            />

            <details class="advanced-fields">
                <summary>
                    <span><span class="adv-chevron">"›"</span>" Advanced"</span>
                    <small>{move || format!("{} / {} / {}", sel.equip.get(), sel.age.get(), sel.metric.get())}</small>
                </summary>

                <div class="advanced-grid">
                    <SelectField label="Equipment" options=sel.equip_opts current=sel.equip set_current=sel.set_equip />
                    <SelectField label="Weight Class" options=sel.wc_opts current=sel.wc set_current=sel.set_wc />
                    <SelectField label="Age Class" options=sel.age_opts current=sel.age set_current=sel.set_age />
                    <SelectField label="Tested Status" options=sel.tested_opts current=sel.tested set_current=sel.set_tested />
                    <SelectField label="Lift" options=sel.lift_opts current=sel.lift set_current=sel.set_lift label_for=lift_code_label />
                    <SelectField label="Metric" options=sel.metric_opts current=sel.metric set_current=sel.set_metric />
                </div>
            </details>

            <button class="btn" disabled=move || inp.has_input_error.get() on:click=on_compute>
                "COMPUTE PERCENTILE →"
            </button>
        </div>
    }
}
