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

    format!(
        r##"<svg width="{}" height="{}" viewBox="0 0 {} {}"
            xmlns="http://www.w3.org/2000/svg"
            preserveAspectRatio="xMidYMid meet">
  <!-- Background -->
  <rect width="100%" height="100%" fill="#ffffff"/>
  
  <!-- Main Content Container -->
  <g transform="translate(50, 50)">
    
    <!-- Header -->
    <text x="0" y="30" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="14" font-weight="700" letter-spacing="2" fill="#000000">
      IRON INSIGHTS
    </text>
    
    <!-- Name -->
    <text x="0" y="120" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="64" font-weight="300" letter-spacing="-1" fill="#000000">
      {}
    </text>
    
    <!-- Separator -->
    <line x1="0" y1="160" x2="900" y2="160" stroke="#000000" stroke-width="2"/>
    
    <!-- Lifts Grid -->
    <g transform="translate(0, 240)">
      <!-- Squat -->
      <g transform="translate(0, 0)">
        <text x="0" y="0" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="12" font-weight="700" letter-spacing="1" fill="#666666">SQUAT</text>
        <text x="0" y="50" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="48" font-weight="400" fill="#000000">{}</text>
        <text x="100" y="50" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="14" fill="#999999">kg</text>
      </g>
      
      <!-- Bench -->
      <g transform="translate(225, 0)">
        <text x="0" y="0" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="12" font-weight="700" letter-spacing="1" fill="#666666">BENCH</text>
        <text x="0" y="50" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="48" font-weight="400" fill="#000000">{}</text>
        <text x="100" y="50" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="14" fill="#999999">kg</text>
      </g>
      
      <!-- Deadlift -->
      <g transform="translate(450, 0)">
        <text x="0" y="0" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="12" font-weight="700" letter-spacing="1" fill="#666666">DEADLIFT</text>
        <text x="0" y="50" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="48" font-weight="400" fill="#000000">{}</text>
        <text x="100" y="50" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="14" fill="#999999">kg</text>
      </g>
      
      <!-- Total -->
      <g transform="translate(675, 0)">
        <text x="0" y="0" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="12" font-weight="700" letter-spacing="1" fill="#000000">TOTAL</text>
        <text x="0" y="50" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="48" font-weight="700" fill="#000000">{}</text>
        <text x="100" y="50" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="14" fill="#999999">kg</text>
      </g>
    </g>
    
    <!-- Secondary Stats -->
    <g transform="translate(0, 380)">
      <!-- DOTS -->
      <g transform="translate(0, 0)">
        <text x="0" y="0" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="12" font-weight="700" letter-spacing="1" fill="#666666">DOTS SCORE</text>
        <text x="0" y="40" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="32" font-weight="300" fill="#000000">{}</text>
      </g>
      
      <!-- Strength Level -->
      <g transform="translate(225, 0)">
        <text x="0" y="0" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="12" font-weight="700" letter-spacing="1" fill="#666666">CLASSIFICATION</text>
        <text x="0" y="40" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="32" font-weight="300" fill="#000000">{}</text>
      </g>
      
      <!-- Bodyweight -->
      <g transform="translate(450, 0)">
        <text x="0" y="0" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="12" font-weight="700" letter-spacing="1" fill="#666666">BODYWEIGHT</text>
        <text x="0" y="40" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="32" font-weight="300" fill="#000000">{:.1}kg</text>
      </g>
    </g>
    
    <!-- Footer -->
    <text x="0" y="500" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-size="12" fill="#999999">
      Generated on {}
    </text>
    
  </g>
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
        chrono::Local::now().format("%B %d, %Y")
    )
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

    format!(
        r##"<svg width="{}" height="{}" viewBox="0 0 {} {}"
            xmlns="http://www.w3.org/2000/svg"
            preserveAspectRatio="xMidYMid meet">
  <defs>
    <linearGradient id="goldGradient" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#D4AF37;stop-opacity:1" />
      <stop offset="50%" style="stop-color:#F7EF8A;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#D4AF37;stop-opacity:1" />
    </linearGradient>
    <pattern id="paper" width="100" height="100" patternUnits="userSpaceOnUse">
      <rect width="100" height="100" fill="#fdfbf7"/>
      <path d="M0 0h100v100H0z" fill="#000" fill-opacity="0.02"/>
    </pattern>
  </defs>
  
  <!-- Background -->
  <rect width="100%" height="100%" fill="url(#paper)"/>
  
  <!-- Border Frame -->
  <rect x="20" y="20" width="960" height="560" fill="none" stroke="#8B0000" stroke-width="4"/>
  <rect x="30" y="30" width="940" height="540" fill="none" stroke="#D4AF37" stroke-width="2"/>
  
  <!-- Corner Ornaments -->
  <path d="M20 20 L80 20 L80 25 L25 25 L25 80 L20 80 Z" fill="#8B0000"/>
  <path d="M980 20 L920 20 L920 25 L975 25 L975 80 L980 80 Z" fill="#8B0000"/>
  <path d="M20 580 L80 580 L80 575 L25 575 L25 520 L20 520 Z" fill="#8B0000"/>
  <path d="M980 580 L920 580 L920 575 L975 575 L975 520 L980 520 Z" fill="#8B0000"/>
  
  <!-- Header -->
  <text x="500" y="80" font-family="Times New Roman, serif" font-size="24" font-weight="bold" letter-spacing="2" text-anchor="middle" fill="#8B0000">
    OFFICIAL LIFTING RECORD
  </text>
  
  <!-- Name -->
  <text x="500" y="140" font-family="Times New Roman, serif" font-size="56" font-weight="bold" text-anchor="middle" fill="#000000">
    {}
  </text>
  
  <!-- Divider -->
  <path d="M300 160 L700 160" stroke="#D4AF37" stroke-width="2"/>
  <circle cx="500" cy="160" r="5" fill="#8B0000"/>
  
  <!-- Stats Table -->
  <g transform="translate(100, 220)">
    <!-- Headers -->
    <rect x="0" y="0" width="800" height="40" fill="#8B0000"/>
    <text x="100" y="28" font-family="Arial, sans-serif" font-size="16" font-weight="bold" text-anchor="middle" fill="#FFFFFF">SQUAT</text>
    <text x="300" y="28" font-family="Arial, sans-serif" font-size="16" font-weight="bold" text-anchor="middle" fill="#FFFFFF">BENCH PRESS</text>
    <text x="500" y="28" font-family="Arial, sans-serif" font-size="16" font-weight="bold" text-anchor="middle" fill="#FFFFFF">DEADLIFT</text>
    <text x="700" y="28" font-family="Arial, sans-serif" font-size="16" font-weight="bold" text-anchor="middle" fill="#FFFFFF">TOTAL</text>
    
    <!-- Values -->
    <rect x="0" y="40" width="800" height="80" fill="#f5f5f5" stroke="#cccccc"/>
    <line x1="200" y1="40" x2="200" y2="120" stroke="#cccccc"/>
    <line x1="400" y1="40" x2="400" y2="120" stroke="#cccccc"/>
    <line x1="600" y1="40" x2="600" y2="120" stroke="#cccccc"/>
    
    <text x="100" y="90" font-family="Times New Roman, serif" font-size="36" font-weight="bold" text-anchor="middle" fill="#000000">{}</text>
    <text x="300" y="90" font-family="Times New Roman, serif" font-size="36" font-weight="bold" text-anchor="middle" fill="#000000">{}</text>
    <text x="500" y="90" font-family="Times New Roman, serif" font-size="36" font-weight="bold" text-anchor="middle" fill="#000000">{}</text>
    <text x="700" y="90" font-family="Times New Roman, serif" font-size="36" font-weight="bold" text-anchor="middle" fill="#8B0000">{}</text>
  </g>
  
  <!-- Secondary Stats Box -->
  <g transform="translate(250, 360)">
    <rect x="0" y="0" width="500" height="100" fill="none" stroke="#D4AF37" stroke-width="2"/>
    <rect x="5" y="5" width="490" height="90" fill="none" stroke="#8B0000" stroke-width="1"/>
    
    <!-- DOTS -->
    <text x="125" y="40" font-family="Arial, sans-serif" font-size="14" font-weight="bold" text-anchor="middle" fill="#666666">DOTS SCORE</text>
    <text x="125" y="80" font-family="Times New Roman, serif" font-size="32" font-weight="bold" text-anchor="middle" fill="#000000">{}</text>
    
    <!-- Divider -->
    <line x1="250" y1="20" x2="250" y2="80" stroke="#cccccc"/>
    
    <!-- Class -->
    <text x="375" y="40" font-family="Arial, sans-serif" font-size="14" font-weight="bold" text-anchor="middle" fill="#666666">CLASSIFICATION</text>
    <text x="375" y="80" font-family="Times New Roman, serif" font-size="28" font-weight="bold" text-anchor="middle" fill="#8B0000">{}</text>
  </g>
  
  <!-- Footer -->
  <text x="500" y="540" font-family="Times New Roman, serif" font-size="14" font-style="italic" text-anchor="middle" fill="#666666">
    Certified by Iron Insights • Bodyweight: {:.1}kg • {} Class • Date: {}
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
        chrono::Local::now().format("%Y-%m-%d")
    )
}
