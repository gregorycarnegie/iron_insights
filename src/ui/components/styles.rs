// src/ui/components/styles.rs - CSS styles as compile-time strings
use maud::{Markup, PreEscaped};

pub fn render_styles() -> Markup {
    PreEscaped(r#"
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
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px;
            border-radius: 8px;
            margin: 20px 0;
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
            background: rgba(255, 255, 255, 0.1);
            border-radius: 5px;
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
    "#.to_string())
}