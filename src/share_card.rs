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
    let card_width = 800;
    let card_height = 700; // Increased height for all lifts
    
    // Colors as string constants to avoid parsing issues
    let bg_start = "#667eea";
    let bg_end = "#764ba2";
    let accent = "#4facfe";

    // Format all lift values
    let squat_display = data.squat.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let bench_display = data.bench.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let deadlift_display = data.deadlift.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let total_display = data.total.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());

    let dots_display = data.dots_score
        .map(|d| format!("{:.1}", d))
        .unwrap_or_else(|| "-".to_string());

    let percentile_display = data.percentile
        .map(|p| format!("{:.0}%", p))
        .unwrap_or_else(|| "-".to_string());

    format!(
        r##"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:{};stop-opacity:1" />
      <stop offset="100%" style="stop-color:{};stop-opacity:1" />
    </linearGradient>
    <filter id="shadow">
      <feDropShadow dx="0" dy="4" stdDeviation="8" flood-opacity="0.3"/>
    </filter>
  </defs>
  
  <rect width="100%" height="100%" fill="url(#bg)" rx="20"/>
  <rect x="0" y="0" width="100%" height="80" fill="{}" rx="20"/>
  
  <text x="40" y="50" font-family="Arial" font-size="32" font-weight="bold" fill="white">
    🏋️ Iron Insights
  </text>
  
  <rect x="40" y="140" width="720" height="520" fill="white" rx="15" filter="url(#shadow)" opacity="0.95"/>
  
  <text x="400" y="190" font-family="Arial" font-size="36" font-weight="bold" text-anchor="middle" fill="#333">
    {}
  </text>
  
  <!-- Main Lifts Grid -->
  <g transform="translate(80, 220)">
    <!-- Squat -->
    <rect x="0" y="0" width="160" height="90" fill="{}" rx="10" opacity="0.2"/>
    <text x="80" y="25" font-family="Arial" font-size="14" text-anchor="middle" fill="#666" font-weight="600">
      🏋️ SQUAT
    </text>
    <text x="80" y="55" font-family="Arial" font-size="32" text-anchor="middle" fill="#333" font-weight="bold">
      {}
    </text>
    <text x="80" y="75" font-family="Arial" font-size="12" text-anchor="middle" fill="#888">
      kg
    </text>
    
    <!-- Bench -->
    <rect x="180" y="0" width="160" height="90" fill="{}" rx="10" opacity="0.2"/>
    <text x="260" y="25" font-family="Arial" font-size="14" text-anchor="middle" fill="#666" font-weight="600">
      💪 BENCH
    </text>
    <text x="260" y="55" font-family="Arial" font-size="32" text-anchor="middle" fill="#333" font-weight="bold">
      {}
    </text>
    <text x="260" y="75" font-family="Arial" font-size="12" text-anchor="middle" fill="#888">
      kg
    </text>
    
    <!-- Deadlift -->
    <rect x="360" y="0" width="160" height="90" fill="{}" rx="10" opacity="0.2"/>
    <text x="440" y="25" font-family="Arial" font-size="14" text-anchor="middle" fill="#666" font-weight="600">
      ⬆️ DEADLIFT
    </text>
    <text x="440" y="55" font-family="Arial" font-size="32" text-anchor="middle" fill="#333" font-weight="bold">
      {}
    </text>
    <text x="440" y="75" font-family="Arial" font-size="12" text-anchor="middle" fill="#888">
      kg
    </text>
    
    <!-- Total -->
    <rect x="540" y="0" width="160" height="90" fill="#ff6b6b" rx="10" opacity="0.3"/>
    <text x="620" y="25" font-family="Arial" font-size="14" text-anchor="middle" fill="#666" font-weight="600">
      🏆 TOTAL
    </text>
    <text x="620" y="55" font-family="Arial" font-size="32" text-anchor="middle" fill="#333" font-weight="bold">
      {}
    </text>
    <text x="620" y="75" font-family="Arial" font-size="12" text-anchor="middle" fill="#888">
      kg
    </text>
  </g>
  
  <!-- DOTS and Level Section -->
  <g transform="translate(150, 340)">
    <rect x="0" y="0" width="200" height="100" fill="{}" rx="10" opacity="0.2"/>
    <text x="100" y="25" font-family="Arial" font-size="16" text-anchor="middle" fill="#666" font-weight="600">
      🎯 DOTS SCORE
    </text>
    <text x="100" y="65" font-family="Arial" font-size="36" text-anchor="middle" fill="#333" font-weight="bold">
      {}
    </text>
    
    <rect x="220" y="0" width="200" height="100" fill="{}" rx="10" opacity="0.2"/>
    <text x="320" y="25" font-family="Arial" font-size="16" text-anchor="middle" fill="#666" font-weight="600">
      💪 STRENGTH LEVEL
    </text>
    <text x="320" y="60" font-family="Arial" font-size="22" text-anchor="middle" fill="#333" font-weight="bold">
      {}
    </text>
    <text x="320" y="80" font-family="Arial" font-size="14" text-anchor="middle" fill="#888">
      {}th Percentile
    </text>
  </g>
  
  <!-- Footer Info -->
  <g transform="translate(100, 480)">
    <text x="300" y="20" font-family="Arial" font-size="14" text-anchor="middle" fill="#666">
      {} • {:.1}kg Bodyweight • {} Class
    </text>
    <text x="300" y="45" font-family="Arial" font-size="12" text-anchor="middle" fill="#999">
      Generated by Iron Insights - Powerlifting Analytics
    </text>
  </g>
  
</svg>"##,
        card_width, card_height, bg_start, bg_end, accent,
        data.name, accent, squat_display, accent, bench_display, accent, deadlift_display, total_display,
        accent, dots_display, accent, data.strength_level, percentile_display,
        data.strength_level, data.bodyweight, data.sex
    )
}

