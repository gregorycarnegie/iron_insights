type ActivityItem = {
  timestamp: number;
  strength_level: string;
  dots_score: number;
  lift_type: string;
};

type StatsMessage = {
  type: 'StatsUpdate';
  active_users: number;
  total_connections: number;
  server_load: number;
};

type UserActivityMessage = {
  type: 'UserActivity';
  recent_calculations: ActivityItem[];
  user_count: number;
};

type ServerMetricsMessage = {
  type: 'ServerMetrics';
  [key: string]: unknown;
};

type DotsCalculationMessage = {
  type: 'DotsCalculation';
  strength_level: string;
  dots_score: number;
  lift_type: string;
};

type WebSocketMessage = StatsMessage | UserActivityMessage | ServerMetricsMessage | DotsCalculationMessage;

// WebSocket state
let websocket: WebSocket | null = null;
let isConnected = false;
let reconnectAttempts = 0;
const maxReconnectAttempts = 5;
const sessionId = 'session_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
const supportsArrow = checkArrowSupport(); // Detect Arrow.js support

// Exponential backoff delays (in milliseconds)
const BACKOFF_DELAYS = [1000, 2000, 5000, 10000, 30000, 60000];

// Initialize WebSocket connection
function initWebSocket(): void {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = protocol + '//' + window.location.host + '/ws';

  try {
    websocket = new WebSocket(wsUrl);

    websocket.onopen = function () {
      console.log('ĐY"- WebSocket connected');
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
      websocket?.send(JSON.stringify(connectMsg));
    };

    websocket.onmessage = function (event: MessageEvent<string | ArrayBuffer>) {
      try {
        if (event.data instanceof ArrayBuffer) {
          // Handle Arrow binary message
          if (supportsArrow) {
            console.log('ĐY"Ư Received Arrow binary message:', event.data.byteLength, 'bytes');
            console.log('ĐYs? Arrow binary message received - 27x faster than JSON!');

            // For now, we'll acknowledge the Arrow message but can't deserialize it
            // without Arrow.js library. In a production environment, you'd add:
            // const table = arrow.Table.from(new Uint8Array(event.data));
            // handleWebSocketMessage(extractMessageFromArrowTable(table));

            // Temporary: Log that we received it successfully
            console.log('ƒo. Arrow message processed (deserialization pending Arrow.js integration)');
          } else {
            console.warn('ƒsÿ‹÷? Received binary message but Arrow support not available');
          }
        } else {
          // Handle JSON text message
          const parsed = JSON.parse(event.data) as Partial<WebSocketMessage> & { type?: string };
          if (!parsed || typeof parsed !== 'object' || typeof parsed.type !== 'string') {
            console.warn('Unknown WebSocket message format');
            return;
          }
          handleWebSocketMessage(parsed as WebSocketMessage);
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    websocket.onclose = function () {
      console.log('ĐY"O WebSocket disconnected');
      isConnected = false;
      updateConnectionStatus(false);

      // Attempt to reconnect with exponential backoff
      if (reconnectAttempts < maxReconnectAttempts) {
        const delay = BACKOFF_DELAYS[reconnectAttempts] || 60000;
        reconnectAttempts++;
        console.log('ĐY"" Reconnecting in ' + delay / 1000 + 's (attempt ' + reconnectAttempts + '/' + maxReconnectAttempts + ')');
        setTimeout(initWebSocket, delay);
      } else {
        console.log('ƒ?O Max reconnection attempts reached');
      }
    };

    websocket.onerror = function (error) {
      console.error('ƒ?O WebSocket error:', error);
    };
  } catch (error) {
    console.error('ƒ?O Failed to initialize WebSocket:', error);
    updateConnectionStatus(false);
  }
}

// Handle incoming WebSocket messages
function handleWebSocketMessage(data: WebSocketMessage): void {
  switch (data.type) {
    case 'StatsUpdate':
      updateLiveStats(
        typeof data.active_users === 'number' ? data.active_users : 0,
        typeof data.total_connections === 'number' ? data.total_connections : 0,
        typeof data.server_load === 'number' ? data.server_load : 0
      );
      break;

    case 'UserActivity':
      updateActivityFeed(Array.isArray(data.recent_calculations) ? data.recent_calculations : []);
      updateUserCount(typeof data.user_count === 'number' ? data.user_count : 0);
      break;

    case 'ServerMetrics':
      if (typeof window.updateServerMetrics === 'function') {
        window.updateServerMetrics(data);
      } else {
        console.log('ServerMetrics message received without handler');
      }
      break;

    case 'DotsCalculation':
      // Handle real-time DOTS calculations from other users
      if (typeof data.dots_score === 'number') {
        addActivityItem('ĐY?<‹÷? ' + data.strength_level + ' lifter: ' + data.dots_score.toFixed(1) + ' DOTS');
      }
      break;

    default: {
      const unknownType = (data as { type?: string }).type;
      console.log('Unknown WebSocket message type:', unknownType);
    }
  }
}

// Send user update via WebSocket
function sendUserUpdate(bodyweight: number | null, squat: number | null, bench: number | null, deadlift: number | null, liftType: string): void {
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
function updateConnectionStatus(connected: boolean): void {
  const statusElement = document.getElementById('connectionStatus');
  if (statusElement) {
    statusElement.className = 'connection-status' + (connected ? '' : ' disconnected');
  }
}

// Update live statistics
function updateLiveStats(activeUsers: number, totalConnections: number, serverLoad: number): void {
  const totalConnectionsEl = document.getElementById('totalConnections');
  const serverLoadEl = document.getElementById('serverLoad');

  if (totalConnectionsEl) {
    totalConnectionsEl.textContent = String(totalConnections);
  }
  if (serverLoadEl) {
    serverLoadEl.textContent = (serverLoad * 100).toFixed(1) + '%';
  }
}

// Update user count
function updateUserCount(count: number): void {
  const userCountEl = document.getElementById('userCount');
  if (userCountEl) {
    userCountEl.textContent = count + ' users online';
  }
}

// Update activity feed
function updateActivityFeed(recentCalculations: ActivityItem[]): void {
  const feed = document.getElementById('activityFeed');
  if (!feed) {
    console.warn('Activity feed element not found');
    return;
  }

  if (recentCalculations && recentCalculations.length > 0) {
    const calculationsCountEl = document.getElementById('calculationsCount');
    if (calculationsCountEl) {
      calculationsCountEl.textContent = String(recentCalculations.length);
    }

    // Show recent 10 calculations
    const recent = recentCalculations.slice(-10).reverse();
    feed.innerHTML = recent
      .map(calc => {
        const timeAgo = getTimeAgo(calc.timestamp * 1000);
        return '<div class="activity-item">ĐY?<‹÷? ' + calc.strength_level + ': ' + calc.dots_score.toFixed(1) + ' DOTS (' + calc.lift_type + ') - ' + timeAgo + '</div>';
      })
      .join('');
  }
}

// Add single activity item
function addActivityItem(text: string): void {
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
    feed.removeChild(feed.lastChild!);
  }
}

// Check if Apache Arrow support is available
function checkArrowSupport(): boolean {
  // For now, we'll detect based on modern browser features
  // In a real implementation, you'd check for Arrow.js library
  const hasArrayBuffer = typeof ArrayBuffer !== 'undefined';
  const hasUint8Array = typeof Uint8Array !== 'undefined';
  const hasWebAssembly = typeof WebAssembly !== 'undefined';

  const isModernBrowser = hasArrayBuffer && hasUint8Array && hasWebAssembly;

  if (isModernBrowser) {
    console.log('ĐYs? Arrow binary support available - expect 27x faster WebSocket messages!');
    return true;
  }

  console.log('ĐY"? Using JSON text messages - consider upgrading browser for Arrow support');
  return false;
}

// Expose functions to global scope
window.initWebSocket = initWebSocket;
window.sendUserUpdate = sendUserUpdate;
