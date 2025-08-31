use maud::{Markup, PreEscaped};

pub fn render_base_styles() -> Markup {
    PreEscaped(r#"
        :root {
            /* Brand palette */
            --primary: #2563eb;
            --primary-dark: #1e40af;
            --primary-light: #60a5fa;
            --primary-rgb: 37, 99, 235;

            --secondary: #10b981;
            --secondary-dark: #059669;
            --secondary-light: #34d399;

            --danger: #ef4444;
            --warning: #f59e0b;

            /* Neutral scale (light mode defaults) */
            --dark: #111827;
            --dark-secondary: #1f2937;
            --dark-tertiary: #374151;
            --light: #f3f4f6;                 /* page background */
            --light-secondary: #f9fafb;       /* surfaces */
            --border: #e5e7eb;

            /* Semantic tokens used across pages */
            --bg: var(--light);
            --surface: #ffffff;
            --surface-secondary: #f9fafb;
            --surface-hover: #f3f4f6;

            --text-primary: #111827;
            --text-secondary: #4b5563;
            --text-tertiary: #6b7280;

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
            background: radial-gradient(1200px 800px at 10% -10%, rgba(var(--primary-rgb),0.07), transparent 60%),
                        radial-gradient(800px 600px at 90% -20%, rgba(16,185,129,0.06), transparent 60%),
                        var(--bg);
            color: var(--text-primary);
            line-height: 1.6;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }
    "#.to_string())
}
