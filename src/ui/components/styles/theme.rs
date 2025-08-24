use maud::{Markup, PreEscaped};

pub fn render_theme_styles() -> Markup {
    PreEscaped(r#"
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
                --text-primary: #374151;
                --text-secondary: #4b5563;
                --text-tertiary: #6b7280;
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