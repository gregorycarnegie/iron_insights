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

    context.set_fill_style_str("#08080a");
    context.fill_rect(0.0, 0.0, 1200.0, 630.0);

    context.set_fill_style_str("#14141a");
    context.fill_rect(40.0, 40.0, 1120.0, 550.0);
    context.set_fill_style_str("rgba(78, 205, 196, 0.08)");
    context.fill_rect(760.0, 40.0, 400.0, 220.0);
    context.set_fill_style_str("rgba(200, 241, 53, 0.06)");
    context.fill_rect(40.0, 40.0, 320.0, 180.0);

    context.set_stroke_style_str("#1c1c24");
    context.stroke_rect(40.0, 40.0, 1120.0, 550.0);

    context.set_fill_style_str("#c8f135");
    context.fill_rect(80.0, 78.0, 186.0, 34.0);
    context.set_fill_style_str("#08080a");
    context.set_font("800 18px 'Barlow Condensed', sans-serif");
    context
        .fill_text("IRON INSIGHTS", 96.0, 101.0)
        .map_err(|_| "Failed to render brand tag.")?;

    context.set_fill_style_str("#f0efe8");
    context.set_font("900 56px 'Barlow Condensed', sans-serif");
    context
        .fill_text("Ranking Snapshot", 80.0, 170.0)
        .map_err(|_| "Failed to render heading text.")?;

    context.set_font("600 36px Barlow, sans-serif");
    let who = if payload.handle.trim().is_empty() {
        "Anonymous Lifter".to_string()
    } else {
        payload.handle.trim().to_string()
    };
    context
        .fill_text(&who, 80.0, 224.0)
        .map_err(|_| "Failed to render handle.")?;

    context.set_fill_style_str("#c8f135");
    context.set_font("900 124px 'Barlow Condensed', sans-serif");
    context
        .fill_text(&format!("{:.1}%", payload.percentile * 100.0), 80.0, 360.0)
        .map_err(|_| "Failed to render percentile.")?;

    context.set_fill_style_str("#f0efe8");
    context.set_font("700 34px 'Barlow Condensed', sans-serif");
    context
        .fill_text("Stronger than the matched field", 80.0, 408.0)
        .map_err(|_| "Failed to render rank label.")?;

    context.set_fill_style_str("#b2b2b8");
    context.set_font("500 28px Barlow, sans-serif");
    context
        .fill_text(
            &format!("Tier {}  |  Focus {}", payload.tier, payload.lift_focus),
            80.0,
            448.0,
        )
        .map_err(|_| "Failed to render tier line.")?;

    context.set_fill_style_str("#7a7a84");
    context.set_font("500 23px 'JetBrains Mono', monospace");
    context
        .fill_text(
            &format!(
                "BW {:.1} kg | S {:.1} | B {:.1} | D {:.1}",
                payload.bodyweight, payload.squat, payload.bench, payload.deadlift
            ),
            80.0,
            492.0,
        )
        .map_err(|_| "Failed to render lift line.")?;

    context.set_fill_style_str("#14141a");
    context.fill_rect(760.0, 104.0, 320.0, 322.0);
    context.set_stroke_style_str("#2a2a34");
    context.stroke_rect(760.0, 104.0, 320.0, 322.0);

    context.set_fill_style_str("#5a5a64");
    context.set_font("600 18px 'JetBrains Mono', monospace");
    context
        .fill_text("Percentile meter", 790.0, 145.0)
        .map_err(|_| "Failed to render meter label.")?;

    context.set_fill_style_str("#0a0a0d");
    context.fill_rect(790.0, 182.0, 260.0, 18.0);
    context.set_fill_style_str("#c8f135");
    context.fill_rect(
        790.0,
        182.0,
        (payload.percentile * 260.0).clamp(0.0, 260.0) as f64,
        18.0,
    );
    context.set_fill_style_str("#7a7a84");
    context.set_font("600 18px 'JetBrains Mono', monospace");
    context
        .fill_text("0", 790.0, 224.0)
        .map_err(|_| "Failed to render meter min.")?;
    context
        .fill_text("100", 1018.0, 224.0)
        .map_err(|_| "Failed to render meter max.")?;

    context.set_fill_style_str("#f0efe8");
    context.set_font("900 64px 'Barlow Condensed', sans-serif");
    context
        .fill_text(&format!("{:.1}", payload.percentile * 100.0), 790.0, 322.0)
        .map_err(|_| "Failed to render meter value.")?;
    context.set_font("700 28px 'Barlow Condensed', sans-serif");
    context
        .fill_text("percentile", 790.0, 358.0)
        .map_err(|_| "Failed to render meter unit.")?;

    context.set_fill_style_str("#7a7a84");
    context.set_font("500 22px 'JetBrains Mono', monospace");
    context
        .fill_text("iron-insights", 80.0, 555.0)
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
    anchor.set_download("iron-insights-ranking.png");
    anchor.click();
    Ok(())
}
