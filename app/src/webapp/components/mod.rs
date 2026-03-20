macro_rules! metric_grid {
    ($( $label:literal => $value:expr ),+ $(,)?) => {
        view! {
            <div class="nerd-metrics-grid">
                $(
                    <p>
                        <strong>$label</strong>
                        {$value}
                    </p>
                )*
            </div>
        }
    };
}

use crate::core::{
    BodyweightConditionedStats, HeatmapBin, HistogramBin, HistogramDensity, HistogramDiagnostics,
};
use crate::webapp::models::{CohortComparisonRow, CompareMode, CrossSexComparison, TrendPoint};
use leptos::html::Canvas;
use leptos::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone)]
pub(super) struct RankingPageSections {
    pub(super) header: RankingHeroSection,
    pub(super) onboarding: OnboardingSections,
    pub(super) result: ResultCardSections,
    pub(super) meet_day: MeetDaySection,
}

#[derive(Clone)]
pub(super) struct RankingHeroSection {
    pub(super) dataset_blurb: Memo<String>,
    pub(super) ranking_cohort_blurb: Memo<String>,
}

#[derive(Clone)]
pub(super) struct MeetDaySection {
    pub(super) squat: ReadSignal<f32>,
    pub(super) bench: ReadSignal<f32>,
    pub(super) deadlift: ReadSignal<f32>,
    pub(super) use_lbs: ReadSignal<bool>,
    pub(super) unit_label: Memo<&'static str>,
}

#[derive(Clone)]
pub(super) struct OnboardingSections {
    pub(super) identity: OnboardingIdentitySection,
    pub(super) lifts: OnboardingLiftSection,
    pub(super) filters: OnboardingFilterSection,
    pub(super) actions: OnboardingActionSection,
}

#[derive(Clone)]
pub(super) struct OnboardingIdentitySection {
    pub(super) sex_options: Memo<Vec<String>>,
    pub(super) sex: ReadSignal<String>,
    pub(super) set_sex: WriteSignal<String>,
    pub(super) equip_options: Memo<Vec<String>>,
    pub(super) equip: ReadSignal<String>,
    pub(super) set_equip: WriteSignal<String>,
    pub(super) unit_label: Memo<&'static str>,
    pub(super) use_lbs: ReadSignal<bool>,
    pub(super) set_use_lbs: WriteSignal<bool>,
}

#[derive(Clone)]
pub(super) struct OnboardingLiftSection {
    pub(super) squat: ReadSignal<f32>,
    pub(super) set_squat: WriteSignal<f32>,
    pub(super) squat_error: ReadSignal<Option<String>>,
    pub(super) set_squat_error: WriteSignal<Option<String>>,
    pub(super) bench: ReadSignal<f32>,
    pub(super) set_bench: WriteSignal<f32>,
    pub(super) bench_error: ReadSignal<Option<String>>,
    pub(super) set_bench_error: WriteSignal<Option<String>>,
    pub(super) deadlift: ReadSignal<f32>,
    pub(super) set_deadlift: WriteSignal<f32>,
    pub(super) deadlift_error: ReadSignal<Option<String>>,
    pub(super) set_deadlift_error: WriteSignal<Option<String>>,
    pub(super) bodyweight: ReadSignal<f32>,
    pub(super) set_bodyweight: WriteSignal<f32>,
    pub(super) bodyweight_error: ReadSignal<Option<String>>,
    pub(super) set_bodyweight_error: WriteSignal<Option<String>>,
}

#[derive(Clone)]
pub(super) struct OnboardingFilterSection {
    pub(super) tested_options: Memo<Vec<String>>,
    pub(super) tested: ReadSignal<String>,
    pub(super) set_tested: WriteSignal<String>,
    pub(super) age_options: Memo<Vec<String>>,
    pub(super) age: ReadSignal<String>,
    pub(super) set_age: WriteSignal<String>,
    pub(super) wc_options: Memo<Vec<String>>,
    pub(super) wc: ReadSignal<String>,
    pub(super) set_wc: WriteSignal<String>,
    pub(super) lift_options: Memo<Vec<String>>,
    pub(super) lift: ReadSignal<String>,
    pub(super) set_lift: WriteSignal<String>,
    pub(super) metric_options: Memo<Vec<String>>,
    pub(super) metric: ReadSignal<String>,
    pub(super) set_metric: WriteSignal<String>,
}

