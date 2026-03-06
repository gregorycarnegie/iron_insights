use wasm_bindgen::JsCast;

#[allow(clippy::too_many_arguments)]
pub(super) fn download_share_png(
    handle: &str,
    bodyweight: f32,
    squat: f32,
    bench: f32,
    deadlift: f32,
    lift_focus: &str,
    percentile: f32,
    tier: &str,
) -> Result<(), String> {
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

    context.set_fill_style_str("#0f2e24");
    context.fill_rect(0.0, 0.0, 1200.0, 630.0);
    context.set_fill_style_str("#153f31");
    context.fill_rect(40.0, 40.0, 1120.0, 550.0);
    context.set_fill_style_str("#f6f2e8");
    context.set_font("700 60px 'Space Grotesk', sans-serif");
    context
        .fill_text("Iron Insights Ranking", 80.0, 130.0)
        .map_err(|_| "Failed to render heading text.")?;

    context.set_font("500 36px 'Space Grotesk', sans-serif");
    let who = if handle.trim().is_empty() {
        "Anonymous Lifter".to_string()
    } else {
        handle.trim().to_string()
    };
    context
        .fill_text(&who, 80.0, 200.0)
        .map_err(|_| "Failed to render handle.")?;

    context.set_font("400 30px 'IBM Plex Mono', monospace");
    context
        .fill_text(
            &format!(
                "BW {:.1}kg | S {:.1} | B {:.1} | D {:.1} | Focus {}",
                bodyweight, squat, bench, deadlift, lift_focus
            ),
            80.0,
            265.0,
        )
        .map_err(|_| "Failed to render lift line.")?;
    context
        .fill_text(
            &format!(
                "Stronger than {:.1}% of lifters | Tier {}",
                percentile * 100.0,
                tier
            ),
            80.0,
            320.0,
        )
        .map_err(|_| "Failed to render rank line.")?;

    context.set_fill_style_str("#d4c6a9");
    context.fill_rect(80.0, 370.0, 760.0, 14.0);
    context.set_fill_style_str("#f6f2e8");
    context.fill_rect(
        80.0,
        370.0,
        (percentile * 760.0).clamp(0.0, 760.0) as f64,
        14.0,
    );

    context.set_fill_style_str("#f6f2e8");
    context.set_font("400 24px 'IBM Plex Mono', monospace");
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
