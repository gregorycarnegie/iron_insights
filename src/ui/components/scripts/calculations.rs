use maud::{Markup, PreEscaped};

pub fn render_calculation_scripts() -> Markup {
    PreEscaped(r#"
        // Helper function to calculate gender-specific DOTS score using WASM
        function calculateDOTS(liftKg, bodyweightKg, sex = null) {
            const sexValue = sex || currentSex;
            const isMale = sexValue === 'M' || sexValue === 'Male';
            
            if (calculate_dots_with_gender_wasm) {
                return calculate_dots_with_gender_wasm(liftKg, bodyweightKg, isMale);
            } else {
                // Fallback JavaScript implementation with gender-specific coefficients
                let a, b, c, d, e;
                if (isMale) {
                    // Male coefficients
                    a = -307.75076;
                    b = 24.0900756;
                    c = -0.1918759221;
                    d = 0.0007391293;
                    e = -0.000001093;
                } else {
                    // Female coefficients
                    a = -57.96288;
                    b = 13.6175032;
                    c = -0.1126655495;
                    d = 0.0005158568;
                    e = -0.0000010706;
                }
                
                const denominator = a + 
                    b * bodyweightKg +
                    c * Math.pow(bodyweightKg, 2) +
                    d * Math.pow(bodyweightKg, 3) +
                    e * Math.pow(bodyweightKg, 4);
                
                return liftKg * 500.0 / denominator;
            }
        }
        
        // Fallback strength level calculation
        function getStrengthLevel(dotsScore, isMale = true) {
            return getStrengthLevelForLift(dotsScore, 'total', isMale);
        }
        
        // Gender and lift-specific strength level calculation
        function getStrengthLevelForLift(dotsScore, liftType, isMale = true) {
            // Realistic DOTS-based strength standards per single lift
            const standards = {
                squat: {
                    male: { beginner: 61, novice: 102, intermediate: 132, advanced: 163, elite: 187, worldClass: Infinity },
                    female: { beginner: 58, novice: 96, intermediate: 125, advanced: 154, elite: 177, worldClass: Infinity }
                },
                bench: {
                    male: { beginner: 41, novice: 69, intermediate: 89, advanced: 110, elite: 127, worldClass: Infinity },
                    female: { beginner: 39, novice: 65, intermediate: 85, advanced: 104, elite: 120, worldClass: Infinity }
                },
                deadlift: {
                    male: { beginner: 63, novice: 105, intermediate: 136, advanced: 167, elite: 192, worldClass: Infinity },
                    female: { beginner: 59, novice: 99, intermediate: 128, advanced: 158, elite: 182, worldClass: Infinity }
                },
                total: {
                    male: { beginner: 200, novice: 300, intermediate: 400, advanced: 500, elite: 600, worldClass: 700 },
                    female: { beginner: 180, novice: 270, intermediate: 360, advanced: 450, elite: 540, worldClass: 630 }
                }
            };
            
            const genderKey = isMale ? 'male' : 'female';
            const liftStandards = standards[liftType] || standards.total;
            const thresholds = liftStandards[genderKey];
            
            if (dotsScore < thresholds.beginner) return "Beginner";
            else if (dotsScore < thresholds.novice) return "Novice";
            else if (dotsScore < thresholds.intermediate) return "Intermediate";
            else if (dotsScore < thresholds.advanced) return "Advanced";
            else if (dotsScore < thresholds.elite) return "Elite";
            else return "World Class";
        }
        
        // Fallback Wilks 2020 calculation
        function calculateWilks(liftKg, bodyweightKg, isMale) {
            const bw = bodyweightKg;
            
            if (isMale) {
                // Male Wilks 2020 coefficients
                const a = 47.46178854, b = 8.472061379, c = 0.07369410346;
                const d = -0.001395833811, e = 7.07665973070743e-06, f = -1.20804336482315e-08;
                const denominator = a + b*bw + c*bw*bw + d*bw*bw*bw + e*bw*bw*bw*bw + f*bw*bw*bw*bw*bw;
                return liftKg * 600.0 / denominator;
            } else {
                // Female Wilks 2020 coefficients
                const a = -125.4255398, b = 13.71219419, c = -0.03307250631;
                const d = -0.001050400051, e = 9.38773881462799e-06, f = -2.3334613884954e-08;
                const denominator = a + b*bw + c*bw*bw + d*bw*bw*bw + e*bw*bw*bw*bw + f*bw*bw*bw*bw*bw;
                return liftKg * 600.0 / denominator;
            }
        }
        
        // Fallback IPF GL Points calculation
        function calculateIPFGLPoints(liftKg, bodyweightKg, isMale) {
            if (isMale) {
                return 1199.72839 / (1025.18162 - 0.00921 * bodyweightKg) * liftKg;
            } else {
                return 610.32796 / (1045.59282 - 0.03048 * bodyweightKg) * liftKg;
            }
        }
        
        // Fallback strength level color
        function getStrengthLevelColor(level) {
            switch (level) {
                case "Untrained": return "\#495057";
                case "Beginner": return "\#6c757d";
                case "Novice": return "\#28a745";
                case "Intermediate": return "\#17a2b8";
                case "Advanced": return "\#ffc107";
                case "Elite": return "\#fd7e14";
                case "World Class": return "\#dc3545";
                default: return "\#6c757d";
            }
        }
        
        // Function to update user metrics display for modern UI
        function updateUserMetrics(bodyweight, userLift) {
            console.log('updateUserMetrics called with:', {bodyweight, userLift, isValid: !!(bodyweight && userLift && !isNaN(bodyweight) && !isNaN(userLift))});
            
            if (bodyweight && userLift && !isNaN(bodyweight) && !isNaN(userLift)) {
                try {
                    let dots, wilks, ipfGLPoints, level, color;
                    const isMale = currentSex === 'M';
                    
                    console.log('Calculating metrics with:', {bodyweight, userLift, isMale});
                    
                    if (calculate_dots_and_level_for_lift_with_gender_wasm && calculate_wilks_wasm && calculate_ipf_gl_points_wasm) {
                        console.log('Using WASM calculations with lift type:', currentLiftType);
                        
                        const dotsResult = calculate_dots_and_level_for_lift_with_gender_wasm(userLift, bodyweight, isMale, currentLiftType);
                        console.log('WASM DOTS result:', dotsResult);
                        
                        dots = dotsResult.dots;
                        level = dotsResult.level;
                        color = dotsResult.color;
                        wilks = calculate_wilks_wasm(userLift, bodyweight, isMale);
                        ipfGLPoints = calculate_ipf_gl_points_wasm(userLift, bodyweight, isMale);
                    } else {
                        console.log('Using fallback calculations');
                        // Fallback calculation
                        dots = calculateDOTS(userLift, bodyweight);
                        wilks = calculateWilks(userLift, bodyweight, isMale);
                        ipfGLPoints = calculateIPFGLPoints(userLift, bodyweight, isMale);
                        level = getStrengthLevelForLift(dots, currentLiftType, isMale);
                        color = getStrengthLevelColor(level);
                    }
                    
                    console.log('Calculated values:', {dots, wilks, ipfGLPoints, level, color});
                    
                    // Update strength level display
                    const strengthLevelElement = document.getElementById('strengthLevel');
                    if (strengthLevelElement) {
                        strengthLevelElement.innerHTML = '<div class="strength-badge" style="background-color: ' + color + ';">🏋️ ' + level + '</div>';
                    }
                    
                    // Update all scoring metrics
                    const userDotsEl = document.getElementById('userDotsScore');
                    const userWilksEl = document.getElementById('userWilks');
                    const userIPFGLPointsEl = document.getElementById('userIPFGLPoints');
                    
                    if (userDotsEl) userDotsEl.textContent = dots.toFixed(1);
                    if (userWilksEl) userWilksEl.textContent = wilks.toFixed(1);
                    if (userIPFGLPointsEl) userIPFGLPointsEl.textContent = ipfGLPoints.toFixed(1);
                    
                    // Performance card is always visible - no need to show/hide
                    
                } catch (error) {
                    console.error('Error calculating user metrics:', error);
                }
            } else {
                console.log('Invalid data for user metrics');
                // Performance card stays visible even with no data
            }
        }
    "#.to_string())
}
