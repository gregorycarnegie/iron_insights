use maud::{Markup, PreEscaped};

pub fn render_responsive_styles() -> Markup {
    PreEscaped(r#"
        /* Responsive Design */
        @media (max-width: 1024px) {
            .main-content {
                grid-template-columns: 1fr;
            }
            
            .sidebar {
                display: none;
                position: fixed;
                top: 65px;
                left: 0;
                bottom: 0;
                width: 280px;
                z-index: 99;
                box-shadow: var(--shadow-lg);
            }
            
            .sidebar.mobile-open {
                display: block;
            }
            
            .chart-grid {
                grid-template-columns: 1fr;
            }
            
            .lift-breakdown {
                grid-template-columns: repeat(2, 1fr);
            }
            
            .mobile-menu-toggle {
                display: block;
            }
        }
        
        @media (max-width: 640px) {
            .header-content {
                padding: 1rem;
            }
            
            .content {
                padding: 1rem;
            }
            
            .stats-grid {
                grid-template-columns: 1fr;
            }
            
            .percentile-grid {
                grid-template-columns: 1fr;
            }
            
            .lift-breakdown {
                grid-template-columns: 1fr;
            }
        }
    "#.to_string())
}