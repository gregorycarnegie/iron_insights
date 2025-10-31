use crate::components::*;
use maud::{DOCTYPE, Markup, html};

pub fn render_onerepmax_page() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (render_head_minimal())
            body {
                div.container {
                    (render_header())
                    div.main-content {
                        div.onerepmax-calculator {
                            div.calculator-header {
                                h1 { "1RM Calculator" }
                                p.description { "Calculate your one-rep max and training percentages using proven formulas." }
                            }

                            div.calculator-content #calculator-content {
                                div.input-section {
                                    h2 { "Enter Your Lift Data" }
                                    div.input-group {
                                        label for="weight" { "Weight Lifted" }
                                        div.input-with-unit {
                                            input #weight type="number" step="0.1" placeholder="100" min="1";
                                            select #unit {
                                                option value="kg" selected { "kg" }
                                                option value="lbs" { "lbs" }
                                            }
                                        }
                                    }

                                    div.input-group {
                                        label for="reps" { "Repetitions" }
                                        input #reps type="number" placeholder="5" min="1" max="20";
                                    }

                                    div.input-group {
                                        label for="formula" { "Calculation Formula" }
                                        select #formula {
                                            option value="epley" selected { "Epley (most common)" }
                                            option value="brzycki" { "Brzycki" }
                                            option value="lander" { "Lander" }
                                            option value="lombardi" { "Lombardi" }
                                            option value="mayhew" { "Mayhew" }
                                            option value="oconner" { "O'Conner" }
                                            option value="wathen" { "Wathen" }
                                        }
                                    }

                                    button #calculate-btn onclick="calculate1RM()" class="btn btn-primary" { "Calculate 1RM" }
                                }

                                div.results-section #results style="display: none;" {
                                    div.onerepmax-result {
                                        h2 { "Your One Rep Max" }
                                        div.onerepmax-display {
                                            span #onerepmax-value { "—" }
                                            span #onerepmax-unit { "kg" }
                                        }
                                        p.formula-used { "Using " span #formula-name { "—" } " formula" }
                                    }

                                    div.percentage-table {
                                        h3 { "Training Percentages" }
                                        table {
                                            thead {
                                                tr {
                                                    th { "Percentage" }
                                                    th { "Weight" }
                                                    th { "Typical Reps" }
                                                    th { "Training Focus" }
                                                }
                                            }
                                            tbody #percentage-table-body {
                                                // Will be populated by JavaScript
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                (crate::components::scripts::render_scripts())
                script { (maud::PreEscaped(r#"
                    // 1RM Calculation Formulas
                    window.formulas = {
                        epley: (weight, reps) => weight * (1 + reps / 30),
                        brzycki: (weight, reps) => weight * 36 / (37 - reps),
                        lander: (weight, reps) => weight * 100 / (101.3 - 2.67123 * reps),
                        lombardi: (weight, reps) => weight * Math.pow(reps, 0.10),
                        mayhew: (weight, reps) => weight * 100 / (52.2 + 41.9 * Math.exp(-0.055 * reps)),
                        oconner: (weight, reps) => weight * (1 + 0.025 * reps),
                        wathen: (weight, reps) => weight * 100 / (48.8 + 53.8 * Math.exp(-0.075 * reps))
                    };

                    window.formulaNames = {
                        epley: 'Epley',
                        brzycki: 'Brzycki',
                        lander: 'Lander',
                        lombardi: 'Lombardi',
                        mayhew: 'Mayhew',
                        oconner: "O'Conner",
                        wathen: 'Wathen'
                    };

                    // Training percentage data with typical reps and focus
                    window.trainingPercentages = [
                        { percent: 100, reps: '1', focus: 'Max Strength' },
                        { percent: 95, reps: '2', focus: 'Max Strength' },
                        { percent: 90, reps: '3-4', focus: 'Strength' },
                        { percent: 85, reps: '5-6', focus: 'Strength' },
                        { percent: 80, reps: '6-8', focus: 'Strength/Power' },
                        { percent: 75, reps: '8-10', focus: 'Strength/Hypertrophy' },
                        { percent: 70, reps: '10-12', focus: 'Hypertrophy' },
                        { percent: 65, reps: '12-15', focus: 'Hypertrophy' },
                        { percent: 60, reps: '15-20', focus: 'Muscular Endurance' },
                        { percent: 55, reps: '20-25', focus: 'Muscular Endurance' },
                        { percent: 50, reps: '25-30', focus: 'Muscular Endurance' }
                    ];

                    window.calculate1RM = function() {
                        const weightInput = document.getElementById('weight');
                        const repsInput = document.getElementById('reps');
                        const formulaSelect = document.getElementById('formula');
                        const unitSelect = document.getElementById('unit');
                        
                        const weight = parseFloat(weightInput.value);
                        const reps = parseInt(repsInput.value);
                        const formula = formulaSelect.value;
                        const unit = unitSelect.value;
                        
                        // Validation
                        if (!weight || weight <= 0) {
                            alert('Please enter a valid weight');
                            return;
                        }
                        
                        if (!reps || reps <= 0 || reps > 20) {
                            alert('Please enter repetitions between 1 and 20');
                            return;
                        }
                        
                        if (reps === 1) {
                            alert('For 1 rep, your 1RM is the weight you lifted!');
                            return;
                        }
                        
                        // Calculate 1RM
                        const oneRepMax = window.formulas[formula](weight, reps);
                        
                        // Display results
                        window.displayResults(oneRepMax, formula, unit);
                    }
                    
                    window.displayResults = function(oneRepMax, formula, unit) {
                        // Show results section
                        document.getElementById('results').style.display = 'block';
                        // Enable two-column layout when results are visible
                        const container = document.getElementById('calculator-content');
                        if (container) {
                            container.classList.add('has-results');
                        }
                        
                        // Display 1RM
                        document.getElementById('onerepmax-value').textContent = oneRepMax.toFixed(1);
                        document.getElementById('onerepmax-unit').textContent = unit;
                        document.getElementById('formula-name').textContent = window.formulaNames[formula];
                        
                        // Generate percentage table
                        const tableBody = document.getElementById('percentage-table-body');
                        tableBody.innerHTML = '';
                        
                        window.trainingPercentages.forEach(data => {
                            const percentageWeight = (oneRepMax * data.percent / 100);
                            const row = document.createElement('tr');
                            
                            row.innerHTML = '<td class="percentage-cell">' + data.percent + '%</td>' +
                                           '<td class="weight-cell">' + percentageWeight.toFixed(1) + ' ' + unit + '</td>' +
                                           '<td class="reps-cell">' + data.reps + '</td>' +
                                           '<td class="focus-cell">' + data.focus + '</td>';
                            
                            tableBody.appendChild(row);
                        });
                        
                        // Scroll to results
                        document.getElementById('results').scrollIntoView({ 
                            behavior: 'smooth', 
                            block: 'start' 
                        });
                    }
                    
                    // Allow Enter key to calculate
                    document.addEventListener('keypress', function(e) {
                        if (e.key === 'Enter' && (e.target.id === 'weight' || e.target.id === 'reps')) {
                            window.calculate1RM();
                        }
                    });
                "#))
                }
                style {
                    r#"
                    .onerepmax-calculator {
                        max-width: 1000px;
                        margin: 0 auto;
                        padding: 2rem;
                    }
                    
                    .calculator-header {
                        text-align: center;
                        margin-bottom: 3rem;
                    }
                    
                    .calculator-header h1 {
                        font-size: 2.5rem;
                        margin-bottom: 1rem;
                        color: var(--text-primary);
                    }
                    
                    .description {
                        font-size: 1.1rem;
                        color: var(--text-secondary);
                        max-width: 600px;
                        margin: 0 auto;
                    }
                    
                    .calculator-content {
                        display: grid;
                        gap: 3rem;
                        grid-template-columns: 1fr;
                        align-items: start;
                    }
                    
                    /* Show inputs and results side-by-side on wider screens */
                    @media (min-width: 900px) {
                        .calculator-content.has-results {
                            grid-template-columns: 1fr 1fr;
                        }
                    }
                    
                    .input-section {
                        background: var(--surface);
                        padding: 2rem;
                        border-radius: 12px;
                        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
                    }
                    
                    .input-section h2 {
                        margin-bottom: 2rem;
                        color: var(--text-primary);
                        font-size: 1.5rem;
                    }
                    
                    .input-group {
                        margin-bottom: 1.5rem;
                    }
                    
                    .input-group label {
                        display: block;
                        margin-bottom: 0.5rem;
                        font-weight: 600;
                        color: var(--text-primary);
                    }
                    
                    .input-with-unit {
                        display: flex;
                        gap: 0.5rem;
                        align-items: stretch;
                    }
                    
                    .input-with-unit input {
                        flex: 3;
                        min-width: 120px;
                    }
                    
                    .input-with-unit select {
                        flex: 1;
                        max-width: 70px;
                        min-width: 60px;
                    }
                    
                    input, select {
                        width: 100%;
                        padding: 0.75rem;
                        border: 2px solid var(--border);
                        border-radius: 8px;
                        font-size: 1rem;
                        background: var(--surface);
                        color: var(--text-primary);
                        transition: border-color 0.2s ease;
                    }
                    
                    input:focus, select:focus {
                        outline: none;
                        border-color: var(--primary);
                        box-shadow: 0 0 0 3px rgba(var(--primary-rgb), 0.1);
                    }
                    
                    .btn {
                        display: inline-block;
                        padding: 0.875rem 2rem;
                        border-radius: 8px;
                        font-weight: 600;
                        font-size: 1.1rem;
                        border: none;
                        cursor: pointer;
                        transition: all 0.2s ease;
                        text-decoration: none;
                        text-align: center;
                    }
                    
                    .btn-primary {
                        background: var(--primary);
                        color: white;
                    }
                    
                    .btn-primary:hover {
                        background: var(--primary-dark);
                        transform: translateY(-2px);
                    }
                    
                    .results-section {
                        background: var(--surface);
                        padding: 2rem;
                        border-radius: 12px;
                        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
                        align-self: start;
                    }
                    
                    .onerepmax-result {
                        text-align: center;
                        margin-bottom: 3rem;
                        padding: 2rem;
                        background: linear-gradient(135deg, var(--primary-light), var(--secondary-light));
                        border-radius: 12px;
                    }
                    
                    .onerepmax-result h2 {
                        margin-bottom: 1rem;
                        color: var(--text-primary);
                    }
                    
                    .onerepmax-display {
                        font-size: 3rem;
                        font-weight: 700;
                        color: var(--primary);
                        margin-bottom: 0.5rem;
                    }
                    
                    .formula-used {
                        color: var(--text-secondary);
                        font-style: italic;
                    }
                    
                    .percentage-table h3 {
                        margin-bottom: 1.5rem;
                        color: var(--text-primary);
                        font-size: 1.3rem;
                    }
                    
                    table {
                        width: 100%;
                        border-collapse: collapse;
                        background: var(--surface);
                        border-radius: 8px;
                        overflow: hidden;
                        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
                    }
                    
                    th {
                        background: var(--surface-secondary);
                        padding: 1rem;
                        text-align: left;
                        font-weight: 600;
                        color: var(--text-primary);
                        border-bottom: 2px solid var(--border);
                    }
                    
                    td {
                        padding: 0.875rem 1rem;
                        border-bottom: 1px solid var(--border);
                        color: var(--text-primary);
                    }
                    
                    tr:hover {
                        background: var(--surface-hover);
                    }
                    
                    .percentage-cell {
                        font-weight: 600;
                        color: var(--primary);
                    }
                    
                    .weight-cell {
                        font-weight: 600;
                        font-size: 1.1rem;
                    }
                    
                    .focus-cell {
                        color: var(--text-secondary);
                        font-style: italic;
                    }
                    
                    @media (max-width: 768px) {
                        .onerepmax-calculator {
                            padding: 1rem;
                        }
                        
                        .calculator-header h1 {
                            font-size: 2rem;
                        }
                        
                        .onerepmax-display {
                            font-size: 2.5rem;
                        }
                        
                        table {
                            font-size: 0.9rem;
                        }
                        
                        th, td {
                            padding: 0.5rem;
                        }
                    }
                    "#
                }
            }
        }
    }
}
