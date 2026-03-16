use super::{
    BodyweightConditionedPanel, ChartsPanel, CohortComparisonPanel, CompareModePanel,
    DistributionControlsPanel, DistributionDiagnosticsPanel, MethodologyBoxPanel,
    NerdsPageSections, OnboardingPanel, PercentileLadderPanel, ProgressPanel, RarityPanel,
    SimulatorPanel, TrendsPanel,
};
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn NerdsPage(page: NerdsPageSections) -> impl IntoView {
    let NerdsPageSections {
        header,
        onboarding,
        cohort,
        distributions,
        targets,
        trends,
        methodology,
    } = page;
    view! {
        <header class="hero">
            <h1>"Stats for Nerds"</h1>
            <p>{move || header.dataset_blurb.get()}</p>
            <p class="intro">
                "Advanced distributions, cohort comparisons, and methodology details for people who want the full story."
            </p>
            <p class="muted">{move || header.nerd_cohort_summary.get()}</p>
        </header>

        <OnboardingPanel form=onboarding />

        <section class="nerd-section">
            <h2>"Cohort"</h2>
            <CompareModePanel
                compare_mode=cohort.compare_mode
                set_compare_mode=cohort.set_compare_mode
                tested=cohort.tested
                set_tested=cohort.set_tested
                equip=cohort.equip
                set_equip=cohort.set_equip
                age=cohort.age
                set_age=cohort.set_age
                compare_summary=cohort.compare_summary
            />
            <CohortComparisonPanel
                rows=cohort.cohort_comparison_rows
                exact_deltas_enabled=cohort.cohort_exact_deltas_enabled
                set_exact_deltas_enabled=cohort.set_cohort_exact_deltas_enabled
                exact_percentiles=cohort.cohort_exact_percentiles
                exact_loading=cohort.cohort_exact_loading
                exact_error=cohort.cohort_exact_error
            />
            <ProgressPanel tracking=cohort.progress />
        </section>

        <section class="nerd-section">
            <h2>"Distributions"</h2>
            <DistributionControlsPanel
                lift_mult=distributions.controls.lift_mult
                set_lift_mult=distributions.controls.set_lift_mult
                bw_mult=distributions.controls.bw_mult
                set_bw_mult=distributions.controls.set_bw_mult
            />
            <DistributionDiagnosticsPanel
                diagnostics=distributions.diagnostics.distribution_diagnostics
                hist_x_label=distributions.diagnostics.hist_x_label
            />
            <PercentileLadderPanel ladder=distributions.ladder />
            <RarityPanel
                density=distributions.rarity_snapshot
                hist_x_label=distributions.diagnostics.hist_x_label
            />
            <BodyweightConditionedPanel
                calculated=distributions.calculated
                conditioned=distributions.bodyweight_conditioned
                use_lbs=distributions.use_lbs
                unit_label=distributions.unit_label
            />
            <ChartsPanel
                rebinned_hist=distributions.rebinned_hist
                user_lift=distributions.user_lift
                hist_x_label=distributions.diagnostics.hist_x_label
                canvas_ref=distributions.canvas_ref
                show_heat_indicator=distributions.show_heat_indicator
                set_show_heat_indicator=distributions.set_show_heat_indicator
            />
        </section>

        <section class="nerd-section">
            <h2>"Targets and Simulations"</h2>
            <SimulatorPanel
                squat_delta=targets.squat_delta
                set_squat_delta=targets.set_squat_delta
                bench_delta=targets.bench_delta
                set_bench_delta=targets.set_bench_delta
                deadlift_delta=targets.deadlift_delta
                set_deadlift_delta=targets.set_deadlift_delta
                projected_total=targets.projected_total
                projected_squat=targets.projected_squat
                projected_bench=targets.projected_bench
                projected_deadlift=targets.projected_deadlift
                use_lbs=targets.use_lbs
                unit_label=targets.unit_label
                projected_percentile=targets.projected_percentile
                projected_rank_tier=targets.projected_rank_tier
                percentile_delta=targets.percentile_delta
                target_percentile=targets.target_percentile
                set_target_percentile=targets.set_target_percentile
                target_kg_needed=targets.target_kg_needed
                target_summary=targets.target_summary
            />
        </section>

        <section class="nerd-section">
            <h2>"Trends"</h2>
            <TrendsPanel
                calculated=trends.calculated
                trend_points=trends.selected_trend_points
                trend_note=trends.trend_note
                current_value=trends.user_lift
                threshold_axis_label=trends.hist_x_label
            />
        </section>

        <section class="nerd-section">
            <h2>"Methodology and Debug"</h2>
            <MethodologyBoxPanel
                exact_slice_key=methodology.exact_slice_key
                shard_key=methodology.shard_key
                dataset_version=methodology.dataset_version
                dataset_revision=methodology.dataset_revision
                histogram_bin_width=methodology.histogram_bin_width
                heatmap_dims=methodology.heatmap_dims
                summary_stats=methodology.summary_stats
                load_timing_blurb=methodology.load_timing_blurb
            />
        </section>
    }
}
