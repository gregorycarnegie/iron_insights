export type SexValue = 'M' | 'F' | 'Male' | 'Female' | 'All';
export type LiftType = 'squat' | 'bench' | 'deadlift' | 'total';
export type EquipmentOption = 'Raw' | 'Wraps' | 'Single-ply' | 'Multi-ply';

export interface HistogramData {
  values: number[];
  counts: number[];
  bins: number[];
  min_val: number;
  max_val: number;
}

export interface ScatterData {
  x: number[];
  y: number[];
  sex: SexValue[];
}

export interface ArrowResponse {
  histogram_data: HistogramData;
  dots_histogram_data: HistogramData;
  scatter_data: ScatterData;
  dots_scatter_data: ScatterData;
  user_percentile: number | null;
  user_dots_percentile: number | null;
  avg_dots: number | null;
  processing_time_ms: number;
  total_records: number;
  is_cached: boolean;
}

export interface ArrowTableColumn<T = number> {
  toArray(): T[];
}

export interface ArrowTable {
  length: number;
  getChildAt<T = number>(index: number): ArrowTableColumn<T> | null;
}

export interface ArrowApi {
  tableFromIPC(data: Iterable<number> | ArrayBuffer | ArrayBufferView): ArrowTable;
}

export type PlotlyTrace = Record<string, unknown> & {
  x?: number[];
  y?: number[];
  values?: number[];
  marker?: Record<string, unknown>;
  type?: string;
};

export interface PlotlyApi {
  react(
    root: string | HTMLElement,
    traces: PlotlyTrace[],
    layout?: Record<string, unknown>,
    config?: Record<string, unknown>
  ): Promise<void> | void;
  relayout(root: string | HTMLElement, updates: Record<string, unknown>): Promise<void> | void;
  restyle(root: string | HTMLElement, updates: Record<string, unknown>): Promise<void> | void;
  toImage(root: string | HTMLElement, opts: Record<string, unknown>): Promise<string>;
}

export interface LazyLoaderDeps {
  Arrow: ArrowApi;
  Plotly: PlotlyApi;
}

export interface FetchParams {
  sex: SexValue;
  lift_type: LiftType | 'total';
  bodyweight: number | null;
  squat: number | null;
  bench: number | null;
  deadlift: number | null;
  equipment: EquipmentOption[];
  years_filter: string;
  federation: string;
  weight_class: string | null;
}
