// src/ui/components/share_card.rs - Share card generation section
use maud::{html, Markup};

pub fn render_share_card_section() -> Markup {
    html! {
        div #shareCardSection .user-input-section style="display: none;" {
            h3 { "🎨 Generate Share Card" }
            p { "Create a beautiful social media card to share your powerlifting achievements!" }
            
            div.user-metrics {
                div.control-group {
                    label { "Your Name:" }
                    input #shareCardName type="text" placeholder="Enter your name" maxlength="30";
                }
                
                div.control-group style="grid-column: span 3;" {
                    label { "Your Powerlifting Numbers (kg):" }
                    div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px; margin-top: 5px;" {
                        input #shareSquat type="number" placeholder="Squat" step="0.5" min="0";
                        input #shareBench type="number" placeholder="Bench" step="0.5" min="0";
                        input #shareDeadlift type="number" placeholder="Deadlift" step="0.5" min="0";
                    }
                }
            }
            
            div.user-metrics style="margin-top: 15px;" {
                div.control-group {
                    label { "Card Theme:" }
                    select #shareCardTheme {
                        option value="default" { "Default" }
                        option value="dark" { "Dark Mode" }
                        option value="minimal" { "Minimal" }
                        option value="powerlifting" { "Competition Style" }
                    }
                }
                
                div.control-group {
                    button onclick="generateShareCard()" style="width: 100%;" {
                        "Generate Share Card 🎨"
                    }
                }
                
                div.control-group {
                    button #downloadButton onclick="downloadShareCard()" 
                           style="width: 100%; background: #28a745;" disabled {
                        "Download SVG 📥"
                    }
                }
            }
            
            // Share Card Preview
            div #shareCardPreview style="margin-top: 20px; text-align: center; display: none;" {
                h4 { "Preview:" }
                div #shareCardContainer style="border: 2px solid #ddd; border-radius: 8px; padding: 20px; background: #f8f9fa; display: inline-block;" {}
            }
        }
    }
}