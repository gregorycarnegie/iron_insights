use maud::{Markup, PreEscaped};

pub fn render_init_scripts() -> Markup {
    PreEscaped(r#"
        let debugMode = false;
        let lastResponse = null;
        let wasmModule = null;
        let calculate_dots_wasm = null;
        let calculate_dots_with_gender_wasm = null;
        let calculate_strength_level_wasm = null;
        let calculate_strength_level_for_lift_wasm = null;
        let calculate_strength_level_for_lift_with_gender_wasm = null;
        let get_strength_level_color_wasm = null;
        let calculate_dots_and_level_wasm = null;
        let calculate_dots_and_level_for_lift_wasm = null;
        let calculate_dots_and_level_for_lift_with_gender_wasm = null;
        let calculate_wilks_wasm = null;
        let calculate_ipf_gl_points_wasm = null;
        let calculate_all_scores_wasm = null;
        let calculate_strength_level_from_percentile_wasm = null;
        
        // UI state management for modern toggle-based controls
        let currentSex = 'M';
        let currentLiftType = 'squat';
        let currentEquipment = ['Raw'];
        let currentTimePeriod = 'last_5_years';
        let currentFederation = 'all';
        let currentBinCount = 50;
        
        // Initialize WASM module
        async function initWasm() {
            try {
                const wasmModule = await import('/static/wasm/iron_insights_wasm.js');
                await wasmModule.default();
                
                // Store WASM functions globally
                calculate_dots_wasm = wasmModule.calculate_dots;
                calculate_dots_with_gender_wasm = wasmModule.calculate_dots_with_gender;
                calculate_strength_level_wasm = wasmModule.calculate_strength_level;
                calculate_strength_level_for_lift_wasm = wasmModule.calculate_strength_level_for_lift;
                calculate_strength_level_for_lift_with_gender_wasm = wasmModule.calculate_strength_level_for_lift_with_gender;
                get_strength_level_color_wasm = wasmModule.get_strength_level_color;
                calculate_dots_and_level_wasm = wasmModule.calculate_dots_and_level;
                calculate_dots_and_level_for_lift_wasm = wasmModule.calculate_dots_and_level_for_lift;
                calculate_dots_and_level_for_lift_with_gender_wasm = wasmModule.calculate_dots_and_level_for_lift_with_gender;
                calculate_wilks_wasm = wasmModule.calculate_wilks;
                calculate_ipf_gl_points_wasm = wasmModule.calculate_ipf_gl_points;
                calculate_all_scores_wasm = wasmModule.calculate_all_scores;
                calculate_strength_level_from_percentile_wasm = wasmModule.calculate_strength_level_from_percentile;
                
                console.log('✅ WASM module loaded successfully');
                return true;
            } catch (error) {
                console.error('❌ Failed to load WASM module:', error);
                console.log('📋 Falling back to JavaScript implementation');
                return false;
            }
        }
        
        // Load Apache Arrow from bundled asset
        let Arrow;
        async function loadArrow() {
            console.log('🔄 Loading Apache Arrow library...');
            try {
                console.log('🔄 Loading Arrow from bundled asset');
                await loadScript('/static/js/dist/arrow.min.js');
                if (typeof window.Arrow !== 'undefined') {
                    Arrow = window.Arrow;
                    console.log('✅ Apache Arrow library loaded successfully from bundled asset');
                    return true;
                }
                throw new Error('Failed to load bundled Arrow library');
            } catch (error) {
                console.error('❌ Failed to load Apache Arrow library:', error);
                return false;
            }
        }
        
        // Helper function to load script
        function loadScript(src) {
            return new Promise((resolve, reject) => {
                const script = document.createElement('script');
                script.src = src;
                script.onload = resolve;
                script.onerror = reject;
                document.head.appendChild(script);
            });
        }
    "#.to_string())
}