#[derive(Clone)]
pub(super) struct OnboardingActionSection {
    pub(super) set_squat_delta: WriteSignal<f32>,
    pub(super) set_bench_delta: WriteSignal<f32>,
    pub(super) set_deadlift_delta: WriteSignal<f32>,
    pub(super) set_share_handle: WriteSignal<String>,
    pub(super) set_calculated: WriteSignal<bool>,
    pub(super) set_reveal_tick: WriteSignal<u64>,
    pub(super) has_input_error: Memo<bool>,
    pub(super) calculating: ReadSignal<bool>,
    pub(super) set_calculating: WriteSignal<bool>,
    pub(super) set_share_status: WriteSignal<Option<String>>,
    pub(super) set_lift_mult: WriteSignal<usize>,
    pub(super) set_bw_mult: WriteSignal<usize>,
}

#[derive(Clone)]
pub(super) struct ResultCardSections {
    pub(super) status: ResultCardStatusSection,
    pub(super) share: ResultCardShareSection,
    pub(super) lifts: ResultCardLiftSection,
}

#[derive(Clone)]
pub(super) struct ResultCardStatusSection {
    pub(super) calculated: ReadSignal<bool>,
    pub(super) percentile: Memo<Option<(f32, usize, u32)>>,
    pub(super) rank_tier: Memo<Option<&'static str>>,
    pub(super) reveal_tick: ReadSignal<u64>,
    pub(super) load_error: ReadSignal<Option<String>>,
    pub(super) unavailable_reason: Memo<Option<String>>,
}

#[derive(Clone)]
pub(super) struct ResultCardShareSection {
    pub(super) show_share: ReadSignal<bool>,
    pub(super) set_show_share: WriteSignal<bool>,
    pub(super) share_url: Memo<Option<String>>,
    pub(super) share_status: ReadSignal<Option<String>>,
    pub(super) set_share_status: WriteSignal<Option<String>>,
    pub(super) share_handle: ReadSignal<String>,
    pub(super) set_share_handle: WriteSignal<String>,
}

#[derive(Clone)]
pub(super) struct ResultCardLiftSection {
    pub(super) bodyweight: ReadSignal<f32>,
    pub(super) squat: ReadSignal<f32>,
    pub(super) bench: ReadSignal<f32>,
    pub(super) deadlift: ReadSignal<f32>,
    pub(super) lift: ReadSignal<String>,
    pub(super) use_lbs: ReadSignal<bool>,
    pub(super) unit_label: Memo<&'static str>,
}

#[derive(Clone)]
pub(super) struct ProgressSections {
    pub(super) result: ProgressResultSection,
    pub(super) selection: ProgressSelectionSection,
    pub(super) lifts: ProgressLiftSection,
    pub(super) display: ProgressDisplaySection,
}

#[derive(Clone)]
pub(super) struct ProgressResultSection {
    pub(super) calculated: ReadSignal<bool>,
    pub(super) percentile: Memo<Option<(f32, usize, u32)>>,
}

#[derive(Clone)]
pub(super) struct ProgressSelectionSection {
    pub(super) sex: ReadSignal<String>,
    pub(super) equip: ReadSignal<String>,
    pub(super) wc: ReadSignal<String>,
    pub(super) age: ReadSignal<String>,
    pub(super) tested: ReadSignal<String>,
    pub(super) lift: ReadSignal<String>,
    pub(super) metric: ReadSignal<String>,
}

#[derive(Clone)]
pub(super) struct ProgressLiftSection {
    pub(super) squat: ReadSignal<f32>,
    pub(super) bench: ReadSignal<f32>,
    pub(super) deadlift: ReadSignal<f32>,
    pub(super) bodyweight: ReadSignal<f32>,
}

#[derive(Clone)]
pub(super) struct ProgressDisplaySection {
    pub(super) use_lbs: ReadSignal<bool>,
    pub(super) unit_label: Memo<&'static str>,
}

#[derive(Clone)]
pub(super) struct PercentileLadderData {
    pub(super) calculated: ReadSignal<bool>,
    pub(super) metric_is_kg_comparable: Memo<bool>,
    pub(super) estimates: PercentileLadderEstimates,
}

#[derive(Clone)]
pub(super) struct PercentileLadderEstimates {
    pub(super) kg_for_next_1pct: Memo<Option<f32>>,
    pub(super) kg_for_next_5pct: Memo<Option<f32>>,
    pub(super) kg_for_next_10pct: Memo<Option<f32>>,
    pub(super) pct_gain_plus_2_5kg: Memo<Option<f32>>,
    pub(super) pct_gain_plus_5kg: Memo<Option<f32>>,
    pub(super) pct_gain_plus_10kg: Memo<Option<f32>>,
}

