use super::shared::Corners;
use crate::{core::calc_1rm, webapp::ui::parse_f32_input};
use leptos::prelude::*;

const PERCENTAGES: &[(&str, u32, &str)] = &[
    ("95%", 95, "1-2 reps · Max singles"),
    ("90%", 90, "2-3 reps · Near-max"),
    ("85%", 85, "3-5 reps · Heavy triples"),
    ("80%", 80, "4-6 reps · Strength"),
    ("75%", 75, "6-8 reps · Strength-hypertrophy"),
    ("70%", 70, "8-10 reps · Hypertrophy"),
    ("65%", 65, "10-12 reps · Volume"),
    ("60%", 60, "12-15 reps · Endurance"),
    ("50%", 50, "15+ reps · Conditioning"),
];

#[derive(Clone, Copy)]
struct FormulaOption {
    id: &'static str,
    name: &'static str,
    equation: &'static str,
    note: &'static str,
}

const FORMULAS: [FormulaOption; 4] = [
    FormulaOption {
        id: "epley",
        name: "Epley",
        equation: "w x (1 + r/30)",
        note: "Simple strength-room default.",
    },
    FormulaOption {
        id: "brzycki",
        name: "Brzycki",
        equation: "w / (1.0278 - 0.0278r)",
        note: "Conservative near lower reps.",
    },
    FormulaOption {
        id: "mayhew",
        name: "Mayhew",
        equation: "100w / (52.2 + 41.9e^-0.055r)",
        note: "Curves reps non-linearly.",
    },
    FormulaOption {
        id: "lombardi",
        name: "Lombardi",
        equation: "w x r^0.1",
        note: "Smooth power relationship.",
    },
];

