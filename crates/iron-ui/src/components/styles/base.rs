use maud::{Markup, PreEscaped};

pub fn render_base_styles() -> Markup {
    PreEscaped(r#"
        :root {
            /* Brand palette - Fallbacks */
            --primary: #3b82f6;
            --primary-dark: #2563eb;
            --primary-light: #60a5fa;
            --primary-rgb: 59, 130, 246;

            --secondary: #10b981;
            --secondary-dark: #059669;
            --secondary-light: #34d399;

            --danger: #ef4444;
            --warning: #f59e0b;

            /* Neutral scale */
            --dark: #020617;
            --light: #f8fafc;
            --border: rgba(148, 163, 184, 0.1);

            /* Semantic tokens */
            --bg: #020617;
            --surface: #0f172a;
            
            --text-primary: #f8fafc;
            --text-secondary: #94a3b8;
            --text-tertiary: #64748b;

            /* Lift-specific colors - Neon Versions */
            --squat-color: #ef4444;
            --bench-color: #3b82f6;
            --deadlift-color: #a855f7;
            --total-color: #10b981;

            /* Shadows & Glows */
            --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
            --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
            --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1);
            --glow-primary: 0 0 20px rgba(59, 130, 246, 0.5);
            --glow-text: 0 0 10px rgba(255, 255, 255, 0.2);
        }
        
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            background-color: var(--bg);
            background-image: 
                radial-gradient(circle at 15% 50%, rgba(59, 130, 246, 0.08), transparent 25%),
                radial-gradient(circle at 85% 30%, rgba(16, 185, 129, 0.08), transparent 25%),
                radial-gradient(circle at 50% 80%, rgba(168, 85, 247, 0.05), transparent 40%);
            background-attachment: fixed;
            color: var(--text-primary);
            line-height: 1.6;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }

        /* Typography Enhancements */
        h1, h2, h3, h4, h5, h6 {
            letter-spacing: -0.025em;
            color: #fff;
        }

        .text-gradient {
            background: linear-gradient(135deg, #fff 0%, #94a3b8 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }

        /* Focus styles */
        *:focus-visible {
            outline: 2px solid var(--primary);
            outline-offset: 2px;
            box-shadow: var(--glow-primary);
        }

        /* Skip link */
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
        .skip-link:focus { top: 6px; }

        /* Utilities */
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

        /* Animations */
        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(10px); }
            to { opacity: 1; transform: translateY(0); }
        }

        @keyframes pulseGlow {
            0%, 100% { box-shadow: 0 0 15px rgba(59, 130, 246, 0.2); }
            50% { box-shadow: 0 0 25px rgba(59, 130, 246, 0.5); }
        }

        .page-transition {
            opacity: 0;
            animation: fadeIn 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards;
        }

        .stagger-children > * {
            opacity: 0;
            animation: fadeIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) forwards;
        }
        .stagger-children > *:nth-child(1) { animation-delay: 0.05s; }
        .stagger-children > *:nth-child(2) { animation-delay: 0.1s; }
        .stagger-children > *:nth-child(3) { animation-delay: 0.15s; }
        .stagger-children > *:nth-child(4) { animation-delay: 0.2s; }
        .stagger-children > *:nth-child(5) { animation-delay: 0.25s; }
        .stagger-children > *:nth-child(6) { animation-delay: 0.3s; }

        .hover-lift {
            transition: transform 0.2s ease, box-shadow 0.2s ease;
        }
        .hover-lift:hover {
            transform: translateY(-2px);
            box-shadow: 0 10px 30px -10px rgba(0, 0, 0, 0.5);
        }

        /* Scroll behavior */
        html { scroll-behavior: smooth; }

        /* Reduced motion */
        @media (prefers-reduced-motion: reduce) {
            *, *::before, *::after {
                animation-duration: 0.01ms !important;
                animation-iteration-count: 1 !important;
                transition-duration: 0.01ms !important;
                scroll-behavior: auto !important;
            }
        }

        /* JS helpers */
        .js-only { display: none; }
        .js .js-only { display: block; }
        .no-js .no-js-hide { display: none; }
        
        .text-warning { color: var(--warning); font-weight: 500; }
    "#.to_string())
}
