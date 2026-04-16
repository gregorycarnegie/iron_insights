use super::shared::Corners;
use crate::webapp::ui::parse_f32_input;
use leptos::prelude::*;

const PLATES_KG: &[f32] = &[50.0, 25.0, 20.0, 15.0, 10.0, 5.0, 2.5, 1.25];
const PLATES_LB: &[f32] = &[100.0, 55.0, 45.0, 35.0, 25.0, 10.0, 5.0, 2.5];

const PLATE_COLORS_KG: &[&str] = &[
    "#e8472b", "#c79a4a", "#3a7fd4", "#4caf50", "#ffffff", "#e8472b", "#3a7fd4", "#c79a4a",
];
const PLATE_COLORS_LB: &[&str] = &[
    "#e8472b", "#e8e3d6", "#3a7fd4", "#4caf50", "#c79a4a", "#3a7fd4", "#e8e3d6", "#c79a4a",
];

#[derive(Clone, PartialEq)]
struct PlateResult {
    plates_per_side: Vec<(f32, usize, &'static str)>,
    loaded_weight: f32,
    remainder: f32,
}

fn calc_plates(target: f32, bar: f32, use_lbs: bool) -> PlateResult {
    let plates = if use_lbs { PLATES_LB } else { PLATES_KG };
    let colors = if use_lbs {
        PLATE_COLORS_LB
    } else {
        PLATE_COLORS_KG
    };
    let mut remaining = ((target - bar) / 2.0).max(0.0);
    let mut result: Vec<(f32, usize, &'static str)> = Vec::new();

    for (i, &plate) in plates.iter().enumerate() {
        if remaining >= plate {
            let count = (remaining / plate) as usize;
            remaining -= count as f32 * plate;
            result.push((plate, count, colors[i]));
        }
    }

    let plates_total: f32 = result.iter().map(|(p, c, _)| p * (*c as f32) * 2.0).sum();
    let loaded = bar + plates_total;
    PlateResult {
        plates_per_side: result,
        loaded_weight: loaded,
        remainder: remaining,
    }
}

#[component]
pub fn PlateCalcPage() -> impl IntoView {
    let (target, set_target) = signal(100.0f32);
    let (bar, set_bar) = signal(20.0f32);
    let (use_lbs, set_use_lbs) = signal(false);

    let result = Memo::new(move |_| calc_plates(target.get(), bar.get(), use_lbs.get()));
    let unit = Memo::new(move |_| if use_lbs.get() { "LB" } else { "KG" });

    view! {
        <section class="page active" id="page-plate">
            <div class="page-head">
                <h1 class="page-title">
                    "Load the " <span class="accent">"bar."</span>
                </h1>
                <p class="page-lede">
                    <span class="serif">"No mental arithmetic."</span>
                    " Enter the target weight — get the exact plates to load each side."
                </p>
            </div>

            <div class="plate-grid">
                // Input panel
                <div class="panel">
                    <Corners />
                    <div class="panel-head">
                        <span><span class="tag">"IN"</span>" TARGET LOAD"</span>
                        <span>"BAR + PLATES"</span>
                    </div>
                    <div class="panel-body input-stack">
                        // Units
                        <div>
                            <label>"Units"</label>
                            <div class="toggle-group">
                                <button
                                    class:on=move || !use_lbs.get()
                                    on:click=move |_| set_use_lbs.set(false)
                                >"Kilograms"</button>
                                <button
                                    class:on=move || use_lbs.get()
                                    on:click=move |_| set_use_lbs.set(true)
                                >"Pounds"</button>
                            </div>
                        </div>

                        // Target weight
                        <div>
                            <label>"Target Weight"</label>
                            <div class="lift-row">
                                <input
                                    type="number"
                                    step="2.5"
                                    min="0"
                                    prop:value=move || target.get()
                                    on:input=move |ev| {
                                        let v = parse_f32_input(&ev);
                                        if v >= 0.0 { set_target.set(v); }
                                    }
                                />
                                <div class="hint">{move || unit.get()}</div>
                            </div>
                        </div>

                        // Bar weight
                        <div>
                            <label>"Bar Weight"</label>
                            <div class="toggle-group">
                                {move || {
                                    let bars: &[(f32, &str)] = if use_lbs.get() {
                                        &[(45.0, "45 lb"), (35.0, "35 lb"), (15.0, "15 lb")]
                                    } else {
                                        &[(20.0, "20 kg"), (15.0, "15 kg"), (10.0, "10 kg")]
                                    };
                                    bars.iter().map(|(b, label)| {
                                        let b_val = *b;
                                        view! {
                                            <button
                                                class:on=move || (bar.get() - b_val).abs() < 0.1
                                                on:click=move |_| set_bar.set(b_val)
                                            >{*label}</button>
                                        }
                                    }).collect_view()
                                }}
                            </div>
                        </div>
                    </div>
                </div>

                // Result column
                <div>
                    // Loaded weight display
                    <div class="rm-display" style="margin-bottom:24px">
                        <div class="content">
                            <div style="font-size:10px;letter-spacing:0.3em;color:var(--ink-dim)">"LOADED WEIGHT"</div>
                            <div class="rm-value">{move || format!("{:.2}", result.get().loaded_weight)}</div>
                            <div class="rm-unit">{move || unit.get()}</div>
                            {move || {
                                let r = result.get().remainder;
                                if r > 0.01 {
                                    view! {
                                        <div style="font-size:11px;color:var(--iron);margin-top:8px">
                                            {format!("Cannot reach target — {:.2} {} short per side", r, unit.get())}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }
                            }}
                        </div>
                    </div>

                    // Bar visualizer
                    <div class="panel" style="margin-bottom:24px">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"VIZ"</span>" BAR VISUALIZER"</span>
                            <span>"EACH SIDE"</span>
                        </div>
                        <div class="panel-body">
                            <div class="plate-bar-wrap">
                                <div class="bar-visual">
                                    // Left sleeve (plates in reverse order for visual)
                                    <div class="sleeve left">
                                        {move || {
                                            let r = result.get();
                                            r.plates_per_side.iter().rev().flat_map(|(plate, count, color)| {
                                                let color = *color;
                                                let plate_label = *plate;
                                                (0..*count).map(move |_| {
                                                    let h = plate_height_px(plate_label);
                                                    view! {
                                                        <div
                                                            class="plate"
                                                            style=format!("height:{h}px;background:{color};")
                                                        >
                                                            <span>{format!("{}", plate_label)}</span>
                                                        </div>
                                                    }
                                                }).collect::<Vec<_>>()
                                            }).collect_view()
                                        }}
                                    </div>
                                    // Bar collar
                                    <div class="bar-center">
                                        <div class="collar"></div>
                                        <div class="shaft"></div>
                                        <div class="collar"></div>
                                    </div>
                                    // Right sleeve
                                    <div class="sleeve right">
                                        {move || {
                                            let r = result.get();
                                            r.plates_per_side.iter().flat_map(|(plate, count, color)| {
                                                let color = *color;
                                                let plate_label = *plate;
                                                (0..*count).map(move |_| {
                                                    let h = plate_height_px(plate_label);
                                                    view! {
                                                        <div
                                                            class="plate"
                                                            style=format!("height:{h}px;background:{color};")
                                                        >
                                                            <span>{format!("{}", plate_label)}</span>
                                                        </div>
                                                    }
                                                }).collect::<Vec<_>>()
                                            }).collect_view()
                                        }}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Plate list
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"LST"</span>" PLATE LIST"</span>
                            <span>"PER SIDE"</span>
                        </div>
                        <div class="panel-body" style="padding:0">
                            {move || {
                                let r = result.get();
                                if r.plates_per_side.is_empty() {
                                    view! {
                                        <div class="notice">"No plates needed — bar weight equals or exceeds target."</div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <table class="rm-table">
                                            <thead>
                                                <tr>
                                                    <th>"PLATE"</th>
                                                    <th>"QTY"</th>
                                                    <th>"SUBTOTAL"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {r.plates_per_side.iter().map(|(plate, count, color)| {
                                                    let subtotal = plate * (*count as f32);
                                                    let color = *color;
                                                    let plate = *plate;
                                                    let count = *count;
                                                    let u = unit.get();
                                                    view! {
                                                        <tr>
                                                            <td class="pct" style=format!("color:{color}")>
                                                                {format!("{} {}", plate, u)}
                                                            </td>
                                                            <td class="wt">{format!("× {}", count)}</td>
                                                            <td class="pct">{format!("{:.2} {}", subtotal, u)}</td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

fn plate_height_px(plate: f32) -> u32 {
    match plate as u32 {
        50 | 100 => 120,
        45 | 55 => 120,
        25 => 100,
        20 => 90,
        15 | 35 => 80,
        10 => 70,
        5 => 55,
        _ => 40,
    }
}