fn generate_dark_card(data: &ShareCardData) -> String {
    let card_width = 800;
    let card_height = 700;

    // Format all lift values
    let squat_display = data.squat.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let bench_display = data.bench.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let deadlift_display = data.deadlift.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let total_display = data.total.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());

    let dots_display = data.dots_score
        .map(|d| format!("{:.1}", d))
        .unwrap_or_else(|| "-".to_string());

    let percentile_display = data.percentile
        .map(|p| format!("{:.0}%", p))
        .unwrap_or_else(|| "-".to_string());

    format!(
        r##"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="darkBg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#1a1a2e;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#16213e;stop-opacity:1" />
    </linearGradient>
  </defs>
  
  <rect width="100%" height="100%" fill="url(#darkBg)" rx="20"/>
  <rect x="0" y="0" width="100%" height="80" fill="#0f3460" rx="20"/>
  
  <text x="40" y="50" font-family="Arial" font-size="32" font-weight="bold" fill="#00d2ff">
    🏋️ Iron Insights
  </text>
  
  <rect x="40" y="140" width="720" height="520" fill="#0f0e17" rx="15" opacity="0.9"/>
  
  <text x="400" y="190" font-family="Arial" font-size="36" font-weight="bold" text-anchor="middle" fill="white">
    {}
  </text>
  
  <!-- Main Lifts Grid Dark Theme -->
  <g transform="translate(80, 220)">
    <!-- Squat -->
    <rect x="0" y="0" width="160" height="90" fill="#e94560" rx="10" opacity="0.3"/>
    <text x="80" y="25" font-family="Arial" font-size="14" text-anchor="middle" fill="#a7a9be">
      🏋️ SQUAT
    </text>
    <text x="80" y="55" font-family="Arial" font-size="32" text-anchor="middle" fill="white" font-weight="bold">
      {}
    </text>
    <text x="80" y="75" font-family="Arial" font-size="12" text-anchor="middle" fill="#a7a9be">
      kg
    </text>
    
    <!-- Bench -->
    <rect x="180" y="0" width="160" height="90" fill="#00d2ff" rx="10" opacity="0.3"/>
    <text x="260" y="25" font-family="Arial" font-size="14" text-anchor="middle" fill="#a7a9be">
      💪 BENCH
    </text>
    <text x="260" y="55" font-family="Arial" font-size="32" text-anchor="middle" fill="white" font-weight="bold">
      {}
    </text>
    <text x="260" y="75" font-family="Arial" font-size="12" text-anchor="middle" fill="#a7a9be">
      kg
    </text>
    
    <!-- Deadlift -->
    <rect x="360" y="0" width="160" height="90" fill="#f25f4c" rx="10" opacity="0.3"/>
    <text x="440" y="25" font-family="Arial" font-size="14" text-anchor="middle" fill="#a7a9be">
      ⬆️ DEADLIFT
    </text>
    <text x="440" y="55" font-family="Arial" font-size="32" text-anchor="middle" fill="white" font-weight="bold">
      {}
    </text>
    <text x="440" y="75" font-family="Arial" font-size="12" text-anchor="middle" fill="#a7a9be">
      kg
    </text>
    
    <!-- Total -->
    <rect x="540" y="0" width="160" height="90" fill="#ff6b95" rx="10" opacity="0.4"/>
    <text x="620" y="25" font-family="Arial" font-size="14" text-anchor="middle" fill="#a7a9be">
      🏆 TOTAL
    </text>
    <text x="620" y="55" font-family="Arial" font-size="32" text-anchor="middle" fill="white" font-weight="bold">
      {}
    </text>
    <text x="620" y="75" font-family="Arial" font-size="12" text-anchor="middle" fill="#a7a9be">
      kg
    </text>
  </g>
  
  <!-- DOTS and Level Section Dark -->
  <g transform="translate(150, 340)">
    <rect x="0" y="0" width="200" height="100" fill="#4c6ef5" rx="10" opacity="0.3"/>
    <text x="100" y="25" font-family="Arial" font-size="16" text-anchor="middle" fill="#a7a9be">
      🎯 DOTS SCORE
    </text>
    <text x="100" y="65" font-family="Arial" font-size="36" text-anchor="middle" fill="white" font-weight="bold">
      {}
    </text>
    
    <rect x="220" y="0" width="200" height="100" fill="#20c997" rx="10" opacity="0.3"/>
    <text x="320" y="25" font-family="Arial" font-size="16" text-anchor="middle" fill="#a7a9be">
      💪 STRENGTH LEVEL
    </text>
    <text x="320" y="60" font-family="Arial" font-size="22" text-anchor="middle" fill="white" font-weight="bold">
      {}
    </text>
    <text x="320" y="80" font-family="Arial" font-size="14" text-anchor="middle" fill="#a7a9be">
      {}th Percentile
    </text>
  </g>
  
  <text x="400" y="580" font-family="Arial" font-size="14" text-anchor="middle" fill="#a7a9be">
    {} • {:.1}kg • {} • Generated by Iron Insights
  </text>
  
</svg>"##,
        card_width, card_height,
        data.name,
        squat_display, bench_display, deadlift_display, total_display,
        dots_display, data.strength_level, percentile_display,
        data.strength_level, data.bodyweight, data.sex
    )
}

fn generate_minimal_card(data: &ShareCardData) -> String {
    let card_width = 800;
    let card_height = 500;

    // Format all lift values
    let squat_display = data.squat.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let bench_display = data.bench.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let deadlift_display = data.deadlift.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let total_display = data.total.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());

    let dots_display = data.dots_score
        .map(|d| format!("{:.1}", d))
        .unwrap_or_else(|| "-".to_string());

    format!(
        r##"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
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
        card_width, card_height,
        data.name,
        squat_display, bench_display, deadlift_display, total_display,
        dots_display, data.strength_level
    )
}

fn generate_powerlifting_card(data: &ShareCardData) -> String {
    let card_width = 800;
    let card_height = 600;

    let squat_display = data.squat.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let bench_display = data.bench.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let deadlift_display = data.deadlift.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let total_display = data.total.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "-".to_string());
    let dots_display = data.dots_score.map(|d| format!("{:.1}", d)).unwrap_or_else(|| "-".to_string());

    format!(
        r##"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
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
        card_width, card_height,
        data.name,
        squat_display, bench_display, deadlift_display, total_display,
        dots_display,
        data.strength_level,
        data.bodyweight, data.sex,
    )
}