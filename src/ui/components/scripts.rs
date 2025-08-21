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

        async function updateCharts() {
            const liftType = document.getElementById('liftType').value;
            const bodyweight = parseFloat(document.getElementById('bodyweight').value);
            const userLift = parseFloat(document.getElementById('userLift').value);
            
            // Get equipment selections
            const selectedEquipment = [];
            if (document.getElementById('equipment-all').checked) {
                selectedEquipment.push("All");
            } else {
                if (document.getElementById('equipment-raw').checked) selectedEquipment.push("Raw");
                if (document.getElementById('equipment-wraps').checked) selectedEquipment.push("Wraps");
                if (document.getElementById('equipment-single-ply').checked) selectedEquipment.push("Single-ply");
                if (document.getElementById('equipment-multi-ply').checked) selectedEquipment.push("Multi-ply");
                if (document.getElementById('equipment-unlimited').checked) selectedEquipment.push("Unlimited");
            }
            
            // Default to Raw if nothing selected
            if (selectedEquipment.length === 0) {
                selectedEquipment.push("Raw");
            }
            
            const params = {
                sex: document.getElementById('sex').value,
                lift_type: liftType,
                bodyweight: bodyweight || null,
                squat: liftType === 'squat' ? userLift : null,
                bench: liftType === 'bench' ? userLift : null,
                deadlift: liftType === 'deadlift' ? userLift : null,
                equipment: selectedEquipment,
                years_filter: document.getElementById('years-filter').value
            };
            
            // Handle total lift type
            if (liftType === 'total' && userLift) {
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
                    liftType === 'squat' ? userLift : null,
                    liftType === 'bench' ? userLift : null,
                    liftType === 'deadlift' ? userLift : null,
                    liftType);
                
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
                
                // Show percentile comparison
                let percentilesHtml = '';
                if (data.user_percentile !== null && data.user_dots_percentile !== null) {
                    const userDots = bodyweight && userLift ? calculateDOTS(userLift, bodyweight) : null;
                    
                    percentilesHtml = '<div class="percentile-comparison">' +
                            '<div class="percentile-card">' +
                                '<div class="percentile-value">' + data.user_percentile + '%</div>' +
                                '<div class="percentile-label">Raw Weight Percentile</div>' +
                            '</div>' +
                            '<div class="percentile-card dots">' +
                                '<div class="percentile-value">' + data.user_dots_percentile + '%</div>' +
                                '<div class="percentile-label">DOTS Percentile</div>' +
                            '</div>' +
                        '</div>' +
                        (userDots ? '<p style="text-align: center; margin-top: 10px;"><strong>Your DOTS Score: ' + userDots.toFixed(1) + '</strong></p>' : '');
                }
                document.getElementById('percentiles').innerHTML = percentilesHtml;
                
                // Show stats with DOTS status
                const dotsStatus = dotsHistogramSuccess && dotsScatterSuccess ? '✅ Working' : '❌ No Data';
                document.getElementById('stats').innerHTML = '<div class="stats">' +
                        '<div class="stat-card">' +
                            '<div class="stat-value">' + data.processing_time_ms + 'ms</div>' +
                            '<div class="stat-label">Processing Time</div>' +
                        '</div>' +
                        '<div class="stat-card">' +
                            '<div class="stat-value">' + data.total_records.toLocaleString() + '</div>' +
                            '<div class="stat-label">Records Analyzed</div>' +
                        '</div>' +
                        '<div class="stat-card">' +
                            '<div class="stat-value">' + (liftType.charAt(0).toUpperCase() + liftType.slice(1)) + '</div>' +
                            '<div class="stat-label">Current Lift</div>' +
                        '</div>' +
                        '<div class="stat-card">' +
                            '<div class="stat-value">' + dotsStatus + '</div>' +
                            '<div class="stat-label">DOTS Charts</div>' +
                        '</div>' +
                    '</div>';
                
            } catch (error) {
                console.error('Error:', error);
                document.getElementById('stats').innerHTML = '<p style="color: red;">Error loading data: ' + error.message + '</p>';
                
                // Show errors on all charts
                showError('histogram', 'Failed to load data');
                showError('dotsHistogram', 'Failed to load data');
                showError('scatter', 'Failed to load data');
                showError('dotsScatter', 'Failed to load data');
            }
        }
        
        // Helper function to calculate gender-specific DOTS score using WASM
        function calculateDOTS(liftKg, bodyweightKg, sex = null) {
            if (calculate_dots_wasm) {
                return calculate_dots_wasm(liftKg, bodyweightKg);
            } else {
                // Fallback JavaScript implementation with gender-specific coefficients
                const currentSex = sex || document.getElementById('sex').value;
                
                let a, b, c, d, e;
                if (currentSex === 'M' || currentSex === 'Male') {
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
        
        // Function to update user metrics display
        function updateUserMetrics(bodyweight, userLift) {
            const userMetricsSection = document.getElementById('userMetrics');
            const strengthBadge = document.getElementById('strengthBadge');
            const userDotsValue = document.getElementById('userDotsValue');
            const userStrengthLevel = document.getElementById('userStrengthLevel');
            
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
                    
                    userDotsValue.textContent = dots.toFixed(1);
                    userStrengthLevel.textContent = level;
                    
                    strengthBadge.innerHTML = '<div class="strength-badge" style="background-color: ' + color + ';">🏋️ ' + level + '</div>';
                    
                    userMetricsSection.style.display = 'block';
                    
                    // Show share card section when user has entered data
                    const shareCardSection = document.getElementById('shareCardSection');
                    shareCardSection.style.display = 'block';
                    
                    // Auto-populate share card inputs
                    populateShareCardInputs();
                    
                } catch (error) {
                    console.error('Error calculating user metrics:', error);
                    userMetricsSection.style.display = 'none';
                    document.getElementById('shareCardSection').style.display = 'none';
                }
            } else {
                userMetricsSection.style.display = 'none';
                document.getElementById('shareCardSection').style.display = 'none';
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
                    sex: document.getElementById('sex').value
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
        
        // Share Card Generation
        let currentShareCardSvg = null;
        
        // Auto-populate share card inputs when user metrics are updated
        function populateShareCardInputs() {
            const bodyweight = parseFloat(document.getElementById('bodyweight').value);
            const userLift = parseFloat(document.getElementById('userLift').value);
            const liftType = document.getElementById('liftType').value;
            
            if (!bodyweight || !userLift) return;
            
            // Auto-populate the current lift based on what user entered in main form
            if (liftType === 'squat') {
                document.getElementById('shareSquat').value = userLift;
            } else if (liftType === 'bench') {
                document.getElementById('shareBench').value = userLift;
            } else if (liftType === 'deadlift') {
                document.getElementById('shareDeadlift').value = userLift;
            }
        }

        async function generateShareCard() {
            const name = document.getElementById('shareCardName').value.trim();
            const theme = document.getElementById('shareCardTheme').value;
            const bodyweight = parseFloat(document.getElementById('bodyweight').value);
            const sex = document.getElementById('sex').value;
            
            // Get individual lift values from share card inputs
            const squat = parseFloat(document.getElementById('shareSquat').value) || null;
            const bench = parseFloat(document.getElementById('shareBench').value) || null;
            const deadlift = parseFloat(document.getElementById('shareDeadlift').value) || null;
            
            if (!name) {
                alert('Please enter your name');
                return;
            }
            
            if (!bodyweight) {
                alert('Please enter your bodyweight');
                return;
            }
            
            if (!squat && !bench && !deadlift) {
                alert('Please enter at least one lift value');
                return;
            }
            
            try {
                // Calculate total
                const total = (squat || 0) + (bench || 0) + (deadlift || 0);
                
                // Calculate DOTS score using the highest single lift or total if all lifts provided
                let bestLift = total;
                if (squat && bench && deadlift) {
                    bestLift = total; // Use total when all lifts are provided
                } else {
                    bestLift = Math.max(squat || 0, bench || 0, deadlift || 0);
                }
                
                const dotsScore = calculateDOTS(bestLift, bodyweight);
                let strengthLevel;
                
                if (calculate_strength_level_wasm) {
                    strengthLevel = calculate_strength_level_wasm(dotsScore);
                } else {
                    strengthLevel = getStrengthLevel(dotsScore);
                }
                
                // Get percentile from last response if available
                const percentile = lastResponse ? lastResponse.user_dots_percentile : null;
                
                // Prepare share card data
                const shareCardData = {
                    name: name,
                    bodyweight: bodyweight,
                    squat: squat,
                    bench: bench,
                    deadlift: deadlift,
                    total: total > 0 ? total : null,
                    dots_score: dotsScore,
                    strength_level: strengthLevel,
                    percentile: percentile,
                    lift_type: 'total', // Always show as total for comprehensive view
                    sex: sex === 'All' ? 'M' : sex,
                    theme: theme
                };
                
                // Send request to generate share card
                const response = await fetch('/api/share-card', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(shareCardData)
                });
                
                if (!response.ok) {
                    throw new Error('Failed to generate share card');
                }
                
                const svgContent = await response.text();
                currentShareCardSvg = svgContent;
                
                // Display preview
                const container = document.getElementById('shareCardContainer');
                container.innerHTML = svgContent;
                document.getElementById('shareCardPreview').style.display = 'block';
                
                // Enable download button
                document.getElementById('downloadButton').disabled = false;
                
                console.log('✅ Share card generated successfully');
                
            } catch (error) {
                console.error('❌ Error generating share card:', error);
                alert('Failed to generate share card: ' + error.message);
            }
        }
        
        function downloadShareCard() {
            if (!currentShareCardSvg) {
                alert('Please generate a share card first');
                return;
            }
            
            try {
                // Create blob and download
                const blob = new Blob([currentShareCardSvg], { type: 'image/svg+xml' });
                const url = URL.createObjectURL(blob);
                
                const name = document.getElementById('shareCardName').value.trim() || 'powerlifter';
                const liftType = document.getElementById('liftType').value;
                const theme = document.getElementById('shareCardTheme').value;
                
                const filename = name.toLowerCase().replace(/\s+/g, '-') + '-' + liftType + '-' + theme + '-card.svg';
                
                const a = document.createElement('a');
                a.href = url;
                a.download = filename;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                
                URL.revokeObjectURL(url);
                
                console.log('✅ Share card downloaded: ' + filename);
                
            } catch (error) {
                console.error('❌ Error downloading share card:', error);
                alert('Failed to download share card');
            }
        }
        
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

        // Handle equipment checkbox logic
        function setupEquipmentFilters() {
            const allCheckbox = document.getElementById('equipment-all');
            const otherCheckboxes = [
                'equipment-raw', 'equipment-wraps', 'equipment-single-ply', 
                'equipment-multi-ply', 'equipment-unlimited'
            ].map(id => document.getElementById(id));
            
            // When "All" is checked, uncheck others
            allCheckbox.addEventListener('change', function() {
                if (this.checked) {
                    otherCheckboxes.forEach(cb => cb.checked = false);
                }
            });
            
            // When any specific equipment is checked, uncheck "All"
            otherCheckboxes.forEach(cb => {
                cb.addEventListener('change', function() {
                    if (this.checked) {
                        allCheckbox.checked = false;
                    }
                });
            });
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