#[component]
pub fn OneRmPage() -> impl IntoView {
    let (weight, set_weight) = signal(140.0f32);
    let (reps, set_reps) = signal(5.0f32);
    let (formula, set_formula) = signal("epley".to_string());

    let rm = Memo::new(move |_| calc_1rm(weight.get(), reps.get(), &formula.get()));
    let formula_label = Memo::new(move |_| match formula.get().as_str() {
        "brzycki" => "w / (1.0278 − 0.0278r) · BRZYCKI",
        "mayhew" => "100w / (52.2 + 41.9e^-0.055r) · MAYHEW",
        "lombardi" => "w × r^0.1 · LOMBARDI",
        _ => "w × (1 + r/30) · EPLEY",
    });
    let send_to_plate_calc = move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("iron_insights_plate_target_kg", &format!("{:.2}", rm.get()));
        }
        let _ = window.location().set_hash("plate-calc");
    };

    view! {
        <section class="page active" id="page-rm">
            <div class="page-head">
                <h1 class="page-title">
                    "One rep. " <span class="accent">"Max."</span>
                </h1>
                <p class="page-lede">
                    <span class="serif">"Don't grind to miss."</span>
                    " Plug in any submaximal set and estimate your true ceiling across five validated formulas."
                </p>
            </div>

            <div class="rm-grid">
                // Input panel
                <div class="panel">
                    <Corners />
                    <div class="panel-head">
                        <span><span class="tag">"IN"</span>" SUBMAXIMAL SET"</span>
                        <span>"WEIGHT × REPS"</span>
                    </div>
                    <div class="panel-body input-stack">
                        <div>
                            <label for="one-rm-weight">"Weight Lifted"</label>
                            <div class="lift-row">
                                <input
                                    id="one-rm-weight"
                                    type="number"
                                    step="2.5"
                                    prop:value=move || weight.get()
                                    on:input=move |ev| {
                                        let v = parse_f32_input(&ev);
                                        if v > 0.0 { set_weight.set(v); }
                                    }
                                />
                                <div class="hint">"KG"</div>
                            </div>
                        </div>
                        <div>
                            <label for="one-rm-reps">"Reps Completed"</label>
                            <input
                                id="one-rm-reps"
                                type="number"
                                min="1"
                                max="30"
                                step="1"
                                prop:value=move || reps.get() as u32
                                on:input=move |ev| {
                                    let v = parse_f32_input(&ev);
                                    if v >= 1.0 { set_reps.set(v); }
                                }
                            />
                        </div>
                        <div>
                            <label for="one-rm-formula">"Formula"</label>
                            <select
                                id="one-rm-formula"
                                on:change=move |ev| set_formula.set(event_target_value(&ev))
                                prop:value=move || formula.get()
                            >
                                {FORMULAS.iter().map(|option| {
                                    let id = option.id;
                                    view! {
                                        <option value=id prop:selected=move || formula.get() == id>
                                            {if id == "epley" { "Epley (default)" } else { option.name }}
                                        </option>
                                    }
                                }).collect_view()}
                            </select>
                        </div>
                    </div>
                </div>

                // Result column
                <div>
                    <div class="rm-display">
                        <div class="content">
                            <p class="chart-summary">
                                {move || {
                                    format!(
                                        "{:.0}kg for {:.0} reps estimates a {:.0}kg one-rep max with the selected formula.",
                                        weight.get(),
                                        reps.get(),
                                        rm.get(),
                                    )
                                }}
                            </p>
                            <div style="font-size:10px;letter-spacing:0.3em;color:var(--ink-dim)">"ESTIMATED ONE REP MAX"</div>
                            <div class="rm-value">{move || format!("{:.0}", rm.get())}</div>
                            <div class="rm-unit">"KG"</div>
                            <div class="rm-formula">"≈ " {move || formula_label.get()}</div>
                            <button
                                type="button"
                                class="btn rm-send-button"
                                on:click=send_to_plate_calc
                            >
                                "LOAD THIS IN PLATE CALC"
                            </button>
                        </div>
                    </div>

                    <div class="panel" style="margin-top:24px">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"ƒ"</span>" FORMULA COMPARISON"</span>
                            <span>"ESTIMATES"</span>
                        </div>
                        <div class="panel-body" style="padding:0">
                            <p class="chart-summary rm-table-summary">
                                "Different equations diverge as reps climb; compare the spread before choosing a training max."
                            </p>
                            <table class="rm-table rm-compare-table">
                                <thead>
                                    <tr>
                                        <th>"Formula"</th>
                                        <th>"Estimated 1RM"</th>
                                        <th>"Equation"</th>
                                        <th>"Use"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {FORMULAS.iter().map(|option| {
                                        let id = option.id;
                                        view! {
                                            <tr class:active=move || formula.get() == id>
                                                <td class="pct">{option.name}</td>
                                                <td class="wt">{move || format!("{:.1} kg", calc_1rm(weight.get(), reps.get(), id))}</td>
                                                <td class="pct">{option.equation}</td>
                                                <td class="pct">{option.note}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    </div>

                    <div class="panel" style="margin-top:24px">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"%"</span>" PRESCRIBED PERCENTAGES"</span>
                            <span>"TRAINING TABLE"</span>
                        </div>
                        <div class="panel-body" style="padding:0">
                            <p class="chart-summary rm-table-summary">
                                {move || {
                                    format!(
                                        "Training weights are scaled from your estimated {:.0}kg max.",
                                        rm.get(),
                                    )
                                }}
                            </p>
                            <table class="rm-table">
                                <thead>
                                    <tr>
                                        <th>"%1RM"</th>
                                        <th>"WEIGHT"</th>
                                        <th>"TYPICAL REPS"</th>
                                        <th>"PURPOSE"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {PERCENTAGES.iter().map(|(pct_label, pct, purpose)| {
                                        let pct_val = *pct as f32 / 100.0;
                                        view! {
                                            <tr>
                                                <td class="pct">{*pct_label}</td>
                                                <td class="wt">{move || format!("{:.1} kg", rm.get() * pct_val)}</td>
                                                <td class="pct">{purpose.split(" · ").next().unwrap_or("")}</td>
                                                <td class="pct">{purpose.split(" · ").last().unwrap_or("")}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
