use crate::core::{HeatmapBin, HistogramBin};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const DEFAULT_HEATMAP_WIDTH: f64 = 800.0;
const DEFAULT_HEATMAP_HEIGHT: f64 = 380.0;
const MEN_COLOR: &str = "#e8472b";
const MEN_COLOR_RGB: &str = "232, 71, 43";
const WOMEN_COLOR_RGB: &str = "53, 208, 255";
const WOMEN_CURVE_COLOR: &str = "#35d0ff";
const USER_MARKER_COLOR: &str = "#e8e3d6";
const SURFACE_COLOR: &str = "#0b0b0d";
const AXIS_COLOR: &str = "#2a2a30";
const TICK_COLOR: &str = "#8a8680";
const LABEL_COLOR: &str = "#f4f1ea";
const LEGEND_BORDER: &str = "#2a2a30";
const HEAT_COLOR_RGB: &str = "232, 71, 43";
const STEEL_BAR_COLOR: &str = "rgba(107,115,128,0.4)";

fn setup_canvas(
    canvas: &HtmlCanvasElement,
    fallback_h: f64,
) -> Option<(CanvasRenderingContext2d, f64, f64)> {
    let Ok(Some(ctx)) = canvas.get_context("2d") else {
        return None;
    };
    let Ok(ctx) = ctx.dyn_into::<CanvasRenderingContext2d>() else {
        return None;
    };

    let css_w = f64::from(canvas.client_width().max(1));
    let css_h = match canvas.client_height() {
        0 => fallback_h.max(1.0),
        value => f64::from(value),
    };
    let dpr = web_sys::window()
        .map_or(1.0, |window| window.device_pixel_ratio())
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
    ctx.clear_rect(0.0, 0.0, f64::from(backing_w), f64::from(backing_h));
    let _ = ctx.scale(dpr, dpr);

    Some((ctx, css_w, css_h))
}

fn histogram_mean_stddev(hist: &HistogramBin) -> Option<(f64, f64)> {
    if hist.counts.is_empty() || hist.base_bin <= 0.0 {
        return None;
    }

    let center =
        |idx: usize| -> f64 { f64::from(hist.min) + (idx as f64 + 0.5) * f64::from(hist.base_bin) };
    let (total, sum_x, sum_x2) = hist.counts.iter().copied().enumerate().fold(
        (0.0_f64, 0.0_f64, 0.0_f64),
        |(total, sum_x, sum_x2), (idx, count)| {
            let weight = f64::from(count);
            let x = center(idx);
            (total + weight, sum_x + weight * x, sum_x2 + weight * x * x)
        },
    );

    if total <= 0.0 {
        return None;
    }

    let mean = sum_x / total;
    let variance = (sum_x2 / total - mean * mean).max(0.0);
    let stddev = variance.sqrt();
    if stddev.is_finite() && stddev > 0.0 {
        Some((mean, stddev))
    } else {
        None
    }
}

fn histogram_normal_params(hist: &HistogramBin) -> Option<(f64, f64)> {
    histogram_mean_stddev(hist).or_else(|| {
        let span = f64::from(hist.max - hist.min);
        (span > 0.0).then_some((f64::from(hist.min + hist.max) * 0.5, span / 6.0))
    })
}

fn normal_peak(value: f64, mean: f64, stddev: f64) -> f64 {
    let z = (value - mean) / stddev.max(0.0001);
    (-0.5 * z * z).exp()
}

fn axis_tick_label(x_label: &str, value: f64) -> String {
    let prefix = if x_label.contains("DOTS") {
        "DOTS"
    } else if x_label.contains("Wilks") {
        "WILKS"
    } else if x_label.contains("GL") {
        "GL"
    } else if x_label.contains("kg") || x_label.contains("KG") {
        "KG"
    } else {
        ""
    };
    let value = format_axis_tick(value as f32);
    if prefix.is_empty() {
        value
    } else {
        format!("{prefix} {value}")
    }
}

