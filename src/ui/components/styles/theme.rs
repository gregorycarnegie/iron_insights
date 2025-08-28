use maud::{Markup, PreEscaped};

pub fn render_theme_styles() -> Markup {
    PreEscaped(r#"
        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
            /* Token overrides tuned for contrast */
            :root {
                --primary: #3b82f6;
                --primary-dark: #2563eb;
                --primary-light: #60a5fa;
                --secondary: #10b981;
                --danger: #ef4444;
                --warning: #f59e0b;

                /* Surfaces */
                --light: #0b1220;            /* page background */
                --light-secondary: #111827;  /* cards/subtle fills */
                --dark: #e5e7eb;             /* text on dark */
                --dark-secondary: #cbd5e1;   /* secondary text */
                --dark-tertiary: #94a3b8;    /* tertiary text */
                --border: #334155;           /* slate-700 */

                /* Text */
                --text-primary: #e5e7eb;     /* high contrast on dark */
                --text-secondary: #cbd5e1;
                --text-tertiary: #94a3b8;
            }

            body {
                background: var(--light);
                color: var(--text-primary);
                color-scheme: dark;
            }

            /* Dark surfaces: make cards and controls readable */
            .container { background: #0f172a; }

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
                background: var(--light-secondary) !important;
                border-color: var(--border) !important;
                color: var(--text-primary) !important;
            }

            .chart-header,
            .stat-label,
            .metric-label,
            .chart-title {
                color: var(--text-secondary) !important;
            }

            .chart-option { color: var(--text-secondary) !important; }
            .chart-option.active { color: #ffffff !important; }

            /* Tables */
            .data-table th {
                color: var(--text-secondary);
                background: rgba(255,255,255,0.02);
            }
            .data-table td { color: var(--dark); }

            /* Inputs */
            .control-group input:focus,
            .control-group select:focus {
                box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.25);
            }
        }
    "#.to_string())
}
