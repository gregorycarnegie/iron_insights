// src/ui/sharecard_page.rs - Dedicated Share Card generator page
use maud::{html, Markup, DOCTYPE, PreEscaped};

use crate::ui::components::{render_head, render_header};

pub fn render_sharecard_page() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head())
            body {
                div.container {
                    (render_header())
                    div.main-content {
                        // Use existing two-column layout: sidebar + content
                        aside.sidebar {
                            h2 { "Create Your Share Card" }
                            p { "Fill in your details, then generate a beautiful SVG card ready to share on social media." }

                            // Form grid
                            div.user-metrics {
                                div.control-group {
                                    label { "Your Name" }
                                    input #shareName type="text" placeholder="Jane Doe" maxlength="32";
                                }
                                div.control-group {
                                    label { "Sex" }
                                    select #shareSex {
                                        option value="M" { "Male" }
                                        option value="F" { "Female" }
                                    }
                                }
                                div.control-group {
                                    label { "Bodyweight (kg)" }
                                    input #shareBodyweight type="number" placeholder="80" step="0.1" min="20" max="350";
                                }
                            }

                            div.user-metrics {
                                div.control-group {
                                    label { "Squat (kg)" }
                                    input #shareSquat type="number" placeholder="—" step="0.5" min="0";
                                }
                                div.control-group {
                                    label { "Bench (kg)" }
                                    input #shareBench type="number" placeholder="—" step="0.5" min="0";
                                }
                                div.control-group {
                                    label { "Deadlift (kg)" }
                                    input #shareDeadlift type="number" placeholder="—" step="0.5" min="0";
                                }
                            }

                            div.user-metrics {
                                div.control-group {
                                    label { "Theme" }
                                    select #shareTheme {
                                        option value="default" { "Default" }
                                        option value="dark" { "Dark Mode" }
                                        option value="minimal" { "Minimal" }
                                        option value="powerlifting" { "Competition Style" }
                                    }
                                }
                                div.control-group {
                                    label { "Lift Type" }
                                    select #shareLiftType {
                                        option value="total" selected { "Total" }
                                        option value="squat" { "Squat" }
                                        option value="bench" { "Bench" }
                                        option value="deadlift" { "Deadlift" }
                                    }
                                }
                                div.control-group {
                                    label { " " }
                                    button onclick="generateShareCard()" style="width: 100%;" { "Generate Share Card 🎨" }
                                }
                            }
                        }

                        // Wide content area for preview
                        section.content {
                            div #shareCardPreview style="display: none;" {
                                h3 { "Preview" }
                                div style="border: 1px solid var(--border); padding: 12px; border-radius: 8px; background: var(--light-secondary); overflow: auto; max-width: 100%;" {
                                    div #shareCardContainer style="display: inline-block; overflow: auto;" {}
                                }
                                div style="margin-top: 12px; display: flex; gap: 8px;" {
                                    button #downloadButton onclick="downloadShareCard()" disabled style="background: #10b981; color: white;" { "Download SVG 📥" }
                                }
                            }
                            // Keep scripts in the page
                            (sharecard_inline_scripts())
                        }
                    }
                }
                // Load global scripts for calculations and utils
                (crate::ui::components::scripts::render_scripts())
            }
        }
    }
}

fn sharecard_inline_scripts() -> Markup {
    PreEscaped(r#"
        <script>
        let shareCardSvgContent = null;

        function parseNumberOrNull(val) {
            const n = parseFloat(val);
            return isNaN(n) ? null : n;
        }

        async function generateShareCard() {
            try {
                const name = (document.getElementById('shareName').value || '').trim();
                const sex = document.getElementById('shareSex').value || 'M';
                const bodyweight = parseNumberOrNull(document.getElementById('shareBodyweight').value);
                const squat = parseNumberOrNull(document.getElementById('shareSquat').value);
                const bench = parseNumberOrNull(document.getElementById('shareBench').value);
                const deadlift = parseNumberOrNull(document.getElementById('shareDeadlift').value);
                const theme = document.getElementById('shareTheme').value || 'default';
                const liftType = document.getElementById('shareLiftType').value || 'total';

                if (!name) {
                    alert('Please enter your name.');
                    return;
                }
                if (!bodyweight) {
                    alert('Please enter your bodyweight.');
                    return;
                }

                // Compute total if possible
                const total = (squat || 0) + (bench || 0) + (deadlift || 0);
                const hasAnyLift = (squat || bench || deadlift);

                // Compute DOTS using existing functions (WASM-backed if available)
                let dots = null;
                if (hasAnyLift && total > 0) {
                    dots = calculateDOTS(total, bodyweight, sex);
                }

                // Strength level (total-based)
                let strengthLevel = '—';
                try {
                    const isMale = sex === 'M' || sex === 'Male';
                    if (dots) {
                        strengthLevel = getStrengthLevelForLift(dots, 'total', isMale);
                    }
                } catch (e) { console.warn('Strength level fallback', e); }

                // IMPORTANT: The API expects flattened fields (serde flatten), not nested under card_data
                const payload = {
                    theme: theme,
                    name: name,
                    bodyweight: bodyweight,
                    squat: squat,
                    bench: bench,
                    deadlift: deadlift,
                    total: hasAnyLift ? total : null,
                    dots_score: dots,
                    strength_level: strengthLevel,
                    percentile: null,
                    lift_type: liftType,
                    sex: sex
                };

                const res = await fetch('/api/share-card', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload)
                });
                if (!res.ok) {
                    throw new Error('Failed to generate share card');
                }
                const svg = await res.text();

                // Render preview
                const container = document.getElementById('shareCardContainer');
                container.innerHTML = svg;
                document.getElementById('shareCardPreview').style.display = 'block';
                const dl = document.getElementById('downloadButton');
                dl.disabled = false;
                shareCardSvgContent = svg;
            } catch (err) {
                console.error(err);
                alert('Error generating share card.');
            }
        }

        function downloadShareCard() {
            try {
                const svg = shareCardSvgContent || document.getElementById('shareCardContainer').innerHTML;
                if (!svg) return;
                const blob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = 'iron-insights-share-card.svg';
                document.body.appendChild(a);
                a.click();
                a.remove();
                URL.revokeObjectURL(url);
            } catch (e) {
                console.error('Download failed', e);
            }
        }
        </script>
    "#.to_string())
}
