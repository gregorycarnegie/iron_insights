import type { ArrowApi, LazyLoaderDeps, PlotlyApi, EquipmentOption, LiftType, SexValue } from './types';

let debugMode = false;
let lastResponse = null;
let calculate_dots_wasm: ((lift: number, bw: number) => number) | null = null;
let calculate_dots_with_gender_wasm: ((lift: number, bw: number, isMale: boolean) => number) | null = null;
let calculate_strength_level_wasm: ((dots: number, isMale: boolean) => string) | null = null;
let calculate_strength_level_for_lift_wasm: ((dots: number, lift: LiftType, isMale: boolean) => string) | null = null;
let calculate_strength_level_for_lift_with_gender_wasm:
  | ((lift: number, bw: number, isMale: boolean, liftType: LiftType) => { dots: number; level: string; color: string })
  | null = null;
let get_strength_level_color_wasm: ((level: string) => string) | null = null;
let calculate_dots_and_level_wasm: ((lift: number, bw: number, isMale: boolean) => { dots: number; level: string; color: string }) | null = null;
let calculate_dots_and_level_for_lift_wasm:
  | ((lift: number, bw: number, isMale: boolean, liftType: LiftType) => { dots: number; level: string; color: string })
  | null = null;
let calculate_dots_and_level_for_lift_with_gender_wasm:
  | ((lift: number, bw: number, isMale: boolean, liftType: LiftType) => { dots: number; level: string; color: string })
  | null = null;
let calculate_wilks_wasm: ((lift: number, bw: number, isMale: boolean) => number) | null = null;
let calculate_ipf_gl_points_wasm: ((lift: number, bw: number, isMale: boolean) => number) | null = null;
let calculate_all_scores_wasm: ((lift: number, bw: number, isMale: boolean) => { dots: number; wilks: number; ipf_gl_points: number }) | null = null;
let calculate_strength_level_from_percentile_wasm: ((percentile: number, isMale: boolean, lift: LiftType) => string) | null = null;

// UI state management for modern toggle-based controls
let currentSex: SexValue = 'All';
let currentLiftType: LiftType = 'squat';
let currentEquipment: EquipmentOption[] = ['Raw'];
let currentTimePeriod = 'last_5_years';
let currentFederation = 'all';
let currentBinCount = 50;
let currentWeightClass = 'All';

// Initialize WASM module with streaming compilation for faster startup
async function initWasm(): Promise<boolean> {
  try {
    // Check if streaming compilation is supported (modern browsers)
    if (typeof WebAssembly.instantiateStreaming === 'function') {
      try {
        console.log('ÐYs? Loading WASM via streaming compilation...');
        // Load the JS glue code first
        const wasmModule: any = await import('/static/wasm/iron_insights_wasm.js');
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

        console.log('ƒo. WASM module loaded successfully via streaming');
        return true;
      } catch (streamError) {
        console.warn('ƒsÿ‹÷? Streaming WASM load failed, falling back to standard load:', streamError);
        // Fall through to standard loading
      }
    }

    // Fallback: Standard WASM loading (older browsers or if streaming fails)
    console.log('ÐY"Ý Loading WASM via standard method...');
    const wasmModule: any = await import('/static/wasm/iron_insights_wasm.js');
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

    console.log('ƒo. WASM module loaded successfully');
    return true;
  } catch (error) {
    console.error('ƒ?O Failed to load WASM module:', error);
    console.log('ÐY"< Falling back to JavaScript implementation');
    return false;
  }
}

// Load analytics dependencies using lazy loader
let Arrow: ArrowApi | null = null;
let Plotly: PlotlyApi | null = null;
async function loadAnalyticsDependencies(): Promise<boolean> {
  console.log('ÐY"" Loading analytics dependencies...');
  try {
    const deps: LazyLoaderDeps = await window.lazyLoader.loadAnalyticsDependencies();
    Arrow = deps.Arrow;
    Plotly = deps.Plotly;
    window.Arrow = Arrow;
    window.Plotly = Plotly;
    console.log('ƒo. Analytics dependencies loaded via lazy loader');
    return true;
  } catch (error) {
    console.error('ƒ?O Failed to load analytics dependencies:', error);
    return false;
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
window.calculate_dots_wasm = calculate_dots_wasm || undefined;
window.calculate_dots_with_gender_wasm = calculate_dots_with_gender_wasm || undefined;
window.calculate_strength_level_wasm = calculate_strength_level_wasm || undefined;
window.calculate_strength_level_for_lift_wasm = calculate_strength_level_for_lift_wasm || undefined;
window.calculate_strength_level_for_lift_with_gender_wasm = calculate_strength_level_for_lift_with_gender_wasm || undefined;
window.get_strength_level_color_wasm = get_strength_level_color_wasm || undefined;
window.calculate_dots_and_level_wasm = calculate_dots_and_level_wasm || undefined;
window.calculate_dots_and_level_for_lift_wasm = calculate_dots_and_level_for_lift_wasm || undefined;
window.calculate_dots_and_level_for_lift_with_gender_wasm = calculate_dots_and_level_for_lift_with_gender_wasm || undefined;
window.calculate_wilks_wasm = calculate_wilks_wasm || undefined;
window.calculate_ipf_gl_points_wasm = calculate_ipf_gl_points_wasm || undefined;
window.calculate_all_scores_wasm = calculate_all_scores_wasm || undefined;
window.calculate_strength_level_from_percentile_wasm = calculate_strength_level_from_percentile_wasm || undefined;