fn format_axis_tick(value: f32) -> String {
    if (value - value.round()).abs() < 0.05 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

pub(super) fn draw_ranking_distribution_canvas(
    canvas: &HtmlCanvasElement,
    hist: &HistogramBin,
    user_value: Option<f32>,
    x_label: &str,
) {
    let Some((ctx, cw, ch)) = setup_canvas(canvas, 320.0) else {
        return;
    };
    let Some((mean, stddev)) = histogram_normal_params(hist) else {
        return;
    };

    ctx.set_fill_style_str(SURFACE_COLOR);
    ctx.fill_rect(0.0, 0.0, cw, ch);

    ctx.set_stroke_style_str("rgba(255,255,255,0.04)");
    ctx.set_line_width(1.0);
    for i in 0..=4 {
        let y = ch * f64::from(i) / 4.0;
        ctx.begin_path();
        ctx.move_to(0.0, y);
        ctx.line_to(cw, y);
        ctx.stroke();
    }

    let x_min = mean - 3.0 * stddev;
    let x_max = mean + 3.0 * stddev;
    let x_span = (x_max - x_min).max(0.0001);
    let bins = 60usize;
    let bin_w = cw / bins as f64;
    let baseline = ch - 20.0;
    let plot_h = (ch - 40.0).max(1.0);
    let your_x = user_value
        .map_or(cw * 0.5, |value| ((f64::from(value) - x_min) / x_span).clamp(0.0, 1.0) * cw);

    let mut vals = Vec::with_capacity(bins);
    let mut max_val = 0.0_f64;
    for i in 0..bins {
        let value = x_min + (i as f64 + 0.5) * x_span / bins as f64;
        let density = normal_peak(value, mean, stddev);
        max_val = max_val.max(density);
        vals.push(density);
    }

    for (i, density) in vals.into_iter().enumerate() {
        let h = density / max_val.max(0.0001) * plot_h;
        let x = i as f64 * bin_w;
        let center_x = (i as f64 + 0.5) * bin_w;
        let fill = if user_value.is_some() && (center_x - your_x).abs() < bin_w {
            MEN_COLOR
        } else if user_value.is_some() && center_x < your_x {
            "rgba(232,71,43,0.35)"
        } else {
            STEEL_BAR_COLOR
        };
        ctx.set_fill_style_str(fill);
        ctx.fill_rect(x + 1.0, baseline - h, (bin_w - 2.0).max(0.5), h);
    }

    if user_value.is_some() {
        ctx.set_stroke_style_str(MEN_COLOR);
        ctx.set_line_width(2.0);
        let dash: wasm_bindgen::JsValue = js_sys::Array::of2(&4.0.into(), &3.0.into()).into();
        let _ = ctx.set_line_dash(&dash);
        ctx.begin_path();
        ctx.move_to(your_x, 0.0);
        ctx.line_to(your_x, baseline);
        ctx.stroke();
        let empty_dash: wasm_bindgen::JsValue = js_sys::Array::new().into();
        let _ = ctx.set_line_dash(&empty_dash);

        ctx.set_fill_style_str(MEN_COLOR);
        ctx.set_font("bold 11px 'Archivo Black', sans-serif");
        ctx.set_text_baseline("alphabetic");
        if your_x > cw - 48.0 {
            ctx.set_text_align("right");
            let _ = ctx.fill_text("YOU", your_x - 6.0, 14.0);
        } else {
            ctx.set_text_align("left");
            let _ = ctx.fill_text("YOU", your_x + 6.0, 14.0);
        }
    }

    ctx.set_fill_style_str(TICK_COLOR);
    ctx.set_font("10px 'JetBrains Mono', monospace");
    ctx.set_text_align("center");
    ctx.set_text_baseline("alphabetic");
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let x = t * cw;
        let value = x_min + t * x_span;
        let _ = ctx.fill_text(&axis_tick_label(x_label, value), x, ch - 4.0);
    }
}

