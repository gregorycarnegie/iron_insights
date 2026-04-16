use super::shared::Corners;
use crate::webapp::helpers::{bodyfat_category, calc_bodyfat_female, calc_bodyfat_male};
use crate::webapp::ui::parse_f32_input;
use leptos::prelude::*;

const MALE_BF_ROWS: [(&str, &str, &str); 6] = [
    ("Essential", "2 - 5%", "#6b7380"),
    ("Elite Athlete", "6 - 10%", "#c79a4a"),
    ("Athlete", "11 - 14%", "#e8b13a"),
    ("Fitness", "15 - 19%", "#8fb04a"),
    ("Average", "20 - 24%", "#e8472b"),
    ("Obese", "25%+", "#b5321d"),
];

const FEMALE_BF_ROWS: [(&str, &str, &str); 6] = [
    ("Essential", "10 - 13%", "#6b7380"),
    ("Elite Athlete", "14 - 17%", "#c79a4a"),
    ("Athlete", "18 - 21%", "#e8b13a"),
    ("Fitness", "22 - 25%", "#8fb04a"),
    ("Average", "26 - 31%", "#e8472b"),
    ("Obese", "32%+", "#b5321d"),
];

fn category_color(category: &str) -> &'static str {
    match category {
        "Essential" => "#6b7380",
        "Elite Athlete" => "#c79a4a",
        "Athlete" => "#e8b13a",
        "Fitness" => "#8fb04a",
        "Average" => "#e8472b",
        _ => "#b5321d",
    }
}

