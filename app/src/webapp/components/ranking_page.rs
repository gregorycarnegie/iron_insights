use super::{FaqPanel, MeetDayPanel, OnboardingPanel, PercentilePanel, ResultCardPanel};
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
#[component]
pub(in crate::webapp) fn RankingPage(
    dataset_blurb: Memo<String>,
    ranking_cohort_blurb: Memo<String>,
    sex_options: Memo<Vec<String>>,
    sex: ReadSignal<String>,
    set_sex: WriteSignal<String>,
    equip_options: Memo<Vec<String>>,
    equip: ReadSignal<String>,
    set_equip: WriteSignal<String>,
    unit_label: Memo<&'static str>,
    use_lbs: ReadSignal<bool>,
    set_use_lbs: WriteSignal<bool>,
    squat: ReadSignal<f32>,
    set_squat: WriteSignal<f32>,
    squat_error: ReadSignal<Option<String>>,
    set_squat_error: WriteSignal<Option<String>>,
    bench: ReadSignal<f32>,
    set_bench: WriteSignal<f32>,
    bench_error: ReadSignal<Option<String>>,
    set_bench_error: WriteSignal<Option<String>>,
    deadlift: ReadSignal<f32>,
    set_deadlift: WriteSignal<f32>,
    deadlift_error: ReadSignal<Option<String>>,
    set_deadlift_error: WriteSignal<Option<String>>,
    bodyweight: ReadSignal<f32>,
    set_bodyweight: WriteSignal<f32>,
    bodyweight_error: ReadSignal<Option<String>>,
    set_bodyweight_error: WriteSignal<Option<String>>,
    set_squat_delta: WriteSignal<f32>,
    set_bench_delta: WriteSignal<f32>,
    set_deadlift_delta: WriteSignal<f32>,
    set_share_handle: WriteSignal<String>,
    set_calculated: WriteSignal<bool>,
    has_input_error: Memo<bool>,
    calculating: ReadSignal<bool>,
    set_calculating: WriteSignal<bool>,
    set_share_status: WriteSignal<Option<String>>,
    tested_options: Memo<Vec<String>>,
    tested: ReadSignal<String>,
    set_tested: WriteSignal<String>,
    age_options: Memo<Vec<String>>,
    age: ReadSignal<String>,
    set_age: WriteSignal<String>,
    wc_options: Memo<Vec<String>>,
    wc: ReadSignal<String>,
    set_wc: WriteSignal<String>,
    lift_options: Memo<Vec<String>>,
    lift: ReadSignal<String>,
    set_lift: WriteSignal<String>,
    metric_options: Memo<Vec<String>>,
    metric: ReadSignal<String>,
    set_metric: WriteSignal<String>,
    set_lift_mult: WriteSignal<usize>,
    set_bw_mult: WriteSignal<usize>,
    calculated: ReadSignal<bool>,
    percentile: Memo<Option<(f32, usize, u32)>>,
    rank_tier: Memo<Option<&'static str>>,
    load_error: ReadSignal<Option<String>>,
    result_unavailable_reason: Memo<Option<String>>,
    show_share: ReadSignal<bool>,
    set_show_share: WriteSignal<bool>,
    share_url: Memo<Option<String>>,
    share_status: ReadSignal<Option<String>>,
    share_handle: ReadSignal<String>,
    percentile_percent: Memo<Option<f32>>,
) -> impl IntoView {
    view! {
        <header class="hero">
            <h1>"How Strong Are You?"</h1>
            <p>{move || dataset_blurb.get()}</p>
            <p class="intro">
                "Enter your lifts and get a clear percentile result. No charts, just your ranking."
            </p>
            <p class="muted">{move || ranking_cohort_blurb.get()}</p>
        </header>

        <OnboardingPanel
            sex_options=sex_options
            sex=sex
            set_sex=set_sex
            equip_options=equip_options
            equip=equip
            set_equip=set_equip
            unit_label=unit_label
            use_lbs=use_lbs
            set_use_lbs=set_use_lbs
            squat=squat
            set_squat=set_squat
            squat_error=squat_error
            set_squat_error=set_squat_error
            bench=bench
            set_bench=set_bench
            bench_error=bench_error
            set_bench_error=set_bench_error
            deadlift=deadlift
            set_deadlift=set_deadlift
            deadlift_error=deadlift_error
            set_deadlift_error=set_deadlift_error
            bodyweight=bodyweight
            set_bodyweight=set_bodyweight
            bodyweight_error=bodyweight_error
            set_bodyweight_error=set_bodyweight_error
            set_squat_delta=set_squat_delta
            set_bench_delta=set_bench_delta
            set_deadlift_delta=set_deadlift_delta
            set_share_handle=set_share_handle
            set_calculated=set_calculated
            has_input_error=has_input_error
            calculating=calculating
            set_calculating=set_calculating
            set_share_status=set_share_status
            tested_options=tested_options
            tested=tested
            set_tested=set_tested
            age_options=age_options
            age=age
            set_age=set_age
            wc_options=wc_options
            wc=wc
            set_wc=set_wc
            lift_options=lift_options
            lift=lift
            set_lift=set_lift
            metric_options=metric_options
            metric=metric
            set_metric=set_metric
            set_lift_mult=set_lift_mult
            set_bw_mult=set_bw_mult
        />
        <ResultCardPanel
            calculated=calculated
            percentile=percentile
            rank_tier=rank_tier
            load_error=load_error
            unavailable_reason=result_unavailable_reason
            show_share=show_share
            set_show_share=set_show_share
            share_url=share_url
            share_status=share_status
            set_share_status=set_share_status
            share_handle=share_handle
            set_share_handle=set_share_handle
            bodyweight=bodyweight
            squat=squat
            bench=bench
            deadlift=deadlift
            lift=lift
        />
        <PercentilePanel
            percentile_percent=percentile_percent
        />
        <MeetDayPanel
            squat=squat
            bench=bench
            deadlift=deadlift
            use_lbs=use_lbs
            unit_label=unit_label
        />
        <FaqPanel />
    }
}
