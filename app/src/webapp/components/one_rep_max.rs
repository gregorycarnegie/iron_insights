use leptos::prelude::*;

use crate::webapp::helpers::{display_to_kg, kg_to_display};

#[component]
pub(in crate::webapp) fn OneRepMaxPanel() -> impl IntoView {
    let (use_lbs, set_use_lbs) = signal(false);
    let (load, set_load) = signal(100.0f32);
    let (reps, set_reps) = signal(5u32);

    let load_kg = Memo::new(move |_| display_to_kg(load.get(), use_lbs.get()));
    let epley_kg = Memo::new(move |_| {
        let r = reps.get() as f32;
        if r <= 0.0 {
            0.0
        } else {
            load_kg.get() * (1.0 + r / 30.0)
        }
    });
    let brzycki_kg = Memo::new(move |_| {
        let r = reps.get() as f32;
        if r <= 1.0 {
            load_kg.get()
        } else if r >= 37.0 {
            0.0
        } else {
            load_kg.get() * (36.0 / (37.0 - r))
        }
    });
    let average_kg = Memo::new(move |_| {
        let a = epley_kg.get();
        let b = brzycki_kg.get();
        if b > 0.0 { (a + b) * 0.5 } else { a }
    });

    let unit = Memo::new(move |_| if use_lbs.get() { "lb" } else { "kg" });

    view! {
        <section class="panel one-rm">
            <h2>"1-Rep Max Calculator"</h2>
            <p class="muted">
                "Estimate your true 1RM from a working set."
            </p>

            <div class="control-row">
                <div class="units-toggle">
                    <span>"Units"</span>
                    <div class="toggle-buttons">
                        <button
                            type="button"
                            class:chip=true
                            class:active=move || !use_lbs.get()
                            on:click=move |_| set_use_lbs.set(false)
                        >
                            "kg"
                        </button>
                        <button
                            type="button"
                            class:chip=true
                            class:active=move || use_lbs.get()
                            on:click=move |_| set_use_lbs.set(true)
                        >
                            "lb"
                        </button>
                    </div>
                </div>
            </div>

            <div class="grid simple one-rm-grid">
                <label>{move || format!("Lifted weight ({})", unit.get())}
                    <input
                        type="number"
                        min="1"
                        max="1000"
                        step="0.5"
                        prop:value=move || load.get().to_string()
                        on:change=move |ev| {
                            if let Ok(value) = event_target_value(&ev).parse::<f32>()
                                && value.is_finite() && value > 0.0
                            {
                                set_load.set(value);
                            }
                        }
                    />
                </label>
                <label>"Reps completed"
                    <input
                        type="number"
                        min="1"
                        max="20"
                        step="1"
                        prop:value=move || reps.get().to_string()
                        on:change=move |ev| {
                            if let Ok(value) = event_target_value(&ev).parse::<u32>()
                                && (1..=20).contains(&value)
                            {
                                set_reps.set(value);
                            }
                        }
                    />
                </label>
            </div>

            <div class="one-rm-results">
                <p class="one-rm-primary">
                    {move || format!(
                        "Estimated 1RM: {:.1} {}",
                        kg_to_display(average_kg.get(), use_lbs.get()),
                        unit.get()
                    )}
                </p>
                <p class="muted">
                    {move || format!(
                        "Epley: {:.1} {} | Brzycki: {:.1} {}",
                        kg_to_display(epley_kg.get(), use_lbs.get()),
                        unit.get(),
                        kg_to_display(brzycki_kg.get(), use_lbs.get()),
                        unit.get()
                    )}
                </p>
            </div>
        </section>
    }
}
