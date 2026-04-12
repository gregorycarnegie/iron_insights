use leptos::ev;
use leptos::html::Canvas;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::webapp::helpers::{display_to_kg, format_input_bound, kg_to_display};

struct PlateSpec {
    weight: f32,
    name: &'static str,
    color: &'static str,
    face_color: &'static str,
    radius: f32,
    thickness: f32,
}

const PLATES: &[PlateSpec] = &[
    PlateSpec {
        weight: 25.0,
        name: "Red 25",
        color: "#e63946",
        face_color: "#f04e5b",
        radius: 1.0,
        thickness: 0.065,
    },
    PlateSpec {
        weight: 20.0,
        name: "Blue 20",
        color: "#457bca",
        face_color: "#5a8fd4",
        radius: 0.93,
        thickness: 0.065,
    },
    PlateSpec {
        weight: 15.0,
        name: "Yellow 15",
        color: "#ffd166",
        face_color: "#ffdb80",
        radius: 0.86,
        thickness: 0.065,
    },
    PlateSpec {
        weight: 10.0,
        name: "Green 10",
        color: "#5cb85c",
        face_color: "#72c672",
        radius: 0.79,
        thickness: 0.055,
    },
    PlateSpec {
        weight: 5.0,
        name: "Black 5",
        color: "#444444",
        face_color: "#555555",
        radius: 0.72,
        thickness: 0.055,
    },
    PlateSpec {
        weight: 2.5,
        name: "Black 2.5",
        color: "#383838",
        face_color: "#4a4a4a",
        radius: 0.36,
        thickness: 0.055,
    },
    PlateSpec {
        weight: 1.25,
        name: "Black 1.25",
        color: "#2a2a2a",
        face_color: "#3c3c3c",
        radius: 0.36,
        thickness: 0.028,
    },
];

struct PlateResult {
    plate_idx: usize,
    count: u32,
}

struct CalcResult {
    plates: Vec<PlateResult>,
    actual: f32,
    remainder: f32,
    warning: Option<String>,
}

fn calculate_plates(target_kg: f32, bar_kg: f32) -> CalcResult {
    let remaining = target_kg - bar_kg;

    if remaining < 0.0 {
        return CalcResult {
            plates: vec![],
            actual: bar_kg,
            remainder: remaining,
            warning: Some(format!("Target is less than bar weight ({bar_kg}kg).")),
        };
    }

    let per_side = remaining / 2.0;
    let mut leftover = per_side;
    let mut plates = Vec::new();

    for (idx, plate) in PLATES.iter().enumerate() {
        if leftover <= 0.0 {
            break;
        }
        let count = (leftover / plate.weight).floor() as u32;
        if count > 0 {
            plates.push(PlateResult {
                plate_idx: idx,
                count,
            });
            leftover = ((leftover - count as f32 * plate.weight) * 1000.0).round() / 1000.0;
        }
    }

    let loaded: f32 = plates
        .iter()
        .map(|p| p.count as f32 * PLATES[p.plate_idx].weight * 2.0)
        .sum();
    let actual = bar_kg + loaded;
    let remainder = ((target_kg - actual) * 1000.0).round() / 1000.0;

    let warning = if remainder > 0.001 {
        Some(format!(
            "Cannot hit {target_kg}kg exactly with standard plates. Loaded: {actual}kg. Short by {remainder}kg."
        ))
    } else {
        None
    };

    CalcResult {
        plates,
        actual,
        remainder,
        warning,
    }
}

