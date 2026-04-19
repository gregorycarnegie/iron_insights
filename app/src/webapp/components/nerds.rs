use super::shared::{Corners, InputForm};
use crate::webapp::charts::draw_heatmap;
use crate::webapp::helpers::kg_to_display;
use crate::webapp::state::AppState;
use leptos::ev;
use leptos::html::Canvas;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::prelude::*;

const REF_SQUAT_PCT: f32 = 35.0;
const REF_BENCH_PCT: f32 = 25.0;
const REF_DEADLIFT_PCT: f32 = 40.0;

fn format_lift_value(value_kg: f32, use_lbs: bool) -> String {
    let shown = kg_to_display(value_kg, use_lbs);
    if (shown - shown.round()).abs() < 0.05 {
        format!("{shown:.0}")
    } else {
        format!("{shown:.1}")
    }
}

fn delta_class(delta: f32) -> &'static str {
    if delta >= 0.0 { "up" } else { "dn" }
}

fn delta_marker(delta: f32) -> &'static str {
    if delta >= 0.0 { "^" } else { "v" }
}

fn triangle_axis_point(
    vertex: (f32, f32),
    center: (f32, f32),
    pct: f32,
    reference_pct: f32,
) -> (f32, f32) {
    let strength = (pct / (reference_pct * 1.35).max(0.0001)).clamp(0.18, 1.0);
    (
        center.0 + (vertex.0 - center.0) * strength,
        center.1 + (vertex.1 - center.1) * strength,
    )
}

fn triangle_points(squat_pct: f32, bench_pct: f32, deadlift_pct: f32) -> String {
    let center = (210.0, 226.0);
    let squat = triangle_axis_point((210.0, 40.0), center, squat_pct, REF_SQUAT_PCT);
    let bench = triangle_axis_point((380.0, 320.0), center, bench_pct, REF_BENCH_PCT);
    let deadlift = triangle_axis_point((40.0, 320.0), center, deadlift_pct, REF_DEADLIFT_PCT);
    format!(
        "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
        squat.0, squat.1, bench.0, bench.1, deadlift.0, deadlift.1
    )
}

fn reference_triangle_points() -> String {
    triangle_points(REF_SQUAT_PCT, REF_BENCH_PCT, REF_DEADLIFT_PCT)
}

fn balance_index(squat_pct: f32, bench_pct: f32, deadlift_pct: f32) -> f32 {
    let deviation = (squat_pct - REF_SQUAT_PCT).abs()
        + (bench_pct - REF_BENCH_PCT).abs()
        + (deadlift_pct - REF_DEADLIFT_PCT).abs();
    (1.0 - deviation / 40.0).clamp(0.0, 1.0)
}

fn archetype(squat_pct: f32, bench_pct: f32, deadlift_pct: f32) -> (&'static str, &'static str) {
    let squat_delta = squat_pct - REF_SQUAT_PCT;
    let bench_delta = bench_pct - REF_BENCH_PCT;
    let deadlift_delta = deadlift_pct - REF_DEADLIFT_PCT;
    let max_abs = squat_delta
        .abs()
        .max(bench_delta.abs())
        .max(deadlift_delta.abs());

    if max_abs < 2.0 {
        (
            "Balanced",
            "Your total is close to the classic powerlifting split.",
        )
    } else if deadlift_delta >= squat_delta && deadlift_delta >= bench_delta {
        (
            "Puller",
            "Deadlift-dominant. Your pull carries more of your total than the reference split.",
        )
    } else if squat_delta >= bench_delta && squat_delta >= deadlift_delta {
        (
            "Squatter",
            "Squat-dominant. Your total leans toward leg drive and position strength.",
        )
    } else {
        (
            "Presser",
            "Bench-dominant. Your press contributes more of your total than the reference split.",
        )
    }
}

