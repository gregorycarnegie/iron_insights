// share_card.rs - SVG Share Card Generator for beautiful social media cards
use serde::{Deserialize, Serialize};

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
    let filled_width = (percentile / 100.0 * bar_width).max(10.0); // Min width for visibility

    let percentile_text = match data.percentile {
        Some(p) => format!("Top {:.1}%", 100.0 - p), // "Top 5%" sounds better than "95th Percentile"
        None => "Unranked".to_string(),
    };

    format!(
        r##"<svg width="{}" height="{}" viewBox="0 0 {} {}"
            xmlns="http://www.w3.org/2000/svg"
            preserveAspectRatio="xMidYMid meet">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#141E30;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#243B55;stop-opacity:1" />
    </linearGradient>
    <linearGradient id="cardShine" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" style="stop-color:#ffffff;stop-opacity:0" />
      <stop offset="50%" style="stop-color:#ffffff;stop-opacity:0.05" />
      <stop offset="100%" style="stop-color:#ffffff;stop-opacity:0" />
    </linearGradient>
    <filter id="glow">
      <feGaussianBlur stdDeviation="2.5" result="coloredBlur"/>
      <feMerge>
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    <filter id="shadow">
      <feDropShadow dx="0" dy="10" stdDeviation="15" flood-color="#000" flood-opacity="0.4"/>
    </filter>
  </defs>
  
  <!-- Background -->
  <rect width="100%" height="100%" fill="url(#bg)"/>
  
  <!-- Decorative Elements -->
  <circle cx="0" cy="0" r="300" fill="#4facfe" opacity="0.05"/>
  <circle cx="1000" cy="600" r="400" fill="#00f2fe" opacity="0.05"/>
  
  <!-- Main Card Container -->
  <rect x="50" y="50" width="900" height="500" rx="20" fill="rgba(255,255,255,0.03)" stroke="rgba(255,255,255,0.1)" stroke-width="1" filter="url(#shadow)"/>
  <rect x="50" y="50" width="900" height="500" rx="20" fill="url(#cardShine)"/>
  
  <!-- Header -->
  <text x="500" y="100" font-family="Inter, Arial, sans-serif" font-size="16" font-weight="600" letter-spacing="2" text-anchor="middle" fill="#4facfe">
    IRON INSIGHTS
  </text>
  
  <!-- Name -->
  <text x="500" y="150" font-family="Inter, Arial, sans-serif" font-size="48" font-weight="800" text-anchor="middle" fill="#ffffff">
    {}
  </text>
  
  <!-- Stats Grid -->
  <g transform="translate(0, 220)">
    <!-- Squat -->
    <g transform="translate(200, 0)">
      <text x="0" y="0" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#8899a6">SQUAT</text>
      <text x="0" y="40" font-family="Inter, Arial, sans-serif" font-size="36" font-weight="700" text-anchor="middle" fill="#ffffff">{}</text>
      <text x="0" y="65" font-family="Inter, Arial, sans-serif" font-size="12" text-anchor="middle" fill="#8899a6">kg</text>
    </g>
    
    <!-- Bench -->
    <g transform="translate(400, 0)">
      <text x="0" y="0" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#8899a6">BENCH</text>
      <text x="0" y="40" font-family="Inter, Arial, sans-serif" font-size="36" font-weight="700" text-anchor="middle" fill="#ffffff">{}</text>
      <text x="0" y="65" font-family="Inter, Arial, sans-serif" font-size="12" text-anchor="middle" fill="#8899a6">kg</text>
    </g>
    
    <!-- Deadlift -->
    <g transform="translate(600, 0)">
      <text x="0" y="0" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#8899a6">DEADLIFT</text>
      <text x="0" y="40" font-family="Inter, Arial, sans-serif" font-size="36" font-weight="700" text-anchor="middle" fill="#ffffff">{}</text>
      <text x="0" y="65" font-family="Inter, Arial, sans-serif" font-size="12" text-anchor="middle" fill="#8899a6">kg</text>
    </g>
    
    <!-- Total -->
    <g transform="translate(800, 0)">
      <text x="0" y="0" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#4facfe">TOTAL</text>
      <text x="0" y="40" font-family="Inter, Arial, sans-serif" font-size="36" font-weight="700" text-anchor="middle" fill="#4facfe" filter="url(#glow)">{}</text>
      <text x="0" y="65" font-family="Inter, Arial, sans-serif" font-size="12" text-anchor="middle" fill="#8899a6">kg</text>
    </g>
  </g>
  
  <!-- Separator -->
  <line x1="200" y1="320" x2="800" y2="320" stroke="#ffffff" stroke-opacity="0.1" stroke-width="1"/>
  
  <!-- Secondary Stats -->
  <g transform="translate(0, 360)">
    <!-- DOTS -->
    <g transform="translate(350, 0)">
      <text x="0" y="20" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#8899a6">DOTS SCORE</text>
      <text x="0" y="60" font-family="Inter, Arial, sans-serif" font-size="42" font-weight="800" text-anchor="middle" fill="#ffffff">{}</text>
    </g>
    
    <!-- Strength Level -->
    <g transform="translate(650, 0)">
      <text x="0" y="20" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#8899a6">STRENGTH LEVEL</text>
      <text x="0" y="60" font-family="Inter, Arial, sans-serif" font-size="32" font-weight="700" text-anchor="middle" fill="#00f2fe">{}</text>
    </g>
  </g>
  
  <!-- Percentile Bar -->
  <g transform="translate(350, 460)">
    <rect x="0" y="0" width="{}" height="6" rx="3" fill="#ffffff" opacity="0.1"/>
    <rect x="0" y="0" width="{}" height="6" rx="3" fill="url(#bg)">
      <stop offset="0%" style="stop-color:#4facfe;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#00f2fe;stop-opacity:1" />
    </rect>
    <text x="{}" y="25" font-family="Inter, Arial, sans-serif" font-size="12" font-weight="500" text-anchor="middle" fill="#8899a6">{}</text>
  </g>
  
  <!-- Footer -->
  <text x="500" y="520" font-family="Inter, Arial, sans-serif" font-size="12" text-anchor="middle" fill="#8899a6" opacity="0.7">
    {:.1}kg Bodyweight • {} Class • {}
  </text>

</svg>"##,
        card_width,
        card_height,
        card_width,
        card_height,
        data.name,
        squat_display,
        bench_display,
        deadlift_display,
        total_display,
        dots_display,
        data.strength_level,
        bar_width,
        filled_width,
        bar_width / 2.0,
        percentile_text,
        data.bodyweight,
        data.sex,
        data.lift_type.to_uppercase()
    )
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

    format!(
        r##"<svg width="{}" height="{}" viewBox="0 0 {} {}"
            xmlns="http://www.w3.org/2000/svg"
            preserveAspectRatio="xMidYMid meet">
  <defs>
    <linearGradient id="neonBg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#000000;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#1a1a1a;stop-opacity:1" />
    </linearGradient>
    <linearGradient id="neonGradient" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" style="stop-color:#ff00cc;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#333399;stop-opacity:1" />
    </linearGradient>
    <filter id="neonGlow">
      <feGaussianBlur stdDeviation="2.5" result="coloredBlur"/>
      <feMerge>
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    <filter id="cardShadow">
      <feDropShadow dx="0" dy="0" stdDeviation="20" flood-color="#ff00cc" flood-opacity="0.15"/>
    </filter>
  </defs>
  
  <!-- Background -->
  <rect width="100%" height="100%" fill="url(#neonBg)"/>
  
  <!-- Grid Lines -->
  <pattern id="grid" width="50" height="50" patternUnits="userSpaceOnUse">
    <path d="M 50 0 L 0 0 0 50" fill="none" stroke="#333" stroke-width="1" opacity="0.3"/>
  </pattern>
  <rect width="100%" height="100%" fill="url(#grid)" />
  
  <!-- Main Card Container -->
  <rect x="50" y="50" width="900" height="500" rx="20" fill="rgba(20, 20, 20, 0.8)" stroke="#333" stroke-width="1" filter="url(#cardShadow)"/>
  
  <!-- Header -->
  <text x="500" y="100" font-family="Inter, Arial, sans-serif" font-size="16" font-weight="600" letter-spacing="4" text-anchor="middle" fill="#ff00cc" filter="url(#neonGlow)">
    IRON INSIGHTS
  </text>
  
  <!-- Name -->
  <text x="500" y="150" font-family="Inter, Arial, sans-serif" font-size="48" font-weight="800" text-anchor="middle" fill="#ffffff" filter="url(#neonGlow)">
    {}
  </text>
  
  <!-- Stats Grid -->
  <g transform="translate(0, 220)">
    <!-- Squat -->
    <g transform="translate(200, 0)">
      <text x="0" y="0" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#666">SQUAT</text>
      <text x="0" y="40" font-family="Inter, Arial, sans-serif" font-size="36" font-weight="700" text-anchor="middle" fill="#ffffff">{}</text>
      <text x="0" y="65" font-family="Inter, Arial, sans-serif" font-size="12" text-anchor="middle" fill="#444">kg</text>
    </g>
    
    <!-- Bench -->
    <g transform="translate(400, 0)">
      <text x="0" y="0" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#666">BENCH</text>
      <text x="0" y="40" font-family="Inter, Arial, sans-serif" font-size="36" font-weight="700" text-anchor="middle" fill="#ffffff">{}</text>
      <text x="0" y="65" font-family="Inter, Arial, sans-serif" font-size="12" text-anchor="middle" fill="#444">kg</text>
    </g>
    
    <!-- Deadlift -->
    <g transform="translate(600, 0)">
      <text x="0" y="0" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#666">DEADLIFT</text>
      <text x="0" y="40" font-family="Inter, Arial, sans-serif" font-size="36" font-weight="700" text-anchor="middle" fill="#ffffff">{}</text>
      <text x="0" y="65" font-family="Inter, Arial, sans-serif" font-size="12" text-anchor="middle" fill="#444">kg</text>
    </g>
    
    <!-- Total -->
    <g transform="translate(800, 0)">
      <text x="0" y="0" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#ff00cc">TOTAL</text>
      <text x="0" y="40" font-family="Inter, Arial, sans-serif" font-size="36" font-weight="700" text-anchor="middle" fill="#ff00cc" filter="url(#neonGlow)">{}</text>
      <text x="0" y="65" font-family="Inter, Arial, sans-serif" font-size="12" text-anchor="middle" fill="#444">kg</text>
    </g>
  </g>
  
  <!-- Separator -->
  <line x1="200" y1="320" x2="800" y2="320" stroke="#333" stroke-width="1"/>
  
  <!-- Secondary Stats -->
  <g transform="translate(0, 360)">
    <!-- DOTS -->
    <g transform="translate(350, 0)">
      <text x="0" y="20" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#666">DOTS SCORE</text>
      <text x="0" y="60" font-family="Inter, Arial, sans-serif" font-size="42" font-weight="800" text-anchor="middle" fill="#ffffff">{}</text>
    </g>
    
    <!-- Strength Level -->
    <g transform="translate(650, 0)">
      <text x="0" y="20" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="600" text-anchor="middle" fill="#666">STRENGTH LEVEL</text>
      <text x="0" y="60" font-family="Inter, Arial, sans-serif" font-size="32" font-weight="700" text-anchor="middle" fill="#333399" filter="url(#neonGlow)">{}</text>
    </g>
  </g>
  
  <!-- Percentile Bar -->
  <g transform="translate(350, 460)">
    <rect x="0" y="0" width="{}" height="4" rx="2" fill="#333"/>
    <rect x="0" y="0" width="{}" height="4" rx="2" fill="url(#neonGradient)" filter="url(#neonGlow)"/>
    <text x="{}" y="25" font-family="Inter, Arial, sans-serif" font-size="12" font-weight="500" text-anchor="middle" fill="#666">{}</text>
  </g>
  
  <!-- Footer -->
  <text x="500" y="520" font-family="Inter, Arial, sans-serif" font-size="12" text-anchor="middle" fill="#444">
    {:.1}kg Bodyweight • {} Class • {}
  </text>

</svg>"##,
        card_width,
        card_height,
        card_width,
        card_height,
        data.name,
        squat_display,
        bench_display,
        deadlift_display,
        total_display,
        dots_display,
        data.strength_level,
        bar_width,
        filled_width,
        bar_width / 2.0,
        percentile_text,
        data.bodyweight,
        data.sex,
        data.lift_type.to_uppercase()
    )
}

