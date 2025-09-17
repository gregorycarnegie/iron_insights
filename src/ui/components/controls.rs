// src/ui/components/controls.rs - Modern sidebar controls
use maud::{Markup, html};

pub fn render_controls() -> Markup {
    html! {
        aside.sidebar #sidebar role="complementary" aria-label="Analytics controls" {
            // Progressive enhancement: form works without JS
            form #analytics-form method="get" action="/analytics" {
            div.control-section {
                h3 { "Athlete Profile" }

                div.control-group {
                    fieldset {
                        legend { "Sex" }
                        // No-JS fallback: regular radio buttons
                        div.no-js-only {
                            label { input type="radio" name="sex" value="M" checked; " Male" }
                            label { input type="radio" name="sex" value="F"; " Female" }
                            label { input type="radio" name="sex" value="All"; " All" }
                        }
                        // Enhanced version with toggle buttons
                        div.toggle-group.js-only role="radiogroup" aria-labelledby="sex-legend" {
                            button.toggle-button.active type="button" data-value="M" onclick="setToggle(this, 'sex')" role="radio" aria-checked="true" tabindex="0" { "Male" }
                            button.toggle-button type="button" data-value="F" onclick="setToggle(this, 'sex')" role="radio" aria-checked="false" tabindex="-1" { "Female" }
                            button.toggle-button type="button" data-value="All" onclick="setToggle(this, 'sex')" role="radio" aria-checked="false" tabindex="-1" { "All" }
                        }
                    }
                }

                div.control-group {
                    label for="bodyweight" { "Bodyweight (kg)" }
                    input #bodyweight name="bodyweight" type="number" placeholder="75" step="0.1" min="30" max="300" aria-describedby="bodyweight-help";
                    span.sr-only #bodyweight-help { "Enter your bodyweight in kilograms, between 30 and 300 kg" }
                }

                div.control-group {
                    label for="weightClass" { "Weight Class" }
                    select #weightClass name="weightClass" aria-describedby="weight-class-help" {
                        option value="All" { "All Classes" }
                        optgroup label="Men's Classes" {
                            option value="59" { "59 kg" }
                            option value="66" { "66 kg" }
                            option value="74" { "74 kg" }
                            option value="83" { "83 kg" }
                            option value="93" { "93 kg" }
                            option value="105" { "105 kg" }
                            option value="120" { "120 kg" }
                            option value="120+" { "120+ kg" }
                        }
                        optgroup label="Women's Classes" {
                            option value="47" { "47 kg" }
                            option value="52" { "52 kg" }
                            option value="57" { "57 kg" }
                            option value="63" { "63 kg" }
                            option value="69" { "69 kg" }
                            option value="76" { "76 kg" }
                            option value="84" { "84 kg" }
                            option value="84+" { "84+ kg" }
                        }
                    }
                    span.sr-only #weight-class-help { "Select your powerlifting weight class or all classes to compare against" }
                }
            }

            div.control-section {
                h3 { "Lift Selection" }

                div.control-group {
                    fieldset {
                        legend { "Lift Type" }
                        div.toggle-group role="radiogroup" aria-labelledby="lift-type-legend" {
                            button.toggle-button.active type="button" data-value="squat" onclick="setToggle(this, 'lift')" role="radio" aria-checked="true" tabindex="0" {
                                "SQ"
                                span.sr-only { " (Squat)" }
                            }
                            button.toggle-button type="button" data-value="bench" onclick="setToggle(this, 'lift')" role="radio" aria-checked="false" tabindex="-1" {
                                "BP"
                                span.sr-only { " (Bench Press)" }
                            }
                            button.toggle-button type="button" data-value="deadlift" onclick="setToggle(this, 'lift')" role="radio" aria-checked="false" tabindex="-1" {
                                "DL"
                                span.sr-only { " (Deadlift)" }
                            }
                            button.toggle-button type="button" data-value="total" onclick="setToggle(this, 'lift')" role="radio" aria-checked="false" tabindex="-1" {
                                "Total"
                                span.sr-only { " (All three lifts combined)" }
                            }
                        }
                    }
                }

                div.control-group {
                    label for="userLift" { "Your Best (kg)" }
                    input #userLift type="number" placeholder="Enter your best lift" step="2.5" min="0" aria-describedby="user-lift-help";
                    span.sr-only #user-lift-help { "Enter your personal best for the selected lift type in kilograms" }
                }
            }

            div.control-section {
                h3 { "Competition Settings" }

                div.control-group {
                    fieldset {
                        legend { "Equipment" }
                        div.checkbox-group role="group" aria-labelledby="equipment-legend" {
                            label.checkbox-label {
                                input #equipment-raw type="checkbox" checked aria-describedby="raw-help";
                                "Raw"
                                span.sr-only #raw-help { "No supportive equipment" }
                            }
                            label.checkbox-label {
                                input #equipment-wraps type="checkbox" aria-describedby="wraps-help";
                                "Wraps"
                                span.sr-only #wraps-help { "Knee wraps allowed" }
                            }
                            label.checkbox-label {
                                input #equipment-single-ply type="checkbox" aria-describedby="single-ply-help";
                                "Single-ply"
                                span.sr-only #single-ply-help { "Single layer supportive equipment" }
                            }
                            label.checkbox-label {
                                input #equipment-multi-ply type="checkbox" aria-describedby="multi-ply-help";
                                "Multi-ply"
                                span.sr-only #multi-ply-help { "Multiple layer supportive equipment" }
                            }
                        }
                    }
                }

                div.control-group {
                    label for="timePeriod" { "Time Period" }
                    select #timePeriod aria-describedby="time-period-help" {
                        option value="all" { "All Time" }
                        option value="2024" { "2024" }
                        option value="2023" { "2023" }
                        option value="last_12_months" { "Last 12 Months" }
                        option value="last_5_years" selected { "Last 5 Years" }
                    }
                    span.sr-only #time-period-help { "Select the time period for filtering competition data" }
                }

                div.control-group {
                    label for="federation" { "Federation" }
                    select #federation aria-describedby="federation-help" {
                        option value="all" { "All Federations" }
                        option value="ipf" { "IPF" }
                        option value="usapl" { "USAPL" }
                        option value="uspa" { "USPA" }
                        option value="wrpf" { "WRPF" }
                    }
                    span.sr-only #federation-help { "Select the powerlifting federation to filter results" }
                }
            }

            // Submit button that works with or without JS
            button.btn.btn-primary.btn-lg type="submit" onclick="updateAnalytics(); return false;" aria-describedby="update-button-help" {
                "Update Analytics"
                span.sr-only #update-button-help { "Apply the selected filters and refresh the analytics charts" }
            }

            // Hidden input for progressive enhancement
            input type="hidden" name="js" value="0" #js-enabled;
            }
        }

        div.sidebar-overlay #sidebarOverlay onclick="toggleSidebar()" {}
    }
}
