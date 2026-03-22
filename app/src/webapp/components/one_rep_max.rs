use leptos::prelude::*;

use crate::webapp::helpers::{display_to_kg, format_input_bound, kg_to_display};

const QUICK_REP_PRESETS: [u32; 4] = [3, 5, 8, 10];

#[derive(Clone, Copy, PartialEq, Eq)]
struct RepMaxTarget {
    reps: u32,
    label: &'static str,
}

const REP_MAX_TARGETS: [RepMaxTarget; 7] = [
    RepMaxTarget {
        reps: 1,
        label: "max single",
    },
    RepMaxTarget {
        reps: 2,
        label: "heavy double",
    },
    RepMaxTarget {
        reps: 3,
        label: "hard triple",
    },
    RepMaxTarget {
        reps: 4,
        label: "heavy four",
    },
    RepMaxTarget {
        reps: 5,
        label: "classic 5RM",
    },
    RepMaxTarget {
        reps: 8,
        label: "solid volume",
    },
    RepMaxTarget {
        reps: 10,
        label: "high-rep push",
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
struct TrainingPercentage {
    percent: u32,
    label: &'static str,
}

const TRAINING_PERCENTAGES: [TrainingPercentage; 7] = [
    TrainingPercentage {
        percent: 60,
        label: "light technique",
    },
    TrainingPercentage {
        percent: 70,
        label: "steady volume",
    },
    TrainingPercentage {
        percent: 75,
        label: "comfortable work",
    },
    TrainingPercentage {
        percent: 80,
        label: "hard work sets",
    },
    TrainingPercentage {
        percent: 85,
        label: "top triples",
    },
    TrainingPercentage {
        percent: 90,
        label: "heavy single or double",
    },
    TrainingPercentage {
        percent: 95,
        label: "near-max single",
    },
];

fn format_input_weight(value: f32) -> String {
    if (value - value.round()).abs() < 0.05 {
        format!("{:.0}", value)
    } else {
        format!("{:.1}", value)
    }
}

fn epley_1rm(load_kg: f32, reps: u32) -> f32 {
    if reps <= 1 {
        load_kg
    } else {
        load_kg * (1.0 + reps as f32 / 30.0)
    }
}

fn brzycki_1rm(load_kg: f32, reps: u32) -> f32 {
    if reps <= 1 {
        load_kg
    } else if reps >= 37 {
        0.0
    } else {
        load_kg * (36.0 / (37.0 - reps as f32))
    }
}

fn blend_factor(reps: u32) -> f32 {
    ((reps as f32 - 8.0) / 2.0).clamp(0.0, 1.0)
}

fn blended_1rm(load_kg: f32, reps: u32) -> f32 {
    if reps <= 1 {
        return load_kg;
    }

    let brzycki = brzycki_1rm(load_kg, reps);
    let epley = epley_1rm(load_kg, reps);

    if reps < 8 {
        brzycki
    } else if reps > 10 {
        epley
    } else {
        brzycki + (epley - brzycki) * blend_factor(reps)
    }
}

fn working_weight_for_reps(one_rm_kg: f32, reps: u32) -> f32 {
    if reps <= 1 {
        return one_rm_kg;
    }

    let brzycki = one_rm_kg * ((37.0 - reps as f32) / 36.0).max(0.0);
    let epley = one_rm_kg / (1.0 + reps as f32 / 30.0);

    if reps < 8 {
        brzycki
    } else if reps > 10 {
        epley
    } else {
        brzycki + (epley - brzycki) * blend_factor(reps)
    }
}

fn estimate_quality_badge(reps: u32) -> &'static str {
    match reps {
        1 => "Actual single entered",
        2..=5 => "Strong estimate",
        6..=10 => "Good estimate",
        11..=15 => "Rougher estimate",
        _ => "Ballpark only",
    }
}

fn estimate_guidance(reps: u32) -> &'static str {
    match reps {
        1 => {
            "A true single is already your best day number, so the tables below use that single as the base."
        }
        2..=5 => {
            "Heavy doubles to fives usually give the cleanest 1RM estimate without needing a true max test."
        }
        6..=10 => {
            "This is still useful for planning, but expect a bit more spread between formulas as reps climb."
        }
        11..=15 => {
            "Use this as a planning number, not a promise. Higher-rep sets are less precise for predicting a max."
        }
        _ => {
            "Very high-rep sets can drift a lot. Treat this as a rough training estimate rather than a meet-day forecast."
        }
    }
}