#[component]
pub fn NerdsPage() -> impl IntoView {
    let app = use_context::<AppState>().expect("AppState must be provided by App");
    let inp = app.input;
    let cmp = app.compute;
    let dataset_blurb = cmp.dataset_blurb;
    let percentile = cmp.percentile;
    let rank_tier = cmp.rank_tier;
    let user_lift = cmp.user_lift;
    let calculated = cmp.calculated;
    let rebinned_heat = cmp.rebinned_heat;
    let hist_x_label = cmp.hist_x_label;
    let slice_summary = cmp.slice_summary;
    let use_lbs = inp.use_lbs;
    let unit_label = inp.unit_label;
    let bodyweight = inp.bodyweight;
    let squat = inp.squat;
    let bench = inp.bench;
    let deadlift = inp.deadlift;

    let canvas_ref: NodeRef<Canvas> = NodeRef::new();
    let (heatmap_resize_tick, set_heatmap_resize_tick) = signal(0u32);
    let resize_handle = window_event_listener(ev::resize, move |_| {
        set_heatmap_resize_tick.update(|tick| *tick = tick.wrapping_add(1));
    });
    on_cleanup(move || resize_handle.remove());

    Effect::new(move |_| {
        let _ = heatmap_resize_tick.get();
        let Some(canvas) = canvas_ref.get() else {
            return;
        };
        let Some(h) = rebinned_heat.get() else {
            return;
        };
        draw_heatmap(
            &canvas,
            &h,
            calculated.get().then(|| user_lift.get()),
            bodyweight.get(),
            &hist_x_label.get(),
        );
    });

    let total_kg = Memo::new(move |_| squat.get() + bench.get() + deadlift.get());
    let squat_pct = Memo::new(move |_| {
        let t = total_kg.get();
        if t > 0.0 {
            squat.get() / t * 100.0
        } else {
            0.0
        }
    });
    let bench_pct = Memo::new(move |_| {
        let t = total_kg.get();
        if t > 0.0 {
            bench.get() / t * 100.0
        } else {
            0.0
        }
    });
    let dl_pct = Memo::new(move |_| {
        let t = total_kg.get();
        if t > 0.0 {
            deadlift.get() / t * 100.0
        } else {
            0.0
        }
    });
    let squat_delta = Memo::new(move |_| squat_pct.get() - REF_SQUAT_PCT);
    let bench_delta = Memo::new(move |_| bench_pct.get() - REF_BENCH_PCT);
    let dl_delta = Memo::new(move |_| dl_pct.get() - REF_DEADLIFT_PCT);
    let user_triangle_points =
        Memo::new(move |_| triangle_points(squat_pct.get(), bench_pct.get(), dl_pct.get()));
    let reference_triangle_points = reference_triangle_points();
    let archetype = Memo::new(move |_| archetype(squat_pct.get(), bench_pct.get(), dl_pct.get()));
    let balance_index =
        Memo::new(move |_| balance_index(squat_pct.get(), bench_pct.get(), dl_pct.get()));

    view! {
        <section class="page active" id="page-nerds">
            <div class="page-head">
                <h1 class="page-title">
                    "Stats for " <span class="accent">"nerds"</span> "."
                </h1>
                <p class="page-lede">
                    <span class="serif">"Distribution, heatmap, and lift ratios"</span>
                    " for your selected cohort. Data from " {move || dataset_blurb.get()} "."
                </p>
            </div>

            <div class="nerd-grid">
                // Left: heatmap panel
                <div>
                    <div class="panel" style="margin-bottom:24px">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"MAP"</span>" LIFT × BODYWEIGHT HEATMAP"</span>
                            <span>"DENSITY"</span>
                        </div>
                        <div class="panel-body">
                            <p class="chart-summary">
                                {move || {
                                    if calculated.get() {
                                        format!(
                                            "Your marker plots {} at {}{} bodyweight against the densest parts of this cohort.",
                                            format_lift_value(user_lift.get(), use_lbs.get()),
                                            format_lift_value(bodyweight.get(), use_lbs.get()),
                                            unit_label.get(),
                                        )
                                    } else {
                                        "Compute first to plot your lift and bodyweight against the cohort heatmap.".to_string()
                                    }
                                }}
                            </p>
                            <div class="heat-wrap">
                                {move || {
                                    if rebinned_heat.get().is_some() {
                                        view! {
                                            <p class="visually-hidden">
                                                {move || {
                                                    format!(
                                                        "Heatmap chart. Your current marker is {} at {}{} bodyweight.",
                                                        format_lift_value(user_lift.get(), use_lbs.get()),
                                                        format_lift_value(bodyweight.get(), use_lbs.get()),
                                                        unit_label.get(),
                                                    )
                                                }}
                                            </p>
                                            <canvas
                                                node_ref=canvas_ref.clone()
                                                style="width:100%;height:100%;display:block"
                                                role="img"
                                                aria-label="Lift by bodyweight cohort heatmap"
                                            ></canvas>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="notice">
                                                {if calculated.get() { "Loading heatmap..." } else { "Compute first to load heatmap." }}
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                    // Lift composition power triangle
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"SHAPE"</span>" POWER TRIANGLE"</span>
                            <span>"LIFT COMPOSITION"</span>
                        </div>
                        <div class="panel-body">
                            <p class="chart-summary">
                                {move || {
                                    format!(
                                        "{}: squat is {:.1}%, bench is {:.1}%, and deadlift is {:.1}% of your total.",
                                        archetype.get().0,
                                        squat_pct.get(),
                                        bench_pct.get(),
                                        dl_pct.get(),
                                    )
                                }}
                            </p>
                            <div
                                class="power-triangle"
                                role="img"
                                aria-label=move || {
                                    format!(
                                        "Power triangle showing squat {:.1} percent, bench {:.1} percent, and deadlift {:.1} percent of total.",
                                        squat_pct.get(),
                                        bench_pct.get(),
                                        dl_pct.get()
                                    )
                                }
                            >
                                <div class="pt-head">
                                    <span><span class="pt-tag">"04"</span>" LIFT COMPOSITION"</span>
                                    <span>
                                        "TOTAL "
                                        {move || format_lift_value(total_kg.get(), use_lbs.get())}
                                        {move || unit_label.get().to_uppercase()}
                                        " / REFERENCE SPLIT"
                                    </span>
                                </div>
                                <div class="pt-title">"Your " <span>"shape"</span> " of strong."</div>

                                <div class="pt-grid">
                                    <div class="pt-svg-wrap">
                                        <svg viewBox="0 0 420 380" width="100%" style="display:block">
                                            <defs>
                                                <pattern id="ptgrid" width="30" height="30" patternUnits="userSpaceOnUse">
                                                    <path d="M 30 0 L 0 0 0 30" fill="none" stroke="#1a1a1f" stroke-width="1" />
                                                </pattern>
                                            </defs>
                                            <rect width="420" height="380" fill="url(#ptgrid)" />

                                            <polygon points="210,40 380,320 40,320" fill="none" stroke="#2a2a30" stroke-width="1" stroke-dasharray="4 4" />
                                            <polygon points="210,110 325,287 95,287" fill="none" stroke="#2a2a30" stroke-width="1" stroke-dasharray="2 3" />
                                            <polygon points="210,180 268,252 152,252" fill="none" stroke="#2a2a30" stroke-width="1" stroke-dasharray="2 3" />

                                            <polygon points="210,40 380,320 40,320" fill="none" stroke="#52504c" stroke-width="1" />
                                            <line x1="210" y1="40" x2="210" y2="320" stroke="#1a1a1f" stroke-width="1" />
                                            <line x1="40" y1="320" x2="380" y2="320" stroke="#1a1a1f" stroke-width="1" />

                                            <polygon points={reference_triangle_points.clone()} fill="#6b7380" fill-opacity="0.15" stroke="#6b7380" stroke-width="1" stroke-dasharray="3 3" />
                                            <text x="128" y="220" font-family="JetBrains Mono, monospace" font-size="9" fill="#52504c" letter-spacing="1.5">"REFERENCE"</text>

                                            <polygon points=move || user_triangle_points.get() fill="#e8472b" fill-opacity="0.18" stroke="#e8472b" stroke-width="2" />

                                            <circle cx="210" cy="86" r="6" fill="#e8472b" />
                                            <circle cx="210" cy="86" r="10" fill="none" stroke="#e8472b" stroke-width="1" opacity="0.4" />
                                            <circle cx="356" cy="309" r="6" fill="#c79a4a" />
                                            <circle cx="356" cy="309" r="10" fill="none" stroke="#c79a4a" stroke-width="1" opacity="0.4" />
                                            <circle cx="79" cy="309" r="6" fill="#6b7380" />
                                            <circle cx="79" cy="309" r="10" fill="none" stroke="#6b7380" stroke-width="1" opacity="0.4" />

                                            <g>
                                                <text x="210" y="22" text-anchor="middle" font-family="Archivo Black, sans-serif" font-size="11" fill="#e8472b" letter-spacing="2.5">"SQUAT"</text>
                                                <text x="210" y="74" text-anchor="middle" font-family="Archivo Black, sans-serif" font-size="18" fill="#f4f1ea">
                                                    {move || format_lift_value(squat.get(), use_lbs.get())}
                                                </text>
                                                <text x="210" y="350" text-anchor="middle" font-family="JetBrains Mono, monospace" font-size="9" fill="#52504c" letter-spacing="1.5">
                                                    {move || format!("{:.1}% OF TOTAL", squat_pct.get())}
                                                </text>
                                            </g>
                                            <g>
                                                <text x="388" y="340" text-anchor="end" font-family="Archivo Black, sans-serif" font-size="11" fill="#c79a4a" letter-spacing="2.5">"BENCH"</text>
                                                <text x="388" y="358" text-anchor="end" font-family="JetBrains Mono, monospace" font-size="9" fill="#52504c" letter-spacing="1">
                                                    {move || format!("{:.1}% / {}{:.1}%", bench_pct.get(), if bench_delta.get() >= 0.0 { "+" } else { "" }, bench_delta.get())}
                                                </text>
                                                <text x="374" y="293" text-anchor="end" font-family="Archivo Black, sans-serif" font-size="18" fill="#f4f1ea">
                                                    {move || format_lift_value(bench.get(), use_lbs.get())}
                                                </text>
                                            </g>
                                            <g>
                                                <text x="32" y="340" font-family="Archivo Black, sans-serif" font-size="11" fill="#6b7380" letter-spacing="2.5">"DEADLIFT"</text>
                                                <text x="32" y="358" font-family="JetBrains Mono, monospace" font-size="9" fill="#52504c" letter-spacing="1">
                                                    {move || format!("{:.1}% / {}{:.1}%", dl_pct.get(), if dl_delta.get() >= 0.0 { "+" } else { "" }, dl_delta.get())}
                                                </text>
                                                <text x="64" y="293" font-family="Archivo Black, sans-serif" font-size="18" fill="#f4f1ea">
                                                    {move || format_lift_value(deadlift.get(), use_lbs.get())}
                                                </text>
                                            </g>
                                        </svg>
                                    </div>

                                    <div class="pt-right">
                                        <div class="pt-arch">
                                            <div class="pt-arch-lbl">"ARCHETYPE"</div>
                                            <div class="pt-arch-name">
                                                "The " <span>{move || archetype.get().0}</span> "."
                                            </div>
                                            <div class="pt-arch-desc">{move || archetype.get().1}</div>
                                        </div>

                                        <div class="pt-contrib">
                                            <div class="pt-contrib-row">
                                                <div class="pt-ic sq">"SQ"</div>
                                                <div class="pt-bar"><div class="pt-fill sq" style=move || format!("width:{:.1}%", squat_pct.get())></div></div>
                                                <div class="pt-pct">
                                                    {move || format!("{:.1}%", squat_pct.get())}
                                                    <span class=move || format!("delta {}", delta_class(squat_delta.get()))>
                                                        {move || format!("{} {:.1}%", delta_marker(squat_delta.get()), squat_delta.get().abs())}
                                                    </span>
                                                </div>
                                            </div>
                                            <div class="pt-contrib-row">
                                                <div class="pt-ic bp">"BP"</div>
                                                <div class="pt-bar"><div class="pt-fill bp" style=move || format!("width:{:.1}%", bench_pct.get())></div></div>
                                                <div class="pt-pct">
                                                    {move || format!("{:.1}%", bench_pct.get())}
                                                    <span class=move || format!("delta {}", delta_class(bench_delta.get()))>
                                                        {move || format!("{} {:.1}%", delta_marker(bench_delta.get()), bench_delta.get().abs())}
                                                    </span>
                                                </div>
                                            </div>
                                            <div class="pt-contrib-row">
                                                <div class="pt-ic dl">"DL"</div>
                                                <div class="pt-bar"><div class="pt-fill dl" style=move || format!("width:{:.1}%", dl_pct.get())></div></div>
                                                <div class="pt-pct">
                                                    {move || format!("{:.1}%", dl_pct.get())}
                                                    <span class=move || format!("delta {}", delta_class(dl_delta.get()))>
                                                        {move || format!("{} {:.1}%", delta_marker(dl_delta.get()), dl_delta.get().abs())}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="pt-foot">
                                            <span>"BALANCE INDEX / " {move || format!("{:.2}", balance_index.get())}</span>
                                            <span>"REF / 35-25-40"</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                // Right: stats panels
                <div class="nerd-side">
                    // Your lift ratios (live from inputs)
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"μ"</span>" LIFT RATIOS"</span>
                            <span>"OF YOUR TOTAL"</span>
                        </div>
                        <div class="panel-body">
                            <p class="chart-summary">
                                {move || {
                                    format!(
                                        "Your total is split {:.1}% squat, {:.1}% bench, and {:.1}% deadlift.",
                                        squat_pct.get(),
                                        bench_pct.get(),
                                        dl_pct.get(),
                                    )
                                }}
                            </p>
                            <div class="dist-bar">
                                <div class="lb">
                                    <span>"SQUAT"</span>
                                    <span>{move || format!("{:.1}%", squat_pct.get())}</span>
                                </div>
                                <div class="tr">
                                    <div class="fl" style=move || format!("width:{:.1}%", squat_pct.get())></div>
                                </div>
                            </div>
                            <div class="dist-bar">
                                <div class="lb">
                                    <span>"BENCH"</span>
                                    <span>{move || format!("{:.1}%", bench_pct.get())}</span>
                                </div>
                                <div class="tr">
                                    <div class="fl" style=move || format!("width:{:.1}%", bench_pct.get())></div>
                                </div>
                            </div>
                            <div class="dist-bar">
                                <div class="lb">
                                    <span>"DEADLIFT"</span>
                                    <span>{move || format!("{:.1}%", dl_pct.get())}</span>
                                </div>
                                <div class="tr">
                                    <div class="fl" style=move || format!("width:{:.1}%", dl_pct.get())></div>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Population stats from cohort
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"P"</span>" COHORT"</span>
                            <span>"SUMMARY"</span>
                        </div>
                        <div class="panel-body" style="font-size:12px;line-height:1.9">
                            {move || match slice_summary.get() {
                                Some(s) => view! {
                                    <div>
                                        <div style="display:flex;justify-content:space-between">
                                            <span style="color:var(--ink-dim)">"TOTAL LIFTERS"</span>
                                            <span style="font-family:'Archivo Black',sans-serif">{s.total.to_string()}</span>
                                        </div>
                                        <div style="display:flex;justify-content:space-between">
                                            <span style="color:var(--ink-dim)">"MIN LIFT"</span>
                                            <span style="font-family:'Archivo Black',sans-serif">{format!("{:.1} KG", s.min_kg)}</span>
                                        </div>
                                        <div style="display:flex;justify-content:space-between">
                                            <span style="color:var(--ink-dim)">"MAX LIFT"</span>
                                            <span style="font-family:'Archivo Black',sans-serif">{format!("{:.1} KG", s.max_kg)}</span>
                                        </div>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="notice">
                                        {if calculated.get() { "Loading cohort summary..." } else { "Compute first." }}
                                    </div>
                                }.into_any(),
                            }}
                        </div>
                    </div>

                    // Your percentile result
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"PCT"</span>" YOUR PERCENTILE"</span>
                            <span>"RESULT"</span>
                        </div>
                        <div class="panel-body">
                            {move || match percentile.get() {
                                Some((p, _, _)) => view! {
                                    <div>
                                        <div class="stat-big">
                                            <div class="label">"PERCENTILE"</div>
                                            <div class="num">{format!("{:.1}", p * 100.0)}<span style="font-size:28px;color:var(--ink-dim)">"ᵗʰ"</span></div>
                                            <div class="sub">{rank_tier.get().unwrap_or("—").to_uppercase()}</div>
                                            <div class="bar-track">
                                                <div class="bar-fill" style=format!("width:{:.1}%", p * 100.0)></div>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="notice">
                                        {if calculated.get() { "Awaiting distribution data..." } else { "Compute first." }}
                                    </div>
                                }.into_any(),
                            }}
                        </div>
                    </div>

                    // Input form
                    <div class="panel">
                        <Corners />
                        <div class="panel-head">
                            <span><span class="tag">"IN"</span>" YOUR NUMBERS"</span>
                            <span>"FILTER"</span>
                        </div>
                        <div class="panel-body">
                            <InputForm />
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
