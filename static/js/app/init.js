
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
let currentSex = 'All';
let currentLiftType = 'squat';
let currentEquipment = ['Raw'];
let currentTimePeriod = 'last_5_years';
let currentFederation = 'all';
let currentBinCount = 50;
let currentWeightClass = 'All';

// Initialize WASM module with streaming compilation for faster startup
async function initWasm() {
    try {
        const wasmUrl = '/static/wasm/iron_insights_wasm_bg.wasm';

        // Check if streaming compilation is supported (modern browsers)
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                console.log('🚀 Loading WASM via streaming compilation...');
                // Load the JS glue code first
                const wasmModule = await import('/static/wasm/iron_insights_wasm.js');
                // Use default initialization which leverages streaming internally
                await wasmModule.default();

                // Store WASM functions globally
                window.calculate_dots_wasm = wasmModule.calculate_dots;
                window.calculate_dots_with_gender_wasm = wasmModule.calculate_dots_with_gender;
                window.calculate_strength_level_wasm = wasmModule.calculate_strength_level;
                window.calculate_strength_level_for_lift_wasm = wasmModule.calculate_strength_level_for_lift;
                window.calculate_strength_level_for_lift_with_gender_wasm = wasmModule.calculate_strength_level_for_lift_with_gender;
                window.get_strength_level_color_wasm = wasmModule.get_strength_level_color;
                window.calculate_dots_and_level_wasm = wasmModule.calculate_dots_and_level;
                window.calculate_dots_and_level_for_lift_wasm = wasmModule.calculate_dots_and_level_for_lift;
                window.calculate_dots_and_level_for_lift_with_gender_wasm = wasmModule.calculate_dots_and_level_for_lift_with_gender;
                window.calculate_wilks_wasm = wasmModule.calculate_wilks;
                window.calculate_ipf_gl_points_wasm = wasmModule.calculate_ipf_gl_points;
                window.calculate_all_scores_wasm = wasmModule.calculate_all_scores;
                window.calculate_strength_level_from_percentile_wasm = wasmModule.calculate_strength_level_from_percentile;

                console.log('✅ WASM module loaded successfully via streaming');
                return true;
            } catch (streamError) {
                console.warn('⚠️ Streaming WASM load failed, falling back to standard load:', streamError);
                // Fall through to standard loading
            }
        }

        // Fallback: Standard WASM loading (older browsers or if streaming fails)
        console.log('📦 Loading WASM via standard method...');
        const wasmModule = await import('/static/wasm/iron_insights_wasm.js');
        await wasmModule.default();

        // Store WASM functions globally
        window.calculate_dots_wasm = wasmModule.calculate_dots;
        window.calculate_dots_with_gender_wasm = wasmModule.calculate_dots_with_gender;
        window.calculate_strength_level_wasm = wasmModule.calculate_strength_level;
        window.calculate_strength_level_for_lift_wasm = wasmModule.calculate_strength_level_for_lift;
        window.calculate_strength_level_for_lift_with_gender_wasm = wasmModule.calculate_strength_level_for_lift_with_gender;
        window.get_strength_level_color_wasm = wasmModule.get_strength_level_color;
        window.calculate_dots_and_level_wasm = wasmModule.calculate_dots_and_level;
        window.calculate_dots_and_level_for_lift_wasm = wasmModule.calculate_dots_and_level_for_lift;
        window.calculate_dots_and_level_for_lift_with_gender_wasm = wasmModule.calculate_dots_and_level_for_lift_with_gender;
        window.calculate_wilks_wasm = wasmModule.calculate_wilks;
        window.calculate_ipf_gl_points_wasm = wasmModule.calculate_ipf_gl_points;
        window.calculate_all_scores_wasm = wasmModule.calculate_all_scores;
        window.calculate_strength_level_from_percentile_wasm = wasmModule.calculate_strength_level_from_percentile;

        console.log('✅ WASM module loaded successfully');
        return true;
    } catch (error) {
        console.error('❌ Failed to load WASM module:', error);
        console.log('📋 Falling back to JavaScript implementation');
        return false;
    }
}

// Load analytics dependencies using lazy loader
let Arrow, Plotly;
async function loadAnalyticsDependencies() {
    console.log('🔄 Loading analytics dependencies...');
    try {
        const deps = await window.lazyLoader.loadAnalyticsDependencies();
        Arrow = deps.Arrow;
        Plotly = deps.Plotly;
        console.log('✅ Analytics dependencies loaded via lazy loader');
        return true;
    } catch (error) {
        console.error('❌ Failed to load analytics dependencies:', error);
    }
}

// Expose functions to global scope
window.initWasm = initWasm;
window.loadAnalyticsDependencies = loadAnalyticsDependencies;

// Expose state variables to global scope
window.currentSex = currentSex;
window.currentLiftType = currentLiftType;
window.currentEquipment = currentEquipment;
window.currentTimePeriod = currentTimePeriod;
window.currentFederation = currentFederation;
window.currentBinCount = currentBinCount;
window.currentWeightClass = currentWeightClass;
window.lastResponse = lastResponse;
window.debugMode = debugMode;

// Expose WASM function placeholders to global scope so they can be updated
window.calculate_dots_wasm = calculate_dots_wasm;
window.calculate_dots_with_gender_wasm = calculate_dots_with_gender_wasm;
window.calculate_strength_level_wasm = calculate_strength_level_wasm;
window.calculate_strength_level_for_lift_wasm = calculate_strength_level_for_lift_wasm;
window.calculate_strength_level_for_lift_with_gender_wasm = calculate_strength_level_for_lift_with_gender_wasm;
window.get_strength_level_color_wasm = get_strength_level_color_wasm;
window.calculate_dots_and_level_wasm = calculate_dots_and_level_wasm;
window.calculate_dots_and_level_for_lift_wasm = calculate_dots_and_level_for_lift_wasm;
window.calculate_dots_and_level_for_lift_with_gender_wasm = calculate_dots_and_level_for_lift_with_gender_wasm;
window.calculate_wilks_wasm = calculate_wilks_wasm;
window.calculate_ipf_gl_points_wasm = calculate_ipf_gl_points_wasm;
window.calculate_all_scores_wasm = calculate_all_scores_wasm;
window.calculate_strength_level_from_percentile_wasm = calculate_strength_level_from_percentile_wasm;

