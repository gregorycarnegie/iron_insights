use super::shared::Corners;
use crate::webapp::{
    helpers::{
        JacksonPollock7SiteSkinfolds, bodyfat_category, calc_bodyfat_female, calc_bodyfat_jp3,
        calc_bodyfat_jp7, calc_bodyfat_male, calc_bodyfat_ymca,
    },
    ui::parse_f32_input,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BodyfatMethod {
    Navy,
    Ymca,
    Skinfold3,
    Skinfold7,
}

impl BodyfatMethod {
    fn label(self) -> &'static str {
        match self {
            BodyfatMethod::Navy => "Navy",
            BodyfatMethod::Ymca => "YMCA",
            BodyfatMethod::Skinfold3 => "Skinfold 3",
            BodyfatMethod::Skinfold7 => "Skinfold 7",
        }
    }

    fn description(self) -> &'static str {
        match self {
            BodyfatMethod::Navy => {
                "Tape estimate using height plus neck and waist, with hip added for women."
            }
            BodyfatMethod::Ymca => {
                "Scale-and-waist estimate; quick, but less useful when muscle mass is high."
            }
            BodyfatMethod::Skinfold3 => {
                "Jackson-Pollock 3-site calipers - chest/abdomen/thigh for men, tricep/suprailiac/thigh for women."
            }
            BodyfatMethod::Skinfold7 => {
                "Jackson-Pollock 7-site calipers - more sites means more tester-skill sensitivity, but typically tighter estimates."
            }
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            BodyfatMethod::Navy => "US Navy tape method",
            BodyfatMethod::Ymca => "YMCA (Wallace-Ross) method",
            BodyfatMethod::Skinfold3 => "Jackson-Pollock 3-site",
            BodyfatMethod::Skinfold7 => "Jackson-Pollock 7-site",
        }
    }
}

const METHODS: [BodyfatMethod; 4] = [
    BodyfatMethod::Navy,
    BodyfatMethod::Ymca,
    BodyfatMethod::Skinfold3,
    BodyfatMethod::Skinfold7,
];

