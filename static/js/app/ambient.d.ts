declare module 'plotly.js-dist-min';

declare module '/static/wasm/iron_insights_wasm.js' {
  import type { LiftType } from './types';

  export default function init(input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<void>;
  export const calculate_dots: (lift: number, bodyweight: number) => number;
  export const calculate_dots_with_gender: (lift: number, bodyweight: number, isMale: boolean) => number;
  export const calculate_strength_level: (dots: number, isMale: boolean) => string;
  export const calculate_strength_level_for_lift: (dots: number, lift: LiftType, isMale: boolean) => string;
  export const calculate_strength_level_for_lift_with_gender: (
    lift: number,
    bodyweight: number,
    isMale: boolean,
    liftType: LiftType
  ) => { dots: number; level: string; color: string };
  export const get_strength_level_color: (level: string) => string;
  export const calculate_dots_and_level: (lift: number, bodyweight: number, isMale: boolean) => { dots: number; level: string; color: string };
  export const calculate_dots_and_level_for_lift: (
    lift: number,
    bodyweight: number,
    isMale: boolean,
    liftType: LiftType
  ) => { dots: number; level: string; color: string };
  export const calculate_dots_and_level_for_lift_with_gender: (
    lift: number,
    bodyweight: number,
    isMale: boolean,
    liftType: LiftType
  ) => { dots: number; level: string; color: string };
  export const calculate_wilks: (lift: number, bodyweight: number, isMale: boolean) => number;
  export const calculate_ipf_gl_points: (lift: number, bodyweight: number, isMale: boolean) => number;
  export const calculate_all_scores: (lift: number, bodyweight: number, isMale: boolean) => {
    dots: number;
    wilks: number;
    ipf_gl_points: number;
  };
  export const calculate_strength_level_from_percentile: (percentile: number, isMale: boolean, lift: LiftType) => string;
}
