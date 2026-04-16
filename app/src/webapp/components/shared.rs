use crate::webapp::helpers::{display_to_kg, format_input_bound, kg_to_display};
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

/// Standard panel wrapper
#[component]
pub(super) fn Panel(
    #[prop(into)] tag: String,
    #[prop(into)] label: String,
    #[prop(into, optional)] right: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="panel">
            <Corners />
            <div class="panel-head">
                <span><span class="tag">{tag}</span>" "{label}</span>
                {right.map(|r| view! { <span>{r}</span> })}
            </div>
            <div class="panel-body">
                {children()}
            </div>
        </div>
    }
}

#[derive(Clone)]
pub(super) struct InputFormCtx {
    pub(super) sex_opts: Memo<Vec<String>>,
    pub(super) sex: ReadSignal<String>,
    pub(super) set_sex: WriteSignal<String>,
    pub(super) equip_opts: Memo<Vec<String>>,
    pub(super) equip: ReadSignal<String>,
    pub(super) set_equip: WriteSignal<String>,
    pub(super) unit_label: Memo<&'static str>,
    pub(super) use_lbs: ReadSignal<bool>,
    pub(super) set_use_lbs: WriteSignal<bool>,
    pub(super) wc_opts: Memo<Vec<String>>,
    pub(super) wc: ReadSignal<String>,
    pub(super) set_wc: WriteSignal<String>,
    pub(super) age_opts: Memo<Vec<String>>,
    pub(super) age: ReadSignal<String>,
    pub(super) set_age: WriteSignal<String>,
    pub(super) tested_opts: Memo<Vec<String>>,
    pub(super) tested: ReadSignal<String>,
    pub(super) set_tested: WriteSignal<String>,
    pub(super) lift_opts: Memo<Vec<String>>,
    pub(super) lift: ReadSignal<String>,
    pub(super) set_lift: WriteSignal<String>,
    pub(super) metric_opts: Memo<Vec<String>>,
    pub(super) metric: ReadSignal<String>,
    pub(super) set_metric: WriteSignal<String>,
    pub(super) squat: ReadSignal<f32>,
    pub(super) set_squat: WriteSignal<f32>,
    pub(super) squat_error: ReadSignal<Option<String>>,
    pub(super) set_squat_error: WriteSignal<Option<String>>,
    pub(super) bench: ReadSignal<f32>,
    pub(super) set_bench: WriteSignal<f32>,
    pub(super) bench_error: ReadSignal<Option<String>>,
    pub(super) set_bench_error: WriteSignal<Option<String>>,
    pub(super) deadlift: ReadSignal<f32>,
    pub(super) set_deadlift: WriteSignal<f32>,
    pub(super) deadlift_error: ReadSignal<Option<String>>,
    pub(super) set_deadlift_error: WriteSignal<Option<String>>,
    pub(super) bodyweight: ReadSignal<f32>,
    pub(super) set_bodyweight: WriteSignal<f32>,
    pub(super) bodyweight_error: ReadSignal<Option<String>>,
    pub(super) set_bodyweight_error: WriteSignal<Option<String>>,
    pub(super) calculated: ReadSignal<bool>,
    pub(super) set_calculated: WriteSignal<bool>,
    pub(super) calculating: ReadSignal<bool>,
    pub(super) set_calculating: WriteSignal<bool>,
    pub(super) has_input_error: Memo<bool>,
    pub(super) reveal_tick: ReadSignal<u64>,
    pub(super) set_reveal_tick: WriteSignal<u64>,
    pub(super) set_squat_delta: WriteSignal<f32>,
    pub(super) set_bench_delta: WriteSignal<f32>,
    pub(super) set_deadlift_delta: WriteSignal<f32>,
    pub(super) set_lift_mult: WriteSignal<usize>,
    pub(super) set_bw_mult: WriteSignal<usize>,
}

