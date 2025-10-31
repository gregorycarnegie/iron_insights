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

        /* Focus styles for keyboard navigation */
        *:focus {
            outline: 2px solid var(--primary);
            outline-offset: 2px;
        }

        /* Skip to main content link */
        .skip-link {
            position: absolute;
            top: -40px;
            left: 6px;
            background: var(--primary);
            color: white;
            padding: 8px;
            text-decoration: none;
            border-radius: 4px;
            z-index: 1000;
        }

        .skip-link:focus {
            top: 6px;
        }

        /* Screen reader only text */
        .sr-only {
            position: absolute;
            width: 1px;
            height: 1px;
            padding: 0;
            margin: -1px;
            overflow: hidden;
            clip: rect(0, 0, 0, 0);
            white-space: nowrap;
            border: 0;
        }

        /* Page transitions and animations */
        .page-transition {
            opacity: 0;
            transform: translateY(20px);
            animation: pageEnter 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
        }

        @keyframes pageEnter {
            to {
                opacity: 1;
                transform: translateY(0);
            }
        }

        /* Stagger animation for child elements */
        .stagger-children > * {
            opacity: 0;
            transform: translateY(20px);
            animation: staggerIn 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
        }

        .stagger-children > *:nth-child(1) { animation-delay: 0.1s; }
        .stagger-children > *:nth-child(2) { animation-delay: 0.2s; }
        .stagger-children > *:nth-child(3) { animation-delay: 0.3s; }
        .stagger-children > *:nth-child(4) { animation-delay: 0.4s; }
        .stagger-children > *:nth-child(5) { animation-delay: 0.5s; }
        .stagger-children > *:nth-child(6) { animation-delay: 0.6s; }

        @keyframes staggerIn {
            to {
                opacity: 1;
                transform: translateY(0);
            }
        }

        /* Smooth scroll behavior */
        html {
            scroll-behavior: smooth;
        }

        /* Reduce motion for accessibility */
        @media (prefers-reduced-motion: reduce) {
            *,
            *::before,
            *::after {
                animation-duration: 0.01ms !important;
                animation-iteration-count: 1 !important;
                transition-duration: 0.01ms !important;
                scroll-behavior: auto !important;
            }

            .page-transition,
            .stagger-children > * {
                animation: none !important;
                opacity: 1 !important;
                transform: none !important;
            }
        }

        /* Progressive enhancement utilities */
        .js-only {
            display: none;
        }

        .js .js-only {
            display: block;
        }

        .no-js .no-js-hide {
            display: none;
        }

        .text-warning {
            color: var(--warning);
            font-weight: 500;
            font-size: 0.9rem;
        }
    "#.to_string())
}
