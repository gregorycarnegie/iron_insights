// src/ui/components/controls.rs - Modern sidebar controls
use maud::{html, Markup};

pub fn render_controls() -> Markup {
    html! {
        aside.sidebar #sidebar {
            div.control-section {
                h3 { "Athlete Profile" }
                
                div.control-group {
                    label { "Sex" }
                    div.toggle-group {
                        button.toggle-button.active data-value="M" onclick="setToggle(this, 'sex')" { "Male" }
                        button.toggle-button data-value="F" onclick="setToggle(this, 'sex')" { "Female" }
                        button.toggle-button data-value="All" onclick="setToggle(this, 'sex')" { "All" }
                    }
                }
                
                div.control-group {
                    label { "Bodyweight (kg)" }
                    input #bodyweight type="number" placeholder="75" step="0.1" min="30" max="300";
                }
                
                div.control-group {
                    label { "Weight Class" }
                    select #weightClass {
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
                }
            }
            
            div.control-section {
                h3 { "Lift Selection" }
                
                div.control-group {
                    label { "Lift Type" }
                    div.toggle-group {
                        button.toggle-button.active data-value="squat" onclick="setToggle(this, 'lift')" { "SQ" }
                        button.toggle-button data-value="bench" onclick="setToggle(this, 'lift')" { "BP" }
                        button.toggle-button data-value="deadlift" onclick="setToggle(this, 'lift')" { "DL" }
                        button.toggle-button data-value="total" onclick="setToggle(this, 'lift')" { "Total" }
                    }
                }
                
                div.control-group {
                    label { "Your Best (kg)" }
                    input #userLift type="number" placeholder="Enter your best lift" step="2.5" min="0";
                }
            }
            
            div.control-section {
                h3 { "Competition Settings" }
                
                div.control-group {
                    label { "Equipment" }
                    div.checkbox-group {
                        label.checkbox-label {
                            input #equipment-raw type="checkbox" checked;
                            "Raw"
                        }
                        label.checkbox-label {
                            input #equipment-wraps type="checkbox";
                            "Wraps"
                        }
                        label.checkbox-label {
                            input #equipment-single-ply type="checkbox";
                            "Single-ply"
                        }
                        label.checkbox-label {
                            input #equipment-multi-ply type="checkbox";
                            "Multi-ply"
                        }
                    }
                }
                
                div.control-group {
                    label { "Time Period" }
                    select #timePeriod {
                        option value="all" { "All Time" }
                        option value="2024" { "2024" }
                        option value="2023" { "2023" }
                        option value="last_12_months" { "Last 12 Months" }
                        option value="last_5_years" selected { "Last 5 Years" }
                    }
                }
                
                div.control-group {
                    label { "Federation" }
                    select #federation {
                        option value="all" { "All Federations" }
                        option value="ipf" { "IPF" }
                        option value="usapl" { "USAPL" }
                        option value="uspa" { "USPA" }
                        option value="wrpf" { "WRPF" }
                    }
                }
            }
            
            button.btn-primary onclick="updateAnalytics()" {
                "Update Analytics"
            }
        }
        
        div.sidebar-overlay #sidebarOverlay onclick="toggleSidebar()" {}
    }
}