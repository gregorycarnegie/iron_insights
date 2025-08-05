// ui.rs - Updated with improved DOTS debugging frontend
pub const HTML_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Iron Insights - Powerlifting Analytics with DOTS</title>
    <script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
    <style>
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; 
            margin: 0; 
            padding: 20px; 
            background: #f5f5f5; 
        }
        .container { 
            max-width: 1400px; 
            margin: 0 auto; 
            background: white; 
            border-radius: 8px; 
            padding: 20px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1); 
        }
        .header { 
            text-align: center; 
            margin-bottom: 30px; 
        }
        .header h1 {
            color: #333;
            margin-bottom: 5px;
        }
        .header p {
            color: #666;
            margin-bottom: 10px;
        }
        .dots-info {
            background: #e3f2fd;
            padding: 10px 15px;
            border-radius: 5px;
            border-left: 4px solid #2196f3;
            margin: 10px 0;
            font-size: 14px;
        }
        .debug-info {
            background: #fff3cd;
            padding: 10px 15px;
            border-radius: 5px;
            border-left: 4px solid #ffc107;
            margin: 10px 0;
            font-size: 12px;
            font-family: monospace;
            display: none;
        }
        .controls { 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); 
            gap: 15px; 
            margin-bottom: 30px;
            background: #f8f9fa;
            padding: 20px;
            border-radius: 5px;
        }
        .control-group {
            display: flex;
            flex-direction: column;
        }
        .control-group label {
            font-weight: 600;
            margin-bottom: 5px;
            color: #333;
        }
        .chart-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin-bottom: 30px;
        }
        .chart { 
            height: 400px; 
            border: 1px solid #ddd;
            border-radius: 5px;
            background: white;
            position: relative;
        }
        .chart-title {
            font-weight: 600;
            padding: 10px;
            background: #f8f9fa;
            border-bottom: 1px solid #ddd;
            margin: 0;
            font-size: 16px;
        }
        .chart-error {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            color: #dc3545;
            text-align: center;
            font-size: 14px;
        }
        button { 
            background: #007bff; 
            color: white; 
            border: none; 
            padding: 12px 24px; 
            border-radius: 5px; 
            cursor: pointer;
            font-weight: 600;
            transition: background-color 0.2s;
        }
        button:hover {
            background: #0056b3;
        }
        button.debug-toggle {
            background: #ffc107;
            color: #212529;
            font-size: 12px;
            padding: 8px 16px;
        }
        input, select { 
            padding: 10px; 
            border: 1px solid #ddd; 
            border-radius: 4px;
            font-size: 14px;
        }
        .stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-top: 20px;
        }
        .stat-card {
            background: #f8f9fa;
            padding: 15px;
            border-radius: 5px;
            border-left: 4px solid #28a745;
            text-align: center;
        }
        .stat-value {
            font-size: 24px;
            font-weight: bold;
            color: #333;
        }
        .stat-label {
            font-size: 14px;
            color: #666;
            margin-top: 5px;
        }
        .percentile-comparison {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 15px;
            margin: 20px 0;
        }
        .percentile-card {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }
        .percentile-card.dots {
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
        }
        .percentile-value {
            font-size: 32px;
            font-weight: bold;
            margin-bottom: 5px;
        }
        .percentile-label {
            font-size: 16px;
            opacity: 0.9;
        }
        @media (max-width: 768px) {
            .chart-grid {
                grid-template-columns: 1fr;
            }
            .percentile-comparison {
                grid-template-columns: 1fr;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🏋️ Iron Insights</h1>
            <p>High-Performance Powerlifting Analytics with DOTS Scoring</p>
            <div class="dots-info">
                <strong>DOTS (Dots Total)</strong> is the modern replacement for Wilks, providing more accurate strength comparisons across different bodyweights and genders using a single, unified formula.
            </div>
            <div id="debugInfo" class="debug-info"></div>
            <button class="debug-toggle" onclick="toggleDebug()">Toggle Debug Info</button>
        </div>
        
        <div class="controls">
            <div class="control-group">
                <label>Sex:</label>
                <select id="sex">
                    <option value="M">Male</option>
                    <option value="F">Female</option>
                    <option value="All">All</option>
                </select>
            </div>
            <div class="control-group">
                <label>Lift Type:</label>
                <select id="liftType">
                    <option value="squat">Squat</option>
                    <option value="bench">Bench Press</option>
                    <option value="deadlift">Deadlift</option>
                    <option value="total">Total</option>
                </select>
            </div>
            <div class="control-group">
                <label>Your Bodyweight (kg):</label>
                <input type="number" id="bodyweight" placeholder="75" step="0.1">
            </div>
            <div class="control-group">
                <label>Your Lift (kg):</label>
                <input type="number" id="userLift" placeholder="150" step="0.5">
            </div>
            <div class="control-group">
                <button onclick="updateCharts()">Update Analytics</button>
            </div>
        </div>
        
        <div class="chart-grid">
            <div>
                <h3 class="chart-title">Raw Weight Distribution</h3>
                <div id="histogram" class="chart">
                    <div id="histogramError" class="chart-error" style="display: none;">
                        No data available for this lift type
                    </div>
                </div>
            </div>
            <div>
                <h3 class="chart-title">DOTS Score Distribution</h3>
                <div id="dotsHistogram" class="chart">
                    <div id="dotsHistogramError" class="chart-error" style="display: none;">
                        No DOTS data available - check data processing
                    </div>
                </div>
            </div>
            <div>
                <h3 class="chart-title">Raw Weight vs Bodyweight</h3>
                <div id="scatter" class="chart">
                    <div id="scatterError" class="chart-error" style="display: none;">
                        No scatter data available
                    </div>
                </div>
            </div>
            <div>
                <h3 class="chart-title">DOTS vs Bodyweight</h3>
                <div id="dotsScatter" class="chart">
                    <div id="dotsScatterError" class="chart-error" style="display: none;">
                        No DOTS scatter data available
                    </div>
                </div>
            </div>
        </div>
        
        <div id="percentiles"></div>
        <div id="stats"></div>
    </div>
    
    <script>
        let debugMode = false;
        let lastResponse = null;
        
        function toggleDebug() {
            debugMode = !debugMode;
            const debugInfo = document.getElementById('debugInfo');
            debugInfo.style.display = debugMode ? 'block' : 'none';
            
            if (debugMode && lastResponse) {
                showDebugInfo(lastResponse);
            }
        }
        
        function showDebugInfo(data) {
            if (!debugMode) return;
            
            const debugInfo = document.getElementById('debugInfo');
            debugInfo.innerHTML = `
                <strong>Debug Information:</strong><br>
                Raw histogram values: ${data.histogram_data.values.length}<br>
                DOTS histogram values: ${data.dots_histogram_data.values.length}<br>
                Raw scatter points: ${data.scatter_data.x.length}<br>
                DOTS scatter points: ${data.dots_scatter_data.x.length}<br>
                Processing time: ${data.processing_time_ms}ms<br>
                Total records: ${data.total_records}<br>
                User percentile: ${data.user_percentile}<br>
                User DOTS percentile: ${data.user_dots_percentile}
            `;
        }
        
        function showError(chartId, message) {
            const errorElement = document.getElementById(chartId + 'Error');
            if (errorElement) {
                errorElement.style.display = 'block';
                errorElement.textContent = message;
            }
        }
        
        function hideError(chartId) {
            const errorElement = document.getElementById(chartId + 'Error');
            if (errorElement) {
                errorElement.style.display = 'none';
            }
        }
        
        function createPlot(chartId, traces, layout, errorMessage = 'No data available') {
            if (!traces || traces.length === 0 || 
                (traces[0].x && traces[0].x.length === 0) ||
                (traces[0].values && traces[0].values.length === 0)) {
                showError(chartId, errorMessage);
                return false;
            }
            
            hideError(chartId);
            Plotly.newPlot(chartId, traces, layout);
            return true;
        }
        
        async function updateCharts() {
            const liftType = document.getElementById('liftType').value;
            const bodyweight = parseFloat(document.getElementById('bodyweight').value);
            const userLift = parseFloat(document.getElementById('userLift').value);
            
            const params = {
                sex: document.getElementById('sex').value,
                lift_type: liftType,
                bodyweight: bodyweight || null,
                squat: liftType === 'squat' ? userLift : null,
                bench: liftType === 'bench' ? userLift : null,
                deadlift: liftType === 'deadlift' ? userLift : null,
                equipment: ["Raw"]
            };
            
            // Handle total lift type
            if (liftType === 'total' && userLift) {
                params.squat = userLift * 0.35;
                params.bench = userLift * 0.25;
                params.deadlift = userLift * 0.40;
            }
            
            try {
                const response = await fetch('/api/visualize', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(params)
                });
                
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                
                const data = await response.json();
                lastResponse = data;
                
                if (debugMode) {
                    showDebugInfo(data);
                }
                
                // Create raw weight histogram
                const histogramTraces = [{
                    x: data.histogram_data.values,
                    type: 'histogram',
                    nbinsx: 50,
                    name: 'Distribution',
                    marker: { color: '#3498db' }
                }];
                
                // Add user input indicator line if user has entered a lift value
                if (userLift && !isNaN(userLift)) {
                    // Calculate approximate max height for the line
                    const maxValue = Math.max(...data.histogram_data.values);
                    const minValue = Math.min(...data.histogram_data.values);
                    const range = maxValue - minValue;
                    const binSize = range / 50; // nbinsx is 50
                    const estimatedMaxCount = data.histogram_data.values.length / 10; // Rough estimate
                    
                    histogramTraces.push({
                        x: [userLift, userLift],
                        y: [0, estimatedMaxCount],
                        mode: 'lines',
                        type: 'scatter',
                        name: 'Your Lift',
                        line: { color: '#e74c3c', width: 3, dash: 'dash' },
                        showlegend: true
                    });
                }
                
                const histogramSuccess = createPlot('histogram', histogramTraces, {
                    title: '',
                    xaxis: { title: 'Weight (kg)' },
                    yaxis: { title: 'Frequency' },
                    margin: { t: 20 }
                }, 'No raw weight data available for this lift type');
                
                // Create DOTS histogram
                const dotsHistogramTraces = [{
                    x: data.dots_histogram_data.values,
                    type: 'histogram',
                    nbinsx: 50,
                    name: 'DOTS Distribution',
                    marker: { color: '#e74c3c' }
                }];
                
                // Add user DOTS score indicator line if user has entered values
                if (userLift && bodyweight && !isNaN(userLift) && !isNaN(bodyweight)) {
                    const userDotsScore = calculateDOTS(userLift, bodyweight);
                    // Calculate approximate max height for the line
                    const estimatedMaxCount = data.dots_histogram_data.values.length / 10; // Rough estimate
                    
                    dotsHistogramTraces.push({
                        x: [userDotsScore, userDotsScore],
                        y: [0, estimatedMaxCount],
                        mode: 'lines',
                        type: 'scatter',
                        name: 'Your DOTS',
                        line: { color: '#f39c12', width: 3, dash: 'dash' },
                        showlegend: true
                    });
                }
                
                const dotsHistogramSuccess = createPlot('dotsHistogram', dotsHistogramTraces, {
                    title: '',
                    xaxis: { title: 'DOTS Score' },
                    yaxis: { title: 'Frequency' },
                    margin: { t: 20 }
                }, 'No DOTS data available - check data processing or try different filters');
                
                // Create scatter plots
                const maleData = data.scatter_data.x.map((x, i) => ({
                    x: x, y: data.scatter_data.y[i], sex: data.scatter_data.sex[i]
                })).filter(d => d.sex === 'M');
                
                const femaleData = data.scatter_data.x.map((x, i) => ({
                    x: x, y: data.scatter_data.y[i], sex: data.scatter_data.sex[i]
                })).filter(d => d.sex === 'F');
                
                const scatterTraces = [];
                if (maleData.length > 0) {
                    scatterTraces.push({
                        x: maleData.map(d => d.x), y: maleData.map(d => d.y),
                        mode: 'markers', type: 'scatter',
                        marker: { size: 3, opacity: 0.6, color: '#3498db' }, name: 'Male'
                    });
                }
                if (femaleData.length > 0) {
                    scatterTraces.push({
                        x: femaleData.map(d => d.x), y: femaleData.map(d => d.y),
                        mode: 'markers', type: 'scatter',
                        marker: { size: 3, opacity: 0.6, color: '#e91e63' }, name: 'Female'
                    });
                }
                
                // Add user point to scatter plot if both values are provided
                if (bodyweight && userLift && !isNaN(bodyweight) && !isNaN(userLift)) {
                    scatterTraces.push({
                        x: [bodyweight],
                        y: [userLift],
                        mode: 'markers',
                        type: 'scatter',
                        name: 'Your Lift',
                        marker: { 
                            size: 12, 
                            color: '#e74c3c', 
                            symbol: 'star',
                            line: { width: 2, color: '#fff' }
                        },
                        showlegend: true
                    });
                }
                
                const scatterSuccess = createPlot('scatter', scatterTraces, {
                    title: '', xaxis: { title: 'Bodyweight (kg)' }, yaxis: { title: 'Weight (kg)' }, margin: { t: 20 }
                }, 'No scatter plot data available');
                
                // Create DOTS scatter plot
                const maleDotsData = data.dots_scatter_data.x.map((x, i) => ({
                    x: x, y: data.dots_scatter_data.y[i], sex: data.dots_scatter_data.sex[i]
                })).filter(d => d.sex === 'M');
                
                const femaleDotsData = data.dots_scatter_data.x.map((x, i) => ({
                    x: x, y: data.dots_scatter_data.y[i], sex: data.dots_scatter_data.sex[i]
                })).filter(d => d.sex === 'F');
                
                const dotsScatterTraces = [];
                if (maleDotsData.length > 0) {
                    dotsScatterTraces.push({
                        x: maleDotsData.map(d => d.x), y: maleDotsData.map(d => d.y),
                        mode: 'markers', type: 'scatter',
                        marker: { size: 3, opacity: 0.6, color: '#3498db' }, name: 'Male'
                    });
                }
                if (femaleDotsData.length > 0) {
                    dotsScatterTraces.push({
                        x: femaleDotsData.map(d => d.x), y: femaleDotsData.map(d => d.y),
                        mode: 'markers', type: 'scatter',
                        marker: { size: 3, opacity: 0.6, color: '#e91e63' }, name: 'Female'
                    });
                }
                
                // Add user point to DOTS scatter plot if both values are provided
                if (bodyweight && userLift && !isNaN(bodyweight) && !isNaN(userLift)) {
                    const userDotsScore = calculateDOTS(userLift, bodyweight);
                    dotsScatterTraces.push({
                        x: [bodyweight],
                        y: [userDotsScore],
                        mode: 'markers',
                        type: 'scatter',
                        name: 'Your DOTS',
                        marker: { 
                            size: 12, 
                            color: '#f39c12', 
                            symbol: 'star',
                            line: { width: 2, color: '#fff' }
                        },
                        showlegend: true
                    });
                }
                
                const dotsScatterSuccess = createPlot('dotsScatter', dotsScatterTraces, {
                    title: '', xaxis: { title: 'Bodyweight (kg)' }, yaxis: { title: 'DOTS Score' }, margin: { t: 20 }
                }, 'No DOTS scatter data available - check DOTS calculations');
                
                // Show percentile comparison
                let percentilesHtml = '';
                if (data.user_percentile !== null && data.user_dots_percentile !== null) {
                    const userDots = bodyweight && userLift ? calculateDOTS(userLift, bodyweight) : null;
                    
                    percentilesHtml = `
                        <div class="percentile-comparison">
                            <div class="percentile-card">
                                <div class="percentile-value">${data.user_percentile}%</div>
                                <div class="percentile-label">Raw Weight Percentile</div>
                            </div>
                            <div class="percentile-card dots">
                                <div class="percentile-value">${data.user_dots_percentile}%</div>
                                <div class="percentile-label">DOTS Percentile</div>
                            </div>
                        </div>
                        ${userDots ? `<p style="text-align: center; margin-top: 10px;"><strong>Your DOTS Score: ${userDots.toFixed(1)}</strong></p>` : ''}
                    `;
                }
                document.getElementById('percentiles').innerHTML = percentilesHtml;
                
                // Show stats with DOTS status
                const dotsStatus = dotsHistogramSuccess && dotsScatterSuccess ? '✅ Working' : '❌ No Data';
                document.getElementById('stats').innerHTML = `
                    <div class="stats">
                        <div class="stat-card">
                            <div class="stat-value">${data.processing_time_ms}ms</div>
                            <div class="stat-label">Processing Time</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-value">${data.total_records.toLocaleString()}</div>
                            <div class="stat-label">Records Analyzed</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-value">${liftType.charAt(0).toUpperCase() + liftType.slice(1)}</div>
                            <div class="stat-label">Current Lift</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-value">${dotsStatus}</div>
                            <div class="stat-label">DOTS Charts</div>
                        </div>
                    </div>
                `;
                
            } catch (error) {
                console.error('Error:', error);
                document.getElementById('stats').innerHTML = `<p style="color: red;">Error loading data: ${error.message}</p>`;
                
                // Show errors on all charts
                showError('histogram', 'Failed to load data');
                showError('dotsHistogram', 'Failed to load data');
                showError('scatter', 'Failed to load data');
                showError('dotsScatter', 'Failed to load data');
            }
        }
        
        // Helper function to calculate DOTS score
        function calculateDOTS(liftKg, bodyweightKg) {
            const a = -307.75076;
            const b = 24.0900756;
            const c = -0.1918759221;
            const d = 0.0007391293;
            const e = -0.000001093;
            
            const denominator = a + 
                b * bodyweightKg +
                c * Math.pow(bodyweightKg, 2) +
                d * Math.pow(bodyweightKg, 3) +
                e * Math.pow(bodyweightKg, 4);
            
            return liftKg * 500.0 / denominator;
        }
        
        // Load initial data
        updateCharts();
    </script>
</body>
</html>
"#;