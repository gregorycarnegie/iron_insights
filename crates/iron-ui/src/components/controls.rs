// src/ui/components/controls.rs - Modern sidebar controls
use chrono::{self, Datelike};
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
                            label { input type="radio" name="sex" value="M"; " Male" }
                            label { input type="radio" name="sex" value="F"; " Female" }
                            label { input type="radio" name="sex" value="All" checked; " All" }
                        }
                        // Enhanced version with toggle buttons
                        div.toggle-group.js-only role="radiogroup" aria-labelledby="sex-legend" {
                            button.toggle-button type="button" data-value="M" onclick="setToggle(this, 'sex')" role="radio" aria-checked="false" tabindex="-1" { "Male" }
                            button.toggle-button type="button" data-value="F" onclick="setToggle(this, 'sex')" role="radio" aria-checked="false" tabindex="-1" { "Female" }
                            button.toggle-button.active type="button" data-value="All" onclick="setToggle(this, 'sex')" role="radio" aria-checked="true" tabindex="0" { "All" }
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
                        optgroup label="IPF Men" {
                            option value="ipf:53" { "-53 kg" }
                            option value="ipf:59" { "-59 kg" }
                            option value="ipf:66" { "-66 kg" }
                            option value="ipf:74" { "-74 kg" }
                            option value="ipf:83" { "-83 kg" }
                            option value="ipf:93" { "-93 kg" }
                            option value="ipf:105" { "-105 kg" }
                            option value="ipf:120" { "-120 kg" }
                            option value="ipf:120+" { "120+ kg" }
                        }
                        optgroup label="IPF Women" {
                            option value="ipf:43" { "-43 kg" }
                            option value="ipf:47" { "-47 kg" }
                            option value="ipf:52" { "-52 kg" }
                            option value="ipf:57" { "-57 kg" }
                            option value="ipf:63" { "-63 kg" }
                            option value="ipf:69" { "-69 kg" }
                            option value="ipf:76" { "-76 kg" }
                            option value="ipf:84" { "-84 kg" }
                            option value="ipf:84+" { "84+ kg" }
                        }
                        optgroup label="Para Men" {
                            option value="para:49" { "-49 kg" }
                            option value="para:54" { "-54 kg" }
                            option value="para:59" { "-59 kg" }
                            option value="para:65" { "-65 kg" }
                            option value="para:72" { "-72 kg" }
                            option value="para:80" { "-80 kg" }
                            option value="para:88" { "-88 kg" }
                            option value="para:97" { "-97 kg" }
                            option value="para:107" { "-107 kg" }
                            option value="para:107+" { "107+ kg" }
                        }
                        optgroup label="Para Women" {
                            option value="para:41" { "-41 kg" }
                            option value="para:45" { "-45 kg" }
                            option value="para:50" { "-50 kg" }
                            option value="para:55" { "-55 kg" }
                            option value="para:61" { "-61 kg" }
                            option value="para:67" { "-67 kg" }
                            option value="para:73" { "-73 kg" }
                            option value="para:79" { "-79 kg" }
                            option value="para:86" { "-86 kg" }
                            option value="para:86+" { "86+ kg" }
                        }
                        optgroup label="WP Men" {
                            option value="wp:62" { "-62 kg" }
                            option value="wp:69" { "-69 kg" }
                            option value="wp:77" { "-77 kg" }
                            option value="wp:85" { "-85 kg" }
                            option value="wp:94" { "-94 kg" }
                            option value="wp:105" { "-105 kg" }
                            option value="wp:120" { "-120 kg" }
                            option value="wp:120+" { "120+ kg" }
                        }
                        optgroup label="WP Women" {
                            option value="wp:48" { "-48 kg" }
                            option value="wp:53" { "-53 kg" }
                            option value="wp:58" { "-58 kg" }
                            option value="wp:64" { "-64 kg" }
                            option value="wp:72" { "-72 kg" }
                            option value="wp:84" { "-84 kg" }
                            option value="wp:100" { "-100 kg" }
                            option value="wp:100+" { "100+ kg" }
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
                    select #timePeriod name="timePeriod" aria-describedby="time-period-help" {
                        option value="all" { "All Time" }
                        option value="current_year" { (chrono::Utc::now().year().to_string()) }
                        option value="previous_year" { ((chrono::Utc::now().year() - 1).to_string()) }
                        option value="last_12_months" { "Last 12 Months" }
                        option value="last_5_years" selected { "Last 5 Years" }
                    }
                    span.sr-only #time-period-help { "Select the time period for filtering competition data" }
                }

                div.control-group {
                    label for="federation" { "Federation" }
                    select #federation name="federation" aria-describedby="federation-help" {
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
