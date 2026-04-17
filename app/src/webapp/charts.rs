use crate::core::{HeatmapBin, HistogramBin};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const DEFAULT_HEATMAP_WIDTH: f64 = 800.0;
const DEFAULT_HEATMAP_HEIGHT: f64 = 380.0;
const MEN_COLOR: &str = "#e8472b";
const MEN_COLOR_RGB: &str = "232, 71, 43";
const WOMEN_COLOR: &str = "#35d0ff";
const WOMEN_COLOR_RGB: &str = "53, 208, 255";
const USER_MARKER_COLOR: &str = "#e8e3d6";
const SURFACE_COLOR: &str = "#0b0b0d";
const AXIS_COLOR: &str = "#2a2a30";
const TICK_COLOR: &str = "#8a8680";
const LABEL_COLOR: &str = "#f4f1ea";
const LEGEND_BG: &str = "#121215";
const LEGEND_BORDER: &str = "#2a2a30";
const HEAT_COLOR_RGB: &str = "232, 71, 43";

fn format_axis_tick(value: f32) -> String {
    if (value - value.round()).abs() < 0.05 {
        format!("{:.0}", value)
    } else {
        format!("{value:.1}")
    }
}

fn histogram_overlay_rects(
    hist: &HistogramBin,
    min_x: f32,
    max_x: f32,
    max_count: f32,
    left: f32,
    top: f32,
    plot_w: f32,
    plot_h: f32,
) -> Vec<(f32, f32, f32, f32)> {
    let x_span = (max_x - min_x).max(0.0001);
    hist.counts
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(idx, count)| {
            if count == 0 {
                return None;
            }
            let bin_start = hist.min + idx as f32 * hist.base_bin;
            let bin_end = (bin_start + hist.base_bin).min(hist.max);
            let x0 = left + ((bin_start - min_x) / x_span).clamp(0.0, 1.0) * plot_w;
            let x1 = left + ((bin_end - min_x) / x_span).clamp(0.0, 1.0) * plot_w;
            let bar_h = (count as f32 / max_count.max(1.0)) * plot_h;
            let y = top + plot_h - bar_h;
            Some((x0, y, (x1 - x0).max(0.8), bar_h))
        })
        .collect()
}

