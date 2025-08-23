use maud::{Markup, PreEscaped};

pub fn render_layout_styles() -> Markup {
    PreEscaped(r#"
        /* Container */
        .container {
            max-width: 1440px;
            margin: 0 auto;
            padding: 0;
            background: white;
            min-height: 100vh;
        }
        
        /* Modern Header */
        .header {
            background: white;
            border-bottom: 1px solid var(--border);
            padding: 0;
            margin-bottom: 0;
            position: sticky;
            top: 0;
            z-index: 100;
            box-shadow: var(--shadow-sm);
        }
        
        .header-content {
            max-width: 1440px;
            margin: 0 auto;
            padding: 1rem 2rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }
        
        .header h1 {
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--dark);
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }
        
        .header h1 .logo {
            color: var(--primary);
        }
        
        .header-nav {
            display: flex;
            gap: 2rem;
            align-items: center;
        }
        
        .header-nav a {
            color: var(--text-secondary);
            text-decoration: none;
            font-weight: 500;
            transition: color 0.2s;
        }
        
        .header-nav a:hover {
            color: var(--primary);
        }
        
        /* Main Content Area */
        .main-content {
            display: grid;
            grid-template-columns: 280px 1fr;
            min-height: calc(100vh - 65px);
        }
        
        /* Sidebar Controls */
        .sidebar {
            background: var(--light-secondary);
            border-right: 1px solid var(--border);
            padding: 1.5rem;
            overflow-y: auto;
        }
        
        .sidebar h3 {
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-tertiary);
            margin-bottom: 1rem;
        }
        
        /* Content Area */
        .content {
            padding: 2rem;
            overflow-y: auto;
        }
        
        /* Mobile Menu Toggle */
        .mobile-menu-toggle {
            display: none;
            padding: 0.5rem;
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.375rem;
            cursor: pointer;
        }
        
        /* Overlay for mobile sidebar */
        .sidebar-overlay {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.5);
            z-index: 98;
        }
        
        .sidebar-overlay.active {
            display: block;
        }
    "#.to_string())
}