fn generate_minimal_card(data: &ShareCardData) -> String {
    let card_width = 800;
    let card_height = 500;

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

    format!(
        r##"<svg width="{}" height="{}" viewBox="0 0 {} {}"
            xmlns="http://www.w3.org/2000/svg"
            preserveAspectRatio="xMidYMid meet">
  <rect width="100%" height="100%" fill="white" rx="8"/>
  <rect width="100%" height="100%" fill="none" stroke="#e5e5e5" stroke-width="2" rx="8"/>
  
  <line x1="60" y1="60" x2="740" y2="60" stroke="#333" stroke-width="2"/>
  
  <text x="60" y="40" font-family="Arial" font-size="18" fill="#666">
    Iron Insights
  </text>
  
  <text x="400" y="120" font-family="Arial" font-size="42" font-weight="bold" text-anchor="middle" fill="#333">
    {}
  </text>
  
  <!-- Clean Minimal Lifts Layout -->
  <g transform="translate(50, 160)">
    <text x="0" y="20" font-family="Arial" font-size="14" fill="#666" font-weight="600">SQUAT</text>
    <text x="0" y="45" font-family="Arial" font-size="36" fill="#333" font-weight="300">{}</text>
    
    <text x="180" y="20" font-family="Arial" font-size="14" fill="#666" font-weight="600">BENCH</text>
    <text x="180" y="45" font-family="Arial" font-size="36" fill="#333" font-weight="300">{}</text>
    
    <text x="360" y="20" font-family="Arial" font-size="14" fill="#666" font-weight="600">DEADLIFT</text>
    <text x="360" y="45" font-family="Arial" font-size="36" fill="#333" font-weight="300">{}</text>
    
    <text x="540" y="20" font-family="Arial" font-size="14" fill="#666" font-weight="600">TOTAL</text>
    <text x="540" y="45" font-family="Arial" font-size="36" fill="#333" font-weight="bold">{}</text>
  </g>
  
  <!-- Separator Line -->
  <line x1="60" y1="240" x2="740" y2="240" stroke="#eee" stroke-width="1"/>
  
  <!-- DOTS and Level -->
  <g transform="translate(150, 270)">
    <text x="0" y="20" font-family="Arial" font-size="14" fill="#666" font-weight="600">DOTS SCORE</text>
    <text x="0" y="50" font-family="Arial" font-size="42" fill="#333" font-weight="300">{}</text>
    
    <text x="300" y="20" font-family="Arial" font-size="14" fill="#666" font-weight="600">STRENGTH LEVEL</text>
    <text x="300" y="50" font-family="Arial" font-size="32" fill="#333" font-weight="600">{}</text>
  </g>
  
  <text x="60" y="420" font-family="Arial" font-size="12" fill="#999">
    Generated by Iron Insights • Powerlifting Analytics
  </text>

</svg>"##,
        card_width,
        card_height,
        card_width,
        card_height,
        data.name,
        squat_display,
        bench_display,
        deadlift_display,
        total_display,
        dots_display,
        data.strength_level
    )
}

