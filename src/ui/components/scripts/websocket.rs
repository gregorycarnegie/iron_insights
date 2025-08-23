use maud::{Markup, PreEscaped};

pub fn render_websocket_scripts() -> Markup {
    PreEscaped(r#"
        // WebSocket state
        let websocket = null;
        let isConnected = false;
        let reconnectAttempts = 0;
        let maxReconnectAttempts = 5;
        let sessionId = 'session_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
        
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
            const totalConnectionsEl = document.getElementById('totalConnections');
            const serverLoadEl = document.getElementById('serverLoad');
            
            if (totalConnectionsEl) {
                totalConnectionsEl.textContent = totalConnections;
            }
            if (serverLoadEl) {
                serverLoadEl.textContent = (serverLoad * 100).toFixed(1) + '%';
            }
        }
        
        // Update user count
        function updateUserCount(count) {
            const userCountEl = document.getElementById('userCount');
            if (userCountEl) {
                userCountEl.textContent = count + ' users online';
            }
        }
        
        // Update activity feed
        function updateActivityFeed(recentCalculations) {
            const feed = document.getElementById('activityFeed');
            if (!feed) {
                console.warn('Activity feed element not found');
                return;
            }
            
            if (recentCalculations && recentCalculations.length > 0) {
                const calculationsCountEl = document.getElementById('calculationsCount');
                if (calculationsCountEl) {
                    calculationsCountEl.textContent = recentCalculations.length;
                }
                
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
            if (!feed) {
                console.warn('Activity feed element not found');
                return;
            }
            const item = document.createElement('div');
            item.className = 'activity-item';
            item.textContent = text + ' - just now';
            feed.insertBefore(item, feed.firstChild);
            
            // Keep only 10 items
            while (feed.children.length > 10) {
                feed.removeChild(feed.lastChild);
            }
        }
    "#.to_string())
}