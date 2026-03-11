use super::{
    BodyweightConditionedPanel, ChartsPanel, CohortComparisonPanel, CompareModePanel,
    CrossSexPanel, DistributionControlsPanel, DistributionDiagnosticsPanel, MethodologyBoxPanel,
    PercentileLadderPanel, ProgressPanel, RarityPanel, SimulatorPanel, TrendsPanel,
};
use crate::core::{
    BodyweightConditionedStats, HistogramBin, HistogramDensity, HistogramDiagnostics,
};
use crate::webapp::models::{CohortComparisonRow, CompareMode, CrossSexComparison, TrendPoint};
use leptos::html::Canvas;
use leptos::prelude::*;
use std::collections::BTreeMap;

#[allow(clippy::too_many_arguments)]
#[component]
pub(in crate::webapp) fn NerdsPage(
    dataset_blurb: Memo<String>,
    nerd_cohort_summary: Memo<String>,
    compare_mode: ReadSignal<CompareMode>,
    set_compare_mode: WriteSignal<CompareMode>,
    tested: ReadSignal<String>,
    set_tested: WriteSignal<String>,
    equip: ReadSignal<String>,
    set_equip: WriteSignal<String>,
    age: ReadSignal<String>,
    set_age: WriteSignal<String>,
    compare_summary: Memo<String>,
    cohort_comparison_rows: Memo<Vec<CohortComparisonRow>>,
    cohort_exact_deltas_enabled: ReadSignal<bool>,
    set_cohort_exact_deltas_enabled: WriteSignal<bool>,
    cohort_exact_percentiles: ReadSignal<BTreeMap<String, Option<f32>>>,
    cohort_exact_loading: ReadSignal<bool>,
    cohort_exact_error: ReadSignal<Option<String>>,
    cross_sex_hist_loading: ReadSignal<bool>,
    cross_sex_comparison: Memo<Result<CrossSexComparison, String>>,
    use_lbs: ReadSignal<bool>,
    unit_label: Memo<&'static str>,
    calculated: ReadSignal<bool>,
    percentile: Memo<Option<(f32, usize, u32)>>,
    sex: ReadSignal<String>,
    wc: ReadSignal<String>,
    lift: ReadSignal<String>,
    metric: ReadSignal<String>,
    squat: ReadSignal<f32>,
    bench: ReadSignal<f32>,
    deadlift: ReadSignal<f32>,
    bodyweight: ReadSignal<f32>,
    lift_mult: ReadSignal<usize>,
    set_lift_mult: WriteSignal<usize>,
    bw_mult: ReadSignal<usize>,
    set_bw_mult: WriteSignal<usize>,
    distribution_diagnostics: Memo<Option<HistogramDiagnostics>>,
    hist_x_label: Memo<String>,
    metric_is_kg_comparable: Memo<bool>,
    kg_for_next_1pct: Memo<Option<f32>>,
    kg_for_next_5pct: Memo<Option<f32>>,
    kg_for_next_10pct: Memo<Option<f32>>,
    pct_gain_plus_2_5kg: Memo<Option<f32>>,
    pct_gain_plus_5kg: Memo<Option<f32>>,
    pct_gain_plus_10kg: Memo<Option<f32>>,
    rarity_snapshot: Memo<Option<HistogramDensity>>,
    bodyweight_conditioned: Memo<Option<BodyweightConditionedStats>>,
    rebinned_hist: Memo<Option<HistogramBin>>,
    user_lift: Memo<f32>,
    canvas_ref: NodeRef<Canvas>,
    squat_delta: ReadSignal<f32>,
    set_squat_delta: WriteSignal<f32>,
    bench_delta: ReadSignal<f32>,
    set_bench_delta: WriteSignal<f32>,
    deadlift_delta: ReadSignal<f32>,
    set_deadlift_delta: WriteSignal<f32>,
    projected_total: Memo<f32>,
    projected_squat: Memo<f32>,
    projected_bench: Memo<f32>,
    projected_deadlift: Memo<f32>,
    projected_percentile: Memo<Option<(f32, usize, u32)>>,
    projected_rank_tier: Memo<Option<&'static str>>,
    percentile_delta: Memo<Option<f32>>,
    target_percentile: ReadSignal<f32>,
    set_target_percentile: WriteSignal<f32>,
    target_kg_needed: Memo<Option<f32>>,
    target_summary: Memo<String>,
    selected_trend_points: Memo<Vec<TrendPoint>>,
    trend_note: Memo<String>,
    exact_slice_key: Memo<String>,
    shard_key: Memo<String>,
    dataset_version: Memo<String>,
    dataset_revision: Memo<String>,
    histogram_bin_width: Memo<Option<f32>>,
    heatmap_dims: Memo<Option<(usize, usize, f32, f32)>>,
    summary_stats: Memo<Option<(u32, f32, f32)>>,
    load_timing_blurb: Memo<String>,
) -> impl IntoView {
    view! {
        <header class="hero">
            <h1>"Stats for Nerds"</h1>
            <p>{move || dataset_blurb.get()}</p>
            <p class="intro">
                "Advanced distributions, cohort comparisons, and methodology details for people who want the full story."
            </p>
            <p class="muted">{move || nerd_cohort_summary.get()}</p>
        </header>

        <section class="nerd-section">
            <h2>"Cohort"</h2>
            <CompareModePanel
                compare_mode=compare_mode
                set_compare_mode=set_compare_mode
                tested=tested
                set_tested=set_tested
                equip=equip
                set_equip=set_equip
                age=age
                set_age=set_age
                compare_summary=compare_summary
            />
            <CohortComparisonPanel
                rows=cohort_comparison_rows
                exact_deltas_enabled=cohort_exact_deltas_enabled
                set_exact_deltas_enabled=set_cohort_exact_deltas_enabled
                exact_percentiles=cohort_exact_percentiles
                exact_loading=cohort_exact_loading
                exact_error=cohort_exact_error
            />
            <CrossSexPanel
                loading=cross_sex_hist_loading
                comparison=cross_sex_comparison
                use_lbs=use_lbs
                unit_label=unit_label
            />
            <ProgressPanel
                calculated=calculated
                percentile=percentile
                sex=sex
                equip=equip
                wc=wc
                age=age
                tested=tested
                lift=lift
                metric=metric
                squat=squat
                bench=bench
                deadlift=deadlift
                bodyweight=bodyweight
                use_lbs=use_lbs
                unit_label=unit_label
            />
        </section>

        <section class="nerd-section">
            <h2>"Distributions"</h2>
            <DistributionControlsPanel
                lift_mult=lift_mult
                set_lift_mult=set_lift_mult
                bw_mult=bw_mult
                set_bw_mult=set_bw_mult
            />
            <DistributionDiagnosticsPanel
                diagnostics=distribution_diagnostics
                hist_x_label=hist_x_label
            />
            <PercentileLadderPanel
                calculated=calculated
                metric_is_kg_comparable=metric_is_kg_comparable
                kg_for_next_1pct=kg_for_next_1pct
                kg_for_next_5pct=kg_for_next_5pct
                kg_for_next_10pct=kg_for_next_10pct
                pct_gain_plus_2_5kg=pct_gain_plus_2_5kg
                pct_gain_plus_5kg=pct_gain_plus_5kg
                pct_gain_plus_10kg=pct_gain_plus_10kg
            />
            <RarityPanel density=rarity_snapshot hist_x_label=hist_x_label />
            <BodyweightConditionedPanel
                calculated=calculated
                conditioned=bodyweight_conditioned
                use_lbs=use_lbs
                unit_label=unit_label
            />
            <ChartsPanel
                rebinned_hist=rebinned_hist
                user_lift=user_lift
                hist_x_label=hist_x_label
                canvas_ref=canvas_ref
            />
        </section>

        <section class="nerd-section">
            <h2>"Targets and Simulations"</h2>
            <SimulatorPanel
                squat_delta=squat_delta
                set_squat_delta=set_squat_delta
                bench_delta=bench_delta
                set_bench_delta=set_bench_delta
                deadlift_delta=deadlift_delta
                set_deadlift_delta=set_deadlift_delta
                projected_total=projected_total
                projected_squat=projected_squat
                projected_bench=projected_bench
                projected_deadlift=projected_deadlift
                use_lbs=use_lbs
                unit_label=unit_label
                projected_percentile=projected_percentile
                projected_rank_tier=projected_rank_tier
                percentile_delta=percentile_delta
                target_percentile=target_percentile
                set_target_percentile=set_target_percentile
                target_kg_needed=target_kg_needed
                target_summary=target_summary
            />
        </section>

        <section class="nerd-section">
            <h2>"Trends"</h2>
            <TrendsPanel
                calculated=calculated
                trend_points=selected_trend_points
                trend_note=trend_note
                current_value=user_lift
            />
        </section>

        <section class="nerd-section">
            <h2>"Methodology and Debug"</h2>
            <MethodologyBoxPanel
                exact_slice_key=exact_slice_key
                shard_key=shard_key
                dataset_version=dataset_version
                dataset_revision=dataset_revision
                histogram_bin_width=histogram_bin_width
                heatmap_dims=heatmap_dims
                summary_stats=summary_stats
                load_timing_blurb=load_timing_blurb
            />
        </section>
    }
}
