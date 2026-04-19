use super::shared::Corners;
use crate::core::{KG_PER_LB, plates_per_side};
use crate::webapp::helpers::{display_to_kg, kg_to_display};
use crate::webapp::ui::parse_f32_input;
use leptos::prelude::*;

#[derive(Clone, PartialEq)]
struct PlateResult {
    plates_per_side: Vec<(f32, usize, &'static str)>,
    per_side_kg: f32,
    achieved_kg: f32,
    remainder_kg: f32,
    below_bar: bool,
}

fn calc_plates(
    target_display: f32,
    bar_kg: f32,
    collar_kg_each: f32,
    use_lbs: bool,
) -> PlateResult {
    let target_kg = display_to_kg(target_display, use_lbs);
    let collar_total_kg = collar_kg_each * 2.0;
    let per_side_kg = (target_kg - bar_kg - collar_total_kg) / 2.0;
    let (raw_plates, remainder_kg) = plates_per_side(per_side_kg);
    let plates_per_side: Vec<(f32, usize, &'static str)> = raw_plates
        .into_iter()
        .map(|(plate, count)| (plate, count, plate_color(plate)))
        .collect();
    let plate_total_kg = plates_per_side
        .iter()
        .map(|(plate, count, _)| plate * *count as f32 * 2.0)
        .sum::<f32>();
    PlateResult {
        plates_per_side,
        per_side_kg,
        achieved_kg: bar_kg + collar_total_kg + plate_total_kg,
        remainder_kg,
        below_bar: per_side_kg < 0.0,
    }
}

#[component]
pub fn PlateCalcPage() -> impl IntoView {
    let (target, set_target) = signal(180.0f32);
    let (bar_kg, set_bar_kg) = signal(20.0f32);
    let (collar_kg_each, set_collar_kg_each) = signal(0.0f32);
    let (use_lbs, set_use_lbs) = signal(false);

    let result = Memo::new(move |_| {
        calc_plates(
            target.get(),
            bar_kg.get(),
            collar_kg_each.get(),
            use_lbs.get(),
        )
    });
    let unit = Memo::new(move |_| if use_lbs.get() { "LB" } else { "KG" });

    let set_unit = move |next_use_lbs: bool| {
        let current_use_lbs = use_lbs.get_untracked();
        if current_use_lbs != next_use_lbs {
            let current = target.get_untracked();
            let converted = if next_use_lbs {
                current / KG_PER_LB
            } else {
                current * KG_PER_LB
            };
            set_target.set((converted * 2.0).round() / 2.0);
        }
        set_use_lbs.set(next_use_lbs);
    };

    view! {
        <section class="page active" id="page-plate">
            <div class="page-head">
                <h1 class="page-title">
                    "Load the " <span class="accent">"bar"</span>"."
                </h1>
                <p class="page-lede">
                    <span class="serif">"Enter target weight."</span>
                    " We'll tell you which plates to throw on - per side - optimised to use the fewest steel discs."
                </p>
            </div>

            <div class="plate-grid">
                <div>
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"01"</span>" CONFIG"</span>
                            <span>"IPF STANDARD"</span>
                        </div>
                        <div class="panel-body input-stack">
                            <div>
                                <label>"Target Weight"</label>
                                <div class="lift-row">
                                    <input
                                        type="number"
                                        step=move || if use_lbs.get() { "0.5" } else { "2.5" }
                                        min="0"
                                        prop:value=move || format_display(target.get())
                                        on:input=move |ev| {
                                            let v = parse_f32_input(&ev);
                                            if v >= 0.0 {
                                                set_target.set(v);
                                            }
                                        }
                                    />
                                    <div class="hint">{move || unit.get()}</div>
                                </div>
                            </div>

                            <div>
                                <label>"Units"</label>
                                <div class="toggle-group">
                                    <button
                                        class:on=move || !use_lbs.get()
                                        on:click=move |_| set_unit(false)
                                    >"KG"</button>
                                    <button
                                        class:on=move || use_lbs.get()
                                        on:click=move |_| set_unit(true)
                                    >"LB"</button>
                                </div>
                            </div>

                            <div>
                                <label>"Bar Weight"</label>
                                <select
                                    prop:value=move || format_display(bar_kg.get())
                                    on:change=move |ev| {
                                        if let Ok(v) = event_target_value(&ev).parse::<f32>() {
                                            set_bar_kg.set(v);
                                        }
                                    }
                                >
                                    <option value="20">"Standard 20kg (Men's)"</option>
                                    <option value="15">"Women's 15kg"</option>
                                    <option value="25">"Safety Squat / Deadlift 25kg"</option>
                                    <option value="10">"Technique Bar 10kg"</option>
                                </select>
                            </div>

                            <div>
                                <label>"Collars"</label>
                                <div class="toggle-group">
                                    <button
                                        class:on=move || collar_kg_each.get().abs() < 0.01
                                        on:click=move |_| set_collar_kg_each.set(0.0)
                                    >"None"</button>
                                    <button
                                        class:on=move || (collar_kg_each.get() - 2.5).abs() < 0.01
                                        on:click=move |_| set_collar_kg_each.set(2.5)
                                    >"2.5kg pair"</button>
                                    <button
                                        class:on=move || (collar_kg_each.get() - 5.0).abs() < 0.01
                                        on:click=move |_| set_collar_kg_each.set(5.0)
                                    >"5kg pair"</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div>
                    <p class="chart-summary chart-summary-standalone">
                        {move || {
                            let r = result.get();
                            if r.plates_per_side.is_empty() {
                                if r.below_bar {
                                    "The target is below the selected bar and collar weight.".to_string()
                                } else {
                                    "The target is just the selected bar and collars.".to_string()
                                }
                            } else {
                                let plates = r
                                    .plates_per_side
                                    .iter()
                                    .map(|(plate, count, _)| {
                                        format!(
                                            "{} x {}{}",
                                            count,
                                            format_plate_value(kg_to_display(*plate, use_lbs.get())),
                                            unit.get(),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                format!("Per side, load {plates} to reach {:.1}{}.", kg_to_display(r.achieved_kg, use_lbs.get()), unit.get())
                            }
                        }}
                    </p>
                    <div class="bar-stage">
                        <div class="bar-wrap">
                            <div class="plates-stack" style="flex-direction:row-reverse">
                                {move || plate_views(result.get().plates_per_side)}
                            </div>
                            <div class="bar-collar"></div>
                            <div class="bar-sleeve"></div>
                            <div class="bar-shaft"></div>
                            <div class="bar-sleeve"></div>
                            <div class="bar-collar"></div>
                            <div class="plates-stack">
                                {move || plate_views(result.get().plates_per_side)}
                            </div>
                        </div>
                        <div class="floor"></div>
                    </div>

                    <div class="plate-readout">
                        <div class="kpi">
                            <div class="l">"TARGET"</div>
                            <div class="v">
                                {move || format!("{:.1}", target.get())}
                                <span>{move || format!(" {}", unit.get())}</span>
                            </div>
                        </div>
                        <div class="kpi">
                            <div class="l">"PER SIDE"</div>
                            <div class="v">
                                {move || {
                                    let r = result.get();
                                    format!("{:.2}", kg_to_display(r.per_side_kg.max(0.0), use_lbs.get()))
                                }}
                                <span>{move || format!(" {}", unit.get())}</span>
                            </div>
                        </div>
                        <div class="kpi">
                            <div class="l">"ACHIEVED"</div>
                            <div class="v">
                                {move || format!("{:.1}", kg_to_display(result.get().achieved_kg, use_lbs.get()))}
                                <span>{move || format!(" {}", unit.get())}</span>
                            </div>
                        </div>
                    </div>

                    {move || {
                        let r = result.get();
                        if r.plates_per_side.is_empty() {
                            let message = if r.below_bar {
                                "TARGET BELOW BAR WEIGHT"
                            } else {
                                "JUST THE BAR"
                            };
                            view! {
                                <div class="plate-list">
                                    <div class="plate-empty">{message}</div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="plate-list">
                                    {r.plates_per_side.into_iter().map(move |(plate, count, color)| {
                                        let display_plate = kg_to_display(plate, use_lbs.get());
                                        view! {
                                            <div class="row">
                                                <div class="swatch" style=format!("background:{color}")></div>
                                                <div>
                                                    <span class="plate-name">{format_plate_value(display_plate)}</span>
                                                    <span class="plate-unit">{format!(" {} PLATE", unit.get())}</span>
                                                </div>
                                                <div class="cnt">
                                                    {format!("x {}", count * 2)}
                                                    <span>{format!("({count}/SIDE)")}</span>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                    {if r.remainder_kg > 0.01 {
                                        view! {
                                            <div class="notice error">
                                                {format!(
                                                    "{:.2} {} short per side",
                                                    kg_to_display(r.remainder_kg, use_lbs.get()),
                                                    unit.get()
                                                )}
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </section>
    }
}

fn plate_views(plates: Vec<(f32, usize, &'static str)>) -> impl IntoView {
    plates
        .into_iter()
        .flat_map(|(plate, count, color)| {
            (0..count).map(move |_| (plate, color)).collect::<Vec<_>>()
        })
        .enumerate()
        .map(|(i, (plate, color))| {
            view! {
                <div
                    class="plate"
                    attr:data-w=format_plate_key(plate)
                    style=format!(
                        "animation-delay:{}ms; --plate-color:{}; height:{}%; width:{}px",
                        i * 30,
                        color,
                        plate_height_pct(plate),
                        plate_width_px(plate),
                    )
                ></div>
            }
        })
        .collect_view()
}

fn plate_color(plate: f32) -> &'static str {
    match plate_key(plate).as_str() {
        "25" => "#e8472b",
        "20" => "#1e4a8f",
        "15" => "#e8b13a",
        "10" => "#2a8f4e",
        "5" => "#d8d8d8",
        "2.5" => "#333",
        "1.25" => "#555",
        _ => "#e8472b",
    }
}

fn plate_height_pct(plate: f32) -> u8 {
    match plate_key(plate).as_str() {
        "25" => 96,
        "20" => 88,
        "15" => 78,
        "10" => 66,
        "5" => 50,
        "2.5" => 38,
        "1.25" => 30,
        _ => 96,
    }
}

fn plate_width_px(plate: f32) -> u8 {
    match plate_key(plate).as_str() {
        "2.5" => 10,
        "1.25" => 8,
        _ => 14,
    }
}

fn format_plate_key(plate: f32) -> String {
    plate_key(plate)
}

fn plate_key(plate: f32) -> String {
    if (plate - plate.round()).abs() < 0.01 {
        format!("{plate:.0}")
    } else if (plate * 2.0 - (plate * 2.0).round()).abs() < 0.01 {
        format!("{plate:.1}")
    } else {
        format!("{plate:.2}")
    }
}

fn format_display(value: f32) -> String {
    if (value - value.round()).abs() < 0.05 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

fn format_plate_value(value: f32) -> String {
    if (value - value.round()).abs() < 0.05 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

#[cfg(test)]
mod tests {
    use super::calc_plates;
    use wasm_bindgen_test::wasm_bindgen_test;

    fn plate_weights(result: &super::PlateResult) -> Vec<f32> {
        result.plates_per_side.iter().map(|(kg, _, _)| *kg).collect()
    }

    fn plate_counts(result: &super::PlateResult) -> Vec<usize> {
        result.plates_per_side.iter().map(|(_, n, _)| *n).collect()
    }

    #[wasm_bindgen_test]
    fn just_the_bar_no_plates() {
        let r = calc_plates(20.0, 20.0, 0.0, false);
        assert!(r.plates_per_side.is_empty());
        assert!(!r.below_bar);
        assert!((r.achieved_kg - 20.0).abs() < 0.01);
        assert!(r.remainder_kg < 0.01);
    }

    #[wasm_bindgen_test]
    fn below_bar_weight_flags_below_bar() {
        let r = calc_plates(15.0, 20.0, 0.0, false);
        assert!(r.below_bar);
        assert!(r.plates_per_side.is_empty());
    }

    #[wasm_bindgen_test]
    fn standard_60kg_produces_one_20kg_plate_per_side() {
        let r = calc_plates(60.0, 20.0, 0.0, false);
        assert_eq!(plate_weights(&r), vec![20.0]);
        assert_eq!(plate_counts(&r), vec![1]);
        assert!((r.achieved_kg - 60.0).abs() < 0.01);
        assert!(r.remainder_kg < 0.01);
    }

    #[wasm_bindgen_test]
    fn collars_reduce_plates() {
        // 60kg target - 20kg bar - 5kg collars (2*2.5) = 35kg on plates → 17.5 per side
        // 17.5kg: 1×15 + 1×2.5 = 17.5
        let r = calc_plates(60.0, 20.0, 2.5, false);
        assert!((r.per_side_kg - 17.5).abs() < 0.01);
    }

    #[wasm_bindgen_test]
    fn lbs_target_is_converted_before_plate_selection() {
        // 396.83 lbs = exactly 180 kg (180 * 2.204622). Use the flag and check
        // the achieved weight is in the right ballpark, confirming kg conversion ran.
        let r_kg = calc_plates(180.0, 20.0, 0.0, false);
        let r_lb = calc_plates(396.832_5, 20.0, 0.0, true);
        // Both should not be below bar
        assert!(!r_lb.below_bar);
        // Achieved weights should be close (within 1 kg) despite lbs round-trip
        assert!((r_lb.achieved_kg - r_kg.achieved_kg).abs() < 1.0,
            "kg={}, lbs-mode={}", r_kg.achieved_kg, r_lb.achieved_kg);
    }

    #[wasm_bindgen_test]
    fn achieved_equals_bar_plus_plates() {
        let r = calc_plates(180.0, 20.0, 0.0, false);
        let plate_total: f32 = r
            .plates_per_side
            .iter()
            .map(|(kg, count, _)| kg * *count as f32 * 2.0)
            .sum();
        assert!((r.achieved_kg - (20.0 + plate_total)).abs() < 0.01);
    }
}
