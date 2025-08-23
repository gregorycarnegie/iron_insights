// src/ui/components/styles.rs - Modern redesign inspired by OpenIPF and WeightxReps
use maud::{Markup, PreEscaped};

pub fn render_styles() -> Markup {
    PreEscaped(r#"
        :root {
            /* Color palette inspired by OpenIPF/WeightxReps */
            --primary: #2563eb;
            --primary-dark: #1e40af;
            --primary-light: #60a5fa;
            --secondary: #10b981;
            --danger: #ef4444;
            --warning: #f59e0b;
            --dark: #111827;
            --dark-secondary: #1f2937;
            --dark-tertiary: #374151;
            --light: #f9fafb;
            --light-secondary: #f3f4f6;
            --border: #e5e7eb;
            --text-primary: #111827;
            --text-secondary: #6b7280;
            --text-tertiary: #9ca3af;
            
            /* Lift-specific colors */
            --squat-color: #dc2626;
            --bench-color: #2563eb;
            --deadlift-color: #7c3aed;
            --total-color: #059669;
            
            /* Shadows */
            --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
            --shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);
            --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
            --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);
        }
        
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', 'Roboto', sans-serif;
            background: var(--light);
            color: var(--text-primary);
            line-height: 1.6;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }
        
        /* Container */
        .container {
            max-width: 1440px;
            margin: 0 auto;
            padding: 0;
            background: white;
            min-height: 100vh;
        }
        
        /* Modern Header */
        .header {
            background: white;
            border-bottom: 1px solid var(--border);
            padding: 0;
            margin-bottom: 0;
            position: sticky;
            top: 0;
            z-index: 100;
            box-shadow: var(--shadow-sm);
        }
        
        .header-content {
            max-width: 1440px;
            margin: 0 auto;
            padding: 1rem 2rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }
        
        .header h1 {
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--dark);
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }
        
        .header h1 .logo {
            color: var(--primary);
        }
        
        .header-nav {
            display: flex;
            gap: 2rem;
            align-items: center;
        }
        
        .header-nav a {
            color: var(--text-secondary);
            text-decoration: none;
            font-weight: 500;
            transition: color 0.2s;
        }
        
        .header-nav a:hover {
            color: var(--primary);
        }
        
        /* Main Content Area */
        .main-content {
            display: grid;
            grid-template-columns: 280px 1fr;
            min-height: calc(100vh - 65px);
        }
        
        /* Sidebar Controls */
        .sidebar {
            background: var(--light-secondary);
            border-right: 1px solid var(--border);
            padding: 1.5rem;
            overflow-y: auto;
        }
        
        .sidebar h3 {
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-tertiary);
            margin-bottom: 1rem;
        }
        
        .control-section {
            margin-bottom: 2rem;
        }
        
        .control-group {
            margin-bottom: 1rem;
        }
        
        .control-group label {
            display: block;
            font-size: 0.875rem;
            font-weight: 500;
            color: var(--text-primary);
            margin-bottom: 0.5rem;
        }
        
        .control-group input[type="number"],
        .control-group input[type="text"],
        .control-group select {
            width: 100%;
            padding: 0.5rem 0.75rem;
            border: 1px solid var(--border);
            border-radius: 0.375rem;
            font-size: 0.875rem;
            background: white;
            transition: all 0.2s;
        }
        
        .control-group input:focus,
        .control-group select:focus {
            outline: none;
            border-color: var(--primary);
            box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
        }
        
        /* Modern Toggle Buttons */
        .toggle-group {
            display: flex;
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.5rem;
            overflow: hidden;
        }
        
        .toggle-button {
            flex: 1;
            padding: 0.5rem;
            border: none;
            background: white;
            color: var(--text-secondary);
            font-size: 0.875rem;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s;
            border-right: 1px solid var(--border);
        }
        
        .toggle-button:last-child {
            border-right: none;
        }
        
        .toggle-button.active {
            background: var(--primary);
            color: white;
        }
        
        .toggle-button:hover:not(.active) {
            background: var(--light-secondary);
        }
        
        /* Checkbox Style */
        .checkbox-group {
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }
        
        .checkbox-label {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            font-size: 0.875rem;
            color: var(--text-primary);
            cursor: pointer;
            padding: 0.25rem 0;
        }
        
        .checkbox-label input[type="checkbox"] {
            width: 1.125rem;
            height: 1.125rem;
            border: 1px solid var(--border);
            border-radius: 0.25rem;
            cursor: pointer;
        }
        
        .checkbox-label input[type="checkbox"]:checked {
            accent-color: var(--primary);
        }
        
        /* Primary Button */
        .btn-primary {
            width: 100%;
            padding: 0.75rem 1.5rem;
            background: var(--primary);
            color: white;
            border: none;
            border-radius: 0.5rem;
            font-size: 0.875rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
            margin-top: 1.5rem;
        }
        
        .btn-primary:hover {
            background: var(--primary-dark);
            transform: translateY(-1px);
            box-shadow: var(--shadow-md);
        }
        
        .btn-primary:active {
            transform: translateY(0);
        }
        
        /* Content Area */
        .content {
            padding: 2rem;
            overflow-y: auto;
        }
        
        /* Stats Cards */
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }
        
        .stat-card {
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.5rem;
            padding: 1.25rem;
            transition: all 0.2s;
        }
        
        .stat-card:hover {
            box-shadow: var(--shadow-md);
            transform: translateY(-2px);
        }
        
        .stat-label {
            font-size: 0.75rem;
            font-weight: 500;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-tertiary);
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
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.5rem;
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
            padding: 0.25rem 0.5rem;
            font-size: 0.75rem;
            border: 1px solid var(--border);
            border-radius: 0.25rem;
            background: white;
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
            background: white;
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
            background: var(--light-secondary);
            border-radius: 0.375rem;
        }
        
        .metric-value {
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--text-primary);
            margin-bottom: 0.25rem;
        }
        
        .metric-label {
            font-size: 0.75rem;
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
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.5rem;
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
        
        /* Tables */
        .data-table {
            width: 100%;
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.5rem;
            overflow: hidden;
        }
        
        .data-table thead {
            background: var(--light-secondary);
            border-bottom: 1px solid var(--border);
        }
        
        .data-table th {
            padding: 0.75rem 1rem;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-tertiary);
            text-align: left;
        }
        
        .data-table td {
            padding: 0.75rem 1rem;
            font-size: 0.875rem;
            color: var(--text-primary);
            border-bottom: 1px solid var(--border);
        }
        
        .data-table tr:last-child td {
            border-bottom: none;
        }
        
        .data-table tr:hover {
            background: var(--light);
        }
        
        /* Pills/Tags */
        .tag {
            display: inline-block;
            padding: 0.25rem 0.75rem;
            font-size: 0.75rem;
            font-weight: 500;
            border-radius: 9999px;
            background: var(--light-secondary);
            color: var(--text-secondary);
        }
        
        .tag.primary { background: rgba(37, 99, 235, 0.1); color: var(--primary); }
        .tag.success { background: rgba(16, 185, 129, 0.1); color: var(--secondary); }
        .tag.danger { background: rgba(239, 68, 68, 0.1); color: var(--danger); }
        .tag.warning { background: rgba(245, 158, 11, 0.1); color: var(--warning); }
        
        /* Loading States */
        .skeleton {
            background: linear-gradient(90deg, var(--light-secondary) 25%, var(--border) 50%, var(--light-secondary) 75%);
            background-size: 200% 100%;
            animation: loading 1.5s infinite;
            border-radius: 0.375rem;
        }
        
        @keyframes loading {
            0% { background-position: 200% 0; }
            100% { background-position: -200% 0; }
        }
        
        /* Tooltips */
        .tooltip {
            position: relative;
            cursor: help;
        }
        
        .tooltip::after {
            content: attr(data-tooltip);
            position: absolute;
            bottom: 100%;
            left: 50%;
            transform: translateX(-50%);
            background: var(--dark);
            color: white;
            padding: 0.5rem 0.75rem;
            border-radius: 0.375rem;
            font-size: 0.75rem;
            white-space: nowrap;
            opacity: 0;
            pointer-events: none;
            transition: opacity 0.2s;
            margin-bottom: 0.5rem;
        }
        
        .tooltip:hover::after {
            opacity: 1;
        }
        
        /* Responsive Design */
        @media (max-width: 1024px) {
            .main-content {
                grid-template-columns: 1fr;
            }
            
            .sidebar {
                display: none;
                position: fixed;
                top: 65px;
                left: 0;
                bottom: 0;
                width: 280px;
                z-index: 99;
                box-shadow: var(--shadow-lg);
            }
            
            .sidebar.mobile-open {
                display: block;
            }
            
            .chart-grid {
                grid-template-columns: 1fr;
            }
            
            .lift-breakdown {
                grid-template-columns: repeat(2, 1fr);
            }
        }
        
        @media (max-width: 640px) {
            .header-content {
                padding: 1rem;
            }
            
            .content {
                padding: 1rem;
            }
            
            .stats-grid {
                grid-template-columns: 1fr;
            }
            
            .percentile-grid {
                grid-template-columns: 1fr;
            }
            
            .lift-breakdown {
                grid-template-columns: 1fr;
            }
        }
        
        /* Mobile Menu Toggle */
        .mobile-menu-toggle {
            display: none;
            padding: 0.5rem;
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.375rem;
            cursor: pointer;
        }
        
        @media (max-width: 1024px) {
            .mobile-menu-toggle {
                display: block;
            }
        }
        
        /* Overlay for mobile sidebar */
        .sidebar-overlay {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.5);
            z-index: 98;
        }
        
        .sidebar-overlay.active {
            display: block;
        }
        
        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
            :root {
                --primary: #3b82f6;
                --primary-dark: #2563eb;
                --primary-light: #60a5fa;
                --secondary: #10b981;
                --danger: #ef4444;
                --warning: #f59e0b;
                --dark: #f9fafb;
                --dark-secondary: #f3f4f6;
                --dark-tertiary: #e5e7eb;
                --light: #111827;
                --light-secondary: #1f2937;
                --border: #374151;
                --text-primary: #f9fafb;
                --text-secondary: #d1d5db;
                --text-tertiary: #9ca3af;
            }
            
            body {
                background: var(--light);
            }
            
            .container {
                background: #0f172a;
            }
        }
    "#.to_string())
}