const AGE_CONTEXT_ROWS: [(&str, &str, &str); 3] = [
    ("20-39", "8-20%", "21-33%"),
    ("40-59", "11-22%", "23-34%"),
    ("60+", "13-25%", "24-36%"),
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
    let (method, set_method) = signal(BodyfatMethod::Navy);
    let (is_male, set_is_male) = signal(true);
    let (height_cm, set_height_cm) = signal(180.0f32);
    let (weight_kg, set_weight_kg) = signal(85.0f32);
    let (neck_cm, set_neck_cm) = signal(40.0f32);
    let (waist_cm, set_waist_cm) = signal(85.0f32);
    let (hip_cm, set_hip_cm) = signal(95.0f32);
    let (age, set_age) = signal(30.0f32);
    let (chest_mm, set_chest_mm) = signal(15.0f32);
    let (abdomen_mm, set_abdomen_mm) = signal(20.0f32);
    let (thigh_mm, set_thigh_mm) = signal(20.0f32);
    let (tricep_mm, set_tricep_mm) = signal(15.0f32);
    let (suprailiac_mm, set_suprailiac_mm) = signal(15.0f32);
    let (subscapular_mm, set_subscapular_mm) = signal(15.0f32);
    let (midaxillary_mm, set_midaxillary_mm) = signal(12.0f32);

    let result = Memo::new(move |_| match method.get() {
        BodyfatMethod::Navy => {
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
        }
        BodyfatMethod::Ymca => calc_bodyfat_ymca(weight_kg.get(), waist_cm.get(), is_male.get()),
        BodyfatMethod::Skinfold3 => {
            let male = is_male.get();
            let (a, b, c) = if male {
                (chest_mm.get(), abdomen_mm.get(), thigh_mm.get())
            } else {
                (tricep_mm.get(), suprailiac_mm.get(), thigh_mm.get())
            };
            calc_bodyfat_jp3(age.get(), weight_kg.get(), male, a, b, c)
        }
        BodyfatMethod::Skinfold7 => calc_bodyfat_jp7(
            age.get(),
            weight_kg.get(),
            is_male.get(),
            JacksonPollock7SiteSkinfolds {
                chest_mm: chest_mm.get(),
                midaxillary_mm: midaxillary_mm.get(),
                tricep_mm: tricep_mm.get(),
                subscapular_mm: subscapular_mm.get(),
                abdomen_mm: abdomen_mm.get(),
                suprailiac_mm: suprailiac_mm.get(),
                thigh_mm: thigh_mm.get(),
            },
        ),
    });

    let bf_pct = Memo::new(move |_| result.get().map(|r| r.body_fat_pct));
    let category = Memo::new(move |_| bf_pct.get().map(|p| bodyfat_category(p, is_male.get())));
    let gauge_offset = Memo::new(move |_| {
        bf_pct.get().map(|p| {
            let t = (p / 40.0).clamp(0.0, 1.0);
            386.2 - (386.2 - 96.5) * t
        })
    });
    let gauge_color = Memo::new(move |_| category.get().map_or("var(--iron)", category_color));

    let panel_units_label = move || match method.get() {
        BodyfatMethod::Navy | BodyfatMethod::Ymca => "CENTIMETRES",
        BodyfatMethod::Skinfold3 | BodyfatMethod::Skinfold7 => "CM + MM",
    };

    view! {
        <section class="page active" id="page-bodyfat">
            <div class="page-head">
                <h1 class="page-title">
                    "Body " <span class="accent">"composition"</span> "."
                </h1>
                <p class="page-lede">
                    <span class="serif">{move || method.get().subtitle()}</span>
                    " - calibrated for powerlifters carrying more muscle than the average test subject."
                </p>
            </div>

            <div class="bf-grid">
                <div>
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"IN"</span>" MEASUREMENTS"</span>
                            <span>{panel_units_label}</span>
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

                            <div>
                                <label>"Method"</label>
                                <div class="toggle-group bf-method-toggle">
                                    {METHODS.iter().map(|m| {
                                        let m = *m;
                                        view! {
                                            <button
                                                class:on=move || method.get() == m
                                                on:click=move |_| set_method.set(m)
                                            >
                                                {m.label()}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>

                            <NumberInput
                                id="bodyfat-weight" label="Weight - kg"
                                value=weight_kg on_change=set_weight_kg min=30.0 max=300.0
                            />

                            {move || match method.get() {
                                BodyfatMethod::Navy => view! {
                                    <>
                                        <NumberInput
                                            id="bodyfat-height" label="Height (cm)"
                                            value=height_cm on_change=set_height_cm min=100.0 max=250.0
                                        />
                                        <NumberInput
                                            id="bodyfat-neck" label="Neck (cm)"
                                            value=neck_cm on_change=set_neck_cm min=20.0 max=80.0
                                        />
                                        <NumberInput
                                            id="bodyfat-waist" label="Waist (cm - navel)"
                                            value=waist_cm on_change=set_waist_cm min=40.0 max=200.0
                                        />
                                        {move || {
                                            if is_male.get() {
                                                view! { <div class="visually-hidden">"Hip measurement is not used for the male Navy formula."</div> }.into_any()
                                            } else {
                                                view! {
                                                    <NumberInput
                                                        id="bodyfat-hips" label="Hips (cm)"
                                                        value=hip_cm on_change=set_hip_cm min=40.0 max=200.0
                                                    />
                                                }.into_any()
                                            }
                                        }}
                                    </>
                                }.into_any(),
                                BodyfatMethod::Ymca => view! {
                                    <NumberInput
                                        id="bodyfat-waist" label="Waist (cm - navel)"
                                        value=waist_cm on_change=set_waist_cm min=40.0 max=200.0
                                    />
                                }.into_any(),
                                BodyfatMethod::Skinfold3 => view! {
                                    <>
                                        <NumberInput
                                            id="bodyfat-age" label="Age (years)"
                                            value=age on_change=set_age min=10.0 max=100.0 step=1.0
                                        />
                                        {move || {
                                            if is_male.get() {
                                                view! {
                                                    <>
                                                        <SkinfoldInput
                                                            id="bodyfat-chest"
                                                            label="Chest (mm)"
                                                            value=chest_mm
                                                            on_change=set_chest_mm
                                                        />
                                                        <SkinfoldInput
                                                            id="bodyfat-abdomen"
                                                            label="Abdomen (mm)"
                                                            value=abdomen_mm
                                                            on_change=set_abdomen_mm
                                                        />
                                                        <SkinfoldInput
                                                            id="bodyfat-thigh"
                                                            label="Thigh (mm)"
                                                            value=thigh_mm
                                                            on_change=set_thigh_mm
                                                        />
                                                    </>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <>
                                                        <SkinfoldInput
                                                            id="bodyfat-tricep"
                                                            label="Tricep (mm)"
                                                            value=tricep_mm
                                                            on_change=set_tricep_mm
                                                        />
                                                        <SkinfoldInput
                                                            id="bodyfat-suprailiac"
                                                            label="Suprailiac (mm)"
                                                            value=suprailiac_mm
                                                            on_change=set_suprailiac_mm
                                                        />
                                                        <SkinfoldInput
                                                            id="bodyfat-thigh"
                                                            label="Thigh (mm)"
                                                            value=thigh_mm
                                                            on_change=set_thigh_mm
                                                        />
                                                    </>
                                                }.into_any()
                                            }
                                        }}
                                    </>
                                }.into_any(),
                                BodyfatMethod::Skinfold7 => view! {
                                    <>
                                        <NumberInput
                                            id="bodyfat-age" label="Age (years)"
                                            value=age on_change=set_age min=10.0 max=100.0 step=1.0
                                        />
                                        <div class="bf-two-col">
                                            <SkinfoldInput
                                                id="bodyfat-chest"
                                                label="Chest (mm)"
                                                value=chest_mm
                                                on_change=set_chest_mm
                                            />
                                            <SkinfoldInput
                                                id="bodyfat-midaxillary"
                                                label="Midaxillary (mm)"
                                                value=midaxillary_mm
                                                on_change=set_midaxillary_mm
                                            />
                                            <SkinfoldInput
                                                id="bodyfat-tricep"
                                                label="Tricep (mm)"
                                                value=tricep_mm
                                                on_change=set_tricep_mm
                                            />
                                            <SkinfoldInput
                                                id="bodyfat-subscapular"
                                                label="Subscapular (mm)"
                                                value=subscapular_mm
                                                on_change=set_subscapular_mm
                                            />
                                            <SkinfoldInput
                                                id="bodyfat-abdomen"
                                                label="Abdomen (mm)"
                                                value=abdomen_mm
                                                on_change=set_abdomen_mm
                                            />
                                            <SkinfoldInput
                                                id="bodyfat-suprailiac"
                                                label="Suprailiac (mm)"
                                                value=suprailiac_mm
                                                on_change=set_suprailiac_mm
                                            />
                                            <SkinfoldInput
                                                id="bodyfat-thigh"
                                                label="Thigh (mm)"
                                                value=thigh_mm
                                                on_change=set_thigh_mm
                                            />
                                        </div>
                                    </>
                                }.into_any(),
                            }}
                        </div>
                    </div>

                    <div class="panel bf-context-panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"?"</span>" METHODS"</span>
                            <span>"CONTEXT"</span>
                        </div>
                        <div class="panel-body">
                            <div class="bf-method-list">
                                {METHODS.iter().map(|m| {
                                    let m = *m;
                                    view! {
                                        <div
                                            class="bf-method"
                                            class:active=move || method.get() == m
                                        >
                                            <div class="nm">{m.label()}</div>
                                            <div class="tx">{m.description()}</div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                            <p class="bf-note">
                                "Tape estimates are best used as a trend. Measure at the same time of day, keep tape tension consistent, and expect normal error from hydration, posture, and site placement."
                            </p>
                        </div>
                    </div>
                </div>

                <div>
                    <div class="bf-display">
                        <p class="chart-summary">
                            {move || match result.get() {
                                Some(r) => format!(
                                    "Your estimate is {:.1}% body fat, with {:.1}kg lean mass and {:.1}kg fat mass.",
                                    r.body_fat_pct,
                                    r.lean_mass_kg,
                                    r.fat_mass_kg,
                                ),
                                None => "Enter valid measurements to estimate body fat, lean mass, and fat mass.".to_string(),
                            }}
                        </p>
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
                                            .get().map_or_else(|| "386.2".to_string(), |offset| format!("{offset:.1}"))
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

                    <div class="panel bf-reference-panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"REF"</span>" AGE CONTEXT"</span>
                            <span>"BROAD RANGES"</span>
                        </div>
                        <div class="panel-body">
                            <p class="chart-summary">
                                "These ranges are broad training context, not a diagnosis. Strength athletes can sit outside them while performing well."
                            </p>
                            <table class="bf-ref-table">
                                <thead>
                                    <tr>
                                        <th>"Age"</th>
                                        <th>"Men"</th>
                                        <th>"Women"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {AGE_CONTEXT_ROWS.iter().map(|(age, men, women)| {
                                        view! {
                                            <tr>
                                                <th scope="row">{*age}</th>
                                                <td>{*men}</td>
                                                <td>{*women}</td>
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

/// A labelled numeric measurement field. Out-of-range input is ignored, leaving
/// the last valid value in the signal (the browser keeps what was typed).
#[component]
fn NumberInput(
    id: &'static str,
    label: &'static str,
    value: ReadSignal<f32>,
    on_change: WriteSignal<f32>,
    min: f32,
    max: f32,
    #[prop(default = 0.5)] step: f32,
) -> impl IntoView {
    view! {
        <div>
            <label for=id>{label}</label>
            <input
                id=id
                type="number"
                step=step
                min=min
                max=max
                prop:value=move || value.get()
                on:input=move |ev| {
                    let v = parse_f32_input(&ev);
                    if (min..=max).contains(&v) {
                        on_change.set(v);
                    }
                }
            />
        </div>
    }
}

/// Skinfold calipers all share the same plausible range.
#[component]
fn SkinfoldInput(
    id: &'static str,
    label: &'static str,
    value: ReadSignal<f32>,
    on_change: WriteSignal<f32>,
) -> impl IntoView {
    view! { <NumberInput id=id label=label value=value on_change=on_change min=2.0 max=60.0 /> }
}
