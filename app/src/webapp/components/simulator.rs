use crate::webapp::helpers::kg_to_display;
use crate::webapp::ui::parse_f32_input;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn SimulatorPanel(
    squat_delta: ReadSignal<f32>,
    set_squat_delta: WriteSignal<f32>,
    bench_delta: ReadSignal<f32>,
    set_bench_delta: WriteSignal<f32>,
    deadlift_delta: ReadSignal<f32>,
    set_deadlift_delta: WriteSignal<f32>,
    projected_total: Memo<f32>,
    projected_squat: Memo<f32>,
    projected_bench: Memo<f32>,
    projected_deadlift: Memo<f32>,
    use_lbs: ReadSignal<bool>,
    unit_label: Memo<&'static str>,
    projected_percentile: Memo<Option<(f32, usize, u32)>>,
    projected_rank_tier: Memo<Option<&'static str>>,
    percentile_delta: Memo<Option<f32>>,
    target_percentile: ReadSignal<f32>,
    set_target_percentile: WriteSignal<f32>,
    target_kg_needed: Memo<Option<f32>>,
    target_summary: Memo<String>,
) -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"What if you got stronger?"</h2>
            <p class="muted">
                "Use sliders to project where small improvements could move your rank."
            </p>
            <div class="sim-grid">
                <label class="sim-control">
                    <span>"Squat change"</span>
                    <input
                        type="range"
                        min="-50"
                        max="50"
                        step="0.5"
                        prop:value=move || squat_delta.get().to_string()
                        on:input=move |ev| set_squat_delta.set(parse_f32_input(&ev).clamp(-50.0, 50.0))
                    />
                    <strong>{move || format!("{:+.1} {}", kg_to_display(squat_delta.get(), use_lbs.get()), unit_label.get())}</strong>
                </label>
                <label class="sim-control">
                    <span>"Bench change"</span>
                    <input
                        type="range"
                        min="-50"
                        max="50"
                        step="0.5"
                        prop:value=move || bench_delta.get().to_string()
                        on:input=move |ev| set_bench_delta.set(parse_f32_input(&ev).clamp(-50.0, 50.0))
                    />
                    <strong>{move || format!("{:+.1} {}", kg_to_display(bench_delta.get(), use_lbs.get()), unit_label.get())}</strong>
                </label>
                <label class="sim-control">
                    <span>"Deadlift change"</span>
                    <input
                        type="range"
                        min="-50"
                        max="50"
                        step="0.5"
                        prop:value=move || deadlift_delta.get().to_string()
                        on:input=move |ev| set_deadlift_delta.set(parse_f32_input(&ev).clamp(-50.0, 50.0))
                    />
                    <strong>{move || format!("{:+.1} {}", kg_to_display(deadlift_delta.get(), use_lbs.get()), unit_label.get())}</strong>
                </label>
            </div>
            <div class="preset-row">
                <button type="button" class="chip" on:click=move |_| set_deadlift_delta.set((deadlift_delta.get_untracked() + 10.0).clamp(-50.0, 50.0))>"+10kg DL"</button>
                <button type="button" class="chip" on:click=move |_| {
                    set_squat_delta.set((squat_delta.get_untracked() + 6.5).clamp(-50.0, 50.0));
                    set_bench_delta.set((bench_delta.get_untracked() + 6.5).clamp(-50.0, 50.0));
                    set_deadlift_delta.set((deadlift_delta.get_untracked() + 7.0).clamp(-50.0, 50.0));
                }>"+20kg total"</button>
                <button type="button" class="chip" on:click=move |_| {
                    set_squat_delta.set((squat_delta.get_untracked() + 5.0).clamp(-50.0, 50.0));
                    set_bench_delta.set((bench_delta.get_untracked() + 5.0).clamp(-50.0, 50.0));
                    set_deadlift_delta.set((deadlift_delta.get_untracked() + 5.0).clamp(-50.0, 50.0));
                }>"Meet PRs"</button>
                <button type="button" class="chip" on:click=move |_| {
                    set_squat_delta.set((squat_delta.get_untracked() + 10.0).clamp(-50.0, 50.0));
                    set_bench_delta.set((bench_delta.get_untracked() + 5.0).clamp(-50.0, 50.0));
                    set_deadlift_delta.set((deadlift_delta.get_untracked() + 12.5).clamp(-50.0, 50.0));
                }>"1-year projection"</button>
            </div>
            <p class="sim-summary">
                {move || format!(
                    "Projected total: {:.1} {} (S {:.1} / B {:.1} / D {:.1})",
                    kg_to_display(projected_total.get(), use_lbs.get()),
                    unit_label.get(),
                    kg_to_display(projected_squat.get(), use_lbs.get()),
                    kg_to_display(projected_bench.get(), use_lbs.get()),
                    kg_to_display(projected_deadlift.get(), use_lbs.get())
                )}
            </p>
            <p class="muted">
                {move || match projected_percentile.get() {
                    Some((pct, rank, total)) => format!(
                        "Projected: {:.1}% percentile, rank ~{} / {}, tier {}",
                        pct * 100.0,
                        rank,
                        total,
                        projected_rank_tier.get().unwrap_or("Unknown")
                    ),
                    None => "Projected percentile will appear after calculation.".to_string(),
                }}
            </p>
            <p class="muted">
                {move || match percentile_delta.get() {
                    Some(delta) => format!("Shift: {:+.2} percentile points", delta * 100.0),
                    None => "Shift: n/a".to_string(),
                }}
            </p>
            <div class="target-planner">
                <h3>"Target Planner"</h3>
                <label class="sim-control">
                    <span>{move || format!("Target percentile: {:.0}%", target_percentile.get())}</span>
                    <input
                        type="range"
                        min="50"
                        max="99"
                        step="1"
                        prop:value=move || target_percentile.get().to_string()
                        on:input=move |ev| set_target_percentile.set(parse_f32_input(&ev).clamp(50.0, 99.0))
                    />
                </label>
                <p class="sim-summary">
                    {move || match target_kg_needed.get() {
                        Some(kg_needed) => format!(
                            "Estimated lift needed: +{:.1} {}",
                            kg_to_display(kg_needed, use_lbs.get()),
                            unit_label.get()
                        ),
                        None => "Estimated lift needed: n/a".to_string(),
                    }}
                </p>
                <p class="muted">{move || target_summary.get()}</p>
                <p class="muted">
                    "Estimate only. Real meet outcomes vary by attempt selection, meet conditions, and cohort changes."
                </p>
            </div>
        </section>
    }
}