#[derive(Clone)]
pub(super) struct MenVsWomenPageSections {
    pub(super) dataset_blurb: Memo<String>,
    pub(super) selection_summary: Memo<String>,
    pub(super) onboarding: OnboardingSections,
    pub(super) controls: DistributionControlsSection,
    pub(super) comparison_loading: ReadSignal<bool>,
    pub(super) comparison: Memo<Result<CrossSexComparison, String>>,
    pub(super) heat_loading: ReadSignal<bool>,
    pub(super) heat_error: ReadSignal<Option<String>>,
    pub(super) male_hist: ReadSignal<Option<HistogramBin>>,
    pub(super) female_hist: ReadSignal<Option<HistogramBin>>,
    pub(super) male_heat: ReadSignal<Option<HeatmapBin>>,
    pub(super) female_heat: ReadSignal<Option<HeatmapBin>>,
    pub(super) hist_x_label: Memo<String>,
    pub(super) user_lift: Memo<f32>,
    pub(super) bodyweight: ReadSignal<f32>,
    pub(super) use_lbs: ReadSignal<bool>,
    pub(super) unit_label: Memo<&'static str>,
    pub(super) progress: ProgressSections,
}

#[derive(Clone)]
pub(super) struct NerdsPageSections {
    pub(super) header: NerdsHeaderSection,
    pub(super) onboarding: OnboardingSections,
    pub(super) cohort: NerdsCohortSection,
    pub(super) distributions: NerdsDistributionSection,
    pub(super) targets: NerdsTargetsSection,
    pub(super) trends: NerdsTrendsSection,
    pub(super) methodology: NerdsMethodologySection,
}

#[derive(Clone)]
pub(super) struct NerdsHeaderSection {
    pub(super) dataset_blurb: Memo<String>,
    pub(super) nerd_cohort_summary: Memo<String>,
}

#[derive(Clone)]
pub(super) struct NerdsCohortSection {
    pub(super) compare_mode: ReadSignal<CompareMode>,
    pub(super) set_compare_mode: WriteSignal<CompareMode>,
    pub(super) tested: ReadSignal<String>,
    pub(super) set_tested: WriteSignal<String>,
    pub(super) equip: ReadSignal<String>,
    pub(super) set_equip: WriteSignal<String>,
    pub(super) age: ReadSignal<String>,
    pub(super) set_age: WriteSignal<String>,
    pub(super) compare_summary: Memo<String>,
    pub(super) cohort_comparison_rows: Memo<Vec<CohortComparisonRow>>,
    pub(super) cohort_exact_deltas_enabled: ReadSignal<bool>,
    pub(super) set_cohort_exact_deltas_enabled: WriteSignal<bool>,
    pub(super) cohort_exact_percentiles: ReadSignal<BTreeMap<String, Option<f32>>>,
    pub(super) cohort_exact_loading: ReadSignal<bool>,
    pub(super) cohort_exact_error: ReadSignal<Option<String>>,
    pub(super) progress: ProgressSections,
}

#[derive(Clone)]
pub(super) struct NerdsDistributionSection {
    pub(super) controls: DistributionControlsSection,
    pub(super) diagnostics: DistributionDiagnosticsSection,
    pub(super) ladder: PercentileLadderData,
    pub(super) rarity_snapshot: Memo<Option<HistogramDensity>>,
    pub(super) bodyweight_conditioned: Memo<Option<BodyweightConditionedStats>>,
    pub(super) rebinned_hist: Memo<Option<HistogramBin>>,
    pub(super) user_lift: Memo<f32>,
    pub(super) canvas_ref: NodeRef<Canvas>,
    pub(super) show_heat_indicator: ReadSignal<bool>,
    pub(super) set_show_heat_indicator: WriteSignal<bool>,
    pub(super) calculated: ReadSignal<bool>,
    pub(super) use_lbs: ReadSignal<bool>,
    pub(super) unit_label: Memo<&'static str>,
}

#[derive(Clone)]
pub(super) struct DistributionControlsSection {
    pub(super) lift_mult: ReadSignal<usize>,
    pub(super) set_lift_mult: WriteSignal<usize>,
    pub(super) bw_mult: ReadSignal<usize>,
    pub(super) set_bw_mult: WriteSignal<usize>,
}

