// share_card.rs - SVG Share Card Generator for beautiful social media cards
use serde::{Deserialize, Serialize};
use svg::Document;

use svg::node::element::{
    Circle, Definitions, Filter, FilterEffectDropShadow, FilterEffectGaussianBlur,
    FilterEffectMerge, FilterEffectMergeNode, Group, Line, LinearGradient, Path, Pattern,
    Rectangle, Stop, Text,
};

fn create_text(
    text: &str,
    x: i32,
    y: i32,
    font_family: &str,
    font_size: i32,
    font_weight: u32,
    fill: &str,
) -> Text {
    Text::new(text)
        .set("x", x)
        .set("y", y)
        .set("font-family", font_family)
        .set("font-size", font_size)
        .set("font-weight", font_weight)
        .set("fill", fill)
}

fn create_centered_text(
    text: &str,
    x: i32,
    y: i32,
    font_family: &str,
    font_size: i32,
    font_weight: u32,
    fill: &str,
) -> Text {
    create_text(text, x, y, font_family, font_size, font_weight, fill).set("text-anchor", "middle")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareCardData {
    pub name: String,
    pub bodyweight: f32,
    pub squat: Option<f32>,
    pub bench: Option<f32>,
    pub deadlift: Option<f32>,
    pub total: Option<f32>,
    pub dots_score: Option<f32>,
    pub strength_level: String,
    pub percentile: Option<f32>,
    pub lift_type: String,
    pub sex: String,
}

pub enum CardTheme {
    Default,
    Dark,
    Minimal,
    Powerlifting,
}

pub fn generate_themed_share_card_svg(data: &ShareCardData, theme: CardTheme) -> String {
    match theme {
        CardTheme::Default => generate_default_card(data),
        CardTheme::Dark => generate_dark_card(data),
        CardTheme::Minimal => generate_minimal_card(data),
        CardTheme::Powerlifting => generate_powerlifting_card(data),
    }
}

fn generate_default_card(data: &ShareCardData) -> String {
    let card_width = 1000;
    let card_height = 600;

    // Format all lift values
    let squat_display = data
        .squat
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let bench_display = data
        .bench
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let deadlift_display = data
        .deadlift
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let total_display = data
        .total
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let dots_display = data
        .dots_score
        .map(|d| format!("{:.1}", d))
        .unwrap_or_else(|| "-".to_string());

    // Percentile Bar Calculation
    let percentile = data.percentile.unwrap_or(0.0);
    let bar_width = 300.0;
    let filled_width = (percentile / 100.0 * bar_width).max(10.0);
    let percentile_text = match data.percentile {
        Some(p) => format!("Top {:.1}%", 100.0 - p),
        None => "Unranked".to_string(),
    };

    let mut doc = Document::new()
        .set("width", card_width)
        .set("height", card_height)
        .set("viewBox", (0, 0, card_width, card_height))
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("preserveAspectRatio", "xMidYMid meet");

    // Definitions
    let defs = Definitions::new()
        .add(
            LinearGradient::new()
                .set("id", "bg")
                .set("x1", "0%")
                .set("y1", "0%")
                .set("x2", "100%")
                .set("y2", "100%")
                .add(
                    Stop::new()
                        .set("offset", "0%")
                        .set("style", "stop-color:#141E30;stop-opacity:1"),
                )
                .add(
                    Stop::new()
                        .set("offset", "100%")
                        .set("style", "stop-color:#243B55;stop-opacity:1"),
                ),
        )
        .add(
            LinearGradient::new()
                .set("id", "cardShine")
                .set("x1", "0%")
                .set("y1", "0%")
                .set("x2", "100%")
                .set("y2", "0%")
                .add(
                    Stop::new()
                        .set("offset", "0%")
                        .set("style", "stop-color:#ffffff;stop-opacity:0"),
                )
                .add(
                    Stop::new()
                        .set("offset", "50%")
                        .set("style", "stop-color:#ffffff;stop-opacity:0.05"),
                )
                .add(
                    Stop::new()
                        .set("offset", "100%")
                        .set("style", "stop-color:#ffffff;stop-opacity:0"),
                ),
        )
        .add(
            Filter::new()
                .set("id", "glow")
                .add(
                    FilterEffectGaussianBlur::new()
                        .set("stdDeviation", 2.5)
                        .set("result", "coloredBlur"),
                )
                .add(
                    FilterEffectMerge::new()
                        .add(FilterEffectMergeNode::new().set("in", "coloredBlur"))
                        .add(FilterEffectMergeNode::new().set("in", "SourceGraphic")),
                ),
        )
        .add(
            Filter::new().set("id", "shadow").add(
                FilterEffectDropShadow::new()
                    .set("dx", 0)
                    .set("dy", 10)
                    .set("stdDeviation", 15)
                    .set("flood-color", "#000")
                    .set("flood-opacity", 0.4),
            ),
        );

    doc = doc.add(defs);

    // Background
    doc = doc.add(
        Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", "url(#bg)"),
    );

    // Decorative Elements
    doc = doc.add(
        Circle::new()
            .set("cx", 0)
            .set("cy", 0)
            .set("r", 300)
            .set("fill", "#4facfe")
            .set("opacity", 0.05),
    );
    doc = doc.add(
        Circle::new()
            .set("cx", 1000)
            .set("cy", 600)
            .set("r", 400)
            .set("fill", "#00f2fe")
            .set("opacity", 0.05),
    );

    // Main Card Container
    doc = doc.add(
        Rectangle::new()
            .set("x", 50)
            .set("y", 50)
            .set("width", 900)
            .set("height", 500)
            .set("rx", 20)
            .set("fill", "rgba(255,255,255,0.03)")
            .set("stroke", "rgba(255,255,255,0.1)")
            .set("stroke-width", 1)
            .set("filter", "url(#shadow)"),
    );
    doc = doc.add(
        Rectangle::new()
            .set("x", 50)
            .set("y", 50)
            .set("width", 900)
            .set("height", 500)
            .set("rx", 20)
            .set("fill", "url(#cardShine)"),
    );

    // Header
    doc = doc.add(create_centered_text(
        "IRON INSIGHTS",
        500,
        50,
        "Inter, Arial, sans-serif",
        14,
        600,
        "#666",
    ));

    // Name
    doc = doc.add(
        create_centered_text(
            data.name.as_str(),
            500,
            150,
            "Inter, Arial, sans-serif",
            48,
            800,
            "#ffffff",
        )
        .set("filter", "url(#glow)"),
    );
    // Stats Grid
    let mut stats_grid = Group::new().set("transform", "translate(0, 220)");

    // Helper to create lift group
    let create_lift_group =
        |x: i32, label: &str, value: &str, unit: &str, color: &str, filter: Option<&str>| {
            let mut g = Group::new().set("transform", format!("translate({}, 0)", x));
            g = g.add(
                Text::new(label)
                    .set("x", 0)
                    .set("y", 0)
                    .set("font-family", "Inter, Arial, sans-serif")
                    .set("font-size", 14)
                    .set("font-weight", 600)
                    .set("text-anchor", "middle")
                    .set("fill", if color == "#ffffff" { "#8899a6" } else { color }),
            );
            let mut val_text = Text::new(value)
                .set("x", 0)
                .set("y", 40)
                .set("font-family", "Inter, Arial, sans-serif")
                .set("font-size", 36)
                .set("font-weight", 700)
                .set("text-anchor", "middle")
                .set("fill", color);
            if let Some(f) = filter {
                val_text = val_text.set("filter", f);
            }
            g = g.add(val_text);
            g = g.add(
                Text::new(unit)
                    .set("x", 0)
                    .set("y", 65)
                    .set("font-family", "Inter, Arial, sans-serif")
                    .set("font-size", 12)
                    .set("text-anchor", "middle")
                    .set("fill", "#8899a6"),
            );
            g
        };

    stats_grid = stats_grid.add(create_lift_group(
        200,
        "SQUAT",
        &squat_display,
        "kg",
        "#ffffff",
        None,
    ));
    stats_grid = stats_grid.add(create_lift_group(
        400,
        "BENCH",
        &bench_display,
        "kg",
        "#ffffff",
        None,
    ));
    stats_grid = stats_grid.add(create_lift_group(
        600,
        "DEADLIFT",
        &deadlift_display,
        "kg",
        "#ffffff",
        None,
    ));
    stats_grid = stats_grid.add(create_lift_group(
        800,
        "TOTAL",
        &total_display,
        "kg",
        "#4facfe",
        Some("url(#glow)"),
    ));

    doc = doc.add(stats_grid);

    // Separator
    doc = doc.add(
        Line::new()
            .set("x1", 200)
            .set("y1", 320)
            .set("x2", 800)
            .set("y2", 320)
            .set("stroke", "#ffffff")
            .set("stroke-opacity", 0.1)
            .set("stroke-width", 1),
    );

    // Secondary Stats
    let mut sec_stats = Group::new().set("transform", "translate(0, 360)");

    // DOTS
    let mut dots_group = Group::new().set("transform", "translate(350, 0)");
    dots_group = dots_group.add(create_centered_text(
        "DOTS SCORE",
        0,
        20,
        "Inter, Arial, sans-serif",
        14,
        600,
        "#666",
    ));
    dots_group = dots_group.add(create_centered_text(
        &dots_display,
        0,
        60,
        "Inter, Arial, sans-serif",
        42,
        800,
        "#ffffff",
    ));
    sec_stats = sec_stats.add(dots_group);

    // Strength Level
    let mut level_group = Group::new().set("transform", "translate(650, 0)");
    level_group = level_group.add(create_centered_text(
        "STRENGTH LEVEL",
        0,
        20,
        "Inter, Arial, sans-serif",
        14,
        600,
        "#666",
    ));
    level_group = level_group.add(
        create_centered_text(
            data.strength_level.as_str(),
            0,
            60,
            "Inter, Arial, sans-serif",
            32,
            700,
            "#00f2fe",
        )
        .set("filter", "url(#glow)"),
    );
    sec_stats = sec_stats.add(level_group);

    doc = doc.add(sec_stats);

    // Percentile Bar
    let mut perc_group = Group::new().set("transform", "translate(350, 460)");
    perc_group = perc_group.add(
        Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", bar_width)
            .set("height", 6)
            .set("rx", 3)
            .set("fill", "#ffffff")
            .set("opacity", 0.1),
    );

    let filled_bar = Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", filled_width)
        .set("height", 6)
        .set("rx", 3)
        .set("fill", "url(#bg)");

    perc_group = perc_group.add(filled_bar.set("fill", "url(#barGradient)"));
    perc_group = perc_group.add(create_centered_text(
        &percentile_text,
        (bar_width / 2.0) as i32,
        25,
        "Inter, Arial, sans-serif",
        12,
        500,
        "#8899a6",
    ));

    doc = doc.add(perc_group);

    // Footer
    let footer_text = format!(
        "{:.1}kg Bodyweight • {} Class • {}",
        data.bodyweight,
        data.sex,
        data.lift_type.to_uppercase()
    );
    doc = doc.add(
        create_centered_text(
            &footer_text,
            500,
            520,
            "Inter, Arial, sans-serif",
            12,
            400,
            "#8899a6",
        )
        .set("opacity", 0.7),
    );

    doc.to_string()
}

fn generate_dark_card(data: &ShareCardData) -> String {
    let card_width = 1000;
    let card_height = 600;

    // Format all lift values
    let squat_display = data
        .squat
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let bench_display = data
        .bench
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let deadlift_display = data
        .deadlift
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let total_display = data
        .total
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let dots_display = data
        .dots_score
        .map(|d| format!("{:.1}", d))
        .unwrap_or_else(|| "-".to_string());

    // Percentile Bar Calculation
    let percentile = data.percentile.unwrap_or(0.0);
    let bar_width = 300.0;
    let filled_width = (percentile / 100.0 * bar_width).max(10.0);
    let percentile_text = match data.percentile {
        Some(p) => format!("Top {:.1}%", 100.0 - p),
        None => "Unranked".to_string(),
    };

    let mut doc = Document::new()
        .set("width", card_width)
        .set("height", card_height)
        .set("viewBox", (0, 0, card_width, card_height))
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("preserveAspectRatio", "xMidYMid meet");

    // Definitions
    let defs = Definitions::new()
        .add(
            LinearGradient::new()
                .set("id", "neonBg")
                .set("x1", "0%")
                .set("y1", "0%")
                .set("x2", "100%")
                .set("y2", "100%")
                .add(
                    Stop::new()
                        .set("offset", "0%")
                        .set("style", "stop-color:#000000;stop-opacity:1"),
                )
                .add(
                    Stop::new()
                        .set("offset", "100%")
                        .set("style", "stop-color:#1a1a1a;stop-opacity:1"),
                ),
        )
        .add(
            LinearGradient::new()
                .set("id", "neonGradient")
                .set("x1", "0%")
                .set("y1", "0%")
                .set("x2", "100%")
                .set("y2", "0%")
                .add(
                    Stop::new()
                        .set("offset", "0%")
                        .set("style", "stop-color:#ff00cc;stop-opacity:1"),
                )
                .add(
                    Stop::new()
                        .set("offset", "100%")
                        .set("style", "stop-color:#333399;stop-opacity:1"),
                ),
        )
        .add(
            Filter::new()
                .set("id", "neonGlow")
                .add(
                    FilterEffectGaussianBlur::new()
                        .set("stdDeviation", 2.5)
                        .set("result", "coloredBlur"),
                )
                .add(
                    FilterEffectMerge::new()
                        .add(FilterEffectMergeNode::new().set("in", "coloredBlur"))
                        .add(FilterEffectMergeNode::new().set("in", "SourceGraphic")),
                ),
        )
        .add(
            Filter::new().set("id", "cardShadow").add(
                FilterEffectDropShadow::new()
                    .set("dx", 0)
                    .set("dy", 0)
                    .set("stdDeviation", 20)
                    .set("flood-color", "#ff00cc")
                    .set("flood-opacity", 0.15),
            ),
        )
        .add(
            Pattern::new()
                .set("id", "grid")
                .set("width", 50)
                .set("height", 50)
                .set("patternUnits", "userSpaceOnUse")
                .add(
                    Path::new()
                        .set("d", "M 50 0 L 0 0 0 50")
                        .set("fill", "none")
                        .set("stroke", "#333")
                        .set("stroke-width", 1)
                        .set("opacity", 0.3),
                ),
        );

    doc = doc.add(defs);

    // Background
    doc = doc.add(
        Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", "url(#neonBg)"),
    );

    // Grid Lines
    doc = doc.add(
        Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", "url(#grid)"),
    );

    // Main Card Container
    doc = doc.add(
        Rectangle::new()
            .set("x", 50)
            .set("y", 50)
            .set("width", 900)
            .set("height", 500)
            .set("rx", 20)
            .set("fill", "rgba(20, 20, 20, 0.8)")
            .set("stroke", "#333")
            .set("stroke-width", 1)
            .set("filter", "url(#cardShadow)"),
    );

    // Header
    doc = doc.add(
        Text::new("IRON INSIGHTS")
            .set("x", 500)
            .set("y", 100)
            .set("font-family", "Inter, Arial, sans-serif")
            .set("font-size", 16)
            .set("font-weight", 600)
            .set("letter-spacing", 4)
            .set("text-anchor", "middle")
            .set("fill", "#ff00cc")
            .set("filter", "url(#neonGlow)"),
    );

    // Name
    doc = doc.add(
        create_centered_text(
            data.name.as_str(),
            500,
            150,
            "Inter, Arial, sans-serif",
            48,
            800,
            "#ffffff",
        )
        .set("filter", "url(#neonGlow)"),
    );

    // Stats Grid
    let mut stats_grid = Group::new().set("transform", "translate(0, 220)");

    let create_lift_group =
        |x: i32, label: &str, value: &str, unit: &str, color: &str, filter: Option<&str>| {
            let mut g = Group::new().set("transform", format!("translate({}, 0)", x));
            g = g.add(create_centered_text(
                label,
                0,
                0,
                "Inter, Arial, sans-serif",
                14,
                600,
                "#666",
            ));
            let mut val_text =
                create_centered_text(value, 0, 40, "Inter, Arial, sans-serif", 36, 700, color);
            if let Some(f) = filter {
                val_text = val_text.set("filter", f);
            }
            g = g.add(val_text);
            g = g.add(create_centered_text(
                unit,
                0,
                65,
                "Inter, Arial, sans-serif",
                12,
                400, // Default weight for unit wasn't specified but likely normal/400 or same as others. Checking original... it didn't set weight, so default.
                "#444",
            ));
            g
        };

    stats_grid = stats_grid.add(create_lift_group(
        200,
        "SQUAT",
        &squat_display,
        "kg",
        "#ffffff",
        None,
    ));
    stats_grid = stats_grid.add(create_lift_group(
        400,
        "BENCH",
        &bench_display,
        "kg",
        "#ffffff",
        None,
    ));
    stats_grid = stats_grid.add(create_lift_group(
        600,
        "DEADLIFT",
        &deadlift_display,
        "kg",
        "#ffffff",
        None,
    ));
    stats_grid = stats_grid.add(create_lift_group(
        800,
        "TOTAL",
        &total_display,
        "kg",
        "#ff00cc",
        Some("url(#neonGlow)"),
    ));

    doc = doc.add(stats_grid);

    // Separator
    doc = doc.add(
        Line::new()
            .set("x1", 200)
            .set("y1", 320)
            .set("x2", 800)
            .set("y2", 320)
            .set("stroke", "#333")
            .set("stroke-width", 1),
    );

    // Secondary Stats
    let mut sec_stats = Group::new().set("transform", "translate(0, 360)");

    // DOTS
    let mut dots_group = Group::new().set("transform", "translate(350, 0)");
    dots_group = dots_group.add(create_centered_text(
        "DOTS SCORE",
        0,
        20,
        "Inter, Arial, sans-serif",
        14,
        600,
        "#666",
    ));
    dots_group = dots_group.add(create_centered_text(
        &dots_display,
        0,
        60,
        "Inter, Arial, sans-serif",
        42,
        800,
        "#ffffff",
    ));
    sec_stats = sec_stats.add(dots_group);

    // Strength Level
    let mut level_group = Group::new().set("transform", "translate(650, 0)");
    level_group = level_group.add(create_centered_text(
        "STRENGTH LEVEL",
        0,
        20,
        "Inter, Arial, sans-serif",
        14,
        600,
        "#666",
    ));
    level_group = level_group.add(
        create_centered_text(
            data.strength_level.as_str(),
            0,
            60,
            "Inter, Arial, sans-serif",
            32,
            700,
            "#333399",
        )
        .set("filter", "url(#neonGlow)"),
    );
    sec_stats = sec_stats.add(level_group);

    doc = doc.add(sec_stats);

    // Percentile Bar
    let mut perc_group = Group::new().set("transform", "translate(350, 460)");
    perc_group = perc_group.add(
        Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", bar_width)
            .set("height", 4)
            .set("rx", 2)
            .set("fill", "#333"),
    );
    perc_group = perc_group.add(
        Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", filled_width)
            .set("height", 4)
            .set("rx", 2)
            .set("fill", "url(#neonGradient)")
            .set("filter", "url(#neonGlow)"),
    );
    perc_group = perc_group.add(create_centered_text(
        &percentile_text,
        (bar_width / 2.0) as i32,
        25,
        "Inter, Arial, sans-serif",
        12,
        500,
        "#666",
    ));

    doc = doc.add(perc_group);

    // Footer
    let footer_text = format!(
        "{:.1}kg Bodyweight • {} Class • {}",
        data.bodyweight,
        data.sex,
        data.lift_type.to_uppercase()
    );
    doc = doc.add(create_centered_text(
        &footer_text,
        500,
        520,
        "Inter, Arial, sans-serif",
        12,
        400,
        "#444",
    ));

    doc.to_string()
}

fn generate_minimal_card(data: &ShareCardData) -> String {
    let card_width = 1000;
    let card_height = 600;

    // Format all lift values
    let squat_display = data
        .squat
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let bench_display = data
        .bench
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let deadlift_display = data
        .deadlift
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let total_display = data
        .total
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let dots_display = data
        .dots_score
        .map(|d| format!("{:.1}", d))
        .unwrap_or_else(|| "-".to_string());

    let mut doc = Document::new()
        .set("width", card_width)
        .set("height", card_height)
        .set("viewBox", (0, 0, card_width, card_height))
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("preserveAspectRatio", "xMidYMid meet");

    // Background
    doc = doc.add(
        Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", "#ffffff"),
    );

    // Main Content Container
    let mut main_group = Group::new().set("transform", "translate(50, 50)");

    // Header
    main_group = main_group.add(
        create_text(
            "IRON INSIGHTS",
            0,
            30,
            "Helvetica Neue, Helvetica, Arial, sans-serif",
            14,
            700,
            "#000000",
        )
        .set("letter-spacing", 2),
    );

    // Name
    main_group = main_group.add(
        create_text(
            data.name.as_str(),
            0,
            120,
            "Helvetica Neue, Helvetica, Arial, sans-serif",
            64,
            300,
            "#000000",
        )
        .set("letter-spacing", -1),
    );

    // Separator
    main_group = main_group.add(
        Line::new()
            .set("x1", 0)
            .set("y1", 160)
            .set("x2", 900)
            .set("y2", 160)
            .set("stroke", "#000000")
            .set("stroke-width", 2),
    );

    // Lifts Grid
    let mut lifts_grid = Group::new().set("transform", "translate(0, 240)");

    let create_lift_group = |x: i32,
                             label: &str,
                             value: &str,
                             unit: &str,
                             label_color: &str,
                             value_color: &str,
                             value_weight: u32| {
        let mut g = Group::new().set("transform", format!("translate({}, 0)", x));
        g = g.add(
            create_text(
                label,
                0,
                0,
                "Helvetica Neue, Helvetica, Arial, sans-serif",
                12,
                700,
                label_color,
            )
            .set("letter-spacing", 1),
        );
        g = g.add(create_text(
            value,
            0,
            50,
            "Helvetica Neue, Helvetica, Arial, sans-serif",
            48,
            value_weight,
            value_color,
        ));
        g = g.add(create_text(
            unit,
            100,
            50,
            "Helvetica Neue, Helvetica, Arial, sans-serif",
            14,
            400, // Default weight
            "#999999",
        ));
        g
    };

    lifts_grid = lifts_grid.add(create_lift_group(
        0,
        "SQUAT",
        &squat_display,
        "kg",
        "#666666",
        "#000000",
        400,
    ));
    lifts_grid = lifts_grid.add(create_lift_group(
        225,
        "BENCH",
        &bench_display,
        "kg",
        "#666666",
        "#000000",
        400,
    ));
    lifts_grid = lifts_grid.add(create_lift_group(
        450,
        "DEADLIFT",
        &deadlift_display,
        "kg",
        "#666666",
        "#000000",
        400,
    ));
    lifts_grid = lifts_grid.add(create_lift_group(
        675,
        "TOTAL",
        &total_display,
        "kg",
        "#000000",
        "#000000",
        700,
    ));

    main_group = main_group.add(lifts_grid);

    // Secondary Stats
    let mut sec_stats = Group::new().set("transform", "translate(0, 380)");

    let create_sec_group = |x: i32, label: &str, value: &str, suffix: &str| {
        let mut g = Group::new().set("transform", format!("translate({}, 0)", x));
        g = g.add(
            create_text(
                label,
                0,
                0,
                "Helvetica Neue, Helvetica, Arial, sans-serif",
                12,
                700,
                "#666666",
            )
            .set("letter-spacing", 1),
        );
        g = g.add(create_text(
            &format!("{}{}", value, suffix),
            0,
            40,
            "Helvetica Neue, Helvetica, Arial, sans-serif",
            32,
            300,
            "#000000",
        ));
        g
    };

    sec_stats = sec_stats.add(create_sec_group(0, "DOTS SCORE", &dots_display, ""));
    sec_stats = sec_stats.add(create_sec_group(
        225,
        "CLASSIFICATION",
        &data.strength_level,
        "",
    ));
    sec_stats = sec_stats.add(create_sec_group(
        450,
        "BODYWEIGHT",
        &format!("{:.1}", data.bodyweight),
        "kg",
    ));

    main_group = main_group.add(sec_stats);

    // Footer
    let footer_text = format!("Generated on {}", chrono::Local::now().format("%B %d, %Y"));
    main_group = main_group.add(
        Text::new(footer_text)
            .set("x", 0)
            .set("y", 500)
            .set(
                "font-family",
                "Helvetica Neue, Helvetica, Arial, sans-serif",
            )
            .set("font-size", 12)
            .set("fill", "#999999"),
    );

    doc = doc.add(main_group);

    doc.to_string()
}

fn generate_powerlifting_card(data: &ShareCardData) -> String {
    let card_width = 1000;
    let card_height = 600;

    let squat_display = data
        .squat
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let bench_display = data
        .bench
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let deadlift_display = data
        .deadlift
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let total_display = data
        .total
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "-".to_string());
    let dots_display = data
        .dots_score
        .map(|d| format!("{:.1}", d))
        .unwrap_or_else(|| "-".to_string());

    let mut doc = Document::new()
        .set("width", card_width)
        .set("height", card_height)
        .set("viewBox", (0, 0, card_width, card_height))
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("preserveAspectRatio", "xMidYMid meet");

    // Defs
    let defs = Definitions::new()
        .add(
            LinearGradient::new()
                .set("id", "goldGradient")
                .set("x1", "0%")
                .set("y1", "0%")
                .set("x2", "100%")
                .set("y2", "100%")
                .add(
                    Stop::new()
                        .set("offset", "0%")
                        .set("style", "stop-color:#D4AF37;stop-opacity:1"),
                )
                .add(
                    Stop::new()
                        .set("offset", "50%")
                        .set("style", "stop-color:#F7EF8A;stop-opacity:1"),
                )
                .add(
                    Stop::new()
                        .set("offset", "100%")
                        .set("style", "stop-color:#D4AF37;stop-opacity:1"),
                ),
        )
        .add(
            Pattern::new()
                .set("id", "paper")
                .set("width", 100)
                .set("height", 100)
                .set("patternUnits", "userSpaceOnUse")
                .add(
                    Rectangle::new()
                        .set("width", 100)
                        .set("height", 100)
                        .set("fill", "#fdfbf7"),
                )
                .add(
                    Path::new()
                        .set("d", "M0 0h100v100H0z")
                        .set("fill", "#000")
                        .set("fill-opacity", 0.02),
                ),
        );
    doc = doc.add(defs);

    // Background
    doc = doc.add(
        Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", "url(#paper)"),
    );

    // Border Frame
    doc = doc.add(
        Rectangle::new()
            .set("x", 20)
            .set("y", 20)
            .set("width", 960)
            .set("height", 560)
            .set("fill", "none")
            .set("stroke", "#8B0000")
            .set("stroke-width", 4),
    );
    doc = doc.add(
        Rectangle::new()
            .set("x", 30)
            .set("y", 30)
            .set("width", 940)
            .set("height", 540)
            .set("fill", "none")
            .set("stroke", "#D4AF37")
            .set("stroke-width", 2),
    );

    // Corner Ornaments
    doc = doc.add(
        Path::new()
            .set("d", "M20 20 L80 20 L80 25 L25 25 L25 80 L20 80 Z")
            .set("fill", "#8B0000"),
    );
    doc = doc.add(
        Path::new()
            .set("d", "M980 20 L920 20 L920 25 L975 25 L975 80 L980 80 Z")
            .set("fill", "#8B0000"),
    );
    doc = doc.add(
        Path::new()
            .set("d", "M20 580 L80 580 L80 575 L25 575 L25 520 L20 520 Z")
            .set("fill", "#8B0000"),
    );
    doc = doc.add(
        Path::new()
            .set(
                "d",
                "M980 580 L920 580 L920 575 L975 575 L975 520 L980 520 Z",
            )
            .set("fill", "#8B0000"),
    );

    // Header
    doc = doc.add(
        create_centered_text(
            "OFFICIAL LIFTING RECORD",
            500,
            80,
            "Times New Roman, serif",
            24,
            700,
            "#8B0000",
        )
        .set("letter-spacing", 2),
    );

    // Name
    doc = doc.add(create_centered_text(
        data.name.as_str(),
        500,
        140,
        "Times New Roman, serif",
        56,
        700,
        "#000000",
    ));

    // Divider
    doc = doc.add(
        Path::new()
            .set("d", "M300 160 L700 160")
            .set("stroke", "#D4AF37")
            .set("stroke-width", 2),
    );
    doc = doc.add(
        Circle::new()
            .set("cx", 500)
            .set("cy", 160)
            .set("r", 5)
            .set("fill", "#8B0000"),
    );

    // Stats Table
    let mut stats_group = Group::new().set("transform", "translate(100, 220)");

    // Headers
    stats_group = stats_group.add(
        Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", 800)
            .set("height", 40)
            .set("fill", "#8B0000"),
    );

    let create_header_text = |x: i32, text: &str| {
        create_centered_text(text, x, 28, "Arial, sans-serif", 16, 700, "#FFFFFF")
    };
    stats_group = stats_group.add(create_header_text(100, "SQUAT"));
    stats_group = stats_group.add(create_header_text(300, "BENCH PRESS"));
    stats_group = stats_group.add(create_header_text(500, "DEADLIFT"));
    stats_group = stats_group.add(create_header_text(700, "TOTAL"));

    // Values
    stats_group = stats_group.add(
        Rectangle::new()
            .set("x", 0)
            .set("y", 40)
            .set("width", 800)
            .set("height", 80)
            .set("fill", "#f5f5f5")
            .set("stroke", "#cccccc"),
    );
    stats_group = stats_group.add(
        Line::new()
            .set("x1", 200)
            .set("y1", 40)
            .set("x2", 200)
            .set("y2", 120)
            .set("stroke", "#cccccc"),
    );
    stats_group = stats_group.add(
        Line::new()
            .set("x1", 400)
            .set("y1", 40)
            .set("x2", 400)
            .set("y2", 120)
            .set("stroke", "#cccccc"),
    );
    stats_group = stats_group.add(
        Line::new()
            .set("x1", 600)
            .set("y1", 40)
            .set("x2", 600)
            .set("y2", 120)
            .set("stroke", "#cccccc"),
    );

    let create_value_text = |x: i32, text: &str, fill: &str| {
        create_centered_text(text, x, 90, "Times New Roman, serif", 36, 700, fill)
    };
    stats_group = stats_group.add(create_value_text(100, &squat_display, "#000000"));
    stats_group = stats_group.add(create_value_text(300, &bench_display, "#000000"));
    stats_group = stats_group.add(create_value_text(500, &deadlift_display, "#000000"));
    stats_group = stats_group.add(create_value_text(700, &total_display, "#8B0000"));

    doc = doc.add(stats_group);

    // Secondary Stats Box
    let mut sec_group = Group::new().set("transform", "translate(250, 360)");
    sec_group = sec_group.add(
        Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", 500)
            .set("height", 100)
            .set("fill", "none")
            .set("stroke", "#D4AF37")
            .set("stroke-width", 2),
    );
    sec_group = sec_group.add(
        Rectangle::new()
            .set("x", 5)
            .set("y", 5)
            .set("width", 490)
            .set("height", 90)
            .set("fill", "none")
            .set("stroke", "#8B0000")
            .set("stroke-width", 1),
    );

    // DOTS
    sec_group = sec_group.add(
        Text::new("DOTS SCORE")
            .set("x", 125)
            .set("y", 40)
            .set("font-family", "Arial, sans-serif")
            .set("font-size", 14)
            .set("font-weight", "bold")
            .set("text-anchor", "middle")
            .set("fill", "#666666"),
    );
    sec_group = sec_group.add(
        Text::new(dots_display)
            .set("x", 125)
            .set("y", 80)
            .set("font-family", "Times New Roman, serif")
            .set("font-size", 32)
            .set("font-weight", "bold")
            .set("text-anchor", "middle")
            .set("fill", "#000000"),
    );

    // Divider
    sec_group = sec_group.add(
        Line::new()
            .set("x1", 250)
            .set("y1", 20)
            .set("x2", 250)
            .set("y2", 80)
            .set("stroke", "#cccccc"),
    );

    // Class
    sec_group = sec_group.add(
        Text::new("CLASSIFICATION")
            .set("x", 375)
            .set("y", 40)
            .set("font-family", "Arial, sans-serif")
            .set("font-size", 14)
            .set("font-weight", "bold")
            .set("text-anchor", "middle")
            .set("fill", "#666666"),
    );
    sec_group = sec_group.add(
        Text::new(data.strength_level.as_str())
            .set("x", 375)
            .set("y", 80)
            .set("font-family", "Times New Roman, serif")
            .set("font-size", 28)
            .set("font-weight", "bold")
            .set("text-anchor", "middle")
            .set("fill", "#8B0000"),
    );

    doc = doc.add(sec_group);

    // Footer
    let footer_text = format!(
        "Certified by Iron Insights • Bodyweight: {:.1}kg • {} Class • Date: {}",
        data.bodyweight,
        data.sex,
        chrono::Local::now().format("%Y-%m-%d")
    );
    doc = doc.add(
        Text::new(footer_text)
            .set("x", 500)
            .set("y", 540)
            .set("font-family", "Times New Roman, serif")
            .set("font-size", 14)
            .set("font-style", "italic")
            .set("text-anchor", "middle")
            .set("fill", "#666666"),
    );

    doc.to_string()
}
