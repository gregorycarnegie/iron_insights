use maud::{Markup, PreEscaped};

pub fn render_component_styles() -> Markup {
    PreEscaped(r#"
        /* Buttons - Enhanced with micro-interactions */
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
            transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
            will-change: transform;
            position: relative;
            overflow: hidden;
        }

        .btn::before {
            content: '';
            position: absolute;
            top: 50%;
            left: 50%;
            width: 0;
            height: 0;
            background: rgba(255, 255, 255, 0.2);
            border-radius: 50%;
            transform: translate(-50%, -50%);
            transition: width 0.3s ease, height 0.3s ease;
        }

        .btn:active::before {
            width: 300px;
            height: 300px;
        }

        .btn:active { 
            transform: translateY(1px) scale(0.98);
            transition: all 0.1s ease;
        }
        
        .btn-primary { 
            background: linear-gradient(135deg, var(--primary), var(--primary-dark));
            color: #fff;
            box-shadow: 0 2px 4px rgba(var(--primary-rgb), 0.2);
        }
        
        .btn-primary:hover { 
            background: linear-gradient(135deg, var(--primary-light), var(--primary));
            box-shadow: 0 8px 25px rgba(var(--primary-rgb), 0.3);
            transform: translateY(-2px);
        }
        
        .btn-secondary { 
            background: linear-gradient(135deg, var(--secondary), var(--secondary-dark));
            color: #fff;
            box-shadow: 0 2px 4px rgba(16, 185, 129, 0.2);
        }
        
        .btn-secondary:hover { 
            background: linear-gradient(135deg, var(--secondary-light), var(--secondary));
            box-shadow: 0 8px 25px rgba(16, 185, 129, 0.3);
            transform: translateY(-2px);
        }
        
        .btn-tertiary { 
            background: var(--surface-secondary);
            color: var(--text-primary);
            border-color: var(--border);
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }
        
        .btn-tertiary:hover { 
            background: var(--surface);
            border-color: var(--primary);
            color: var(--primary);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
            transform: translateY(-1px);
        }

        /* Enhanced glass/elevated cards with micro-interactions */
        .glass-card {
            background: linear-gradient(180deg, rgba(255,255,255,0.04), rgba(255,255,255,0.02)) , var(--surface);
            border: 1px solid rgba(148, 163, 184, 0.18);
            border-radius: 0.75rem;
            box-shadow: 0 4px 6px rgba(2,6,23,0.05), 0 1px 3px rgba(2,6,23,0.1);
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            position: relative;
            overflow: hidden;
        }

        .glass-card::before {
            content: '';
            position: absolute;
            top: 0;
            left: -100%;
            width: 100%;
            height: 100%;
            background: linear-gradient(90deg, transparent, rgba(var(--primary-rgb), 0.03), transparent);
            transition: left 0.5s ease;
        }

        .glass-card:hover::before {
            left: 100%;
        }
        
        .card-hover:hover { 
            transform: translateY(-8px) scale(1.02);
            box-shadow: 0 20px 40px rgba(2,6,23,0.15), 0 8px 16px rgba(var(--primary-rgb), 0.1);
            border-color: rgba(var(--primary-rgb), 0.2);
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
            transform: scale(1.1) rotate(2deg);
            transition: transform 0.3s cubic-bezier(0.68, -0.55, 0.265, 1.55);
        }

        .feature-card .icon-wrap {
            transition: transform 0.3s ease;
        }

        /* Stat cards with pulse effect */
        .stat-card {
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            position: relative;
        }

        .stat-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 12px 24px rgba(var(--primary-rgb), 0.15);
        }

        .stat-card:hover .stat-number {
            transform: scale(1.05);
            color: var(--primary);
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
            font-size: 0.9rem;
            font-weight: 500;
            color: var(--text-secondary);
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
        
        /* Enhanced Toggle Buttons with micro-interactions */
        .toggle-group {
            display: flex;
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.5rem;
            overflow: hidden;
            position: relative;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            transition: box-shadow 0.2s ease;
        }

        .toggle-group:hover {
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
        }
        
        .toggle-button {
            flex: 1;
            padding: 0.65rem 0.5rem;
            border: none;
            background: white;
            color: var(--text-secondary);
            font-size: 0.875rem;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            border-right: 1px solid var(--border);
            position: relative;
            z-index: 1;
        }
        
        .toggle-button:last-child {
            border-right: none;
        }
        
        .toggle-button.active {
            background: linear-gradient(135deg, var(--primary), var(--primary-dark));
            color: white;
            box-shadow: 0 2px 4px rgba(var(--primary-rgb), 0.3);
            transform: scale(1.02);
        }
        
        .toggle-button:hover:not(.active) {
            background: var(--light-secondary);
            transform: scale(1.01);
            color: var(--primary);
        }

        .toggle-button:focus {
            outline: 2px solid var(--primary);
            outline-offset: 2px;
            z-index: 2;
        }

        .toggle-button:active {
            transform: scale(0.98);
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
            gap: 0.625rem; /* add a touch more space */
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
            margin-right: 0.25rem; /* fallback spacing for browsers without flex gap support */
        }
        
        .checkbox-label input[type="checkbox"]:checked {
            accent-color: var(--primary);
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
            font-weight: 500;
            border-radius: 9999px;
            background: var(--light-secondary);
            color: var(--text-secondary);
        }
        
        .tag.primary { background: rgba(37, 99, 235, 0.1); color: var(--primary); }
        .tag.success { background: rgba(16, 185, 129, 0.1); color: var(--secondary); }
        .tag.danger { background: rgba(239, 68, 68, 0.1); color: var(--danger); }
        .tag.warning { background: rgba(245, 158, 11, 0.1); color: var(--warning); }
        
        /* Loading States - Enhanced Skeletons */
        .skeleton {
            background: linear-gradient(90deg, var(--surface-secondary) 25%, rgba(var(--primary-rgb), 0.08) 50%, var(--surface-secondary) 75%);
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
            background: rgba(var(--primary-rgb), 0.1);
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
                rgba(var(--primary-rgb), 0.15) 0%,
                rgba(var(--primary-rgb), 0.08) 30%,
                rgba(var(--primary-rgb), 0.05) 60%,
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
            background: rgba(255, 255, 255, 0.8);
            display: flex;
            align-items: center;
            justify-content: center;
            border-radius: inherit;
        }
    "#.to_string())
}