fn generate_powerlifting_card(data: &ShareCardData) -> String {
    let card_width = 800;
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

    format!(
        r##"<svg width="{}" height="{}" viewBox="0 0 {} {}"
            xmlns="http://www.w3.org/2000/svg"
            preserveAspectRatio="xMidYMid meet">
  <defs>
    <linearGradient id="powerliftingBg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#2c1810;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#8b4513;stop-opacity:1" />
    </linearGradient>
  </defs>
  
  <rect width="100%" height="100%" fill="url(#powerliftingBg)" rx="15"/>
  <rect x="0" y="0" width="100%" height="100" fill="#d2691e" rx="15"/>
  
  <text x="50" y="55" font-family="Arial" font-size="36" font-weight="bold" fill="white">
    🏋️ POWERLIFTING MEET CARD
  </text>
  
  <text x="400" y="150" font-family="Arial" font-size="42" font-weight="bold" text-anchor="middle" fill="#fffaf0">
    {}
  </text>
  
  <rect x="50" y="180" width="700" height="360" fill="#fffaf0" rx="10" opacity="0.95"/>
  
  <g transform="translate(80, 220)">
    <rect x="0" y="0" width="640" height="40" fill="#8b4513"/>
    <text x="80" y="25" font-family="Arial" font-size="16" font-weight="bold" text-anchor="middle" fill="white">SQUAT</text>
    <text x="240" y="25" font-family="Arial" font-size="16" font-weight="bold" text-anchor="middle" fill="white">BENCH</text>
    <text x="400" y="25" font-family="Arial" font-size="16" font-weight="bold" text-anchor="middle" fill="white">DEADLIFT</text>
    <text x="560" y="25" font-family="Arial" font-size="16" font-weight="bold" text-anchor="middle" fill="white">TOTAL</text>
    
    <rect x="0" y="40" width="640" height="60" fill="#f5deb3" stroke="#8b4513"/>
    <text x="80" y="75" font-family="Arial" font-size="28" font-weight="bold" text-anchor="middle" fill="#2c1810">{}</text>
    <text x="240" y="75" font-family="Arial" font-size="28" font-weight="bold" text-anchor="middle" fill="#2c1810">{}</text>
    <text x="400" y="75" font-family="Arial" font-size="28" font-weight="bold" text-anchor="middle" fill="#2c1810">{}</text>
    <text x="560" y="75" font-family="Arial" font-size="28" font-weight="bold" text-anchor="middle" fill="#2c1810">{}</text>
    
    <rect x="0" y="120" width="640" height="80" fill="#2c1810"/>
    <text x="320" y="145" font-family="Arial" font-size="18" font-weight="bold" text-anchor="middle" fill="#d2691e">
      DOTS SCORE
    </text>
    <text x="320" y="185" font-family="Arial" font-size="48" font-weight="bold" text-anchor="middle" fill="#fffaf0">
      {}
    </text>
    
    <rect x="0" y="220" width="640" height="60" fill="#d2691e"/>
    <text x="320" y="240" font-family="Arial" font-size="16" font-weight="bold" text-anchor="middle" fill="white">
      STRENGTH CLASSIFICATION
    </text>
    <text x="320" y="265" font-family="Arial" font-size="32" font-weight="bold" text-anchor="middle" fill="white">
      {}
    </text>
  </g>
  
  <text x="400" y="575" font-family="Arial" font-size="14" text-anchor="middle" fill="#fffaf0" opacity="0.8">
    Generated by Iron Insights • Bodyweight: {:.1}kg • {}-Class
  </text>
</svg>"##,
        card_width,
        card_height,
        card_width,
        card_height,
        data.name,
        squat_display,
        bench_display,
        deadlift_display,
        total_display,
        dots_display,
        data.strength_level,
        data.bodyweight,
        data.sex,
    )
}
