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
                --primary-rgb: 59, 130, 246;
                --secondary: #10b981;
                --secondary-light: #34d399;
                --secondary-dark: #059669;
                --danger: #ef4444;
                --warning: #f59e0b;

                /* Surfaces */
                --light: #0b1220;            /* page background */
                --light-secondary: #0f172a;  /* cards/subtle fills */
                --bg: #0b1220;
                --surface: #0f172a;
                --surface-secondary: #0b1220;
                --surface-hover: #111827;
                --border: #334155;           /* slate-700 */

                /* Text */
                --text-primary: #e5e7eb;     /* high contrast on dark */
                --text-secondary: #cbd5e1;
                --text-tertiary: #94a3b8;
            }

            body {
                background:
                    radial-gradient(1000px 600px at -10% -10%, rgba(var(--primary-rgb), 0.10), transparent 60%),
                    radial-gradient(800px 500px at 110% -20%, rgba(16,185,129,0.08), transparent 60%),
                    var(--light);
                color: var(--text-primary);
                color-scheme: dark;
            }

            /* Dark surfaces: make cards and controls readable */
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

            /* Glass header on dark */
            .header {
                background: rgba(2, 6, 23, 0.55);
                -webkit-backdrop-filter: saturate(180%) blur(10px);
                backdrop-filter: saturate(180%) blur(10px);
                border-bottom: 1px solid rgba(148, 163, 184, 0.12);
            }
            .header-nav a {
                color: var(--text-secondary);
            }
            .header-nav a.active, .header-nav a:hover {
                color: #fff;
            }
        }
    "#.to_string())
}
