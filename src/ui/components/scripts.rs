// src/ui/components/scripts.rs - JavaScript functionality
use maud::{Markup, PreEscaped};

pub fn render_scripts() -> Markup {
    PreEscaped(r#"
    <script>
        let debugMode = false;
        let lastResponse = null;
        let wasmModule = null;
        let calculate_dots_wasm = null;
        let calculate_strength_level_wasm = null;
        let get_strength_level_color_wasm = null;
        let calculate_dots_and_level_wasm = null;
        
        // UI state management for modern toggle-based controls
        let currentSex = 'M';
        let currentLiftType = 'squat';
        let currentEquipment = ['Raw'];
        let currentTimePeriod = 'last_5_years';
        let currentFederation = 'all';
        
        // WebSocket state
        let websocket = null;
        let isConnected = false;
        let reconnectAttempts = 0;
        let maxReconnectAttempts = 5;
        let sessionId = 'session_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
        
        // Initialize WASM module
        async function initWasm() {
            try {
                const wasmModule = await import('/static/wasm/iron_insights_wasm.js');
                await wasmModule.default();
                
                // Store WASM functions globally
                calculate_dots_wasm = wasmModule.calculate_dots;
                calculate_strength_level_wasm = wasmModule.calculate_strength_level;
                get_strength_level_color_wasm = wasmModule.get_strength_level_color;
                calculate_dots_and_level_wasm = wasmModule.calculate_dots_and_level;
                
                console.log('✅ WASM module loaded successfully');
                return true;
            } catch (error) {
                console.error('❌ Failed to load WASM module:', error);
                console.log('📋 Falling back to JavaScript implementation');
                return false;
            }
        }
        
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
            debugInfo.innerHTML = '<strong>Debug Information:</strong><br>' +
                'Raw histogram values: ' + data.histogram_data.values.length + '<br>' +
                'DOTS histogram values: ' + data.dots_histogram_data.values.length + '<br>' +
                'Raw scatter points: ' + data.scatter_data.x.length + '<br>' +
                'DOTS scatter points: ' + data.dots_scatter_data.x.length + '<br>' +
                'Processing time: ' + data.processing_time_ms + 'ms<br>' +
                'Total records: ' + data.total_records + '<br>' +
                'User percentile: ' + data.user_percentile + '<br>' +
                'User DOTS percentile: ' + data.user_dots_percentile;
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
            // Enhanced Plotly config for GPU acceleration
            const config = {
                displayModeBar: false,
                staticPlot: false,
                responsive: true,
                webGlRenderer: true
            };
            Plotly.react(chartId, traces, layout, config);
            return true;
        }
        
        // Function to fetch and parse comprehensive Arrow data
        async function fetchArrowData(params) {
            // Check if Arrow is available
            if (!Arrow || typeof Arrow.tableFromIPC !== 'function') {
                throw new Error('Apache Arrow library not loaded or tableFromIPC not available');
            }
            
            try {
                const response = await fetch('/api/visualize-arrow', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(params)
                });
                
                if (!response.ok) {
                    throw new Error('HTTP error! status: ' + response.status);
                }
                
                const arrayBuffer = await response.arrayBuffer();
                const uint8Array = new Uint8Array(arrayBuffer);
                
                // Parse Arrow IPC stream
                const table = Arrow.tableFromIPC(uint8Array);
                
                if (!table || table.length === 0) {
                    throw new Error('No Arrow data received');
                }
                
                // Parse data by data_type
                const result = {
                    histogram_data: { values: [], counts: [], bins: [], min_val: 0, max_val: 0 },
                    scatter_data: { x: [], y: [], sex: [] },
                    dots_histogram_data: { values: [], counts: [], bins: [], min_val: 0, max_val: 0 },
                    dots_scatter_data: { x: [], y: [], sex: [] },
                    user_percentile: parseFloat(response.headers.get('X-User-Percentile')) || null,
                    user_dots_percentile: parseFloat(response.headers.get('X-User-Dots-Percentile')) || null,
                    processing_time_ms: parseInt(response.headers.get('X-Processing-Time-Ms')) || 0,
                    total_records: parseInt(response.headers.get('X-Total-Records')) || 0
                };
                
                // Get column arrays from the table using getChildAt
                const dataTypes = table.getChildAt(0).toArray();  // data_type column
                const histValues = table.getChildAt(1).toArray(); // hist_values column
                const histCounts = table.getChildAt(2).toArray(); // hist_counts column  
                const histBins = table.getChildAt(3).toArray();   // hist_bins column
                const scatterX = table.getChildAt(4).toArray();   // scatter_x column
                const scatterY = table.getChildAt(5).toArray();   // scatter_y column
                const scatterSex = table.getChildAt(6).toArray(); // scatter_sex column
                
                for (let i = 0; i < dataTypes.length; i++) {
                    const dataType = dataTypes[i];
                    
                    if (dataType === 'raw_histogram') {
                        if (histValues[i] > 0) result.histogram_data.values.push(histValues[i]);
                        if (histCounts[i] > 0) result.histogram_data.counts.push(histCounts[i]);
                        if (histBins[i] > 0) result.histogram_data.bins.push(histBins[i]);
                    } else if (dataType === 'raw_scatter') {
                        if (scatterX[i] > 0 && scatterY[i] > 0) {
                            result.scatter_data.x.push(scatterX[i]);
                            result.scatter_data.y.push(scatterY[i]);
                            result.scatter_data.sex.push(scatterSex[i]);
                        }
                    } else if (dataType === 'dots_histogram') {
                        if (histValues[i] > 0) result.dots_histogram_data.values.push(histValues[i]);
                        if (histCounts[i] > 0) result.dots_histogram_data.counts.push(histCounts[i]);
                        if (histBins[i] > 0) result.dots_histogram_data.bins.push(histBins[i]);
                    } else if (dataType === 'dots_scatter') {
                        if (scatterX[i] > 0 && scatterY[i] > 0) {
                            result.dots_scatter_data.x.push(scatterX[i]);
                            result.dots_scatter_data.y.push(scatterY[i]);
                            result.dots_scatter_data.sex.push(scatterSex[i]);
                        }
                    }
                }
                
                // Calculate min/max for histograms
                if (result.histogram_data.values.length > 0) {
                    result.histogram_data.min_val = Math.min(...result.histogram_data.values);
                    result.histogram_data.max_val = Math.max(...result.histogram_data.values);
                }
                if (result.dots_histogram_data.values.length > 0) {
                    result.dots_histogram_data.min_val = Math.min(...result.dots_histogram_data.values);
                    result.dots_histogram_data.max_val = Math.max(...result.dots_histogram_data.values);
                }
                
                console.log('✅ Arrow IPC data parsed successfully:', result);
                return result;
                
            } catch (error) {
                console.error('❌ Arrow data fetch error:', error);
                throw error;
            }
        }

        // Modern UI control functions
        function setToggle(element, type) {
            // Remove active class from siblings
            element.parentElement.querySelectorAll('.toggle-button').forEach(btn => {
                btn.classList.remove('active');
            });
            // Add active class to clicked element
            element.classList.add('active');
            
            // Update global state
            const value = element.getAttribute('data-value');
            if (type === 'sex') {
                currentSex = value;
            } else if (type === 'lift') {
                currentLiftType = value;
            }
            
            // Update charts when toggle changes
            updateCharts();
        }
        
        function updateEquipment() {
            // Get all checked equipment checkboxes
            currentEquipment = [];
            if (document.getElementById('equipment-raw').checked) currentEquipment.push("Raw");
            if (document.getElementById('equipment-wraps').checked) currentEquipment.push("Wraps");
            if (document.getElementById('equipment-single-ply').checked) currentEquipment.push("Single-ply");
            if (document.getElementById('equipment-multi-ply').checked) currentEquipment.push("Multi-ply");
            
            // Default to Raw if nothing selected
            if (currentEquipment.length === 0) {
                currentEquipment.push("Raw");
            }
            
            updateCharts();
        }
        
        function updateAnalytics() {
            updateCharts();
        }

        async function updateCharts() {
            const bodyweight = parseFloat(document.getElementById('bodyweight').value);
            const userLift = parseFloat(document.getElementById('userLift').value);
            
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
                    nbinsx: 50,
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
                
                const histogramSuccess = createPlot('weightDistribution', histogramTraces, {
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
                }
                
                // Update user metrics display
                updateUserMetrics(bodyweight, userLift);
                
                // Send user update via WebSocket for real-time activity
                sendUserUpdate(bodyweight, 
                    currentLiftType === 'squat' ? userLift : null,
                    currentLiftType === 'bench' ? userLift : null,
                    currentLiftType === 'deadlift' ? userLift : null,
                    currentLiftType);
                
                const dotsHistogramSuccess = createPlot('dotsDistribution', dotsHistogramTraces, {
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
                
                const scatterSuccess = createPlot('bodyweightScatter', scatterTraces, {
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
                
                // Update DOTS score if available
                const userDots = bodyweight && userLift ? calculateDOTS(userLift, bodyweight) : null;
                if (userDots) {
                    document.getElementById('avgDots').textContent = userDots.toFixed(1);
                    document.getElementById('userDotsScore').textContent = userDots.toFixed(1);
                    document.getElementById('userPerformance').style.display = 'block';
                    document.getElementById('liftBreakdown').style.display = 'grid';
                } else {
                    document.getElementById('userPerformance').style.display = 'none';
                    document.getElementById('liftBreakdown').style.display = 'none';
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
        
        // Helper function to calculate gender-specific DOTS score using WASM
        function calculateDOTS(liftKg, bodyweightKg, sex = null) {
            if (calculate_dots_wasm) {
                return calculate_dots_wasm(liftKg, bodyweightKg);
            } else {
                // Fallback JavaScript implementation with gender-specific coefficients
                const sexValue = sex || currentSex;
                
                let a, b, c, d, e;
                if (sexValue === 'M' || sexValue === 'Male') {
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
        
        // Function to update user metrics display for modern UI
        function updateUserMetrics(bodyweight, userLift) {
            if (bodyweight && userLift && !isNaN(bodyweight) && !isNaN(userLift)) {
                try {
                    let dots, level, color;
                    
                    if (calculate_dots_and_level_wasm) {
                        const result = calculate_dots_and_level_wasm(userLift, bodyweight);
                        dots = result.dots;
                        level = result.level;
                        color = result.color;
                    } else {
                        // Fallback calculation
                        dots = calculateDOTS(userLift, bodyweight);
                        level = getStrengthLevel(dots);
                        color = getStrengthLevelColor(level);
                    }
                    
                    // Update strength level display
                    const strengthLevelElement = document.getElementById('strengthLevel');
                    if (strengthLevelElement) {
                        strengthLevelElement.innerHTML = '<div class="strength-badge" style="background-color: ' + color + ';">🏋️ ' + level + '</div>';
                    }
                    
                    // Update individual lift values in breakdown
                    if (currentLiftType === 'squat') {
                        document.getElementById('userSquatValue').textContent = userLift;
                    } else if (currentLiftType === 'bench') {
                        document.getElementById('userBenchValue').textContent = userLift;
                    } else if (currentLiftType === 'deadlift') {
                        document.getElementById('userDeadliftValue').textContent = userLift;
                    }
                    
                } catch (error) {
                    console.error('Error calculating user metrics:', error);
                }
            }
        }
        
        // Fallback strength level calculation
        function getStrengthLevel(dotsScore) {
            if (dotsScore < 200.0) return "Beginner";
            else if (dotsScore < 300.0) return "Novice";
            else if (dotsScore < 400.0) return "Intermediate";
            else if (dotsScore < 500.0) return "Advanced";
            else if (dotsScore < 600.0) return "Elite";
            else return "World Class";
        }
        
        // Fallback strength level color
        function getStrengthLevelColor(level) {
            return level === "Elite" ? "orange" : "blue";
        }
        
        // Initialize WebSocket connection
        function initWebSocket() {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = protocol + '//' + window.location.host + '/ws';
            
            try {
                websocket = new WebSocket(wsUrl);
                
                websocket.onopen = function(event) {
                    console.log('🔗 WebSocket connected');
                    isConnected = true;
                    reconnectAttempts = 0;
                    updateConnectionStatus(true);
                    
                    // Send connection message
                    const connectMsg = {
                        type: 'Connect',
                        session_id: sessionId,
                        user_agent: navigator.userAgent
                    };
                    websocket.send(JSON.stringify(connectMsg));
                };
                
                websocket.onmessage = function(event) {
                    try {
                        const data = JSON.parse(event.data);
                        handleWebSocketMessage(data);
                    } catch (error) {
                        console.error('Failed to parse WebSocket message:', error);
                    }
                };
                
                websocket.onclose = function(event) {
                    console.log('🔌 WebSocket disconnected');
                    isConnected = false;
                    updateConnectionStatus(false);
                    
                    // Attempt to reconnect
                    if (reconnectAttempts < maxReconnectAttempts) {
                        reconnectAttempts++;
                        console.log('🔄 Attempting to reconnect... (' + reconnectAttempts + '/' + maxReconnectAttempts + ')');
                        setTimeout(initWebSocket, 2000 * reconnectAttempts);
                    }
                };
                
                websocket.onerror = function(error) {
                    console.error('❌ WebSocket error:', error);
                };
                
            } catch (error) {
                console.error('❌ Failed to initialize WebSocket:', error);
                updateConnectionStatus(false);
            }
        }
        
        // Handle incoming WebSocket messages
        function handleWebSocketMessage(data) {
            switch (data.type) {
                case 'StatsUpdate':
                    updateLiveStats(data.active_users, data.total_connections, data.server_load);
                    break;
                    
                case 'UserActivity':
                    updateActivityFeed(data.recent_calculations);
                    updateUserCount(data.user_count);
                    break;
                    
                case 'ServerMetrics':
                    updateServerMetrics(data);
                    break;
                    
                case 'DotsCalculation':
                    // Handle real-time DOTS calculations from other users
                    addActivityItem('🏋️ ' + data.strength_level + ' lifter: ' + data.dots_score.toFixed(1) + ' DOTS');
                    break;
                    
                default:
                    console.log('Unknown WebSocket message type:', data.type);
            }
        }
        
        // Send user update via WebSocket
        function sendUserUpdate(bodyweight, squat, bench, deadlift, liftType) {
            if (isConnected && websocket) {
                const updateMsg = {
                    type: 'UserUpdate',
                    bodyweight: bodyweight,
                    squat: squat,
                    bench: bench,
                    deadlift: deadlift,
                    lift_type: liftType,
                    sex: currentSex
                };
                websocket.send(JSON.stringify(updateMsg));
            }
        }
        
        // Update connection status indicator
        function updateConnectionStatus(connected) {
            const statusElement = document.getElementById('connectionStatus');
            if (statusElement) {
                statusElement.className = 'connection-status' + (connected ? '' : ' disconnected');
            }
        }
        
        // Update live statistics
        function updateLiveStats(activeUsers, totalConnections, serverLoad) {
            document.getElementById('totalConnections').textContent = totalConnections;
            document.getElementById('serverLoad').textContent = (serverLoad * 100).toFixed(1) + '%';
        }
        
        // Update user count
        function updateUserCount(count) {
            document.getElementById('userCount').textContent = count + ' users online';
        }
        
        // Update activity feed
        function updateActivityFeed(recentCalculations) {
            const feed = document.getElementById('activityFeed');
            if (recentCalculations && recentCalculations.length > 0) {
                document.getElementById('calculationsCount').textContent = recentCalculations.length;
                
                // Show recent 10 calculations
                const recent = recentCalculations.slice(-10).reverse();
                feed.innerHTML = recent.map(calc => {
                    const timeAgo = getTimeAgo(calc.timestamp * 1000);
                    return '<div class="activity-item">🏋️ ' + calc.strength_level + ': ' + calc.dots_score.toFixed(1) + ' DOTS (' + calc.lift_type + ') - ' + timeAgo + '</div>';
                }).join('');
            }
        }
        
        // Add single activity item
        function addActivityItem(text) {
            const feed = document.getElementById('activityFeed');
            const item = document.createElement('div');
            item.className = 'activity-item';
            item.textContent = text + ' - just now';
            feed.insertBefore(item, feed.firstChild);
            
            // Keep only 10 items
            while (feed.children.length > 10) {
                feed.removeChild(feed.lastChild);
            }
        }
        
        // Get human-readable time ago
        function getTimeAgo(timestamp) {
            const now = Date.now();
            const diff = now - timestamp;
            const seconds = Math.floor(diff / 1000);
            
            if (seconds < 60) return 'just now';
            if (seconds < 3600) return Math.floor(seconds / 60) + 'm ago';
            if (seconds < 86400) return Math.floor(seconds / 3600) + 'h ago';
            return Math.floor(seconds / 86400) + 'd ago';
        }
        
        // Removed share card functionality - not part of modern UI
        
        // Load Apache Arrow dynamically
        let Arrow;
        async function loadArrow() {
            console.log('🔄 Loading Apache Arrow library...');
            try {
                // Try multiple CDNs
                const cdnUrls = [
                    'https://cdn.jsdelivr.net/npm/apache-arrow/Arrow.es2015.min.js',
                    'https://unpkg.com/apache-arrow@21.0.0/Arrow.es2015.min.js',
                    'https://unpkg.com/apache-arrow@14.0.2/dist/umd/Arrow.js'
                ];
                
                for (const url of cdnUrls) {
                    try {
                        console.log('🔄 Trying to load Arrow from:', url);
                        await loadScript(url);
                        if (typeof window.Arrow !== 'undefined') {
                            Arrow = window.Arrow;
                            console.log('✅ Apache Arrow library loaded successfully from:', url);
                            return true;
                        }
                    } catch (e) {
                        console.log('⚠️  Failed to load from', url, ':', e.message);
                    }
                }
                throw new Error('All CDNs failed');
            } catch (error) {
                console.error('❌ Failed to load Apache Arrow library:', error);
                return false;
            }
        }
        
        // Helper function to load script
        function loadScript(src) {
            return new Promise((resolve, reject) => {
                const script = document.createElement('script');
                script.src = src;
                script.onload = resolve;
                script.onerror = reject;
                document.head.appendChild(script);
            });
        }

        // Handle equipment checkbox logic for modern UI
        function setupEquipmentFilters() {
            const equipmentCheckboxes = [
                'equipment-raw', 'equipment-wraps', 'equipment-single-ply', 'equipment-multi-ply'
            ];
            
            equipmentCheckboxes.forEach(id => {
                const checkbox = document.getElementById(id);
                if (checkbox) {
                    checkbox.addEventListener('change', updateEquipment);
                }
            });
        }
        
        // Chart control functions for modern UI
        function changeBins(element, chartId) {
            const bins = parseInt(element.getAttribute('data-bins'));
            
            // Update active state
            element.parentElement.querySelectorAll('.chart-option').forEach(btn => {
                btn.classList.remove('active');
            });
            element.classList.add('active');
            
            // Re-render chart with new bins
            updateCharts();
        }
        
        function switchRankings(element) {
            const type = element.getAttribute('data-type');
            
            // Update active state
            element.parentElement.querySelectorAll('.chart-option').forEach(btn => {
                btn.classList.remove('active');
            });
            element.classList.add('active');
            
            // Update rankings table based on type
            updateRankingsTable(type);
        }
        
        function toggleTrendline(chartId) {
            // Implement trendline toggle
            console.log('Toggle trendline for', chartId);
        }
        
        function togglePoints(chartId) {
            // Implement points toggle
            console.log('Toggle points for', chartId);
        }
        
        function updateRankingsTable(type) {
            // Implementation for updating rankings table
            console.log('Update rankings table with type:', type);
        }

        // Initialize WASM and load initial data
        async function init() {
            await loadArrow();
            await initWasm();
            initWebSocket();
            setupEquipmentFilters();
            updateCharts();
        }
        
        // Start the application when page loads
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', init);
        } else {
            init();
        }
    </script>
    "#.to_string())
}