#[component]
pub(in crate::webapp) fn OneRepMaxPanel() -> impl IntoView {
    let (use_lbs, set_use_lbs) = signal(false);
    let (load_kg, set_load_kg) = signal(100.0f32);
    let (reps, set_reps) = signal(5u32);

    let display_load = Memo::new(move |_| kg_to_display(load_kg.get(), use_lbs.get()));
    let epley_kg = Memo::new(move |_| epley_1rm(load_kg.get(), reps.get()));
    let brzycki_kg = Memo::new(move |_| brzycki_1rm(load_kg.get(), reps.get()));
    let estimate_kg = Memo::new(move |_| blended_1rm(load_kg.get(), reps.get()));
    let formula_range_kg = Memo::new(move |_| {
        let epley = epley_kg.get();
        let brzycki = brzycki_kg.get();
        (epley.min(brzycki), epley.max(brzycki))
    });
    let set_intensity_pct = Memo::new(move |_| {
        let estimate = estimate_kg.get();
        if estimate > 0.0 {
            (load_kg.get() / estimate) * 100.0
        } else {
            0.0
        }
    });
    let unit = Memo::new(move |_| if use_lbs.get() { "lb" } else { "kg" });
    let load_input_max = Memo::new(move |_| format_input_bound(2000.0, use_lbs.get()));
    let load_input_step = Memo::new(move |_| if use_lbs.get() { "1" } else { "0.5" });

    view! {
        <section class="panel one-rm">
            <div class="one-rm-header">
                <div class="one-rm-header-copy">
                    <h2>"1-Rep Max Calculator"</h2>
                    <p class="muted one-rm-intro">
                        "Turn a hard recent set into a realistic max estimate, rep targets, and training weights."
                    </p>
                </div>
                <p class="one-rm-badge">{move || estimate_quality_badge(reps.get())}</p>
            </div>

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
                        prop:min="1"
                        prop:max=move || load_input_max.get()
                        prop:step=move || load_input_step.get()
                        prop:value=move || format_input_weight(display_load.get())
                        on:change=move |ev| {
                            if let Ok(value) = event_target_value(&ev).parse::<f32>()
                                && value.is_finite() && value > 0.0
                            {
                                set_load_kg.set(display_to_kg(value, use_lbs.get()));
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

            <div class="preset-row one-rm-presets">
                <span class="one-rm-presets-label">"Quick reps"</span>
                <For
                    each=move || QUICK_REP_PRESETS.to_vec()
                    key=|rep| *rep
                    let:rep_preset
                >
                    <button
                        type="button"
                        class="chip"
                        class:active=move || reps.get() == rep_preset
                        on:click=move |_| set_reps.set(rep_preset)
                    >
                        {format!("{} reps", rep_preset)}
                    </button>
                </For>
            </div>

            <div class="one-rm-summary">
                <p class="one-rm-eyebrow">"Estimated 1RM"</p>
                <p class="one-rm-primary">
                    {move || format!(
                        "{:.1} {}",
                        kg_to_display(estimate_kg.get(), use_lbs.get()),
                        unit.get()
                    )}
                </p>
                <p class="one-rm-secondary">
                    {move || {
                        let (low, high) = formula_range_kg.get();
                        format!(
                            "Common-formula range: {:.1}-{:.1} {}",
                            kg_to_display(low, use_lbs.get()),
                            kg_to_display(high, use_lbs.get()),
                            unit.get()
                        )
                    }}
                </p>
                <p class="muted one-rm-note">
                    {move || format!(
                        "Your set is about {:.0}% of this estimate. {}",
                        set_intensity_pct.get(),
                        estimate_guidance(reps.get())
                    )}
                </p>

                <div class="one-rm-stats">
                    <div class="one-rm-stat">
                        <span>"Working set"</span>
                        <strong>{move || format!(
                            "{:.1} {} x {}",
                            display_load.get(),
                            unit.get(),
                            reps.get()
                        )}</strong>
                    </div>
                    <div class="one-rm-stat">
                        <span>"Set intensity"</span>
                        <strong>{move || format!("{:.0}%", set_intensity_pct.get())}</strong>
                    </div>
                    <div class="one-rm-stat">
                        <span>"80% work sets"</span>
                        <strong>{move || format!(
                            "{:.1} {}",
                            kg_to_display(estimate_kg.get() * 0.8, use_lbs.get()),
                            unit.get()
                        )}</strong>
                    </div>
                    <div class="one-rm-stat">
                        <span>"90% heavy work"</span>
                        <strong>{move || format!(
                            "{:.1} {}",
                            kg_to_display(estimate_kg.get() * 0.9, use_lbs.get()),
                            unit.get()
                        )}</strong>
                    </div>
                </div>
            </div>

            <div class="one-rm-tools">
                <div class="one-rm-card">
                    <h3>"Rep Max Targets"</h3>
                    <p class="muted">
                        "Approximate weights for common rep targets, based on your estimated max."
                    </p>
                    <div class="one-rm-table">
                        <For
                            each=move || REP_MAX_TARGETS.to_vec()
                            key=|row| row.reps
                            let:row
                        >
                            <div class="one-rm-table-row" class:active=move || reps.get() == row.reps>
                                <span class="one-rm-table-main">{format!("{} reps", row.reps)}</span>
                                <span class="one-rm-table-note">{row.label}</span>
                                <strong class="one-rm-table-value">
                                    {move || format!(
                                        "{:.1} {}",
                                        kg_to_display(
                                            working_weight_for_reps(estimate_kg.get(), row.reps),
                                            use_lbs.get()
                                        ),
                                        unit.get()
                                    )}
                                </strong>
                            </div>
                        </For>
                    </div>
                </div>

                <div class="one-rm-card">
                    <h3>"Training Percentages"</h3>
                    <p class="muted">
                        "Useful checkpoints if you want to turn the estimate into warm-ups or work sets."
                    </p>
                    <div class="one-rm-table">
                        <For
                            each=move || TRAINING_PERCENTAGES.to_vec()
                            key=|row| row.percent
                            let:row
                        >
                            <div class="one-rm-table-row">
                                <span class="one-rm-table-main">{format!("{}%", row.percent)}</span>
                                <span class="one-rm-table-note">{row.label}</span>
                                <strong class="one-rm-table-value">
                                    {move || format!(
                                        "{:.1} {}",
                                        kg_to_display(
                                            estimate_kg.get() * (row.percent as f32 / 100.0),
                                            use_lbs.get()
                                        ),
                                        unit.get()
                                    )}
                                </strong>
                            </div>
                        </For>
                    </div>
                </div>
            </div>

            <details class="advanced one-rm-details">
                <summary>"See formula details"</summary>
                <p class="muted">
                    "Common calculators drift a bit as reps go up. This estimate uses a lower-rep formula under 8 reps, a higher-rep formula above 10 reps, and blends them in between so you get one clean number."
                </p>
                <p class="muted">
                    {move || format!(
                        "Brzycki: {:.1} {}. Epley: {:.1} {}. Displayed estimate: {:.1} {}.",
                        kg_to_display(brzycki_kg.get(), use_lbs.get()),
                        unit.get(),
                        kg_to_display(epley_kg.get(), use_lbs.get()),
                        unit.get(),
                        kg_to_display(estimate_kg.get(), use_lbs.get()),
                        unit.get()
                    )}
                </p>
                <p class="muted">
                    "Best rule of thumb: hard sets of 1-10 reps are useful for planning, while higher-rep sets are better treated as rough estimates."
                </p>
            </details>
        </section>
    }
}
