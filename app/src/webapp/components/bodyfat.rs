use super::shared::Corners;
use crate::webapp::helpers::{bodyfat_category, calc_bodyfat_female, calc_bodyfat_male};
use crate::webapp::ui::parse_f32_input;
use leptos::prelude::*;

#[component]
pub fn BodyfatPage() -> impl IntoView {
    let (is_male, set_is_male) = signal(true);
    let (height_cm, set_height_cm) = signal(175.0f32);
    let (weight_kg, set_weight_kg) = signal(80.0f32);
    let (neck_cm, set_neck_cm) = signal(38.0f32);
    let (waist_cm, set_waist_cm) = signal(85.0f32);
    let (hip_cm, set_hip_cm) = signal(95.0f32);

    let result = Memo::new(move |_| {
        if is_male.get() {
            calc_bodyfat_male(
                height_cm.get(),
                weight_kg.get(),
                neck_cm.get(),
                waist_cm.get(),
            )
        } else {
            calc_bodyfat_female(
                height_cm.get(),
                weight_kg.get(),
                neck_cm.get(),
                waist_cm.get(),
                hip_cm.get(),
            )
        }
    });

    let bf_pct = Memo::new(move |_| result.get().map(|r| r.body_fat_pct));
    let category = Memo::new(move |_| bf_pct.get().map(|p| bodyfat_category(p, is_male.get())));

    // SVG gauge arc: 0% → 50% maps to -135deg → +135deg
    let gauge_rotation = Memo::new(move |_| {
        bf_pct.get().map(|p| {
            let clamped = p.clamp(3.0, 45.0);
            let t = (clamped - 3.0) / (45.0 - 3.0);
            -135.0 + t * 270.0
        })
    });

    view! {
        <section class="page active" id="page-bodyfat">
            <div class="page-head">
                <h1 class="page-title">
                    "Body " <span class="accent">"composition."</span>
                </h1>
                <p class="page-lede">
                    <span class="serif">"US Navy tape method."</span>
                    " Circumference measurements give an accurate body fat estimate without calipers or DEXA."
                </p>
            </div>

            <div class="bf-grid">
                // Input panel
                <div class="panel">
                    <Corners />
                    <div class="panel-head">
                        <span><span class="tag">"IN"</span>" MEASUREMENTS"</span>
                        <span>"TAPE METHOD"</span>
                    </div>
                    <div class="panel-body input-stack">
                        // Sex
                        <div>
                            <label>"Sex"</label>
                            <div class="toggle-group">
                                <button
                                    class:on=move || is_male.get()
                                    on:click=move |_| set_is_male.set(true)
                                >"Male"</button>
                                <button
                                    class:on=move || !is_male.get()
                                    on:click=move |_| set_is_male.set(false)
                                >"Female"</button>
                            </div>
                        </div>

                        <div>
                            <label>"Height"</label>
                            <div class="lift-row">
                                <input
                                    type="number" step="0.5" min="100" max="250"
                                    prop:value=move || height_cm.get()
                                    on:input=move |ev| {
                                        let v = parse_f32_input(&ev);
                                        if v >= 100.0 && v <= 250.0 { set_height_cm.set(v); }
                                    }
                                />
                                <div class="hint">"CM"</div>
                            </div>
                        </div>

                        <div>
                            <label>"Body Weight"</label>
                            <div class="lift-row">
                                <input
                                    type="number" step="0.5" min="30" max="300"
                                    prop:value=move || weight_kg.get()
                                    on:input=move |ev| {
                                        let v = parse_f32_input(&ev);
                                        if v >= 30.0 && v <= 300.0 { set_weight_kg.set(v); }
                                    }
                                />
                                <div class="hint">"KG"</div>
                            </div>
                        </div>

                        <div>
                            <label>"Neck Circumference"</label>
                            <div class="lift-row">
                                <input
                                    type="number" step="0.5" min="20" max="80"
                                    prop:value=move || neck_cm.get()
                                    on:input=move |ev| {
                                        let v = parse_f32_input(&ev);
                                        if v >= 20.0 && v <= 80.0 { set_neck_cm.set(v); }
                                    }
                                />
                                <div class="hint">"CM"</div>
                            </div>
                        </div>

                        <div>
                            <label>"Waist Circumference"</label>
                            <div class="lift-row">
                                <input
                                    type="number" step="0.5" min="40" max="200"
                                    prop:value=move || waist_cm.get()
                                    on:input=move |ev| {
                                        let v = parse_f32_input(&ev);
                                        if v >= 40.0 && v <= 200.0 { set_waist_cm.set(v); }
                                    }
                                />
                                <div class="hint">"CM"</div>
                            </div>
                        </div>

                        // Hip: females only
                        {move || if !is_male.get() {
                            view! {
                                <div>
                                    <label>"Hip Circumference"</label>
                                    <div class="lift-row">
                                        <input
                                            type="number" step="0.5" min="40" max="200"
                                            prop:value=move || hip_cm.get()
                                            on:input=move |ev| {
                                                let v = parse_f32_input(&ev);
                                                if v >= 40.0 && v <= 200.0 { set_hip_cm.set(v); }
                                            }
                                        />
                                        <div class="hint">"CM"</div>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                    </div>
                </div>

                // Results column
                <div>
                    // Gauge display
                    <div class="rm-display" style="margin-bottom:24px">
                        <div class="content">
                            <div style="font-size:10px;letter-spacing:0.3em;color:var(--ink-dim)">"BODY FAT"</div>
                            {move || match bf_pct.get() {
                                Some(pct) => view! {
                                    <div>
                                        <div class="rm-value">{format!("{:.1}", pct)}</div>
                                        <div class="rm-unit">"%"</div>
                                        <div class="rm-formula" style="color:var(--chalk)">
                                            {category.get().unwrap_or("—")}
                                        </div>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="rm-value" style="font-size:32px;color:var(--ink-mute)">"—"</div>
                                }.into_any(),
                            }}
                        </div>
                    </div>

                    // SVG gauge
                    {move || match gauge_rotation.get() {
                        Some(rot) => view! {
                            <div class="bf-gauge-wrap" style="margin-bottom:24px;display:flex;justify-content:center">
                                <svg viewBox="0 0 220 130" width="220" height="130" style="overflow:visible">
                                    // Background arc
                                    <path
                                        d="M 20 110 A 90 90 0 1 1 200 110"
                                        fill="none"
                                        stroke="var(--panel-border)"
                                        stroke-width="12"
                                        stroke-linecap="round"
                                    />
                                    // Colored arc (iron)
                                    <path
                                        d="M 20 110 A 90 90 0 1 1 200 110"
                                        fill="none"
                                        stroke="var(--iron)"
                                        stroke-width="12"
                                        stroke-linecap="round"
                                        stroke-dasharray="283"
                                        stroke-dashoffset=move || {
                                            let t = (rot + 135.0) / 270.0;
                                            format!("{:.1}", 283.0 * (1.0 - t))
                                        }
                                    />
                                    // Needle
                                    <g transform=format!("translate(110,110) rotate({})", rot)>
                                        <line x1="0" y1="0" x2="0" y2="-72"
                                            stroke="var(--chalk)" stroke-width="2" stroke-linecap="round" />
                                        <circle cx="0" cy="0" r="5" fill="var(--chalk)" />
                                    </g>
                                    // Zone labels
                                    <text x="18" y="126" fill="var(--ink-dim)" font-size="9" font-family="JetBrains Mono,monospace">"LOW"</text>
                                    <text x="174" y="126" fill="var(--ink-dim)" font-size="9" font-family="JetBrains Mono,monospace">"HIGH"</text>
                                </svg>
                            </div>
                        }.into_any(),
                        None => view! { <div></div> }.into_any(),
                    }}

                    // Breakdown panel
                    <div class="panel" style="margin-bottom:24px">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"∑"</span>" COMPOSITION BREAKDOWN"</span>
                            <span>"LEAN · FAT"</span>
                        </div>
                        <div class="panel-body">
                            {move || match result.get() {
                                Some(r) => view! {
                                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:24px">
                                        <div>
                                            <div style="font-size:10px;letter-spacing:0.2em;color:var(--ink-dim);margin-bottom:6px">"LEAN MASS"</div>
                                            <div style="font-family:'Archivo Black',sans-serif;font-size:28px;color:var(--chalk)">
                                                {format!("{:.1}", r.lean_mass_kg)}
                                                <span style="font-size:14px;color:var(--ink-dim)">" KG"</span>
                                            </div>
                                        </div>
                                        <div>
                                            <div style="font-size:10px;letter-spacing:0.2em;color:var(--ink-dim);margin-bottom:6px">"FAT MASS"</div>
                                            <div style="font-family:'Archivo Black',sans-serif;font-size:28px;color:var(--iron)">
                                                {format!("{:.1}", r.fat_mass_kg)}
                                                <span style="font-size:14px;color:var(--ink-dim)">" KG"</span>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="notice">"Enter measurements to compute."</div>
                                }.into_any(),
                            }}
                        </div>
                    </div>

                    // Reference table
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"REF"</span>" BODY FAT CATEGORIES"</span>
                            <span>{move || if is_male.get() { "MALE" } else { "FEMALE" }}</span>
                        </div>
                        <div class="panel-body" style="padding:0">
                            <table class="rm-table">
                                <thead>
                                    <tr>
                                        <th>"CATEGORY"</th>
                                        <th>"RANGE"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        let rows: &[(&str, &str, &str)] = if is_male.get() {
                                            &[
                                                ("Essential Fat", "2–5%", ""),
                                                ("Athletic", "6–13%", ""),
                                                ("Fitness", "14–17%", ""),
                                                ("Average", "18–24%", ""),
                                                ("Obese", "25%+", ""),
                                            ]
                                        } else {
                                            &[
                                                ("Essential Fat", "10–13%", ""),
                                                ("Athletic", "14–20%", ""),
                                                ("Fitness", "21–24%", ""),
                                                ("Average", "25–31%", ""),
                                                ("Obese", "32%+", ""),
                                            ]
                                        };
                                        let current_cat = category.get().unwrap_or("—");
                                        rows.iter().map(|(cat, range, _)| {
                                            let is_current = *cat == current_cat;
                                            view! {
                                                <tr>
                                                    <td class="pct" style=if is_current { "color:var(--iron)" } else { "" }>
                                                        {if is_current { format!("→ {}", cat) } else { cat.to_string() }}
                                                    </td>
                                                    <td class="wt">{*range}</td>
                                                </tr>
                                            }
                                        }).collect_view()
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
