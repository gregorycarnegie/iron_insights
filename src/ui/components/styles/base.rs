use maud::{Markup, PreEscaped};

pub fn render_base_styles() -> Markup {
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
    "#.to_string())
}