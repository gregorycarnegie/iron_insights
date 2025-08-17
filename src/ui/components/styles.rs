// src/ui/components/styles.rs - CSS styles as compile-time strings
use maud::{Markup, PreEscaped};

pub fn render_styles() -> Markup {
    PreEscaped(r#"
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; 
            margin: 0; 
            padding: 20px; 
            background: linear-gradient(-45deg, #f5f5f5, #e8f4f8, #f0f8ff, #f5f5f5);
            background-size: 400% 400%;
            animation: gradientShift 15s ease infinite;
            min-height: 100vh;
            transform: translateZ(0);
        }
        .container { 
            max-width: 1400px; 
            margin: 0 auto; 
            background: linear-gradient(135deg, rgba(255,255,255,0.95) 0%, rgba(248,250,252,0.95) 50%, rgba(255,255,255,0.95) 100%); 
            border-radius: 8px; 
            padding: 20px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1); 
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255,255,255,0.2);
            transform: translateZ(0);
            will-change: transform;
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
            background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 50%, #f8f9fa 100%);
            background-size: 200% 200%;
            animation: subtleGradientShift 8s ease infinite;
            padding: 20px;
            border-radius: 5px;
            transform: translateZ(0);
            border: 1px solid rgba(255,255,255,0.3);
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
            background: linear-gradient(135deg, rgba(255,255,255,0.98) 0%, rgba(248,250,252,0.98) 50%, rgba(255,255,255,0.98) 100%);
            background-size: 200% 200%;
            animation: chartGradientShift 12s ease infinite;
            position: relative;
            transform: translateZ(0);
            will-change: transform;
            backface-visibility: hidden;
            box-shadow: 0 2px 8px rgba(0,0,0,0.05);
        }
        .chart-title {
            font-weight: 600;
            padding: 10px;
            background: linear-gradient(90deg, #f8f9fa 0%, #e9ecef 50%, #f8f9fa 100%);
            background-size: 200% 100%;
            animation: titleGradientShift 6s ease infinite;
            border-bottom: 1px solid #ddd;
            margin: 0;
            font-size: 16px;
            transform: translateZ(0);
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
            background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 25%, #f1f3f4 50%, #e9ecef 75%, #f8f9fa 100%);
            background-size: 300% 300%;
            animation: statCardGradient 10s ease infinite;
            padding: 15px;
            border-radius: 5px;
            border-left: 4px solid #28a745;
            text-align: center;
            transform: translateZ(0);
            will-change: transform;
            backface-visibility: hidden;
            box-shadow: 0 2px 6px rgba(0,0,0,0.05);
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
        .strength-badge {
            display: inline-block;
            padding: 8px 16px;
            border-radius: 20px;
            font-weight: bold;
            font-size: 14px;
            color: white;
            margin: 10px 0;
            text-align: center;
        }
        .user-input-section {
            background: #f8f9fa;
            border-left: 4px solid #007bff;
            padding: 15px;
            margin: 20px 0;
            border-radius: 5px;
        }
        .user-metrics {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 15px;
            margin-top: 15px;
        }
        .metric-display {
            text-align: center;
            padding: 10px;
            background: white;
            border-radius: 5px;
            border: 1px solid #ddd;
        }
        .metric-value {
            font-size: 20px;
            font-weight: bold;
            color: #333;
        }
        .metric-label {
            font-size: 12px;
            color: #666;
            margin-top: 5px;
        }
        .realtime-panel {
            background: linear-gradient(-45deg, #667eea 0%, #764ba2 25%, #5a67d8 50%, #667eea 75%, #764ba2 100%);
            background-size: 400% 400%;
            animation: realtimeGradientShift 8s ease infinite;
            color: white;
            padding: 20px;
            border-radius: 8px;
            margin: 20px 0;
            transform: translateZ(0);
            will-change: transform;
            backface-visibility: hidden;
            box-shadow: 0 4px 15px rgba(102,126,234,0.3);
        }
        .realtime-title {
            font-size: 18px;
            font-weight: bold;
            margin-bottom: 15px;
            display: flex;
            align-items: center;
            gap: 10px;
        }
        .connection-status {
            display: inline-block;
            width: 10px;
            height: 10px;
            border-radius: 50%;
            background: #28a745;
            margin-right: 8px;
        }
        .connection-status.disconnected {
            background: #dc3545;
        }
        .activity-feed {
            background: rgba(255, 255, 255, 0.1);
            border-radius: 5px;
            padding: 15px;
            max-height: 200px;
            overflow-y: auto;
        }
        .activity-item {
            padding: 8px 0;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
            font-size: 14px;
        }
        .activity-item:last-child {
            border-bottom: none;
        }
        .live-stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
            gap: 15px;
            margin-top: 15px;
        }
        .live-stat {
            text-align: center;
            padding: 10px;
            background: linear-gradient(135deg, rgba(255, 255, 255, 0.1) 0%, rgba(255, 255, 255, 0.2) 50%, rgba(255, 255, 255, 0.1) 100%);
            background-size: 200% 200%;
            animation: liveStatGradient 7s ease infinite;
            border-radius: 5px;
            transform: translateZ(0);
        }
        .live-stat-value {
            font-size: 24px;
            font-weight: bold;
            margin-bottom: 5px;
        }
        .live-stat-label {
            font-size: 12px;
            opacity: 0.9;
        }
        .percentile-comparison {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 15px;
            margin: 20px 0;
        }
        .percentile-card {
            background: linear-gradient(-45deg, #667eea 0%, #764ba2 25%, #5a67d8 50%, #667eea 75%, #764ba2 100%);
            background-size: 300% 300%;
            animation: percentileGradientShift 9s ease infinite;
            color: white;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
            transform: translateZ(0);
            will-change: transform;
            backface-visibility: hidden;
            box-shadow: 0 4px 12px rgba(102,126,234,0.25);
        }
        .percentile-card.dots {
            background: linear-gradient(-45deg, #f093fb 0%, #f5576c 25%, #ff6b9d 50%, #f093fb 75%, #f5576c 100%);
            background-size: 300% 300%;
            animation: dotsPercentileGradientShift 9s ease infinite;
            box-shadow: 0 4px 12px rgba(240,147,251,0.25);
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
        /* GPU-accelerated hover animations */
        .chart:hover {
            transform: translateZ(0) scale(1.02);
            transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            box-shadow: 0 8px 25px rgba(0,0,0,0.15);
        }
        
        .stat-card:hover {
            transform: translateZ(0) translateY(-2px);
            transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }
        
        .percentile-card:hover {
            transform: translateZ(0) scale(1.05);
            transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            box-shadow: 0 8px 25px rgba(0,0,0,0.2);
        }
        
        button:hover {
            background: #0056b3;
            transform: translateZ(0) translateY(-1px);
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            box-shadow: 0 4px 12px rgba(0,123,255,0.3);
        }
        
        /* GPU-accelerated gradient animations */
        @keyframes gradientShift {
            0% { background-position: 0% 50%; }
            50% { background-position: 100% 50%; }
            100% { background-position: 0% 50%; }
        }
        
        @keyframes subtleGradientShift {
            0% { background-position: 0% 0%; }
            50% { background-position: 100% 100%; }
            100% { background-position: 0% 0%; }
        }
        
        @keyframes chartGradientShift {
            0% { background-position: 0% 50%; }
            33% { background-position: 100% 50%; }
            66% { background-position: 50% 100%; }
            100% { background-position: 0% 50%; }
        }
        
        @keyframes titleGradientShift {
            0% { background-position: 0% 50%; }
            50% { background-position: 100% 50%; }
            100% { background-position: 0% 50%; }
        }
        
        @keyframes statCardGradient {
            0% { background-position: 0% 0%; }
            25% { background-position: 100% 0%; }
            50% { background-position: 100% 100%; }
            75% { background-position: 0% 100%; }
            100% { background-position: 0% 0%; }
        }
        
        @keyframes realtimeGradientShift {
            0% { background-position: 0% 50%; }
            25% { background-position: 25% 75%; }
            50% { background-position: 100% 50%; }
            75% { background-position: 75% 25%; }
            100% { background-position: 0% 50%; }
        }
        
        @keyframes liveStatGradient {
            0% { background-position: 0% 50%; }
            50% { background-position: 100% 50%; }
            100% { background-position: 0% 50%; }
        }
        
        @keyframes percentileGradientShift {
            0% { background-position: 0% 50%; }
            33% { background-position: 100% 50%; }
            66% { background-position: 50% 100%; }
            100% { background-position: 0% 50%; }
        }
        
        @keyframes dotsPercentileGradientShift {
            0% { background-position: 0% 50%; }
            33% { background-position: 100% 50%; }
            66% { background-position: 50% 100%; }
            100% { background-position: 0% 50%; }
        }
        
        .connection-status {
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            transform: translateZ(0);
        }
        
        .connection-status:not(.disconnected) {
            animation: statusPulse 2s ease-in-out infinite;
        }
        
        @keyframes statusPulse {
            0%, 100% { transform: translateZ(0) scale(1); opacity: 1; }
            50% { transform: translateZ(0) scale(1.2); opacity: 0.8; }
        }
        
        .activity-item {
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            transform: translateZ(0);
        }
        
        .activity-item:hover {
            transform: translateZ(0) translateX(5px);
            background: rgba(255, 255, 255, 0.1);
            border-radius: 3px;
            padding-left: 5px;
        }
        
        .live-stat {
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            transform: translateZ(0);
        }
        
        .live-stat:hover {
            transform: translateZ(0) scale(1.1);
            background: rgba(255, 255, 255, 0.2);
        }
        
        .metric-display {
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            transform: translateZ(0);
        }
        
        .metric-display:hover {
            transform: translateZ(0) translateY(-2px);
            box-shadow: 0 4px 12px rgba(0,0,0,0.1);
            border-color: #007bff;
        }
        
        @media (max-width: 768px) {
            .chart-grid {
                grid-template-columns: 1fr;
            }
            .percentile-comparison {
                grid-template-columns: 1fr;
            }
            /* Reduce animations on mobile for performance */
            .chart:hover, .stat-card:hover, .percentile-card:hover {
                transform: none;
                transition: none;
            }
        }
    "#.to_string())
}