#![allow(dead_code)]
use wasm_bindgen::JsCast;

pub(super) struct ShareImagePayload<'a> {
    pub(super) handle: &'a str,
    pub(super) bodyweight: f32,
    pub(super) squat: f32,
    pub(super) bench: f32,
    pub(super) deadlift: f32,
    pub(super) lift_focus: &'a str,
    pub(super) percentile: f32,
    pub(super) tier: &'a str,
}

pub(super) fn download_share_png(payload: ShareImagePayload<'_>) -> Result<(), String> {
    let Some(window) = web_sys::window() else {
        return Err("No browser window.".to_string());
    };
    let Some(document) = window.document() else {
        return Err("No browser document.".to_string());
    };

    let canvas = document
        .create_element("canvas")
        .map_err(|_| "Failed to create canvas.")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| "Failed to create share canvas.")?;
    canvas.set_width(1200);
    canvas.set_height(630);

    let context = canvas
        .get_context("2d")
        .map_err(|_| "Failed to get 2d context.")?
        .ok_or_else(|| "No 2d context available.".to_string())?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| "Invalid canvas rendering context.")?;

    // Background
    context.set_fill_style_str("#0b0b0d");
    context.fill_rect(0.0, 0.0, 1200.0, 630.0);

    context.set_fill_style_str("#121215");
    context.fill_rect(40.0, 40.0, 1120.0, 550.0);

    // Iron glow
    context.set_fill_style_str("rgba(232, 71, 43, 0.06)");
    context.fill_rect(40.0, 40.0, 400.0, 550.0);

    context.set_stroke_style_str("#2a2a30");
    context.stroke_rect(40.0, 40.0, 1120.0, 550.0);

    // Corner accents
    context.set_stroke_style_str("#e8472b");
    context.set_line_width(1.5);
    context.begin_path();
    context.move_to(48.0, 60.0);
    context.line_to(48.0, 48.0);
    context.line_to(60.0, 48.0);
    context.stroke();
    context.begin_path();
    context.move_to(1148.0, 60.0);
    context.line_to(1148.0, 48.0);
    context.line_to(1136.0, 48.0);
    context.stroke();

    // Brand bar
    context.set_fill_style_str("#e8472b");
    context.fill_rect(80.0, 72.0, 12.0, 36.0);
    context.set_fill_style_str("#e8e3d6");
    context.fill_rect(72.0, 80.0, 4.0, 20.0);
    context.fill_rect(92.0, 80.0, 4.0, 20.0);

    context.set_fill_style_str("#f4f1ea");
    context.set_font("700 20px 'JetBrains Mono', monospace");
    context
        .fill_text("IRONSCALE", 112.0, 96.0)
        .map_err(|_| "Failed to render brand.")?;

    context.set_fill_style_str("#52504c");
    context.set_font("500 13px 'JetBrains Mono', monospace");
    context
        .fill_text("// WHERE YOU STAND", 112.0, 114.0)
        .map_err(|_| "Failed to render tagline.")?;

    // Main percentile
    context.set_fill_style_str("#e8472b");
    context.set_font("900 110px 'Archivo Black', sans-serif");
    context
        .fill_text(&format!("{:.1}%", payload.percentile * 100.0), 80.0, 320.0)
        .map_err(|_| "Failed to render percentile.")?;

    context.set_fill_style_str("#f4f1ea");
    context.set_font("400 26px 'JetBrains Mono', monospace");
    context
        .fill_text("STRONGER THAN THE MATCHED FIELD", 80.0, 368.0)
        .map_err(|_| "Failed to render rank label.")?;

    // Tier
    context.set_fill_style_str("#8a8680");
    context.set_font("400 18px 'JetBrains Mono', monospace");
    context
        .fill_text(
            &format!(
                "TIER {} · FOCUS {}",
                payload.tier.to_uppercase(),
                payload.lift_focus
            ),
            80.0,
            408.0,
        )
        .map_err(|_| "Failed to render tier.")?;

    // Lifts
    context
        .fill_text(
            &format!(
                "BW {:.1} · S {:.1} · B {:.1} · D {:.1} KG",
                payload.bodyweight, payload.squat, payload.bench, payload.deadlift
            ),
            80.0,
            440.0,
        )
        .map_err(|_| "Failed to render lifts.")?;

    // Handle
    if !payload.handle.trim().is_empty() {
        context.set_fill_style_str("#e8e3d6");
        context.set_font("400 22px 'JetBrains Mono', monospace");
        context
            .fill_text(payload.handle.trim(), 80.0, 490.0)
            .map_err(|_| "Failed to render handle.")?;
    }

    // Percentile bar (right side)
    context.set_fill_style_str("#1a1a1f");
    context.fill_rect(780.0, 100.0, 320.0, 380.0);
    context.set_stroke_style_str("#2a2a30");
    context.stroke_rect(780.0, 100.0, 320.0, 380.0);

    context.set_fill_style_str("#52504c");
    context.set_font("400 11px 'JetBrains Mono', monospace");
    context
        .fill_text("PERCENTILE", 800.0, 132.0)
        .map_err(|_| "Failed to render meter label.")?;

    // Track
    context.set_fill_style_str("#0b0b0d");
    context.fill_rect(800.0, 152.0, 280.0, 12.0);
    context.set_fill_style_str("#e8472b");
    context.fill_rect(
        800.0,
        152.0,
        (payload.percentile * 280.0).clamp(0.0, 280.0) as f64,
        12.0,
    );

    context.set_fill_style_str("#f4f1ea");
    context.set_font("900 80px 'Archivo Black', sans-serif");
    context
        .fill_text(&format!("{:.1}", payload.percentile * 100.0), 800.0, 310.0)
        .map_err(|_| "Failed to render meter value.")?;

    context.set_fill_style_str("#8a8680");
    context.set_font("400 16px 'JetBrains Mono', monospace");
    context
        .fill_text("PERCENTILE", 800.0, 342.0)
        .map_err(|_| "Failed to render meter unit.")?;

    // Watermark
    context.set_fill_style_str("#3a3a42");
    context.set_font("400 13px 'JetBrains Mono', monospace");
    context
        .fill_text("IRONSCALE · WHERE YOU STAND", 80.0, 565.0)
        .map_err(|_| "Failed to render watermark.")?;

    let data_url = canvas
        .to_data_url_with_type("image/png")
        .map_err(|_| "Failed to export PNG.")?;
    let anchor = document
        .create_element("a")
        .map_err(|_| "Failed to create download link.")?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "Failed to create download anchor.")?;
    anchor.set_href(&data_url);
    anchor.set_download("ironscale-ranking.png");
    anchor.click();
    Ok(())
}