#[component]
pub fn BodyfatPage() -> impl IntoView {
    let (is_male, set_is_male) = signal(true);
    let (height_cm, set_height_cm) = signal(180.0f32);
    let (weight_kg, set_weight_kg) = signal(85.0f32);
    let (neck_cm, set_neck_cm) = signal(40.0f32);
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
    let gauge_offset = Memo::new(move |_| {
        bf_pct.get().map(|p| {
            let t = (p / 40.0).clamp(0.0, 1.0);
            386.2 - (386.2 - 96.5) * t
        })
    });
    let gauge_color =
        Memo::new(move |_| category.get().map(category_color).unwrap_or("var(--iron)"));

    view! {
        <section class="page active" id="page-bodyfat">
            <div class="page-head">
                <h1 class="page-title">
                    "Body " <span class="accent">"composition"</span> "."
                </h1>
                <p class="page-lede">
                    <span class="serif">"US Navy tape method"</span>
                    " - calibrated for powerlifters carrying more muscle than the average test subject."
                </p>
            </div>

            <div class="bf-grid">
                <div class="panel">
                    <Corners />
                    <div class="panel-head">
                        <span><span class="tag">"IN"</span>" MEASUREMENTS"</span>
                        <span>"CENTIMETRES"</span>
                    </div>
                    <div class="panel-body bf-inputs">
                        <div>
                            <label>"Sex"</label>
                            <div class="toggle-group">
                                <button
                                    class:on=move || is_male.get()
                                    on:click=move |_| set_is_male.set(true)
                                >
                                    "Male"
                                </button>
                                <button
                                    class:on=move || !is_male.get()
                                    on:click=move |_| set_is_male.set(false)
                                >
                                    "Female"
                                </button>
                            </div>
                        </div>

                        <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px">
                            <div>
                                <label>"Height"</label>
                                <input
                                    type="number"
                                    step="0.5"
                                    min="100"
                                    max="250"
                                    prop:value=move || height_cm.get()
                                    on:input=move |ev| {
                                        let v = parse_f32_input(&ev);
                                        if v >= 100.0 && v <= 250.0 {
                                            set_height_cm.set(v);
                                        }
                                    }
                                />
                            </div>
                            <div>
                                <label>"Weight - kg"</label>
                                <input
                                    type="number"
                                    step="0.5"
                                    min="30"
                                    max="300"
                                    prop:value=move || weight_kg.get()
                                    on:input=move |ev| {
                                        let v = parse_f32_input(&ev);
                                        if v >= 30.0 && v <= 300.0 {
                                            set_weight_kg.set(v);
                                        }
                                    }
                                />
                            </div>
                        </div>

                        <div>
                            <label>"Neck (cm)"</label>
                            <input
                                type="number"
                                step="0.5"
                                min="20"
                                max="80"
                                prop:value=move || neck_cm.get()
                                on:input=move |ev| {
                                    let v = parse_f32_input(&ev);
                                    if v >= 20.0 && v <= 80.0 {
                                        set_neck_cm.set(v);
                                    }
                                }
                            />
                        </div>

                        <div>
                            <label>"Waist (cm - navel)"</label>
                            <input
                                type="number"
                                step="0.5"
                                min="40"
                                max="200"
                                prop:value=move || waist_cm.get()
                                on:input=move |ev| {
                                    let v = parse_f32_input(&ev);
                                    if v >= 40.0 && v <= 200.0 {
                                        set_waist_cm.set(v);
                                    }
                                }
                            />
                        </div>

                        {move || {
                            if !is_male.get() {
                                view! {
                                    <div>
                                        <label>"Hips (cm)"</label>
                                        <input
                                            type="number"
                                            step="0.5"
                                            min="40"
                                            max="200"
                                            prop:value=move || hip_cm.get()
                                            on:input=move |ev| {
                                                let v = parse_f32_input(&ev);
                                                if v >= 40.0 && v <= 200.0 {
                                                    set_hip_cm.set(v);
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div style="display:none"></div> }.into_any()
                            }
                        }}
                    </div>
                </div>

                <div>
                    <div class="bf-display">
                        <div class="bf-gauge">
                            <svg viewBox="0 0 200 200">
                                <circle
                                    class="track"
                                    cx="100"
                                    cy="100"
                                    r="82"
                                    stroke-dasharray="386.2"
                                    stroke-dashoffset="128.7"
                                />
                                <circle
                                    class="fill"
                                    cx="100"
                                    cy="100"
                                    r="82"
                                    stroke-dasharray="386.2"
                                    stroke-dashoffset=move || {
                                        gauge_offset
                                            .get()
                                            .map(|offset| format!("{offset:.1}"))
                                            .unwrap_or_else(|| "386.2".to_string())
                                    }
                                    style=move || format!("stroke: {}", gauge_color.get())
                                />
                            </svg>
                            <div class="center">
                                {move || match bf_pct.get() {
                                    Some(pct) => view! {
                                        <div class="pct">
                                            <span>{format!("{pct:.1}")}</span>
                                            <span class="sign">"%"</span>
                                        </div>
                                    }.into_any(),
                                    None => view! {
                                        <div class="pct">
                                            <span>"--"</span>
                                            <span class="sign">"%"</span>
                                        </div>
                                    }.into_any(),
                                }}
                                <div class="cat">
                                    {move || {
                                        category
                                            .get()
                                            .unwrap_or("Enter Measurements")
                                            .to_ascii_uppercase()
                                    }}
                                </div>
                            </div>
                        </div>

                        {move || match result.get() {
                            Some(r) => view! {
                                <div class="bf-mass-grid">
                                    <div class="mini-stat">
                                        <div class="l">"LEAN MASS"</div>
                                        <div class="v">
                                            {format!("{:.1}", r.lean_mass_kg)}
                                            <span class="unit">"KG"</span>
                                        </div>
                                    </div>
                                    <div class="mini-stat">
                                        <div class="l">"FAT MASS"</div>
                                        <div class="v">
                                            {format!("{:.1}", r.fat_mass_kg)}
                                            <span class="unit">"KG"</span>
                                        </div>
                                    </div>
                                </div>
                            }.into_any(),
                            None => view! {
                                <div class="notice">"Enter measurements to compute."</div>
                            }.into_any(),
                        }}
                    </div>

                    <div class="bf-categories">
                        {move || {
                            let rows = if is_male.get() {
                                &MALE_BF_ROWS
                            } else {
                                &FEMALE_BF_ROWS
                            };
                            let current_cat = category.get().unwrap_or("");
                            rows.iter()
                                .map(|(cat, range, color)| {
                                    let is_current = *cat == current_cat;
                                    let class_name = if is_current {
                                        "bf-cat active"
                                    } else {
                                        "bf-cat"
                                    };

                                    view! {
                                        <div class=class_name style=format!("--dc:{color}")>
                                            <div class="dot"></div>
                                            <div class="nm">{cat.to_ascii_uppercase()}</div>
                                            <div class="rn">{*range}</div>
                                        </div>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>
            </div>
        </section>
    }
}
