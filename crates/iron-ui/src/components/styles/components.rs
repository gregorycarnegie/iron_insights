use maud::{Markup, PreEscaped};

pub fn render_component_styles() -> Markup {
    PreEscaped(r#"
        /* Buttons - Enhanced with micro-interactions & Glows */
        .btn {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            gap: 0.5rem;
            padding: 0.65rem 1.1rem;
            border-radius: 0.5rem;
            font-weight: 600;
            font-size: 0.95rem;
            border: 1px solid transparent;
            text-decoration: none;
            cursor: pointer;
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            will-change: transform, box-shadow;
            position: relative;
            overflow: hidden;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }

        /* Larger button variant used by calculators */
        .btn-lg {
            padding: 0.875rem 2rem;
            font-size: 1.1rem;
        }

        .btn::before {
            content: '';
            position: absolute;
            top: 0;
            left: -100%;
            width: 100%;
            height: 100%;
            background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
            transition: left 0.5s ease;
        }

        .btn:hover::before {
            left: 100%;
        }

        .btn:active { 
            transform: translateY(1px) scale(0.98);
        }
        
        .btn-primary { 
            background: linear-gradient(135deg, var(--primary), var(--primary-dark));
            color: #fff;
            box-shadow: 0 0 15px rgba(59, 130, 246, 0.4);
            border: 1px solid rgba(255, 255, 255, 0.1);
        }
        
        .btn-primary:hover { 
            background: linear-gradient(135deg, var(--primary-light), var(--primary));
            box-shadow: 0 0 25px rgba(59, 130, 246, 0.6), 0 0 5px rgba(255, 255, 255, 0.5);
            transform: translateY(-2px);
            border-color: rgba(255, 255, 255, 0.3);
            text-shadow: 0 0 8px rgba(255, 255, 255, 0.5);
        }
        
        .btn-secondary { 
            background: linear-gradient(135deg, var(--secondary), var(--secondary-dark));
            color: #fff;
            box-shadow: 0 0 15px rgba(16, 185, 129, 0.4);
            border: 1px solid rgba(255, 255, 255, 0.1);
        }
        
        .btn-secondary:hover { 
            background: linear-gradient(135deg, var(--secondary-light), var(--secondary));
            box-shadow: 0 0 25px rgba(16, 185, 129, 0.6);
            transform: translateY(-2px);
            border-color: rgba(255, 255, 255, 0.3);
        }
        
        .btn-tertiary { 
            background: rgba(255, 255, 255, 0.05);
            color: var(--text-secondary);
            border: 1px solid var(--glass-border);
            backdrop-filter: blur(4px);
        }
        
        .btn-tertiary:hover { 
            background: rgba(255, 255, 255, 0.1);
            border-color: var(--primary);
            color: #fff;
            box-shadow: 0 0 15px rgba(59, 130, 246, 0.3);
            transform: translateY(-1px);
        }

        /* Enhanced glass/elevated cards with micro-interactions */
        .glass-card {
            background: var(--glass-bg);
            backdrop-filter: blur(12px);
            -webkit-backdrop-filter: blur(12px);
            border: 1px solid var(--glass-border);
            border-radius: 1rem;
            box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.36);
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            position: relative;
            overflow: hidden;
        }

        .glass-card::after {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 1px;
            background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
        }
        
        .card-hover:hover { 
            transform: translateY(-5px) scale(1.01);
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4), 0 0 20px rgba(59, 130, 246, 0.2);
            border-color: rgba(59, 130, 246, 0.3);
        }

        /* Feature cards with enhanced interactions */
        .feature-card {
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            transform-origin: center;
            position: relative;
        }

        .feature-card:hover {
            transform: translateY(-6px);
        }

        .feature-card:hover .icon-wrap {
            transform: scale(1.1) rotate(5deg);
            filter: drop-shadow(0 0 8px var(--primary));
        }

        .feature-card .icon-wrap {
            transition: all 0.3s ease;
        }

        /* Stat cards with pulse effect */
        .stat-card {
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            position: relative;
        }

        .stat-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 0 30px rgba(59, 130, 246, 0.15);
            border-color: var(--primary);
        }

        .stat-card:hover .stat-number {
            transform: scale(1.05);
            color: #fff;
            text-shadow: 0 0 10px var(--primary);
        }

        .stat-number {
            transition: all 0.2s ease;
        }

        .control-section {
            margin-bottom: 2rem;
        }
        
        .control-group {
            margin-bottom: 1rem;
        }
        
        .control-group label {
            display: block;
            font-size: 0.85rem;
            font-weight: 600;
            color: var(--text-secondary);
            margin-bottom: 0.5rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }
        
        .control-group input[type="number"],
        .control-group input[type="text"],
        .control-group select {
            width: 100%;
            padding: 0.6rem 0.85rem;
            border: 1px solid var(--glass-border);
            border-radius: 0.5rem;
            font-size: 0.9rem;
            background: rgba(0, 0, 0, 0.2);
            color: #fff;
            transition: all 0.3s ease;
        }
        
        .control-group input:focus,
        .control-group select:focus {
            outline: none;
            border-color: var(--primary);
            box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2), 0 0 15px rgba(59, 130, 246, 0.3);
            background: rgba(0, 0, 0, 0.4);
        }
        
        /* Enhanced Toggle Buttons with micro-interactions */
        .toggle-group {
            display: flex;
            background: rgba(0, 0, 0, 0.3);
            border: 1px solid var(--glass-border);
            border-radius: 0.5rem;
            overflow: hidden;
            position: relative;
            padding: 2px;
        }
        
        .toggle-button {
            flex: 1;
            padding: 0.5rem 0.5rem;
            border: none;
            background: transparent;
            color: var(--text-secondary);
            font-size: 0.85rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.3s ease;
            border-radius: 0.3rem;
            position: relative;
            z-index: 1;
        }
        
        .toggle-button.active {
            background: var(--surface-active);
            color: #fff;
            box-shadow: 0 1px 3px rgba(0,0,0,0.2);
        }
        
        .toggle-button:hover:not(.active) {
            color: #fff;
            background: rgba(255, 255, 255, 0.05);
        }

        .toggle-button:focus {
            outline: none;
            color: #fff;
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
            gap: 0.625rem;
            font-size: 0.9rem;
            color: var(--text-secondary);
            cursor: pointer;
            padding: 0.25rem 0;
            transition: color 0.2s;
        }

        .checkbox-label:hover {
            color: #fff;
        }
        
        .checkbox-label input[type="checkbox"] {
            width: 1.125rem;
            height: 1.125rem;
            border: 1px solid var(--glass-border);
            border-radius: 0.25rem;
            cursor: pointer;
            margin-right: 0.25rem;
            background: rgba(0,0,0,0.3);
            appearance: none;
            -webkit-appearance: none;
            position: relative;
            transition: all 0.2s;
        }
        
        .checkbox-label input[type="checkbox"]:checked {
            background: var(--primary);
            border-color: var(--primary);
            box-shadow: 0 0 10px var(--primary-glow);
        }

        .checkbox-label input[type="checkbox"]:checked::after {
            content: '✓';
            position: absolute;
            color: white;
            font-size: 0.8rem;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
        }
        
        /* Sidebar primary action */
        .btn-primary {
            width: 100%;
            margin-top: 1.5rem;
        }
        
        /* Pills/Tags */
        .tag {
            display: inline-block;
            padding: 0.25rem 0.75rem;
            font-size: 0.75rem;
            font-weight: 600;
            border-radius: 9999px;
            background: rgba(255, 255, 255, 0.05);
            color: var(--text-secondary);
            border: 1px solid var(--glass-border);
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }
        
        .tag.primary { background: rgba(59, 130, 246, 0.15); color: #60a5fa; border-color: rgba(59, 130, 246, 0.3); }
        .tag.success { background: rgba(16, 185, 129, 0.15); color: #34d399; border-color: rgba(16, 185, 129, 0.3); }
        .tag.danger { background: rgba(239, 68, 68, 0.15); color: #f87171; border-color: rgba(239, 68, 68, 0.3); }
        .tag.warning { background: rgba(245, 158, 11, 0.15); color: #fbbf24; border-color: rgba(245, 158, 11, 0.3); }
        
        /* Loading States - Enhanced Skeletons */
        .skeleton {
            background: linear-gradient(90deg, rgba(255,255,255,0.03) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.03) 75%);
            background-size: 200% 100%;
            animation: skeleton-shimmer 2s infinite;
            border-radius: 0.375rem;
            position: relative;
            overflow: hidden;
        }
        
        @keyframes skeleton-shimmer {
            0% { background-position: 200% 0; }
            100% { background-position: -200% 0; }
        }

        /* Content-aware skeleton variants */
        .skeleton-text {
            height: 1.2em;
            width: 100%;
            margin-bottom: 0.5rem;
        }

        .skeleton-text.short { width: 60%; }
        .skeleton-text.medium { width: 80%; }
        .skeleton-text.long { width: 95%; }

        .skeleton-title {
            height: 2rem;
            width: 70%;
            margin-bottom: 1rem;
            border-radius: 0.5rem;
        }

        .skeleton-button {
            height: 2.5rem;
            width: 120px;
            border-radius: 0.5rem;
        }

        .skeleton-card {
            height: 200px;
            width: 100%;
            border-radius: 0.75rem;
            margin-bottom: 1rem;
        }

        .skeleton-stat {
            height: 4rem;
            width: 100%;
            border-radius: 0.5rem;
            margin-bottom: 0.5rem;
        }

        .skeleton-chart {
            height: 400px;
            width: 100%;
            border-radius: 0.75rem;
            position: absolute;
            inset: 0;
            margin: 0;
            z-index: 1;
            transition: opacity 0.3s ease;
        }

        .skeleton-chart.loaded {
            opacity: 0;
            pointer-events: none;
        }

        .skeleton-chart::before {
            content: '';
            position: absolute;
            top: 20px;
            left: 20px;
            right: 20px;
            height: 20px;
            background: rgba(255, 255, 255, 0.05);
            border-radius: 4px;
            animation: skeleton-shimmer 2s infinite;
        }

        .skeleton-chart::after {
            content: '';
            position: absolute;
            bottom: 20px;
            left: 20px;
            right: 20px;
            top: 60px;
            background: linear-gradient(
                to top,
                rgba(255, 255, 255, 0.05) 0%,
                rgba(255, 255, 255, 0.02) 30%,
                transparent 100%
            );
            border-radius: 8px;
        }

        /* Skeleton states */
        .skeleton-container {
            opacity: 1;
            transition: opacity 0.3s ease;
        }

        .skeleton-container.loaded {
            opacity: 0;
            pointer-events: none;
        }

        .content-container {
            opacity: 0;
            transition: opacity 0.3s ease 0.1s;
        }

        .content-container.loaded {
            opacity: 1;
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
            background: rgba(0, 0, 0, 0.9);
            color: white;
            padding: 0.5rem 0.75rem;
            border-radius: 0.375rem;
            font-size: 0.75rem;
            white-space: nowrap;
            opacity: 0;
            pointer-events: none;
            transition: opacity 0.2s;
            margin-bottom: 0.5rem;
            border: 1px solid var(--glass-border);
            backdrop-filter: blur(4px);
        }
        
        .tooltip:hover::after {
            opacity: 1;
        }

        /* No-JS fallback styles */
        .no-js-only {
            display: block;
        }

        .js .no-js-only {
            display: none;
        }

        .no-js-only label {
            display: block;
            margin-bottom: 0.5rem;
            padding: 0.5rem;
            border: 1px solid var(--border);
            border-radius: 0.25rem;
            cursor: pointer;
            transition: background-color 0.2s ease;
        }

        .no-js-only label:hover {
            background-color: var(--surface-hover);
        }

        .no-js-only input[type="radio"] {
            margin-right: 0.5rem;
        }

        /* Enhanced form styling for progressive enhancement */
        form:invalid .btn-primary {
            opacity: 0.6;
            cursor: not-allowed;
            box-shadow: none;
        }

        /* Loading state for forms */
        .form-loading {
            position: relative;
            pointer-events: none;
        }

        .form-loading::after {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.7);
            display: flex;
            align-items: center;
            justify-content: center;
            border-radius: inherit;
            backdrop-filter: blur(2px);
        }
    "#.to_string())
}