fn draw_barbell(canvas: &HtmlCanvasElement, plates: &[PlateResult], _bar_kg: f32) {
    let dpr = web_sys::window()
        .map(|w| w.device_pixel_ratio())
        .unwrap_or(1.0);

    let css_w = canvas.client_width() as f64;
    let css_h = canvas.client_height() as f64;
    if css_w < 1.0 || css_h < 1.0 {
        return;
    }

    let buf_w = (css_w * dpr).round() as u32;
    let buf_h = (css_h * dpr).round() as u32;
    canvas.set_width(buf_w);
    canvas.set_height(buf_h);

    let Some(ctx): Option<CanvasRenderingContext2d> = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into().ok())
    else {
        return;
    };

    let _ = ctx.scale(dpr, dpr);
    let w = css_w;
    let h = css_h;

    // Clear
    ctx.set_fill_style_str("#0c0c0f");
    ctx.fill_rect(0.0, 0.0, w, h);

    let cy = h / 2.0;
    let bar_total_len = w * 0.92;
    let bar_x_start = (w - bar_total_len) / 2.0;
    let bar_x_end = bar_x_start + bar_total_len;

    let bar_radius = 3.0;
    let sleeve_radius = 5.0;
    let collar_radius = 7.0;
    let max_plate_h = h * 0.42;

    let sleeve_start_frac = 0.22;
    let sleeve_start_left = bar_x_start + bar_total_len * sleeve_start_frac;
    let sleeve_start_right = bar_x_end - bar_total_len * sleeve_start_frac;
    let collar_w = 6.0;

    // Bar shaft
    ctx.set_fill_style_str("#888888");
    ctx.fill_rect(
        bar_x_start,
        cy - bar_radius,
        bar_total_len,
        bar_radius * 2.0,
    );

    // Sleeves
    ctx.set_fill_style_str("#999999");
    ctx.fill_rect(
        bar_x_start,
        cy - sleeve_radius,
        sleeve_start_left - bar_x_start,
        sleeve_radius * 2.0,
    );
    ctx.fill_rect(
        sleeve_start_right,
        cy - sleeve_radius,
        bar_x_end - sleeve_start_right,
        sleeve_radius * 2.0,
    );

    // Collars (at inner edge of sleeves)
    ctx.set_fill_style_str("#555555");
    ctx.fill_rect(
        sleeve_start_left - collar_w,
        cy - collar_radius,
        collar_w,
        collar_radius * 2.0,
    );
    ctx.fill_rect(
        sleeve_start_right,
        cy - collar_radius,
        collar_w,
        collar_radius * 2.0,
    );

    // End caps
    ctx.set_fill_style_str("#666666");
    ctx.fill_rect(
        bar_x_start,
        cy - sleeve_radius * 0.85,
        3.0,
        sleeve_radius * 1.7,
    );
    ctx.fill_rect(
        bar_x_end - 3.0,
        cy - sleeve_radius * 0.85,
        3.0,
        sleeve_radius * 1.7,
    );

    // Plates
    let gap = 1.5;
    let plate_thickness_base = 12.0;
    let thin_plate_thickness = 6.0;

    let mut offset_left = sleeve_start_left - collar_w - 2.0;
    let mut offset_right = sleeve_start_right + collar_w + 2.0;

    for pr in plates {
        let spec = &PLATES[pr.plate_idx];
        let plate_h = max_plate_h * spec.radius as f64;
        let thick = if spec.thickness < 0.04 {
            thin_plate_thickness
        } else {
            plate_thickness_base
        };

        for _ in 0..pr.count {
            // Left side (plates grow leftward)
            ctx.set_fill_style_str(spec.color);
            ctx.fill_rect(offset_left - thick, cy - plate_h, thick, plate_h * 2.0);
            ctx.set_fill_style_str(spec.face_color);
            ctx.fill_rect(offset_left - thick, cy - plate_h, 2.0, plate_h * 2.0);

            // Right side (plates grow rightward)
            ctx.set_fill_style_str(spec.color);
            ctx.fill_rect(offset_right, cy - plate_h, thick, plate_h * 2.0);
            ctx.set_fill_style_str(spec.face_color);
            ctx.fill_rect(offset_right + thick - 2.0, cy - plate_h, 2.0, plate_h * 2.0);

            // Center hole suggestion
            ctx.set_fill_style_str("#111111");
            let hole_r = 4.0;
            ctx.fill_rect(offset_left - thick, cy - hole_r, thick, hole_r * 2.0);
            ctx.fill_rect(offset_right, cy - hole_r, thick, hole_r * 2.0);

            offset_left -= thick + gap;
            offset_right += thick + gap;
        }
    }
}

fn format_weight_input(v: f32) -> String {
    if (v - v.round()).abs() < 0.05 {
        format!("{:.0}", v)
    } else if ((v * 2.0) - (v * 2.0).round()).abs() < 0.05 {
        format!("{:.1}", v)
    } else {
        format!("{:.2}", v)
    }
}

