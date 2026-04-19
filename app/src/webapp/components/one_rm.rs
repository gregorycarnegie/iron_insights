use super::shared::Corners;
use crate::core::calc_1rm;
use crate::webapp::ui::parse_f32_input;
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

#[component]
pub fn OneRmPage() -> impl IntoView {
    let (weight, set_weight) = signal(140.0f32);
    let (reps, set_reps) = signal(5.0f32);
    let (formula, set_formula) = signal("epley".to_string());

    let rm = Memo::new(move |_| calc_1rm(weight.get(), reps.get(), &formula.get()));
    let formula_label = Memo::new(move |_| match formula.get().as_str() {
        "brzycki" => "w / (1.0278 − 0.0278r) · BRZYCKI",
        "lander" => "100w / (101.3 − 2.67r) · LANDER",
        "lombardi" => "w × r^0.1 · LOMBARDI",
        "oconner" => "w × (1 + r/40) · O'CONNER",
        _ => "w × (1 + r/30) · EPLEY",
    });

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
                            <label>"Weight Lifted"</label>
                            <div class="lift-row">
                                <input
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
                            <label>"Reps Completed"</label>
                            <input
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
                            <label>"Formula"</label>
                            <select
                                on:change=move |ev| set_formula.set(event_target_value(&ev))
                                prop:value=move || formula.get()
                            >
                                <option value="epley" prop:selected=move || formula.get() == "epley">"Epley (default)"</option>
                                <option value="brzycki" prop:selected=move || formula.get() == "brzycki">"Brzycki"</option>
                                <option value="lander" prop:selected=move || formula.get() == "lander">"Lander"</option>
                                <option value="lombardi" prop:selected=move || formula.get() == "lombardi">"Lombardi"</option>
                                <option value="oconner" prop:selected=move || formula.get() == "oconner">"O'Conner"</option>
                            </select>
                        </div>
                    </div>
                </div>

                // Result column
                <div>
                    <div class="rm-display">
                        <div class="content">
                            <div style="font-size:10px;letter-spacing:0.3em;color:var(--ink-dim)">"ESTIMATED ONE REP MAX"</div>
                            <div class="rm-value">{move || format!("{:.0}", rm.get())}</div>
                            <div class="rm-unit">"KG"</div>
                            <div class="rm-formula">"≈ " {move || formula_label.get()}</div>
                        </div>
                    </div>

                    <div class="panel" style="margin-top:24px">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"%"</span>" PRESCRIBED PERCENTAGES"</span>
                            <span>"TRAINING TABLE"</span>
                        </div>
                        <div class="panel-body" style="padding:0">
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
