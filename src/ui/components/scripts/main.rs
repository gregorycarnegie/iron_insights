use maud::{Markup, PreEscaped};

pub fn render_main_scripts() -> Markup {
    PreEscaped(r#"
        async function updateCharts() {
            console.log('updateCharts called');
            const bodyweight = parseFloat(document.getElementById('bodyweight').value);
            
            const userLiftElement = document.getElementById('userLift');
            console.log('userLift element:', userLiftElement);
            console.log('userLift element exists:', !!userLiftElement);
            
            if (!userLiftElement) {
                console.error('userLift input element not found!');
                return;
            }
            
            const userLiftInput = userLiftElement.value;
            console.log('userLiftInput raw value:', `"${userLiftInput}"`);
            console.log('userLiftInput length:', userLiftInput.length);
            
            const userLift = parseSum(userLiftInput);
            console.log('userLift parsed value:', userLift);
            
            // Show calculated sum in tooltip if it's different from input
            if (userLiftInput && userLiftInput !== userLift.toString() && !isNaN(userLift)) {
                const userLiftElement = document.getElementById('userLift');
                if (userLiftElement && (/[+\-]/.test(userLiftInput) && userLiftInput !== userLift.toString())) {
                    userLiftElement.title = `Expression "${userLiftInput}" = ${userLift} kg`;
                    console.log('Set tooltip:', userLiftElement.title);
                } else if (userLiftElement) {
                    userLiftElement.title = '';
                }
            }
            
            const params = {
                sex: currentSex,
                lift_type: currentLiftType,
                bodyweight: bodyweight || null,
                squat: currentLiftType === 'squat' ? userLift : null,
                bench: currentLiftType === 'bench' ? userLift : null,
                deadlift: currentLiftType === 'deadlift' ? userLift : null,
                equipment: currentEquipment,
                years_filter: currentTimePeriod
            };
            
            // Handle total lift type
            if (currentLiftType === 'total' && userLift) {
                params.squat = userLift * 0.35;
                params.bench = userLift * 0.25;
                params.deadlift = userLift * 0.40;
            }
            
            try {
                console.log('📊 Fetching data using Arrow IPC exclusively...');
                const data = await fetchArrowData(params);
                lastResponse = data;
                
                if (debugMode) {
                    showDebugInfo(data);
                }
                
                // Create raw weight histogram
                const histogramTraces = [{
                    x: data.histogram_data.values,
                    type: 'histogram',
                    nbinsx: currentBinCount,
                    name: 'Distribution',
                    marker: { color: '#3498db' }
                }];
                
                // Add user input indicator line if user has entered a lift value
                if (userLift && !isNaN(userLift)) {
                    const estimatedMaxCount = data.histogram_data.values.length / 10;
                    
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
                
                const histogramSuccess = createPlotEnhanced('weightDistribution', histogramTraces, {
                    title: '',
                    xaxis: { title: 'Weight (kg)' },
                    yaxis: { title: 'Frequency' },
                    margin: { t: 20 }
                }, 'No raw weight data available for this lift type');
                
                // Create DOTS histogram
                const dotsHistogramTraces = [{
                    x: data.dots_histogram_data.values,
                    type: 'histogram',
                    nbinsx: currentBinCount,
                    name: 'DOTS Distribution',
                    marker: { color: '#e74c3c' }
                }];
                
                // Add user DOTS score indicator line if user has entered values
                if (userLift && bodyweight && !isNaN(userLift) && !isNaN(bodyweight)) {
                    console.log('Adding DOTS indicator with:', {userLift, bodyweight, currentSex});
                    
                    let userDotsScore;
                    try {
                        userDotsScore = calculateDOTS(userLift, bodyweight, currentSex);
                        console.log('Calculated DOTS score:', userDotsScore);
                    } catch (error) {
                        console.error('Error calculating DOTS:', error);
                        // Fallback calculation without currentSex dependency
                        const isMale = currentSex === 'M' || currentSex === 'Male';
                        const a = isMale ? -307.75076 : -57.96288;
                        const b = isMale ? 24.0900756 : 13.6175032;
                        const c = isMale ? -0.1918759221 : -0.1126655495;
                        const d = isMale ? 0.0007391293 : 0.0005158568;
                        const e = isMale ? -0.000001093 : -0.0000010706;
                        
                        const denominator = a + 
                            b * bodyweight +
                            c * Math.pow(bodyweight, 2) +
                            d * Math.pow(bodyweight, 3) +
                            e * Math.pow(bodyweight, 4);
                        
                        userDotsScore = userLift * 500.0 / denominator;
                        console.log('Fallback DOTS calculation:', userDotsScore);
                    }
                    
                    if (userDotsScore && !isNaN(userDotsScore)) {
                        const estimatedMaxCount = data.dots_histogram_data.values.length / 10;
                        
                        dotsHistogramTraces.push({
                            x: [userDotsScore, userDotsScore],
                            y: [0, estimatedMaxCount],
                            mode: 'lines',
                            type: 'scatter',
                            name: 'Your DOTS',
                            line: { color: '#f39c12', width: 3, dash: 'dash' },
                            showlegend: true
                        });
                        
                        console.log('Added DOTS indicator trace:', {
                            x: [userDotsScore, userDotsScore],
                            y: [0, estimatedMaxCount]
                        });
                    } else {
                        console.warn('Invalid DOTS score calculated:', userDotsScore);
                    }
                }
                
                // Update user metrics display
                updateUserMetrics(bodyweight, userLift);
                
                // Send user update via WebSocket for real-time activity
                sendUserUpdate(bodyweight, 
                    currentLiftType === 'squat' ? userLift : null,
                    currentLiftType === 'bench' ? userLift : null,
                    currentLiftType === 'deadlift' ? userLift : null,
                    currentLiftType);
                
                const dotsHistogramSuccess = createPlotEnhanced('dotsDistribution', dotsHistogramTraces, {
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
                        mode: 'markers', type: 'scattergl',
                        marker: { size: 3, opacity: 0.6, color: '#3498db' }, name: 'Male'
                    });
                }
                if (femaleData.length > 0) {
                    scatterTraces.push({
                        x: femaleData.map(d => d.x), y: femaleData.map(d => d.y),
                        mode: 'markers', type: 'scattergl',
                        marker: { size: 3, opacity: 0.6, color: '#e91e63' }, name: 'Female'
                    });
                }
                
                // Add user point on top of GL points: use a GL marker + SVG text star overlay
                if (bodyweight && userLift && !isNaN(bodyweight) && !isNaN(userLift)) {
                    // WebGL marker for proper layering with scattergl traces
                    scatterTraces.push({
                        x: [bodyweight],
                        y: [userLift],
                        mode: 'markers',
                        type: 'scattergl',
                        name: 'Your Lift',
                        marker: {
                            size: 14,
                            color: '#e74c3c',
                            symbol: 'x', // scattergl-safe symbol
                            line: { width: 2, color: '#fff' }
                        },
                        showlegend: true
                    });
                    // SVG text overlay star to ensure visibility above WebGL layer
                    scatterTraces.push({
                        x: [bodyweight],
                        y: [userLift],
                        mode: 'text',
                        type: 'scatter',
                        text: ['★'],
                        textposition: 'middle center',
                        textfont: { size: 22, color: '#e74c3c' },
                        hoverinfo: 'skip',
                        showlegend: false
                    });
                }
                
                const scatterSuccess = createPlotEnhanced('bodyweightScatter', scatterTraces, {
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
                        mode: 'markers', type: 'scattergl',
                        marker: { size: 3, opacity: 0.6, color: '#3498db' }, name: 'Male'
                    });
                }
                if (femaleDotsData.length > 0) {
                    dotsScatterTraces.push({
                        x: femaleDotsData.map(d => d.x), y: femaleDotsData.map(d => d.y),
                        mode: 'markers', type: 'scattergl',
                        marker: { size: 3, opacity: 0.6, color: '#e91e63' }, name: 'Female'
                    });
                }
                
                // Add user point to DOTS scatter with GL marker + SVG star overlay
                if (bodyweight && userLift && !isNaN(bodyweight) && !isNaN(userLift)) {
                    const userDotsScore = calculateDOTS(userLift, bodyweight);
                    // WebGL marker
                    dotsScatterTraces.push({
                        x: [bodyweight],
                        y: [userDotsScore],
                        mode: 'markers',
                        type: 'scattergl',
                        name: 'Your DOTS',
                        marker: {
                            size: 14,
                            color: '#f39c12',
                            symbol: 'x',
                            line: { width: 2, color: '#fff' }
                        },
                        showlegend: true
                    });
                    // SVG text overlay star
                    dotsScatterTraces.push({
                        x: [bodyweight],
                        y: [userDotsScore],
                        mode: 'text',
                        type: 'scatter',
                        text: ['★'],
                        textposition: 'middle center',
                        textfont: { size: 22, color: '#f39c12' },
                        hoverinfo: 'skip',
                        showlegend: false
                    });
                }
                
                const dotsScatterSuccess = createPlotEnhanced('dotsScatter', dotsScatterTraces, {
                    title: '', xaxis: { title: 'Bodyweight (kg)' }, yaxis: { title: 'DOTS Score' }, margin: { t: 20 }
                }, 'No DOTS scatter data available - check DOTS calculations');
                
                // Update percentile display in modern UI
                if (data.user_percentile !== null && data.user_dots_percentile !== null) {
                    document.getElementById('rawPercentile').textContent = data.user_percentile + '%';
                    document.getElementById('dotsPercentile').textContent = data.user_dots_percentile + '%';
                    document.getElementById('percentileGrid').style.display = 'grid';
                } else {
                    document.getElementById('percentileGrid').style.display = 'none';
                }
                
                // Update modern UI stats grid
                document.getElementById('processingTime').textContent = data.processing_time_ms + 'ms';
                document.getElementById('recordsAnalyzed').textContent = data.total_records.toLocaleString();
                document.getElementById('totalAthletes').textContent = data.total_records.toLocaleString();
                
                // Update average DOTS display if element exists
                if (bodyweight && userLift && !isNaN(bodyweight) && !isNaN(userLift)) {
                    const avgDotsEl = document.getElementById('avgDots');
                    if (avgDotsEl) {
                        const dots = calculateDOTS(userLift, bodyweight);
                        avgDotsEl.textContent = dots.toFixed(1);
                    }
                }
                
            } catch (error) {
                console.error('Error:', error);
                document.getElementById('processingTime').textContent = 'Error';
                document.getElementById('recordsAnalyzed').textContent = 'Error';
                
                // Show errors on all charts
                showError('weightDistribution', 'Failed to load data');
                showError('dotsDistribution', 'Failed to load data');
                showError('bodyweightScatter', 'Failed to load data');
                showError('dotsScatter', 'Failed to load data');
            }
        }
        
        // Initialize WASM and load initial data (analytics page only)
        async function init() {
            // Gate: only run on analytics page to reduce TBT on others
            const weightDist = document.getElementById('weightDistribution');
            const dotsScatter = document.getElementById('dotsScatter');
            const isAnalytics = weightDist && dotsScatter;

            if (!isAnalytics) {
                console.log('📍 Not an analytics page, skipping heavy initialization');
                return; // Skip heavy analytics initialization on non-analytics pages
            }

            console.log('📊 Analytics page detected, initializing...');

            await initWasm();
            // Load analytics dependencies using lazy loader for performance
            setTimeout(async () => {
                try {
                    await loadAnalyticsDependencies();
                    initWebSocket();
                    setupEquipmentFilters();
                    setupInputDebugger();
                    // Schedule chart rendering with a small delay to avoid long tasks during TTI window
                    setTimeout(() => {
                        updateCharts();
                        // Setup crossfiltering after charts are fully rendered and ready
                        setTimeout(() => {
                            console.log('Setting up chart crossfiltering...');
                            setupChartCrossfiltering();
                        }, 2000);
                    }, 0);
                } catch (error) {
                    console.error('❌ Failed to load analytics dependencies:', error);
                    // Show error message to user
                    const container = document.querySelector('.main-content');
                    if (container) {
                        container.innerHTML = '<div style="text-align: center; padding: 2rem; color: #dc3545;">Failed to load required dependencies. Please refresh the page.</div>';
                    }
                }
            }, 0);
        }
        
        // Start the application when page loads
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', init);
        } else {
            init();
        }
    "#.to_string())
}
