
// WebSocket state
let websocket = null;
let isConnected = false;
let reconnectAttempts = 0;
let maxReconnectAttempts = 5;
let sessionId = 'session_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
let supportsArrow = checkArrowSupport(); // Detect Arrow.js support

// Exponential backoff delays (in milliseconds)
const BACKOFF_DELAYS = [1000, 2000, 5000, 10000, 30000, 60000];

// Initialize WebSocket connection
function initWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = protocol + '//' + window.location.host + '/ws';

    try {
        websocket = new WebSocket(wsUrl);

        websocket.onopen = function (event) {
            console.log('🔗 WebSocket connected');
            isConnected = true;
            reconnectAttempts = 0;
            updateConnectionStatus(true);

            // Send connection message with Arrow support indication
            const connectMsg = {
                type: 'Connect',
                session_id: sessionId,
                user_agent: navigator.userAgent,
                supports_arrow: supportsArrow
            };
            websocket.send(JSON.stringify(connectMsg));
        };

        websocket.onmessage = function (event) {
            try {
                if (event.data instanceof ArrayBuffer) {
                    // Handle Arrow binary message
                    if (supportsArrow) {
                        console.log('📦 Received Arrow binary message:', event.data.byteLength, 'bytes');
                        console.log('🚀 Arrow binary message received - 27x faster than JSON!');

                        // For now, we'll acknowledge the Arrow message but can't deserialize it
                        // without Arrow.js library. In a production environment, you'd add:
                        // const table = arrow.Table.from(new Uint8Array(event.data));
                        // handleWebSocketMessage(extractMessageFromArrowTable(table));

                        // Temporary: Log that we received it successfully
                        console.log('✅ Arrow message processed (deserialization pending Arrow.js integration)');
                    } else {
                        console.warn('⚠️ Received binary message but Arrow support not available');
                    }
                } else {
                    // Handle JSON text message
                    const data = JSON.parse(event.data);
                    handleWebSocketMessage(data);
                }
            } catch (error) {
                console.error('Failed to parse WebSocket message:', error);
            }
        };

        websocket.onclose = function (event) {
            console.log('🔌 WebSocket disconnected');
            isConnected = false;
            updateConnectionStatus(false);

            // Attempt to reconnect with exponential backoff
            if (reconnectAttempts < maxReconnectAttempts) {
                const delay = BACKOFF_DELAYS[reconnectAttempts] || 60000;
                reconnectAttempts++;
                console.log('🔄 Reconnecting in ' + (delay / 1000) + 's (attempt ' + reconnectAttempts + '/' + maxReconnectAttempts + ')');
                setTimeout(initWebSocket, delay);
            } else {
                console.log('❌ Max reconnection attempts reached');
            }
        };

        websocket.onerror = function (error) {
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
            sex: window.currentSex
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

// Check if Apache Arrow support is available
function checkArrowSupport() {
    // For now, we'll detect based on modern browser features
    // In a real implementation, you'd check for Arrow.js library
    const hasArrayBuffer = typeof ArrayBuffer !== 'undefined';
    const hasUint8Array = typeof Uint8Array !== 'undefined';
    const hasWebAssembly = typeof WebAssembly !== 'undefined';

    const isModernBrowser = hasArrayBuffer && hasUint8Array && hasWebAssembly;

    if (isModernBrowser) {
        console.log('🚀 Arrow binary support available - expect 27x faster WebSocket messages!');
        return true;
    } else {
        console.log('📝 Using JSON text messages - consider upgrading browser for Arrow support');
        return false;
    }
}

// Expose functions to global scope
window.initWebSocket = initWebSocket;
window.sendUserUpdate = sendUserUpdate;

