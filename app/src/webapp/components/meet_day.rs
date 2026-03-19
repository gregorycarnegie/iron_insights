use crate::webapp::helpers::kg_to_display;
use leptos::prelude::*;

fn round_to_step(value: f32, step: f32) -> f32 {
    if step <= 0.0 {
        return value;
    }
    (value / step).round() * step
}

fn attempt_plan(base_kg: f32, aggressive: bool) -> (f32, f32, f32) {
    let (o, s, t) = if aggressive {
        (0.92, 0.98, 1.03)
    } else {
        (0.90, 0.96, 1.00)
    };
    (
        round_to_step((base_kg * o).clamp(0.0, 600.0), 2.5),
        round_to_step((base_kg * s).clamp(0.0, 600.0), 2.5),
        round_to_step((base_kg * t).clamp(0.0, 600.0), 2.5),
    )
}

#[component]
pub(in crate::webapp) fn MeetDayPanel(
    squat: ReadSignal<f32>,
    bench: ReadSignal<f32>,
    deadlift: ReadSignal<f32>,
    use_lbs: ReadSignal<bool>,
    unit_label: Memo<&'static str>,
) -> impl IntoView {
    let (aggressive_mode, set_aggressive_mode) = signal(false);
    let squat_plan = Memo::new(move |_| attempt_plan(squat.get(), aggressive_mode.get()));
    let bench_plan = Memo::new(move |_| attempt_plan(bench.get(), aggressive_mode.get()));
    let deadlift_plan = Memo::new(move |_| attempt_plan(deadlift.get(), aggressive_mode.get()));

    let identity = Memo::new(move |_| {
        let s = squat.get().max(0.0);
        let b = bench.get().max(0.0);
        let d = deadlift.get().max(0.0);
        let total = s + b + d;
        if total <= 0.0 {
            return "Add your lifts to see your identity.".to_string();
        }
        let s_share = s / total;
        let b_share = b / total;
        let d_share = d / total;
        if s_share >= b_share && s_share >= d_share {
            format!("Squat-dominant profile ({:.1}% of total).", s_share * 100.0)
        } else if b_share >= s_share && b_share >= d_share {
            format!("Bench-dominant profile ({:.1}% of total).", b_share * 100.0)
        } else {
            format!(
                "Deadlift-dominant profile ({:.1}% of total).",
                d_share * 100.0
            )
        }
    });

    view! {
        <section class="panel meet-day">
            <div class="panel-titlebar">
                <div>
                    <h2>"Meet-Day Scorecard"</h2>
                    <p class="muted">
                        "Suggested openers, seconds, and thirds from your current numbers."
                    </p>
                </div>
            </div>
            <div class="toggle-buttons meet-toggle-row">
                <button
                    type="button"
                    class:chip=true
                    class:active=move || !aggressive_mode.get()
                    on:click=move |_| set_aggressive_mode.set(false)
                >
                    "Conservative"
                </button>
                <button
                    type="button"
                    class:chip=true
                    class:active=move || aggressive_mode.get()
                    on:click=move |_| set_aggressive_mode.set(true)
                >
                    "Aggressive"
                </button>
            </div>
            <div class="meet-grid meet-scorecard">
                <div class="meet-item">
                    <div class="meet-item-header">
                        <h3>"Squat"</h3>
                        <p class="meet-current">
                            {move || format!(
                                "Current {:.1} {}",
                                kg_to_display(squat.get(), use_lbs.get()),
                                unit_label.get()
                            )}
                        </p>
                    </div>
                    <div class="meet-attempts">
                        <div class="meet-attempt meet-attempt--opener">
                            <span>"Opener"</span>
                            <strong>{move || format!("{:.1} {}", kg_to_display(squat_plan.get().0, use_lbs.get()), unit_label.get())}</strong>
                        </div>
                        <div class="meet-attempt">
                            <span>"Second"</span>
                            <strong>{move || format!("{:.1} {}", kg_to_display(squat_plan.get().1, use_lbs.get()), unit_label.get())}</strong>
                        </div>
                        <div class="meet-attempt">
                            <span>"Third"</span>
                            <strong>{move || format!("{:.1} {}", kg_to_display(squat_plan.get().2, use_lbs.get()), unit_label.get())}</strong>
                        </div>
                    </div>
                </div>
                <div class="meet-item">
                    <div class="meet-item-header">
                        <h3>"Bench"</h3>
                        <p class="meet-current">
                            {move || format!(
                                "Current {:.1} {}",
                                kg_to_display(bench.get(), use_lbs.get()),
                                unit_label.get()
                            )}
                        </p>
                    </div>
                    <div class="meet-attempts">
                        <div class="meet-attempt meet-attempt--opener">
                            <span>"Opener"</span>
                            <strong>{move || format!("{:.1} {}", kg_to_display(bench_plan.get().0, use_lbs.get()), unit_label.get())}</strong>
                        </div>
                        <div class="meet-attempt">
                            <span>"Second"</span>
                            <strong>{move || format!("{:.1} {}", kg_to_display(bench_plan.get().1, use_lbs.get()), unit_label.get())}</strong>
                        </div>
                        <div class="meet-attempt">
                            <span>"Third"</span>
                            <strong>{move || format!("{:.1} {}", kg_to_display(bench_plan.get().2, use_lbs.get()), unit_label.get())}</strong>
                        </div>
                    </div>
                </div>
                <div class="meet-item">
                    <div class="meet-item-header">
                        <h3>"Deadlift"</h3>
                        <p class="meet-current">
                            {move || format!(
                                "Current {:.1} {}",
                                kg_to_display(deadlift.get(), use_lbs.get()),
                                unit_label.get()
                            )}
                        </p>
                    </div>
                    <div class="meet-attempts">
                        <div class="meet-attempt meet-attempt--opener">
                            <span>"Opener"</span>
                            <strong>{move || format!("{:.1} {}", kg_to_display(deadlift_plan.get().0, use_lbs.get()), unit_label.get())}</strong>
                        </div>
                        <div class="meet-attempt">
                            <span>"Second"</span>
                            <strong>{move || format!("{:.1} {}", kg_to_display(deadlift_plan.get().1, use_lbs.get()), unit_label.get())}</strong>
                        </div>
                        <div class="meet-attempt">
                            <span>"Third"</span>
                            <strong>{move || format!("{:.1} {}", kg_to_display(deadlift_plan.get().2, use_lbs.get()), unit_label.get())}</strong>
                        </div>
                    </div>
                </div>
            </div>
            <p class="meet-summary">{move || identity.get()}</p>
        </section>
    }
}