fn draw_curve_layer(
    ctx: &CanvasRenderingContext2d,
    cw: f64,
    ch: f64,
    x_min: f64,
    x_max: f64,
    mean: f64,
    stddev: f64,
    color: &str,
    label: &str,
) {
    let baseline = ch - 40.0;
    let plot_h = (ch - 70.0).max(1.0);
    let x_span = (x_max - x_min).max(0.0001);
    let step = 2.0_f64;
    let mut pts = Vec::with_capacity((cw / step).ceil() as usize + 2);
    let mut x = 0.0_f64;
    while x <= cw {
        let value = x_min + (x / cw) * x_span;
        let y = baseline - normal_peak(value, mean, stddev) * plot_h;
        pts.push((x, y));
        x += step;
    }
    if pts.last().is_none_or(|(last_x, _)| *last_x < cw) {
        let value = x_max;
        pts.push((cw, baseline - normal_peak(value, mean, stddev) * plot_h));
    }

    let gradient = ctx.create_linear_gradient(0.0, 40.0, 0.0, ch);
    let _ = gradient.add_color_stop(0.0, &format!("{color}66"));
    let _ = gradient.add_color_stop(1.0, &format!("{color}00"));
    ctx.set_fill_style_canvas_gradient(&gradient);
    ctx.begin_path();
    ctx.move_to(0.0, baseline);
    for (x, y) in &pts {
        ctx.line_to(*x, *y);
    }
    ctx.line_to(cw, baseline);
    ctx.close_path();
    ctx.fill();

    ctx.set_stroke_style_str(color);
    ctx.set_line_width(2.0);
    ctx.begin_path();
    for (idx, (x, y)) in pts.iter().enumerate() {
        if idx == 0 {
            ctx.move_to(*x, *y);
        } else {
            ctx.line_to(*x, *y);
        }
    }
    ctx.stroke();

    let mean_x = ((mean - x_min) / x_span).clamp(0.0, 1.0) * cw;
    let label_x = (mean_x - 20.0).clamp(4.0, cw - 76.0);
    ctx.set_fill_style_str(color);
    ctx.set_font("bold 12px 'Archivo Black', sans-serif");
    ctx.set_text_align("left");
    ctx.set_text_baseline("alphabetic");
    let _ = ctx.fill_text(label, label_x, baseline - plot_h - 4.0);
}

