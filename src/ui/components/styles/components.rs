use maud::{Markup, PreEscaped};

pub fn render_component_styles() -> Markup {
    PreEscaped(r#"
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
        
        /* Modern Toggle Buttons */
        .toggle-group {
            display: flex;
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.5rem;
            overflow: hidden;
        }
        
        .toggle-button {
            flex: 1;
            padding: 0.5rem;
            border: none;
            background: white;
            color: var(--text-secondary);
            font-size: 0.875rem;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s;
            border-right: 1px solid var(--border);
        }
        
        .toggle-button:last-child {
            border-right: none;
        }
        
        .toggle-button.active {
            background: var(--primary);
            color: white;
        }
        
        .toggle-button:hover:not(.active) {
            background: var(--light-secondary);
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
        
        /* Primary Button */
        .btn-primary {
            width: 100%;
            padding: 0.75rem 1.5rem;
            background: var(--primary);
            color: white;
            border: none;
            border-radius: 0.5rem;
            font-size: 0.875rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
            margin-top: 1.5rem;
        }
        
        .btn-primary:hover {
            background: var(--primary-dark);
            transform: translateY(-1px);
            box-shadow: var(--shadow-md);
        }
        
        .btn-primary:active {
            transform: translateY(0);
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
        
        /* Loading States */
        .skeleton {
            background: linear-gradient(90deg, var(--light-secondary) 25%, var(--border) 50%, var(--light-secondary) 75%);
            background-size: 200% 100%;
            animation: loading 1.5s infinite;
            border-radius: 0.375rem;
        }
        
        @keyframes loading {
            0% { background-position: 200% 0; }
            100% { background-position: -200% 0; }
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
    "#.to_string())
}