#[component]
pub(super) fn InputForm(ctx: InputFormCtx) -> impl IntoView {
    let InputFormCtx {
        sex_opts,
        sex,
        set_sex,
        equip_opts,
        equip,
        set_equip,
        unit_label,
        use_lbs,
        set_use_lbs,
        wc_opts,
        wc,
        set_wc,
        age_opts,
        age,
        set_age,
        tested_opts,
        tested,
        set_tested,
        lift_opts,
        lift,
        set_lift,
        metric_opts,
        metric,
        set_metric,
        squat,
        set_squat,
        squat_error,
        set_squat_error,
        bench,
        set_bench,
        bench_error,
        set_bench_error,
        deadlift,
        set_deadlift,
        deadlift_error,
        set_deadlift_error,
        bodyweight,
        set_bodyweight,
        bodyweight_error,
        set_bodyweight_error,
        calculated,
        set_calculated,
        calculating,
        set_calculating,
        has_input_error,
        reveal_tick,
        set_reveal_tick,
        set_squat_delta,
        set_bench_delta,
        set_deadlift_delta,
        set_lift_mult,
        set_bw_mult,
    } = ctx;

    let on_compute = move |_| {
        set_squat_delta.set(0.0);
        set_bench_delta.set(0.0);
        set_deadlift_delta.set(0.0);
        set_lift_mult.set(4);
        set_bw_mult.set(5);
        set_calculating.set(true);
        let tick = reveal_tick.get_untracked();
        set_reveal_tick.set(tick.wrapping_add(1));
        set_calculated.set(true);
        set_calculating.set(false);
    };

    view! {
        <div class="input-stack">
            // Sex
            <div>
                <label>"Sex"</label>
                <div class="toggle-group">
                    {move || {
                        sex_opts.get().into_iter().map(|opt| {
                            let opt_clone = opt.clone();
                            view! {
                                <button
                                    class:on=move || sex.get() == opt
                                    on:click=move |_| set_sex.set(opt_clone.clone())
                                >
                                    {if opt_clone == "M" { "Male" } else { "Female" }}
                                </button>
                            }
                        }).collect_view()
                    }}
                </div>
            </div>

            // Equipment
            <div>
                <label>"Equipment"</label>
                <select
                    on:change=move |ev| {
                        set_equip.set(event_target_value(&ev));
                    }
                    prop:value=move || equip.get()
                >
                    {move || equip_opts.get().into_iter().map(|opt| {
                        let opt_clone = opt.clone();
                        view! { <option value={opt_clone.clone()} prop:selected=move || equip.get() == opt_clone>{opt}</option> }
                    }).collect_view()}
                </select>
            </div>

            // Units
            <div>
                <label>"Units"</label>
                <div class="toggle-group">
                    <button class:on=move || !use_lbs.get() on:click=move |_| set_use_lbs.set(false)>"Kilograms"</button>
                    <button class:on=move || use_lbs.get() on:click=move |_| set_use_lbs.set(true)>"Pounds"</button>
                </div>
            </div>

            // Bodyweight
            <div>
                <label>"Bodyweight"</label>
                <div class="lift-row">
                    <input
                        type="number"
                        step="0.5"
                        prop:value=move || format_input_bound(bodyweight.get(), use_lbs.get())
                        on:input=move |ev| {
                            let raw = parse_f32_input(&ev);
                            let kg = display_to_kg(raw, use_lbs.get());
                            if kg < 35.0 || kg > 300.0 {
                                set_bodyweight_error.set(Some(format!("Enter 35–300 kg.")));
                            } else {
                                set_bodyweight_error.set(None);
                                set_bodyweight.set(kg);
                            }
                        }
                    />
                    <div class="hint">{move || unit_label.get().to_uppercase()}</div>
                </div>
                {move || bodyweight_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            // Squat
            <div>
                <label>"Squat"</label>
                <div class="lift-row">
                    <input
                        type="number"
                        step="2.5"
                        prop:value=move || format_input_bound(squat.get(), use_lbs.get())
                        on:input=move |ev| {
                            let raw = parse_f32_input(&ev);
                            let kg = display_to_kg(raw, use_lbs.get());
                            if kg < 0.0 || kg > 600.0 {
                                set_squat_error.set(Some("Enter 0–600 kg.".to_string()));
                            } else {
                                set_squat_error.set(None);
                                set_squat.set(kg);
                            }
                        }
                    />
                    <div class="hint">{move || unit_label.get().to_uppercase()}</div>
                </div>
                {move || squat_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            // Bench
            <div>
                <label>"Bench Press"</label>
                <div class="lift-row">
                    <input
                        type="number"
                        step="2.5"
                        prop:value=move || format_input_bound(bench.get(), use_lbs.get())
                        on:input=move |ev| {
                            let raw = parse_f32_input(&ev);
                            let kg = display_to_kg(raw, use_lbs.get());
                            if kg < 0.0 || kg > 600.0 {
                                set_bench_error.set(Some("Enter 0–600 kg.".to_string()));
                            } else {
                                set_bench_error.set(None);
                                set_bench.set(kg);
                            }
                        }
                    />
                    <div class="hint">{move || unit_label.get().to_uppercase()}</div>
                </div>
                {move || bench_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            // Deadlift
            <div>
                <label>"Deadlift"</label>
                <div class="lift-row">
                    <input
                        type="number"
                        step="2.5"
                        prop:value=move || format_input_bound(deadlift.get(), use_lbs.get())
                        on:input=move |ev| {
                            let raw = parse_f32_input(&ev);
                            let kg = display_to_kg(raw, use_lbs.get());
                            if kg < 0.0 || kg > 600.0 {
                                set_deadlift_error.set(Some("Enter 0–600 kg.".to_string()));
                            } else {
                                set_deadlift_error.set(None);
                                set_deadlift.set(kg);
                            }
                        }
                    />
                    <div class="hint">{move || unit_label.get().to_uppercase()}</div>
                </div>
                {move || deadlift_error.get().map(|e| view! { <p class="notice error">{e}</p> })}
            </div>

            // Filters
            <div>
                <label>"Weight Class"</label>
                <select on:change=move |ev| set_wc.set(event_target_value(&ev)) prop:value=move || wc.get()>
                    {move || wc_opts.get().into_iter().map(|opt| {
                        let opt_c = opt.clone();
                        view! { <option value={opt_c.clone()} prop:selected=move || wc.get() == opt_c>{opt}</option> }
                    }).collect_view()}
                </select>
            </div>

            <div>
                <label>"Age Class"</label>
                <select on:change=move |ev| set_age.set(event_target_value(&ev)) prop:value=move || age.get()>
                    {move || age_opts.get().into_iter().map(|opt| {
                        let opt_c = opt.clone();
                        view! { <option value={opt_c.clone()} prop:selected=move || age.get() == opt_c>{opt}</option> }
                    }).collect_view()}
                </select>
            </div>

            <div>
                <label>"Tested Status"</label>
                <select on:change=move |ev| set_tested.set(event_target_value(&ev)) prop:value=move || tested.get()>
                    {move || tested_opts.get().into_iter().map(|opt| {
                        let opt_c = opt.clone();
                        view! { <option value={opt_c.clone()} prop:selected=move || tested.get() == opt_c>{opt}</option> }
                    }).collect_view()}
                </select>
            </div>

            <div>
                <label>"Lift"</label>
                <select on:change=move |ev| set_lift.set(event_target_value(&ev)) prop:value=move || lift.get()>
                    {move || lift_opts.get().into_iter().map(|opt| {
                        let label = match opt.as_str() {
                            "S" => "Squat", "B" => "Bench", "D" => "Deadlift",
                            "T" => "Total", _ => "Unknown",
                        };
                        let opt_c = opt.clone();
                        view! { <option value={opt_c.clone()} prop:selected=move || lift.get() == opt_c>{label}</option> }
                    }).collect_view()}
                </select>
            </div>

            <div>
                <label>"Metric"</label>
                <select on:change=move |ev| set_metric.set(event_target_value(&ev)) prop:value=move || metric.get()>
                    {move || metric_opts.get().into_iter().map(|opt| {
                        let opt_c = opt.clone();
                        view! { <option value={opt_c.clone()} prop:selected=move || metric.get() == opt_c>{opt}</option> }
                    }).collect_view()}
                </select>
            </div>

            <button
                class="btn"
                disabled=move || has_input_error.get() || calculating.get()
                on:click=on_compute
            >
                {move || if calculating.get() { "COMPUTING..." } else { "COMPUTE PERCENTILE →" }}
            </button>
        </div>
    }
}
