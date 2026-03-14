use crate::core::{HeatmapBin, HistogramBin};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const DEFAULT_HEATMAP_WIDTH: f64 = 800.0;
const DEFAULT_HEATMAP_HEIGHT: f64 = 420.0;

pub(super) fn render_histogram_svg(hist: &HistogramBin, user_value: f32, x_label: &str) -> AnyView {
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

    let marker_x = left
        + ((user_value - hist.min) / (hist.max - hist.min).max(0.0001)).clamp(0.0, 1.0) * plot_w;
    let bars: Vec<(usize, u32)> = hist.counts.iter().copied().enumerate().collect();
    let x_mid = (hist.min + hist.max) * 0.5;
    let y_tick_mid = (max_count * 0.5).round() as u32;

    view! {
        <svg class="hist" viewBox="0 0 760 240" preserveAspectRatio="none">
            <rect x="0" y="0" width={w.to_string()} height={h.to_string()} fill="#f7f5ef" />
            <line x1={left.to_string()} y1={(top + plot_h).to_string()} x2={(left + plot_w).to_string()} y2={(top + plot_h).to_string()} stroke="#8a8a84" stroke-width="1" />
            <line x1={left.to_string()} y1={top.to_string()} x2={left.to_string()} y2={(top + plot_h).to_string()} stroke="#8a8a84" stroke-width="1" />
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
                            fill="#154734"
                        />
                    }
                })
                .collect_view()}
            <line x1={marker_x.to_string()} y1={top.to_string()} x2={marker_x.to_string()} y2={(top + plot_h).to_string()} stroke="#d6452b" stroke-width="3" />

            <text x={left.to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill="#4b4b44" text-anchor="middle">{format!("{:.0}", hist.min)}</text>
            <text x={(left + plot_w * 0.5).to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill="#4b4b44" text-anchor="middle">{format!("{:.0}", x_mid)}</text>
            <text x={(left + plot_w).to_string()} y={(top + plot_h + 18.0).to_string()} font-size="11" fill="#4b4b44" text-anchor="middle">{format!("{:.0}", hist.max)}</text>

            <text x={(left - 8.0).to_string()} y={(top + plot_h).to_string()} font-size="11" fill="#4b4b44" text-anchor="end">{ "0" }</text>
            <text x={(left - 8.0).to_string()} y={(top + plot_h * 0.5 + 4.0).to_string()} font-size="11" fill="#4b4b44" text-anchor="end">{y_tick_mid.to_string()}</text>
            <text x={(left - 8.0).to_string()} y={(top + 4.0).to_string()} font-size="11" fill="#4b4b44" text-anchor="end">{(max_count.round() as u32).to_string()}</text>

            <text x={(left + plot_w * 0.5).to_string()} y={(h - 4.0).to_string()} font-size="12" fill="#20342c" text-anchor="middle">{x_label.to_string()}</text>
            <text x="14" y={(top + plot_h * 0.5).to_string()} font-size="12" fill="#20342c" text-anchor="middle" transform={format!("rotate(-90,14,{})", top + plot_h * 0.5)}>"Lifter count"</text>

            <rect x={(w - 142.0).to_string()} y="10" width="132" height="34" rx="6" fill="#ffffff" stroke="#d5d2c7" />
            <rect x={(w - 132.0).to_string()} y="19" width="14" height="6" fill="#154734" />
            <text x={(w - 112.0).to_string()} y="25" font-size="11" fill="#20342c">"Distribution"</text>
            <line x1={(w - 132.0).to_string()} y1="34" x2={(w - 118.0).to_string()} y2="34" stroke="#d6452b" stroke-width="3" />
            <text x={(w - 112.0).to_string()} y="37" font-size="11" fill="#20342c">"Your value"</text>
        </svg>
    }
    .into_any()
}

pub(super) fn draw_heatmap(
    canvas: &HtmlCanvasElement,
    heat: &HeatmapBin,
    user_lift: f32,
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

    ctx.set_fill_style_str("#fcfaf4");
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
            let color = format!("rgba(11, 89, 160, {a})");
            ctx.set_fill_style_str(&color);
            ctx.fill_rect(
                left + x as f64 * cell_w,
                top + plot_h - ((y + 1) as f64 * cell_h),
                cell_w,
                cell_h,
            );
        }
    }

    let x = left
        + ((user_lift - heat.min_x) / (heat.max_x - heat.min_x).max(0.0001)).clamp(0.0, 1.0) as f64
            * plot_w;
    let y = top + plot_h
        - (((user_bw - heat.min_y) / (heat.max_y - heat.min_y).max(0.0001)).clamp(0.0, 1.0) as f64
            * plot_h);

    ctx.begin_path();
    ctx.set_fill_style_str("#d6452b");
    let _ = ctx.arc(x, y, 5.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();

    ctx.set_stroke_style_str("#8a8a84");
    ctx.begin_path();
    ctx.move_to(left, top + plot_h);
    ctx.line_to(left + plot_w, top + plot_h);
    ctx.move_to(left, top);
    ctx.line_to(left, top + plot_h);
    ctx.stroke();

    ctx.set_fill_style_str("#4b4b44");
    ctx.set_font("11px Space Grotesk, sans-serif");
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

    ctx.set_fill_style_str("#20342c");
    ctx.set_font("12px Space Grotesk, sans-serif");
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
        ctx.set_fill_style_str(&format!("rgba(11, 89, 160, {alpha})"));
        let y0 = legend_y + t0 * legend_h;
        let h0 = (t1 - t0) * legend_h;
        ctx.fill_rect(legend_x, y0, 14.0, h0 + 0.5);
    }
    ctx.set_stroke_style_str("#bdb8a7");
    ctx.stroke_rect(legend_x, legend_y, 14.0, legend_h);

    ctx.set_fill_style_str("#20342c");
    ctx.set_font("11px Space Grotesk, sans-serif");
    ctx.set_text_align("left");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text("Density", legend_x - 2.0, legend_y - 10.0);
    let _ = ctx.fill_text("High", legend_x + 20.0, legend_y + 2.0);
    let _ = ctx.fill_text("Low", legend_x + 20.0, legend_y + legend_h - 2.0);

    ctx.begin_path();
    ctx.set_fill_style_str("#d6452b");
    let _ = ctx.arc(
        legend_x + 7.0,
        legend_y + legend_h + 16.0,
        4.0,
        0.0,
        std::f64::consts::PI * 2.0,
    );
    ctx.fill();
    ctx.set_fill_style_str("#20342c");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text("You", legend_x + 20.0, legend_y + legend_h + 16.0);
}
