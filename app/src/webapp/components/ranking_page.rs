use super::{
    FaqPanel, MeetDayPanel, OnboardingPanel, PercentilePanel, RankingPageSections, ResultCardPanel,
};
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn RankingPage(page: RankingPageSections) -> impl IntoView {
    let RankingPageSections {
        header,
        onboarding,
        result,
        meet_day,
        percentile_percent,
    } = page;
    view! {
        <header class="hero">
            <h1>"How Strong Are You?"</h1>
            <p>{move || header.dataset_blurb.get()}</p>
            <p class="intro">
                "Enter your lifts and get a clear percentile result. No charts, just your ranking."
            </p>
            <p class="muted">{move || header.ranking_cohort_blurb.get()}</p>
        </header>

        <OnboardingPanel form=onboarding />
        <ResultCardPanel card=result />
        <PercentilePanel percentile_percent=percentile_percent />
        <MeetDayPanel
            squat=meet_day.squat
            bench=meet_day.bench
            deadlift=meet_day.deadlift
            use_lbs=meet_day.use_lbs
            unit_label=meet_day.unit_label
        />
        <FaqPanel />
    }
}
