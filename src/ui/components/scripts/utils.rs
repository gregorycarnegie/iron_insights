use maud::{Markup, PreEscaped};

pub fn render_utility_scripts() -> Markup {
    PreEscaped(
        r#"
        // Helper function to parse and calculate sums like "340+270+190"
        function parseSum(input) {
            console.log('parseSum called with input:', input, 'type:', typeof input);
            
            if (typeof input !== 'string') {
                const result = parseFloat(input);
                console.log('parseSum: non-string input, returning:', result);
                return result;
            }
            
            const trimmedInput = input.trim();
            if (!trimmedInput) {
                console.log('parseSum: empty input, returning NaN');
                return NaN;
            }
            
            // If it doesn't contain + or -, just parse as a regular number
            if (!trimmedInput.includes('+') && !trimmedInput.match(/\d\s*-\s*\d/)) {
                const result = parseFloat(trimmedInput);
                console.log('parseSum: no operators found, returning:', result);
                return result;
            }
            
            console.log('parseSum: sum expression detected, parsing...');
            
            try {
                // Simple approach: use eval on sanitized input for safety
                // First sanitize: only allow numbers, +, -, ., and spaces
                const sanitized = trimmedInput.replace(/[^0-9+\-\.\s]/g, '');
                if (sanitized !== trimmedInput.replace(/\s/g, '').replace(/\s/g, '')) {
                    console.log('parseSum: contains invalid characters, falling back');
                    return parseFloat(input);
                }
                
                // Replace multiple operators and clean up
                const expression = sanitized.replace(/\s/g, '').replace(/([+\-]){2,}/g, '+');
                console.log('parseSum: sanitized expression:', expression);
                
                // Use Function constructor for safer evaluation (no access to scope)
                const result = new Function('return ' + expression)();
                
                if (typeof result !== 'number' || !isFinite(result)) {
                    console.log('parseSum: invalid result, falling back');
                    return parseFloat(input);
                }
                
                console.log('parseSum: calculated result:', result);
                return result;
                
            } catch (error) {
                console.warn('Failed to parse sum expression:', input, error);
                return parseFloat(input); // Fall back to regular parsing
            }
        }
        
        // Get human-readable time ago
        function getTimeAgo(timestamp) {
            const now = Date.now();
            const diff = now - timestamp;
            const seconds = Math.floor(diff / 1000);
            
            if (seconds < 60) return 'just now';
            if (seconds < 3600) return Math.floor(seconds / 60) + 'm ago';
            if (seconds < 86400) return Math.floor(seconds / 3600) + 'h ago';
            return Math.floor(seconds / 86400) + 'd ago';
        }
    "#
        .to_string(),
    )
}
