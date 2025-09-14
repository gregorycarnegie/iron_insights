use maud::{Markup, PreEscaped};

pub fn render_chart_styles() -> Markup {
    PreEscaped(r#"
        /* Stats Cards */
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }
        
        .stat-card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 0.75rem;
            padding: 1.25rem;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        
        .stat-card:hover {
            box-shadow: var(--shadow-md);
            transform: translateY(-2px);
        }
        
        .stat-label {
            font-size: 0.8125rem;
            font-weight: 500;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-secondary);
            margin-bottom: 0.25rem;
        }
        
        .stat-value {
            font-size: 1.875rem;
            font-weight: 700;
            color: var(--text-primary);
            line-height: 1;
        }
        
        .stat-change {
            font-size: 0.875rem;
            margin-top: 0.5rem;
            display: flex;
            align-items: center;
            gap: 0.25rem;
        }
        
        .stat-change.positive {
            color: var(--secondary);
        }
        
        .stat-change.negative {
            color: var(--danger);
        }
        
        /* Chart Grid */
        .chart-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 1.5rem;
            margin-bottom: 2rem;
        }
        
        .chart-container {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 0.75rem;
            overflow: hidden;
        }
        
        .chart-header {
            padding: 1rem 1.25rem;
            border-bottom: 1px solid var(--border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .chart-title {
            font-size: 0.875rem;
            font-weight: 600;
            color: var(--text-primary);
            margin: 0;
        }
        
        .chart-options {
            display: flex;
            gap: 0.5rem;
        }
        
        .chart-option {
            padding: 0.35rem 0.6rem;
            font-size: 0.8125rem;
            border: 1px solid var(--border);
            border-radius: 0.25rem;
            background: var(--surface);
            color: var(--text-secondary);
            cursor: pointer;
            transition: all 0.2s;
        }
        
        .chart-option:hover {
            background: var(--light-secondary);
        }
        
        .chart-option.active {
            background: var(--primary);
            color: white;
            border-color: var(--primary);
        }
        
        .chart {
            height: 400px;
            padding: 1rem;
            position: relative;
        }
        
        .chart-error {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            text-align: center;
            color: var(--text-tertiary);
            font-size: 0.875rem;
        }
        
        /* Percentile Cards */
        .percentile-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1rem;
            margin: 2rem 0;
        }
        
        .percentile-card {
            background: linear-gradient(135deg, var(--primary), var(--primary-dark));
            color: white;
            border-radius: 0.5rem;
            padding: 1.5rem;
            position: relative;
            overflow: hidden;
        }
        
        .percentile-card::before {
            content: '';
            position: absolute;
            top: -50%;
            right: -50%;
            width: 200%;
            height: 200%;
            background: radial-gradient(circle, rgba(255,255,255,0.1) 0%, transparent 70%);
        }
        
        .percentile-card.dots {
            background: linear-gradient(135deg, var(--secondary), #059669);
        }
        
        .percentile-value {
            font-size: 2.5rem;
            font-weight: 700;
            margin-bottom: 0.25rem;
            position: relative;
        }
        
        .percentile-label {
            font-size: 0.875rem;
            opacity: 0.9;
            position: relative;
        }
        
        /* User Metrics */
        .user-metrics-card {
            background: var(--surface);
            border: 2px solid var(--primary);
            border-radius: 0.5rem;
            padding: 1.5rem;
            margin: 2rem 0;
        }
        
        .user-metrics-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1.5rem;
        }
        
        .user-metrics-title {
            font-size: 1.125rem;
            font-weight: 600;
            color: var(--text-primary);
        }
        
        .strength-badge {
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.5rem 1rem;
            border-radius: 2rem;
            font-size: 0.875rem;
            font-weight: 600;
            color: white;
        }
        
        .strength-badge.beginner { background: var(--text-tertiary); }
        .strength-badge.novice { background: var(--secondary); }
        .strength-badge.intermediate { background: var(--primary-light); }
        .strength-badge.advanced { background: var(--warning); }
        .strength-badge.elite { background: #ea580c; }
        .strength-badge.world-class { background: var(--danger); }
        
        .user-metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 1rem;
        }
        
        .metric-display {
            text-align: center;
            padding: 1rem;
            background: var(--surface-secondary);
            border-radius: 0.375rem;
        }
        
        .metric-value {
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--text-primary);
            margin-bottom: 0.25rem;
        }
        
        .metric-label {
            font-size: 0.8125rem;
            color: var(--text-secondary);
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }
        
        /* Lift Breakdown */
        .lift-breakdown {
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 1rem;
            margin: 2rem 0;
        }
        
        .lift-card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 0.75rem;
            padding: 1.25rem;
            text-align: center;
            position: relative;
            overflow: hidden;
        }
        
        .lift-card::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 4px;
        }
        
        .lift-card.squat::before { background: var(--squat-color); }
        .lift-card.bench::before { background: var(--bench-color); }
        .lift-card.deadlift::before { background: var(--deadlift-color); }
        .lift-card.total::before { background: var(--total-color); }
        
        .lift-icon {
            font-size: 1.5rem;
            margin-bottom: 0.5rem;
        }
        
        .lift-name {
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-tertiary);
            margin-bottom: 0.5rem;
        }
        
        .lift-value {
            font-size: 1.875rem;
            font-weight: 700;
            color: var(--text-primary);
        }
        
        .lift-unit {
            font-size: 0.875rem;
            color: var(--text-secondary);
            margin-left: 0.25rem;
        }

        /* Export Controls */
        .export-controls {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 0.75rem;
            padding: 1.5rem;
            margin-bottom: 2rem;
        }

        .export-section h4 {
            margin: 0 0 1rem 0;
            font-size: 1rem;
            font-weight: 600;
            color: var(--text-primary);
        }

        .export-buttons {
            display: flex;
            gap: 0.75rem;
            flex-wrap: wrap;
        }

        .export-all-btn, .export-data-btn, .export-original-btn {
            padding: 0.5rem 1rem;
            font-size: 0.875rem;
            font-weight: 500;
            border: 1px solid var(--primary);
            border-radius: 0.5rem;
            background: var(--primary);
            color: white;
            cursor: pointer;
            transition: all 0.2s;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            text-decoration: none;
        }

        .export-all-btn:hover, .export-data-btn:hover, .export-original-btn:hover {
            background: var(--primary-dark);
            border-color: var(--primary-dark);
            transform: translateY(-1px);
            box-shadow: var(--shadow-sm);
        }

        .export-data-btn {
            background: var(--secondary);
            border-color: var(--secondary);
        }

        .export-data-btn:hover {
            background: #059669;
            border-color: #059669;
        }

        .export-original-btn {
            background: var(--warning);
            border-color: var(--warning);
        }

        .export-original-btn:hover {
            background: #d97706;
            border-color: #d97706;
        }

        /* Export Dropdown */
        .export-dropdown {
            position: relative;
            display: inline-block;
        }

        .export-btn {
            background: var(--warning) !important;
            border-color: var(--warning) !important;
            color: white !important;
        }

        .export-btn:hover {
            background: #d97706 !important;
            border-color: #d97706 !important;
        }

        .export-menu {
            position: absolute;
            top: 100%;
            right: 0;
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 0.375rem;
            box-shadow: var(--shadow-lg);
            z-index: 1000;
            min-width: 120px;
            margin-top: 0.25rem;
        }

        .export-menu button {
            display: block;
            width: 100%;
            padding: 0.5rem 0.75rem;
            font-size: 0.875rem;
            border: none;
            background: transparent;
            color: var(--text-primary);
            cursor: pointer;
            text-align: left;
            transition: background-color 0.2s;
        }

        .export-menu button:hover {
            background: var(--light-primary);
        }

        .export-menu button:first-child {
            border-radius: 0.375rem 0.375rem 0 0;
        }

        .export-menu button:last-child {
            border-radius: 0 0 0.375rem 0.375rem;
        }

        /* Chart interaction indicators */
        .chart-container.interactive {
            border-color: var(--primary);
            box-shadow: 0 0 0 1px var(--primary);
        }

        .chart-selection-info {
            position: absolute;
            top: 1rem;
            right: 1rem;
            background: rgba(0, 0, 0, 0.8);
            color: white;
            padding: 0.5rem 0.75rem;
            border-radius: 0.25rem;
            font-size: 0.8125rem;
            z-index: 10;
            pointer-events: none;
            opacity: 0;
            transition: opacity 0.2s;
        }

        .chart-selection-info.visible {
            opacity: 1;
        }
    "#.to_string())
}
