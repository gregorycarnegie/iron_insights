use maud::{Markup, PreEscaped};

pub fn render_table_styles() -> Markup {
    PreEscaped(r#"
        /* Tables */
        .data-table {
            width: 100%;
            background: white;
            border: 1px solid var(--border);
            border-radius: 0.5rem;
            overflow: hidden;
        }
        
        .data-table thead {
            background: var(--light-secondary);
            border-bottom: 1px solid var(--border);
        }
        
        .data-table th {
            padding: 0.75rem 1rem;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-tertiary);
            text-align: left;
        }
        
        .data-table td {
            padding: 0.75rem 1rem;
            font-size: 0.875rem;
            color: var(--text-primary);
            border-bottom: 1px solid var(--border);
        }
        
        .data-table tr:last-child td {
            border-bottom: none;
        }
        
        .data-table tr:hover {
            background: var(--light);
        }
    "#.to_string())
}