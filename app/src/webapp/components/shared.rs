use crate::webapp::helpers::{display_to_kg, format_input_bound};
use crate::webapp::state::AppState;
use crate::webapp::ui::parse_f32_input;
use leptos::prelude::*;

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
                <label>"Bodyweight"</label>
                <div class="lift-row">
                    <input
                        type="number"
                        step="0.5"
                        prop:value=move || format_input_bound(inp.bodyweight.get(), inp.use_lbs.get())
                        on:input=move |ev| {
                            let raw = parse_f32_input(&ev);
                            let kg = display_to_kg(raw, inp.use_lbs.get());
                            if !(35.0..=300.0).contains(&kg) {
                                inp.set_bodyweight_error.set(Some("Enter 35–300 kg.".to_string()));
                            } else {
                                inp.set_bodyweight_error.set(None);
                                inp.set_bodyweight.set(kg);
                            }
                        }
                    />
                    <div class="hint">{move || inp.unit_label.get().to_uppercase()}</div>
                </div>
                {move || inp.bodyweight_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            // Squat
            <div>
                <label>"Squat"</label>
                <div class="lift-row">
                    <input
                        type="number"
                        step="2.5"
                        prop:value=move || format_input_bound(inp.squat.get(), inp.use_lbs.get())
                        on:input=move |ev| {
                            let raw = parse_f32_input(&ev);
                            let kg = display_to_kg(raw, inp.use_lbs.get());
                            if !(0.0..=600.0).contains(&kg) {
                                inp.set_squat_error.set(Some("Enter 0–600 kg.".to_string()));
                            } else {
                                inp.set_squat_error.set(None);
                                inp.set_squat.set(kg);
                            }
                        }
                    />
                    <div class="hint">{move || inp.unit_label.get().to_uppercase()}</div>
                </div>
                {move || inp.squat_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            // Bench
            <div>
                <label>"Bench Press"</label>
                <div class="lift-row">
                    <input
                        type="number"
                        step="2.5"
                        prop:value=move || format_input_bound(inp.bench.get(), inp.use_lbs.get())
                        on:input=move |ev| {
                            let raw = parse_f32_input(&ev);
                            let kg = display_to_kg(raw, inp.use_lbs.get());
                            if !(0.0..=600.0).contains(&kg) {
                                inp.set_bench_error.set(Some("Enter 0–600 kg.".to_string()));
                            } else {
                                inp.set_bench_error.set(None);
                                inp.set_bench.set(kg);
                            }
                        }
                    />
                    <div class="hint">{move || inp.unit_label.get().to_uppercase()}</div>
                </div>
                {move || inp.bench_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            // Deadlift
            <div>
                <label>"Deadlift"</label>
                <div class="lift-row">
                    <input
                        type="number"
                        step="2.5"
                        prop:value=move || format_input_bound(inp.deadlift.get(), inp.use_lbs.get())
                        on:input=move |ev| {
                            let raw = parse_f32_input(&ev);
                            let kg = display_to_kg(raw, inp.use_lbs.get());
                            if !(0.0..=600.0).contains(&kg) {
                                inp.set_deadlift_error.set(Some("Enter 0–600 kg.".to_string()));
                            } else {
                                inp.set_deadlift_error.set(None);
                                inp.set_deadlift.set(kg);
                            }
                        }
                    />
                    <div class="hint">{move || inp.unit_label.get().to_uppercase()}</div>
                </div>
                {move || inp.deadlift_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            <details class="advanced-fields">
                <summary>
                    <span>"Advanced"</span>
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