#[component]
pub(in crate::webapp) fn PlateCalcPanel() -> impl IntoView {
    let (use_lbs, set_use_lbs) = signal(false);
    let (target_kg, set_target_kg) = signal(100.0f32);
    let (bar_kg, set_bar_kg) = signal(20.0f32);
    let (resize_tick, set_resize_tick) = signal(0u32);

    let canvas_ref: NodeRef<Canvas> = NodeRef::new();
    let resize_handle = window_event_listener(ev::resize, move |_| {
        set_resize_tick.update(|tick| *tick = tick.wrapping_add(1));
    });
    on_cleanup(move || resize_handle.remove());

    let display_target = Memo::new(move |_| kg_to_display(target_kg.get(), use_lbs.get()));
    let display_bar = Memo::new(move |_| kg_to_display(bar_kg.get(), use_lbs.get()));
    let unit = Memo::new(move |_| if use_lbs.get() { "lb" } else { "kg" });

    let result = Memo::new(move |_| {
        let t = target_kg.get();
        let b = bar_kg.get();
        let r = calculate_plates(t, b);
        (
            r.plates
                .iter()
                .map(|p| (p.plate_idx, p.count))
                .collect::<Vec<_>>(),
            r.actual,
            r.remainder,
            r.warning,
        )
    });

    let plates_data = Memo::new(move |_| result.get().0);
    let actual = Memo::new(move |_| result.get().1);
    let remainder = Memo::new(move |_| result.get().2);
    let warning = Memo::new(move |_| result.get().3);

    // Canvas redraw effect
    Effect::new(move || {
        let _ = resize_tick.get();
        let pdata = plates_data.get();
        let bar = bar_kg.get();
        let Some(canvas_el) = canvas_ref.get() else {
            return;
        };
        let canvas_el: &HtmlCanvasElement = &canvas_el;

        let plates: Vec<PlateResult> = pdata
            .iter()
            .map(|(idx, count)| PlateResult {
                plate_idx: *idx,
                count: *count,
            })
            .collect();
        draw_barbell(canvas_el, &plates, bar);
    });

    let slider_max = Memo::new(move |_| if use_lbs.get() { 1540.0 } else { 700.0 });
    let slider_min = Memo::new(move |_| if use_lbs.get() { 45.0 } else { 20.0 });
    let slider_step = Memo::new(move |_| if use_lbs.get() { 2.5 } else { 1.25 });
    let target_input_max = Memo::new(move |_| format_input_bound(2000.0, use_lbs.get()));
    let target_input_step = Memo::new(move |_| if use_lbs.get() { "2.5" } else { "1.25" });
    let bar_input_max = Memo::new(move |_| format_input_bound(100.0, use_lbs.get()));
    let bar_input_step = Memo::new(move |_| if use_lbs.get() { "1" } else { "0.5" });

    view! {
        <section class="panel plate-calc">
            <div class="plate-calc-header">
                <div class="plate-calc-header-copy">
                    <h2>"Plate Calculator"</h2>
                    <p class="muted plate-calc-intro">
                        "Enter a target weight and see exactly which plates to load on each side of the bar."
                    </p>
                </div>
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

            <div class="plate-calc-target">
                <p class="plate-calc-eyebrow">"Target Weight"</p>
                <p class="plate-calc-big-number">
                    {move || format!("{}", format_weight_input(display_target.get()))}
                    <span class="plate-calc-unit">{move || unit.get()}</span>
                </p>
                <input
                    type="range"
                    class="plate-calc-slider"
                    prop:min=move || slider_min.get().to_string()
                    prop:max=move || slider_max.get().to_string()
                    prop:step=move || slider_step.get().to_string()
                    prop:value=move || display_target.get().to_string()
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<f32>() {
                            set_target_kg.set(display_to_kg(v, use_lbs.get()));
                        }
                    }
                />
            </div>

            <div class="grid simple plate-calc-inputs">
                <label>{move || format!("Target weight ({})", unit.get())}
                    <input
                        type="number"
                        prop:min="0"
                        prop:max=move || target_input_max.get()
                        prop:step=move || target_input_step.get()
                        prop:value=move || format_weight_input(display_target.get())
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f32>()
                                && v.is_finite() && v >= 0.0
                            {
                                set_target_kg.set(display_to_kg(v, use_lbs.get()));
                            }
                        }
                    />
                </label>
                <label>{move || format!("Bar weight ({})", unit.get())}
                    <input
                        type="number"
                        prop:min="0"
                        prop:max=move || bar_input_max.get()
                        prop:step=move || bar_input_step.get()
                        prop:value=move || format_weight_input(display_bar.get())
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f32>()
                                && v.is_finite() && v >= 0.0
                            {
                                set_bar_kg.set(display_to_kg(v, use_lbs.get()));
                            }
                        }
                    />
                </label>
            </div>

            <div class="plate-calc-canvas-wrap">
                <div class="plate-calc-visual">
                    <div class="plate-calc-canvas-panel">
                        <canvas node_ref=canvas_ref class="plate-calc-canvas" />
                        <p class="plate-calc-canvas-label">"Side view — plates per side"</p>
                    </div>

                    {move || {
                        let pdata = plates_data.get();
                        let use_lbs = use_lbs.get();
                        let unit = unit.get();

                        if pdata.is_empty() {
                            None
                        } else {
                            Some(view! {
                                <aside class="plate-calc-key">
                                    <div class="plate-calc-key-heading">
                                        <span class="plate-calc-key-title">"Key"</span>
                                        <span class="plate-calc-key-subtitle">"Per side"</span>
                                    </div>
                                    <div class="plate-calc-key-items">
                                        {pdata.into_iter().map(|(idx, count)| {
                                            let spec = &PLATES[idx];
                                            let color = spec.color.to_string();
                                            let label = format!(
                                                "{} {}",
                                                format_weight_input(kg_to_display(spec.weight, use_lbs)),
                                                unit,
                                            );

                                            view! {
                                                <div class="plate-calc-key-item">
                                                    <span class="plate-calc-key-swatch" style=format!("background:{color}") />
                                                    <span class="plate-calc-key-label">{label}</span>
                                                    <strong class="plate-calc-key-count">{format!("x{count}")}</strong>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </aside>
                            })
                        }
                    }}
                </div>
            </div>

            <div class="plate-calc-breakdown">
                <h3>"Plates per side"</h3>
                {move || {
                    let pdata = plates_data.get();
                    let t = target_kg.get();
                    let b = bar_kg.get();

                    if pdata.is_empty() && t <= b {
                        view! {
                            <p class="muted">"No plates needed — bar only."</p>
                        }.into_any()
                    } else if pdata.is_empty() {
                        view! {
                            <p class="muted">"No plates needed."</p>
                        }.into_any()
                    } else {
                        view! {
                            <div class="plate-calc-rows">
                                {pdata.into_iter().map(|(idx, count)| {
                                    let spec = &PLATES[idx];
                                    let color = spec.color.to_string();
                                    let name = spec.name;
                                    let weight = spec.weight;
                                    view! {
                                        <div class="plate-calc-row">
                                            <div class="plate-calc-swatch" style=format!("background:{color}") />
                                            <div class="plate-calc-plate-info">
                                                <span class="plate-calc-plate-name">{name}</span>
                                                <span class="plate-calc-plate-sub">
                                                    {move || format!("{:.1} {} each", kg_to_display(weight, use_lbs.get()), unit.get())}
                                                </span>
                                            </div>
                                            <strong class="plate-calc-plate-count">{format!("{count} × 2")}</strong>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <div class="plate-calc-summary">
                <div class="plate-calc-stat">
                    <span>"Actual loaded"</span>
                    <strong>{move || format!("{:.1} {}", kg_to_display(actual.get(), use_lbs.get()), unit.get())}</strong>
                </div>
                <div class="plate-calc-stat">
                    <span>"Remainder"</span>
                    <strong>{move || {
                        let r = remainder.get();
                        if r.abs() < 0.001 { format!("0 {}", unit.get()) }
                        else { format!("{:.2} {}", kg_to_display(r, use_lbs.get()), unit.get()) }
                    }}</strong>
                </div>
            </div>

            {move || warning.get().map(|msg| view! {
                <div class="plate-calc-warning">
                    {msg}
                </div>
            })}
        </section>
    }
}
