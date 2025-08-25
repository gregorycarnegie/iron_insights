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
        function getStrengthLevel(dotsScore) {
            return getStrengthLevelForLift(dotsScore, 'total');
        }
        
        // Lift-specific strength level calculation
        function getStrengthLevelForLift(dotsScore, liftType) {
            switch (liftType) {
                case 'squat':
                    if (dotsScore < 150.0) return "Beginner";
                    else if (dotsScore < 225.0) return "Novice";
                    else if (dotsScore < 300.0) return "Intermediate";
                    else if (dotsScore < 375.0) return "Advanced";
                    else if (dotsScore < 450.0) return "Elite";
                    else return "World Class";
                case 'bench':
                    if (dotsScore < 100.0) return "Beginner";
                    else if (dotsScore < 150.0) return "Novice";
                    else if (dotsScore < 200.0) return "Intermediate";
                    else if (dotsScore < 250.0) return "Advanced";
                    else if (dotsScore < 300.0) return "Elite";
                    else return "World Class";
                case 'deadlift':
                    if (dotsScore < 175.0) return "Beginner";
                    else if (dotsScore < 262.5) return "Novice";
                    else if (dotsScore < 350.0) return "Intermediate";
                    else if (dotsScore < 437.5) return "Advanced";
                    else if (dotsScore < 525.0) return "Elite";
                    else return "World Class";
                default: // 'total' and others
                    if (dotsScore < 200.0) return "Beginner";
                    else if (dotsScore < 300.0) return "Novice";
                    else if (dotsScore < 400.0) return "Intermediate";
                    else if (dotsScore < 500.0) return "Advanced";
                    else if (dotsScore < 600.0) return "Elite";
                    else return "World Class";
            }
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
                    
                    if (calculate_all_scores_wasm) {
                        // Use percentile from the data if available, otherwise fallback to approximate percentile
                        const userPercentile = lastResponse && lastResponse.user_percentile ? lastResponse.user_percentile : 50.0;
                        console.log('Using percentile:', userPercentile, 'from lastResponse:', !!lastResponse);
                        
                        const result = calculate_all_scores_wasm(userLift, bodyweight, isMale, userPercentile);
                        console.log('WASM result:', result);
                        
                        dots = result.dots;
                        wilks = result.wilks;
                        ipfGLPoints = result.ipf_gl_points;
                        level = result.level;
                        color = result.color;
                    } else {
                        console.log('Using fallback calculations');
                        // Fallback calculation
                        dots = calculateDOTS(userLift, bodyweight);
                        wilks = calculateWilks(userLift, bodyweight, isMale);
                        ipfGLPoints = calculateIPFGLPoints(userLift, bodyweight, isMale);
                        level = getStrengthLevelForLift(dots, currentLiftType);
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