pub(super) fn render_histogram_svg(
    hist: &HistogramBin,
    user_value: Option<f32>,
    x_label: &str,
) -> AnyView {
    let aria_label = if user_value.is_some() {
        format!("Histogram showing lifter count by {x_label}, with a marker for your input.")
    } else {
        format!("Histogram showing lifter count by {x_label}.")
    };
    let max_count = hist.counts.iter().copied().max().unwrap_or(1) as f32;
    let w = 760.0f32;
    let h = 240.0f32;
    let left = 52.0f32;
    let right = 16.0f32;
    let top = 12.0f32;
    let bottom = 34.0f32;
    let plot_w = (w - left - right).max(1.0);
    let plot_h = (h - top - bottom).max(1.0);
    let bar_w = (plot_w / hist.counts.len().max(1) as f32).max(1.0);

    let marker_x = user_value.map(|value| {
        left + ((value - hist.min) / (hist.max - hist.min).max(0.0001)).clamp(0.0, 1.0) * plot_w
    });
    let marker_view = if let Some(marker_x) = marker_x {
        view! {
            <line
                x1={marker_x.to_string()}
                y1={top.to_string()}
                x2={marker_x.to_string()}
                y2={(top + plot_h).to_string()}
                stroke={USER_MARKER_COLOR}
                stroke-width="2"
            />
        }
        .into_any()
    } else {
        ().into_any()
    };
    let marker_legend_view = if user_value.is_some() {
        view! {
            <>
                <line
                    x1={(w - 132.0).to_string()}
                    y1="34"
                    x2={(w - 118.0).to_string()}
                    y2="34"
                    stroke={USER_MARKER_COLOR}
                    stroke-width="2"
                />
                <text x={(w - 112.0).to_string()} y="37" font-size="11" fill={LABEL_COLOR}>"Your value"</text>
            </>
        }
        .into_any()
    } else {
        ().into_any()
    };
    let bars: Vec<(usize, u32)> = hist.counts.iter().copied().enumerate().collect();
    let x_mid = (hist.min + hist.max) * 0.5;
    let y_tick_mid = (max_count * 0.5).round() as u32;

    view! {
        <svg class="hist" viewBox="0 0 760 240" preserveAspectRatio="none" role="img" aria-label={aria_label}>
            <rect x="0" y="0" width={w.to_string()} height={h.to_string()} fill={SURFACE_COLOR} />
            <line x1={left.to_string()} y1={(top + plot_h).to_string()} x2={(left + plot_w).to_string()} y2={(top + plot_h).to_string()} stroke={AXIS_COLOR} stroke-width="1" />
            <line x1={left.to_string()} y1={top.to_string()} x2={left.to_string()} y2={(top + plot_h).to_string()} stroke={AXIS_COLOR} stroke-width="1" />
            {bars
                .into_iter()
                .map(|(i, c)| {
                    let bh = (c as f32 / max_count) * plot_h;
                    let x = left + i as f32 * bar_w;
                    let y = top + plot_h - bh;
                    view! {
                        <rect
                            x={x.to_string()}
                            y={y.to_string()}
                            width={(bar_w - 1.0).max(0.5).to_string()}
                            height={bh.to_string()}
                            fill={MEN_COLOR}
                            fill-opacity="0.7"
                        />
                    }
                })
                .collect_view()}
            {marker_view}

            <text x={left.to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="middle">{format!("{:.0}", hist.min)}</text>
            <text x={(left + plot_w * 0.5).to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="middle">{format!("{:.0}", x_mid)}</text>
            <text x={(left + plot_w).to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="middle">{format!("{:.0}", hist.max)}</text>

            <text x={(left - 8.0).to_string()} y={(top + plot_h).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="end">{ "0" }</text>
            <text x={(left - 8.0).to_string()} y={(top + plot_h * 0.5 + 4.0).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="end">{y_tick_mid.to_string()}</text>
            <text x={(left - 8.0).to_string()} y={(top + 4.0).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="end">{(max_count.round() as u32).to_string()}</text>

            <text x={(left + plot_w * 0.5).to_string()} y={(h - 4.0).to_string()} font-size="12" fill={LABEL_COLOR} text-anchor="middle">{x_label.to_string()}</text>
            <text x="14" y={(top + plot_h * 0.5).to_string()} font-size="12" fill={LABEL_COLOR} text-anchor="middle" transform={format!("rotate(-90,14,{})", top + plot_h * 0.5)}>"Count"</text>

            <rect
                x={(w - 142.0).to_string()}
                y="10"
                width="132"
                height={if user_value.is_some() { "34" } else { "20" }}
                fill={LEGEND_BG}
                stroke={LEGEND_BORDER}
            />
            <rect x={(w - 132.0).to_string()} y="19" width="14" height="6" fill={MEN_COLOR} />
            <text x={(w - 112.0).to_string()} y="25" font-size="11" fill={LABEL_COLOR}>"Distribution"</text>
            {marker_legend_view}
        </svg>
    }
    .into_any()
}

pub(super) fn render_dual_histogram_svg(
    male_hist: &HistogramBin,
    female_hist: &HistogramBin,
    user_value: Option<f32>,
    x_label: &str,
) -> AnyView {
    let aria_label =
        format!("Overlayed histogram comparing male and female lifter count by {x_label}.");
    let max_count = male_hist
        .counts
        .iter()
        .chain(female_hist.counts.iter())
        .copied()
        .max()
        .unwrap_or(1) as f32;
    let w = 760.0f32;
    let h = 240.0f32;
    let left = 52.0f32;
    let right = 18.0f32;
    let top = 12.0f32;
    let bottom = 34.0f32;
    let plot_w = (w - left - right).max(1.0);
    let plot_h = (h - top - bottom).max(1.0);
    let min_x = male_hist.min.min(female_hist.min);
    let max_x = male_hist.max.max(female_hist.max);
    let x_span = (max_x - min_x).max(0.0001);
    let user_marker_x =
        user_value.map(|value| left + ((value - min_x) / x_span).clamp(0.0, 1.0) * plot_w);
    let user_marker_view = if let Some(marker_x) = user_marker_x {
        view! {
            <line
                x1={marker_x.to_string()}
                y1={top.to_string()}
                x2={marker_x.to_string()}
                y2={(top + plot_h).to_string()}
                stroke={USER_MARKER_COLOR}
                stroke-width="2"
            />
        }
        .into_any()
    } else {
        ().into_any()
    };
    let user_marker_legend_view = if user_value.is_some() {
        view! {
            <>
                <line
                    x1={(w - 166.0).to_string()}
                    y1="50"
                    x2={(w - 152.0).to_string()}
                    y2="50"
                    stroke={USER_MARKER_COLOR}
                    stroke-width="2"
                />
                <text x={(w - 146.0).to_string()} y="53" font-size="11" fill={LABEL_COLOR}>"Your input"</text>
            </>
        }
        .into_any()
    } else {
        ().into_any()
    };
    let y_tick_mid = (max_count * 0.5).round() as u32;
    let male_bars = histogram_overlay_rects(
        male_hist, min_x, max_x, max_count, left, top, plot_w, plot_h,
    );
    let female_bars = histogram_overlay_rects(
        female_hist,
        min_x,
        max_x,
        max_count,
        left,
        top,
        plot_w,
        plot_h,
    );

    view! {
        <svg class="hist" viewBox="0 0 760 240" preserveAspectRatio="none" role="img" aria-label={aria_label}>
            <rect x="0" y="0" width={w.to_string()} height={h.to_string()} fill={SURFACE_COLOR} />
            <line x1={left.to_string()} y1={(top + plot_h).to_string()} x2={(left + plot_w).to_string()} y2={(top + plot_h).to_string()} stroke={AXIS_COLOR} stroke-width="1" />
            <line x1={left.to_string()} y1={top.to_string()} x2={left.to_string()} y2={(top + plot_h).to_string()} stroke={AXIS_COLOR} stroke-width="1" />

            {female_bars
                .into_iter()
                .map(|(x, y, width, height)| {
                    view! {
                        <rect
                            x={x.to_string()}
                            y={y.to_string()}
                            width={width.to_string()}
                            height={height.to_string()}
                            fill={WOMEN_COLOR}
                            fill-opacity="0.45"
                        />
                    }
                })
                .collect_view()}
            {male_bars
                .into_iter()
                .map(|(x, y, width, height)| {
                    view! {
                        <rect
                            x={x.to_string()}
                            y={y.to_string()}
                            width={width.to_string()}
                            height={height.to_string()}
                            fill={MEN_COLOR}
                            fill-opacity="0.5"
                        />
                    }
                })
                .collect_view()}

            {user_marker_view}

            <text x={left.to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="middle">{format_axis_tick(min_x)}</text>
            <text x={(left + plot_w * 0.5).to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="middle">{format_axis_tick((min_x + max_x) * 0.5)}</text>
            <text x={(left + plot_w).to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="middle">{format_axis_tick(max_x)}</text>

            <text x={(left - 8.0).to_string()} y={(top + plot_h).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="end">{ "0" }</text>
            <text x={(left - 8.0).to_string()} y={(top + plot_h * 0.5 + 4.0).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="end">{y_tick_mid.to_string()}</text>
            <text x={(left - 8.0).to_string()} y={(top + 4.0).to_string()} font-size="11" fill={TICK_COLOR} text-anchor="end">{(max_count.round() as u32).to_string()}</text>

            <text x={(left + plot_w * 0.5).to_string()} y={(h - 4.0).to_string()} font-size="12" fill={LABEL_COLOR} text-anchor="middle">{x_label.to_string()}</text>
            <text x="14" y={(top + plot_h * 0.5).to_string()} font-size="12" fill={LABEL_COLOR} text-anchor="middle" transform={format!("rotate(-90,14,{})", top + plot_h * 0.5)}>"Count"</text>

            <rect
                x={(w - 176.0).to_string()}
                y="10"
                width="166"
                height={if user_value.is_some() { "58" } else { "44" }}
                fill={LEGEND_BG}
                stroke={LEGEND_BORDER}
            />
            <rect x={(w - 166.0).to_string()} y="19" width="14" height="6" fill={MEN_COLOR} fill-opacity="0.5" />
            <text x={(w - 146.0).to_string()} y="25" font-size="11" fill={LABEL_COLOR}>"Men"</text>
            <rect x={(w - 166.0).to_string()} y="33" width="14" height="6" fill={WOMEN_COLOR} fill-opacity="0.45" />
            <text x={(w - 146.0).to_string()} y="39" font-size="11" fill={LABEL_COLOR}>"Women"</text>
            {user_marker_legend_view}
        </svg>
    }
    .into_any()
}

pub(super) fn draw_heatmap(
    canvas: &HtmlCanvasElement,
    heat: &HeatmapBin,
    user_lift: Option<f32>,
    user_bw: f32,
    x_label: &str,
) {
    let Ok(Some(ctx)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(ctx) = ctx.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let css_w = canvas.client_width().max(1) as f64;
    let css_h = match canvas.client_height() {
        0 => (css_w * (DEFAULT_HEATMAP_HEIGHT / DEFAULT_HEATMAP_WIDTH))
            .round()
            .max(1.0),
        value => value as f64,
    };
    let dpr = web_sys::window()
        .map(|window| window.device_pixel_ratio())
        .unwrap_or(1.0)
        .max(1.0);
    let backing_w = (css_w * dpr).round().max(1.0) as u32;
    let backing_h = (css_h * dpr).round().max(1.0) as u32;

    if canvas.width() != backing_w {
        canvas.set_width(backing_w);
    }
    if canvas.height() != backing_h {
        canvas.set_height(backing_h);
    }

    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    ctx.clear_rect(0.0, 0.0, backing_w as f64, backing_h as f64);
    let _ = ctx.scale(dpr, dpr);

    let cw = css_w;
    let ch = css_h;
    let left = 58.0f64;
    let right = 96.0f64;
    let top = 18.0f64;
    let bottom = 44.0f64;
    let plot_w = (cw - left - right).max(1.0);
    let plot_h = (ch - top - bottom).max(1.0);

    ctx.set_fill_style_str(SURFACE_COLOR);
    ctx.fill_rect(0.0, 0.0, cw, ch);

    if heat.width == 0 || heat.height == 0 || heat.grid.is_empty() {
        return;
    }

    let max_cell = heat.grid.iter().copied().max().unwrap_or(1) as f64;
    let cell_w = plot_w / heat.width as f64;
    let cell_h = plot_h / heat.height as f64;

    for y in 0..heat.height {
        for x in 0..heat.width {
            let idx = y * heat.width + x;
            let v = heat.grid[idx] as f64;
            if v <= 0.0 {
                continue;
            }
            let a = (v / max_cell).clamp(0.05, 1.0);
            let color = format!("rgba({HEAT_COLOR_RGB}, {a})");
            ctx.set_fill_style_str(&color);
            ctx.fill_rect(
                left + x as f64 * cell_w,
                top + plot_h - ((y + 1) as f64 * cell_h),
                cell_w,
                cell_h,
            );
        }
    }

    if let Some(user_lift) = user_lift {
        let x = left
            + ((user_lift - heat.min_x) / (heat.max_x - heat.min_x).max(0.0001)).clamp(0.0, 1.0)
                as f64
                * plot_w;
        let y = top + plot_h
            - (((user_bw - heat.min_y) / (heat.max_y - heat.min_y).max(0.0001)).clamp(0.0, 1.0)
                as f64
                * plot_h);

        ctx.begin_path();
        ctx.set_fill_style_str(USER_MARKER_COLOR);
        let _ = ctx.arc(x, y, 5.0, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    }

    ctx.set_stroke_style_str(AXIS_COLOR);
    ctx.begin_path();
    ctx.move_to(left, top + plot_h);
    ctx.line_to(left + plot_w, top + plot_h);
    ctx.move_to(left, top);
    ctx.line_to(left, top + plot_h);
    ctx.stroke();

    ctx.set_fill_style_str(TICK_COLOR);
    ctx.set_font("11px 'JetBrains Mono', monospace");
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");
    let _ = ctx.fill_text(&format!("{:.0}", heat.min_x), left, top + plot_h + 8.0);
    let _ = ctx.fill_text(
        &format!("{:.0}", (heat.min_x + heat.max_x) * 0.5),
        left + plot_w * 0.5,
        top + plot_h + 8.0,
    );
    let _ = ctx.fill_text(
        &format!("{:.0}", heat.max_x),
        left + plot_w,
        top + plot_h + 8.0,
    );

    ctx.set_text_align("right");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(&format!("{:.0}", heat.min_y), left - 8.0, top + plot_h);
    let _ = ctx.fill_text(
        &format!("{:.0}", (heat.min_y + heat.max_y) * 0.5),
        left - 8.0,
        top + plot_h * 0.5,
    );
    let _ = ctx.fill_text(&format!("{:.0}", heat.max_y), left - 8.0, top);

    ctx.set_fill_style_str(LABEL_COLOR);
    ctx.set_font("12px 'JetBrains Mono', monospace");
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");
    let _ = ctx.fill_text(x_label, left + plot_w * 0.5, ch - 18.0);

    ctx.save();
    let _ = ctx.translate(16.0, top + plot_h * 0.5);
    let _ = ctx.rotate(-std::f64::consts::FRAC_PI_2);
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");
    let _ = ctx.fill_text("Bodyweight (kg)", 0.0, 0.0);
    ctx.restore();

    let legend_x = left + plot_w + 22.0;
    let legend_y = top + 30.0;
    let legend_h = (plot_h - 50.0).max(40.0);
    let steps = 24usize;
    for i in 0..steps {
        let t0 = i as f64 / steps as f64;
        let t1 = (i + 1) as f64 / steps as f64;
        let alpha = 0.08 + (1.0 - t0) * (1.0 - 0.08);
        ctx.set_fill_style_str(&format!("rgba({HEAT_COLOR_RGB}, {alpha})"));
        let y0 = legend_y + t0 * legend_h;
        let h0 = (t1 - t0) * legend_h;
        ctx.fill_rect(legend_x, y0, 14.0, h0 + 0.5);
    }
    ctx.set_stroke_style_str(LEGEND_BORDER);
    ctx.stroke_rect(legend_x, legend_y, 14.0, legend_h);

    ctx.set_fill_style_str(LABEL_COLOR);
    ctx.set_font("11px 'JetBrains Mono', monospace");
    ctx.set_text_align("left");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text("Density", legend_x - 2.0, legend_y - 10.0);
    let _ = ctx.fill_text("High", legend_x + 20.0, legend_y + 2.0);
    let _ = ctx.fill_text("Low", legend_x + 20.0, legend_y + legend_h - 2.0);

    if user_lift.is_some() {
        ctx.begin_path();
        ctx.set_fill_style_str(USER_MARKER_COLOR);
        let _ = ctx.arc(
            legend_x + 7.0,
            legend_y + legend_h + 16.0,
            4.0,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.fill();
        ctx.set_fill_style_str(LABEL_COLOR);
        ctx.set_text_baseline("middle");
        let _ = ctx.fill_text("You", legend_x + 20.0, legend_y + legend_h + 16.0);
    }
}

fn draw_overlay_heat_layer(
    ctx: &CanvasRenderingContext2d,
    heat: &HeatmapBin,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    left: f64,
    top: f64,
    plot_w: f64,
    plot_h: f64,
    color_rgb: &str,
) {
    if heat.width == 0 || heat.height == 0 || heat.grid.is_empty() {
        return;
    }

    let max_cell = heat.grid.iter().copied().max().unwrap_or(1) as f64;
    let x_span = (max_x - min_x).max(0.0001);
    let y_span = (max_y - min_y).max(0.0001);

    for y in 0..heat.height {
        for x in 0..heat.width {
            let idx = y * heat.width + x;
            let value = heat.grid[idx] as f64;
            if value <= 0.0 {
                continue;
            }

            let cell_min_x = heat.min_x as f64 + x as f64 * heat.base_x as f64;
            let cell_max_x = (cell_min_x + heat.base_x as f64).min(heat.max_x as f64);
            let cell_min_y = heat.min_y as f64 + y as f64 * heat.base_y as f64;
            let cell_max_y = (cell_min_y + heat.base_y as f64).min(heat.max_y as f64);

            let x0 = left + ((cell_min_x - min_x) / x_span).clamp(0.0, 1.0) * plot_w;
            let x1 = left + ((cell_max_x - min_x) / x_span).clamp(0.0, 1.0) * plot_w;
            let y0 = top + plot_h - ((cell_max_y - min_y) / y_span).clamp(0.0, 1.0) * plot_h;
            let y1 = top + plot_h - ((cell_min_y - min_y) / y_span).clamp(0.0, 1.0) * plot_h;
            let alpha = (0.08 + (value / max_cell) * 0.62).clamp(0.08, 0.70);

            ctx.set_fill_style_str(&format!("rgba({color_rgb}, {alpha:.3})"));
            ctx.fill_rect(x0, y0, (x1 - x0).max(1.0), (y1 - y0).max(1.0));
        }
    }
}

fn draw_circle_marker(ctx: &CanvasRenderingContext2d, x: f64, y: f64, color: &str) {
    ctx.begin_path();
    ctx.set_fill_style_str(color);
    let _ = ctx.arc(x, y, 5.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();
    ctx.set_stroke_style_str(LABEL_COLOR);
    ctx.stroke();
}

pub(super) fn draw_cross_sex_heatmap_overlay(
    canvas: &HtmlCanvasElement,
    male_heat: &HeatmapBin,
    female_heat: &HeatmapBin,
    user_value: Option<f32>,
    user_bw: f32,
    x_label: &str,
) {
    let Ok(Some(ctx)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(ctx) = ctx.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let css_w = canvas.client_width().max(1) as f64;
    let css_h = match canvas.client_height() {
        0 => (css_w * (DEFAULT_HEATMAP_HEIGHT / DEFAULT_HEATMAP_WIDTH))
            .round()
            .max(1.0),
        value => value as f64,
    };
    let dpr = web_sys::window()
        .map(|window| window.device_pixel_ratio())
        .unwrap_or(1.0)
        .max(1.0);
    let backing_w = (css_w * dpr).round().max(1.0) as u32;
    let backing_h = (css_h * dpr).round().max(1.0) as u32;

    if canvas.width() != backing_w {
        canvas.set_width(backing_w);
    }
    if canvas.height() != backing_h {
        canvas.set_height(backing_h);
    }

    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    ctx.clear_rect(0.0, 0.0, backing_w as f64, backing_h as f64);
    let _ = ctx.scale(dpr, dpr);

    let cw = css_w;
    let ch = css_h;
    let left = 58.0f64;
    let right = 132.0f64;
    let top = 18.0f64;
    let bottom = 44.0f64;
    let plot_w = (cw - left - right).max(1.0);
    let plot_h = (ch - top - bottom).max(1.0);

    ctx.set_fill_style_str(SURFACE_COLOR);
    ctx.fill_rect(0.0, 0.0, cw, ch);

    let min_x = male_heat.min_x.min(female_heat.min_x) as f64;
    let max_x = male_heat.max_x.max(female_heat.max_x) as f64;
    let min_y = male_heat.min_y.min(female_heat.min_y) as f64;
    let max_y = male_heat.max_y.max(female_heat.max_y) as f64;
    if max_x <= min_x || max_y <= min_y {
        return;
    }

    let _ = ctx.set_global_composite_operation("lighter");
    draw_overlay_heat_layer(
        &ctx,
        female_heat,
        min_x,
        max_x,
        min_y,
        max_y,
        left,
        top,
        plot_w,
        plot_h,
        WOMEN_COLOR_RGB,
    );
    draw_overlay_heat_layer(
        &ctx,
        male_heat,
        min_x,
        max_x,
        min_y,
        max_y,
        left,
        top,
        plot_w,
        plot_h,
        MEN_COLOR_RGB,
    );
    let _ = ctx.set_global_composite_operation("source-over");

    let x_span = (max_x - min_x).max(0.0001);
    let y_span = (max_y - min_y).max(0.0001);
    if let Some(value) = user_value {
        let marker_y =
            top + plot_h - (((user_bw as f64 - min_y) / y_span).clamp(0.0, 1.0) * plot_h);
        let marker_x = left + (((value as f64 - min_x) / x_span).clamp(0.0, 1.0) * plot_w);
        draw_circle_marker(&ctx, marker_x, marker_y, USER_MARKER_COLOR);
    }

    ctx.set_stroke_style_str(AXIS_COLOR);
    ctx.begin_path();
    ctx.move_to(left, top + plot_h);
    ctx.line_to(left + plot_w, top + plot_h);
    ctx.move_to(left, top);
    ctx.line_to(left, top + plot_h);
    ctx.stroke();

    ctx.set_fill_style_str(TICK_COLOR);
    ctx.set_font("11px 'JetBrains Mono', monospace");
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");
    let _ = ctx.fill_text(&format_axis_tick(min_x as f32), left, top + plot_h + 8.0);
    let _ = ctx.fill_text(
        &format_axis_tick(((min_x + max_x) * 0.5) as f32),
        left + plot_w * 0.5,
        top + plot_h + 8.0,
    );
    let _ = ctx.fill_text(
        &format_axis_tick(max_x as f32),
        left + plot_w,
        top + plot_h + 8.0,
    );

    ctx.set_text_align("right");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(&format_axis_tick(min_y as f32), left - 8.0, top + plot_h);
    let _ = ctx.fill_text(
        &format_axis_tick(((min_y + max_y) * 0.5) as f32),
        left - 8.0,
        top + plot_h * 0.5,
    );
    let _ = ctx.fill_text(&format_axis_tick(max_y as f32), left - 8.0, top);

    ctx.set_fill_style_str(LABEL_COLOR);
    ctx.set_font("12px 'JetBrains Mono', monospace");
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");
    let _ = ctx.fill_text(x_label, left + plot_w * 0.5, ch - 18.0);

    ctx.save();
    let _ = ctx.translate(16.0, top + plot_h * 0.5);
    let _ = ctx.rotate(-std::f64::consts::FRAC_PI_2);
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");
    let _ = ctx.fill_text("Bodyweight (kg)", 0.0, 0.0);
    ctx.restore();

    let legend_x = left + plot_w + 20.0;
    let legend_y = top + 26.0;
    ctx.set_fill_style_str(LABEL_COLOR);
    ctx.set_font("11px 'JetBrains Mono', monospace");
    ctx.set_text_align("left");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text("Overlay", legend_x, legend_y - 12.0);

    ctx.set_fill_style_str(&format!("rgba({MEN_COLOR_RGB}, 0.55)"));
    ctx.fill_rect(legend_x, legend_y, 14.0, 10.0);
    let _ = ctx.fill_text("Men", legend_x + 20.0, legend_y + 5.0);

    ctx.set_fill_style_str(&format!("rgba({WOMEN_COLOR_RGB}, 0.50)"));
    ctx.fill_rect(legend_x, legend_y + 18.0, 14.0, 10.0);
    let _ = ctx.fill_text("Women", legend_x + 20.0, legend_y + 23.0);

    if user_value.is_some() {
        draw_circle_marker(&ctx, legend_x + 7.0, legend_y + 46.0, USER_MARKER_COLOR);
        let _ = ctx.fill_text("You", legend_x + 20.0, legend_y + 46.0);
    }
}