#[derive(Clone)]
pub(super) struct DistributionDiagnosticsSection {
    pub(super) distribution_diagnostics: Memo<Option<HistogramDiagnostics>>,
    pub(super) hist_x_label: Memo<String>,
}

#[derive(Clone)]
pub(super) struct NerdsTargetsSection {
    pub(super) squat_delta: ReadSignal<f32>,
    pub(super) set_squat_delta: WriteSignal<f32>,
    pub(super) bench_delta: ReadSignal<f32>,
    pub(super) set_bench_delta: WriteSignal<f32>,
    pub(super) deadlift_delta: ReadSignal<f32>,
    pub(super) set_deadlift_delta: WriteSignal<f32>,
    pub(super) projected_total: Memo<f32>,
    pub(super) projected_squat: Memo<f32>,
    pub(super) projected_bench: Memo<f32>,
    pub(super) projected_deadlift: Memo<f32>,
    pub(super) use_lbs: ReadSignal<bool>,
    pub(super) unit_label: Memo<&'static str>,
    pub(super) projected_percentile: Memo<Option<(f32, usize, u32)>>,
    pub(super) projected_rank_tier: Memo<Option<&'static str>>,
    pub(super) percentile_delta: Memo<Option<f32>>,
    pub(super) target_percentile: ReadSignal<f32>,
    pub(super) set_target_percentile: WriteSignal<f32>,
    pub(super) target_kg_needed: Memo<Option<f32>>,
    pub(super) target_summary: Memo<String>,
}

#[derive(Clone)]
pub(super) struct NerdsTrendsSection {
    pub(super) calculated: ReadSignal<bool>,
    pub(super) selected_trend_points: Memo<Vec<TrendPoint>>,
    pub(super) trend_note: Memo<String>,
    pub(super) user_lift: Memo<f32>,
    pub(super) hist_x_label: Memo<String>,
}

#[derive(Clone)]
pub(super) struct NerdsMethodologySection {
    pub(super) exact_slice_key: Memo<String>,
    pub(super) shard_key: Memo<String>,
    pub(super) dataset_version: Memo<String>,
    pub(super) dataset_revision: Memo<String>,
    pub(super) histogram_bin_width: Memo<Option<f32>>,
    pub(super) heatmap_dims: Memo<Option<(usize, usize, f32, f32)>>,
    pub(super) summary_stats: Memo<Option<(u32, f32, f32)>>,
    pub(super) load_timing_blurb: Memo<String>,
}

mod bodyweight_conditioned;
mod charts_panel;
mod cohort_comparison;
mod compare_mode;
mod cross_sex_charts;
mod cross_sex_panel;
mod distribution_controls;
mod distribution_diagnostics;
mod faq;
mod logo;
mod meet_day;
mod men_vs_women_page;
mod methodology_box;
mod nerds_page;
mod onboarding;
mod one_rep_max;
mod one_rm_page;
mod percentile;
mod percentile_ladder;
mod plate_calc;
mod plate_calc_page;
mod progress;
mod ranking_page;
mod rarity_panel;
mod result_card;
mod simulator;
mod trends;

pub(super) use bodyweight_conditioned::BodyweightConditionedPanel;
pub(super) use charts_panel::ChartsPanel;
pub(super) use cohort_comparison::CohortComparisonPanel;
pub(super) use compare_mode::CompareModePanel;
pub(super) use cross_sex_charts::CrossSexChartsPanel;
pub(super) use cross_sex_panel::CrossSexPanel;
pub(super) use distribution_controls::DistributionControlsPanel;
pub(super) use distribution_diagnostics::DistributionDiagnosticsPanel;
pub(super) use faq::FaqPanel;
pub(super) use logo::LogoMark;
pub(super) use meet_day::MeetDayPanel;
pub(super) use men_vs_women_page::MenVsWomenPage;
pub(super) use methodology_box::MethodologyBoxPanel;
pub(super) use nerds_page::NerdsPage;
pub(super) use onboarding::OnboardingPanel;
pub(super) use one_rep_max::OneRepMaxPanel;
pub(super) use one_rm_page::OneRmPage;
pub(super) use percentile_ladder::PercentileLadderPanel;
pub(super) use plate_calc::PlateCalcPanel;
pub(super) use plate_calc_page::PlateCalcPage;
pub(super) use progress::ProgressPanel;
pub(super) use ranking_page::RankingPage;
pub(super) use rarity_panel::RarityPanel;
pub(super) use result_card::ResultCardPanel;
pub(super) use simulator::SimulatorPanel;
pub(super) use trends::TrendsPanel;
