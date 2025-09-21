/* tslint:disable */
/* eslint-disable */
/**
 * Calculate DOTS score for a given lift and bodyweight (legacy function - defaults to male)
 */
export function calculate_dots(lift_kg: number, bodyweight_kg: number): number;
/**
 * Calculate DOTS score for a given lift, bodyweight, and gender
 */
export function calculate_dots_with_gender(lift_kg: number, bodyweight_kg: number, is_male: boolean): number;
/**
 * Calculate Wilks 2020 score for a given lift, bodyweight, and sex
 */
export function calculate_wilks(lift_kg: number, bodyweight_kg: number, is_male: boolean): number;
/**
 * Calculate IPF GL Points (Good-Lift Points) for a given lift, bodyweight, and sex
 */
export function calculate_ipf_gl_points(lift_kg: number, bodyweight_kg: number, is_male: boolean): number;
/**
 * Calculate strength level based on percentile
 */
export function calculate_strength_level_from_percentile(percentile: number): string;
/**
 * Legacy strength level calculation (for backward compatibility)
 */
export function calculate_strength_level(dots_score: number): string;
/**
 * Legacy lift-specific strength level calculations (for backward compatibility)
 */
export function calculate_strength_level_for_lift(dots_score: number, lift_type: string): string;
/**
 * Gender-aware strength level calculation with realistic thresholds
 */
export function calculate_strength_level_for_lift_with_gender(dots_score: number, lift_type: string, is_male: boolean): string;
/**
 * Get strength level color for UI styling
 */
export function get_strength_level_color(level: string): string;
/**
 * Combined function to calculate DOTS and strength level (defaults to total)
 */
export function calculate_dots_and_level(lift_kg: number, bodyweight_kg: number): any;
/**
 * Combined function to calculate DOTS and lift-specific strength level
 */
export function calculate_dots_and_level_for_lift(lift_kg: number, bodyweight_kg: number, lift_type: string): any;
/**
 * Combined function to calculate DOTS and lift-specific strength level with gender
 */
export function calculate_dots_and_level_for_lift_with_gender(lift_kg: number, bodyweight_kg: number, is_male: boolean, lift_type: string): any;
/**
 * Combined function to calculate all scoring systems at once
 */
export function calculate_all_scores(lift_kg: number, bodyweight_kg: number, is_male: boolean, percentile: number): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly calculate_dots: (a: number, b: number) => number;
  readonly calculate_dots_with_gender: (a: number, b: number, c: number) => number;
  readonly calculate_wilks: (a: number, b: number, c: number) => number;
  readonly calculate_ipf_gl_points: (a: number, b: number, c: number) => number;
  readonly calculate_strength_level_from_percentile: (a: number) => [number, number];
  readonly calculate_strength_level: (a: number) => [number, number];
  readonly calculate_strength_level_for_lift: (a: number, b: number, c: number) => [number, number];
  readonly calculate_strength_level_for_lift_with_gender: (a: number, b: number, c: number, d: number) => [number, number];
  readonly get_strength_level_color: (a: number, b: number) => [number, number];
  readonly calculate_dots_and_level: (a: number, b: number) => any;
  readonly calculate_dots_and_level_for_lift: (a: number, b: number, c: number, d: number) => any;
  readonly calculate_dots_and_level_for_lift_with_gender: (a: number, b: number, c: number, d: number, e: number) => any;
  readonly calculate_all_scores: (a: number, b: number, c: number, d: number) => any;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