pub(super) fn draw_dual_normal_curve_canvas(
    canvas: &HtmlCanvasElement,
    male_hist: &HistogramBin,
    female_hist: &HistogramBin,
    user_value: Option<f32>,
    x_label: &str,
) {
    let Some((ctx, cw, ch)) = setup_canvas(canvas, 320.0) else {
        return;
    };
    let Some((male_mean, male_stddev)) = histogram_normal_params(male_hist) else {
        return;
    };
    let Some((female_mean, female_stddev)) = histogram_normal_params(female_hist) else {
        return;
    };

    let mut x_min = (male_mean - 3.0 * male_stddev).min(female_mean - 3.0 * female_stddev);
    let mut x_max = (male_mean + 3.0 * male_stddev).max(female_mean + 3.0 * female_stddev);
    if x_max <= x_min {
        x_min = f64::from(male_hist.min.min(female_hist.min));
        x_max = f64::from(male_hist.max.max(female_hist.max));
    }
    if let Some(value) = user_value {
        let value = f64::from(value);
        x_min = x_min.min(value);
        x_max = x_max.max(value);
    }
    let x_span = (x_max - x_min).max(0.0001);

    ctx.set_fill_style_str(SURFACE_COLOR);
    ctx.fill_rect(0.0, 0.0, cw, ch);

    ctx.set_stroke_style_str("rgba(255,255,255,0.05)");
    ctx.set_line_width(1.0);
    for i in 0..=8 {
        let x = cw * f64::from(i) / 8.0;
        ctx.begin_path();
        ctx.move_to(x, 20.0);
        ctx.line_to(x, ch - 40.0);
        ctx.stroke();
    }

    draw_curve_layer(
        &ctx,
        cw,
        ch,
        x_min,
        x_max,
        female_mean,
        female_stddev,
        WOMEN_CURVE_COLOR,
        "FEMALE",
    );
    draw_curve_layer(
        &ctx,
        cw,
        ch,
        x_min,
        x_max,
        male_mean,
        male_stddev,
        MEN_COLOR,
        "MALE",
    );

    if let Some(value) = user_value {
        let marker_x = ((f64::from(value) - x_min) / x_span).clamp(0.0, 1.0) * cw;
        ctx.set_stroke_style_str(USER_MARKER_COLOR);
        ctx.set_line_width(1.5);
        let dash: wasm_bindgen::JsValue = js_sys::Array::of2(&4.0.into(), &3.0.into()).into();
        let _ = ctx.set_line_dash(&dash);
        ctx.begin_path();
        ctx.move_to(marker_x, 20.0);
        ctx.line_to(marker_x, ch - 40.0);
        ctx.stroke();
        let empty_dash: wasm_bindgen::JsValue = js_sys::Array::new().into();
        let _ = ctx.set_line_dash(&empty_dash);
    }

    ctx.set_fill_style_str(TICK_COLOR);
    ctx.set_font("10px 'JetBrains Mono', monospace");
    ctx.set_text_align("center");
    ctx.set_text_baseline("alphabetic");
    for i in 0..=8 {
        let t = f64::from(i) / 8.0;
        let x = t * cw;
        let value = x_min + t * x_span;
        let _ = ctx.fill_text(&format_axis_tick(value as f32), x, ch - 20.0);
    }
    let _ = ctx.fill_text(&x_label.to_uppercase(), cw * 0.5, ch - 4.0);
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

    let css_w = f64::from(canvas.client_width().max(1));
    let css_h = match canvas.client_height() {
        0 => (css_w * (DEFAULT_HEATMAP_HEIGHT / DEFAULT_HEATMAP_WIDTH))
            .round()
            .max(1.0),
        value => f64::from(value),
    };
    let dpr = web_sys::window()
        .map_or(1.0, |window| window.device_pixel_ratio())
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
    ctx.clear_rect(0.0, 0.0, f64::from(backing_w), f64::from(backing_h));
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

    let max_cell = f64::from(heat.grid.iter().copied().max().unwrap_or(1));
    let cell_w = plot_w / heat.width as f64;
    let cell_h = plot_h / heat.height as f64;

    for y in 0..heat.height {
        for x in 0..heat.width {
            let idx = y * heat.width + x;
            let v = f64::from(heat.grid[idx]);
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
            + f64::from(((user_lift - heat.min_x) / (heat.max_x - heat.min_x).max(0.0001)).clamp(0.0, 1.0))
                * plot_w;
        let y = top + plot_h
            - (f64::from(((user_bw - heat.min_y) / (heat.max_y - heat.min_y).max(0.0001)).clamp(0.0, 1.0))
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

    let max_cell = f64::from(heat.grid.iter().copied().max().unwrap_or(1));
    let x_span = (max_x - min_x).max(0.0001);
    let y_span = (max_y - min_y).max(0.0001);

    for y in 0..heat.height {
        for x in 0..heat.width {
            let idx = y * heat.width + x;
            let value = f64::from(heat.grid[idx]);
            if value <= 0.0 {
                continue;
            }

            let cell_min_x = f64::from(heat.min_x) + x as f64 * f64::from(heat.base_x);
            let cell_max_x = (cell_min_x + f64::from(heat.base_x)).min(f64::from(heat.max_x));
            let cell_min_y = f64::from(heat.min_y) + y as f64 * f64::from(heat.base_y);
            let cell_max_y = (cell_min_y + f64::from(heat.base_y)).min(f64::from(heat.max_y));

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

    let css_w = f64::from(canvas.client_width().max(1));
    let css_h = match canvas.client_height() {
        0 => (css_w * (DEFAULT_HEATMAP_HEIGHT / DEFAULT_HEATMAP_WIDTH))
            .round()
            .max(1.0),
        value => f64::from(value),
    };
    let dpr = web_sys::window()
        .map_or(1.0, |window| window.device_pixel_ratio())
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
    ctx.clear_rect(0.0, 0.0, f64::from(backing_w), f64::from(backing_h));
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

    let min_x = f64::from(male_heat.min_x.min(female_heat.min_x));
    let max_x = f64::from(male_heat.max_x.max(female_heat.max_x));
    let min_y = f64::from(male_heat.min_y.min(female_heat.min_y));
    let max_y = f64::from(male_heat.max_y.max(female_heat.max_y));
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
            top + plot_h - (((f64::from(user_bw) - min_y) / y_span).clamp(0.0, 1.0) * plot_h);
        let marker_x = left + (((f64::from(value) - min_x) / x_span).clamp(0.0, 1.0) * plot_w);
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
