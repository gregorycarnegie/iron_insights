use maud::{Markup, PreEscaped};

pub fn render_theme_styles() -> Markup {
    PreEscaped(r#"
        /* Dark mode support - Default for Cyberpunk Theme */
        :root {
            /* Core Palette - Deep Space & Neon */
            --primary: #3b82f6;
            --primary-dark: #2563eb;
            --primary-light: #60a5fa;
            --primary-glow: rgba(59, 130, 246, 0.5);
            --primary-rgb: 59, 130, 246;

            --secondary: #10b981;
            --secondary-light: #34d399;
            --secondary-dark: #059669;
            --secondary-glow: rgba(16, 185, 129, 0.5);

            --accent-pink: #ec4899;
            --accent-cyan: #06b6d4;
            --accent-purple: #8b5cf6;

            --danger: #ef4444;
            --warning: #f59e0b;

            /* Surfaces - Glass & Void */
            --bg: #020617;               /* slate-950 */
            --surface: rgba(15, 23, 42, 0.7); /* slate-900 with opacity */
            --surface-hover: rgba(30, 41, 59, 0.8);
            --surface-active: rgba(51, 65, 85, 0.9);
            
            --glass-bg: rgba(15, 23, 42, 0.6);
            --glass-border: rgba(148, 163, 184, 0.1);
            --glass-shine: rgba(255, 255, 255, 0.05);

            --border: rgba(148, 163, 184, 0.1);
            
            /* Text */
            --text-primary: #f8fafc;     /* slate-50 */
            --text-secondary: #94a3b8;   /* slate-400 */
            --text-tertiary: #64748b;    /* slate-500 */
            --text-glow: 0 0 10px rgba(255, 255, 255, 0.1);
        }

        @media (prefers-color-scheme: light) {
             /* Keep dark theme as default for this aesthetic, but provide high-contrast light overrides if strictly needed. 
                For now, we enforce the dark cyberpunk look as the primary brand identity. */
        }

        body {
            background-color: var(--bg);
            color: var(--text-primary);
            color-scheme: dark;
        }

        /* Glassmorphism Utilities */
        .glass-panel {
            background: var(--glass-bg);
            backdrop-filter: blur(12px);
            -webkit-backdrop-filter: blur(12px);
            border: 1px solid var(--glass-border);
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
        }

        /* Component Overrides */
        .container { background: transparent; }

        .stat-card,
        .chart-container,
        .user-metrics-card,
        .lift-card,
        .metric-display,
        .toggle-group,
        .chart-option,
        .control-group input[type="number"],
        .control-group input[type="text"],
        .control-group select,
        table.data-table,
        .sidebar {
            background: var(--surface) !important;
            backdrop-filter: blur(12px);
            -webkit-backdrop-filter: blur(12px);
            border: 1px solid var(--glass-border) !important;
            color: var(--text-primary) !important;
            box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.36);
        }

        .chart-header,
        .stat-label,
        .metric-label,
        .chart-title {
            color: var(--text-secondary) !important;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            font-size: 0.75rem;
            font-weight: 600;
        }

        .chart-option { 
            color: var(--text-secondary) !important; 
            background: transparent !important;
            border: 1px solid transparent !important;
        }
        
        .chart-option:hover {
            color: var(--text-primary) !important;
            background: var(--surface-hover) !important;
        }

        .chart-option.active { 
            color: #ffffff !important; 
            background: rgba(59, 130, 246, 0.2) !important;
            border: 1px solid var(--primary) !important;
            box-shadow: 0 0 15px var(--primary-glow);
        }

        /* Tables */
        .data-table th {
            color: var(--text-secondary);
            background: rgba(0,0,0,0.2);
            text-transform: uppercase;
            font-size: 0.75rem;
            letter-spacing: 0.05em;
        }
        .data-table td { 
            color: var(--text-primary); 
            border-bottom: 1px solid var(--glass-border);
        }
        .data-table tr:hover td {
            background: var(--surface-hover);
        }

        /* Inputs */
        .control-group input,
        .control-group select {
            transition: all 0.3s ease;
        }

        .control-group input:focus,
        .control-group select:focus {
            box-shadow: 0 0 0 2px var(--primary-dark), 0 0 15px var(--primary-glow);
            border-color: var(--primary);
            background: var(--surface-active) !important;
        }

        /* Header */
        .header {
            background: rgba(2, 6, 23, 0.8);
            -webkit-backdrop-filter: saturate(180%) blur(16px);
            backdrop-filter: saturate(180%) blur(16px);
            border-bottom: 1px solid var(--glass-border);
        }
        .header-nav a {
            color: var(--text-secondary);
            transition: color 0.3s ease, text-shadow 0.3s ease;
        }
        .header-nav a.active, .header-nav a:hover {
            color: #fff;
            text-shadow: 0 0 8px rgba(255,255,255,0.5);
        }
    "#.to_string())
}
