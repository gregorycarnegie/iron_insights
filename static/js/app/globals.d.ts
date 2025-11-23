import type {
  ArrowApi,
  ArrowResponse,
  EquipmentOption,
  FetchParams,
  LazyLoaderDeps,
  LiftType,
  PlotlyApi,
  SexValue
} from './types';

declare global {
  interface Window {
    Arrow: ArrowApi | null;
    Plotly: PlotlyApi | null;
    lazyLoader: { loadAnalyticsDependencies(): Promise<LazyLoaderDeps> };

    // UI + state
    currentSex: SexValue;
    currentLiftType: LiftType;
    currentEquipment: EquipmentOption[];
    currentTimePeriod: string;
    currentFederation: string;
    currentBinCount: number;
    currentWeightClass: string;
    lastResponse: ArrowResponse | null;
    debugMode: boolean;

    // WASM exports (populated at runtime)
    calculate_dots_wasm?: (lift: number, bw: number) => number;
    calculate_dots_with_gender_wasm?: (lift: number, bw: number, isMale: boolean) => number;
    calculate_strength_level_wasm?: (dots: number, isMale: boolean) => string;
    calculate_strength_level_for_lift_wasm?: (dots: number, lift: LiftType, isMale: boolean) => string;
    calculate_strength_level_for_lift_with_gender_wasm?: (lift: number, bw: number, isMale: boolean, liftType: LiftType) => {
      dots: number;
      level: string;
      color: string;
    };
    get_strength_level_color_wasm?: (level: string) => string;
    calculate_dots_and_level_wasm?: (lift: number, bw: number, isMale: boolean) => { dots: number; level: string; color: string };
    calculate_dots_and_level_for_lift_wasm?: (lift: number, bw: number, isMale: boolean, liftType: LiftType) => { dots: number; level: string; color: string };
    calculate_dots_and_level_for_lift_with_gender_wasm?: (
      lift: number,
      bw: number,
      isMale: boolean,
      liftType: LiftType
    ) => { dots: number; level: string; color: string };
    calculate_wilks_wasm?: (lift: number, bw: number, isMale: boolean) => number;
    calculate_ipf_gl_points_wasm?: (lift: number, bw: number, isMale: boolean) => number;
    calculate_all_scores_wasm?: (lift: number, bw: number, isMale: boolean) => { dots: number; wilks: number; ipf_gl_points: number };
    calculate_strength_level_from_percentile_wasm?: (percentile: number, isMale: boolean, lift: LiftType) => string;
    initWasm?: () => Promise<boolean>;
    loadAnalyticsDependencies?: () => Promise<boolean>;

    // App helpers
    fetchArrowData: (params: FetchParams) => Promise<ArrowResponse>;
    updateUserMetrics: (bodyweight: number | null, userLift: number | null) => void;
    calculateDOTS: (lift: number, bw: number, sex?: SexValue | null) => number;
    batchCalculateDOTS: (lifts: Array<{ lift: number; bw: number; sex: SexValue }>) => number[];
    getStrengthLevel: (dotsScore: number, isMale?: boolean) => string;
    getStrengthLevelForLift: (dotsScore: number, liftType: LiftType, isMale?: boolean) => string;
    calculateWilks: (liftKg: number, bodyweightKg: number, isMale: boolean) => number;
    calculateIPFGLPoints: (liftKg: number, bodyweightKg: number, isMale: boolean) => number;
    getStrengthLevelColor: (level: string) => string;
    updateServerMetrics?: (data: unknown) => void;
    initWebSocket: () => void;
    sendUserUpdate: (bodyweight: number | null, squat: number | null, bench: number | null, deadlift: number | null, liftType: string) => void;
    changeBins: (element: HTMLElement, chartId: string) => void;
    switchRankings: (element: HTMLElement) => void;
    toggleTrendline: (chartId: string) => void;
    togglePoints: (chartId: string) => void;
    exportAllCharts: (format?: 'png' | 'svg' | 'jpeg') => Promise<void>;
    exportDataAsCSV: () => void;
    toggleExportDropdown: (button: HTMLElement) => void;
    setupChartCrossfiltering: () => void;
    createPlotEnhanced: (chartId: string, traces: any[], layout: Record<string, unknown>, errorMessage?: string) => boolean;
    showError: (chartId: string, message: string) => void;
    hideError: (chartId: string) => void;
    hideChartSkeleton: (chartId: string) => void;
    setToggle: (element: HTMLElement, type: 'sex' | 'lift') => boolean;
    updateEquipment: () => void;
    updateAnalytics: () => void;
    toggleSidebar: () => void;
    toggleDebug: () => void;
    setupEquipmentFilters: () => void;
    setupWeightClassFilter: () => void;
    setupTimePeriodFilter: () => void;
    setupFederationFilter: () => void;
    setupInputDebugger: () => void;
    showDebugInfo: (data: ArrowResponse) => void;

    parseSum: (input: string | number) => number;
    getTimeAgo: (timestamp: number) => string;

    // App entrypoints
    updateCharts: () => Promise<void>;
    init: () => Promise<void>;
  }
}

export {};
