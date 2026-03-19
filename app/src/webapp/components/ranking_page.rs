use super::{FaqPanel, MeetDayPanel, OnboardingPanel, RankingPageSections, ResultCardPanel};
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn RankingPage(page: RankingPageSections) -> impl IntoView {
    let RankingPageSections {
        header,
        onboarding,
        result,
        meet_day,
    } = page;
    view! {
        <header class="hero">
            <p class="hero-tag">"// percentile ranking"</p>
            <h1>
                "How "
                <span>"Strong"</span>
                <br />
                "Are You?"
            </h1>
            <p class="intro">
                "Enter your lifts and get a clear percentile result. No charts, just your ranking."
            </p>
            <div class="hero-badges">
                <p class="data-badge">
                    <strong>{move || header.dataset_blurb.get()}</strong>
                </p>
                <p class="data-badge">{move || header.ranking_cohort_blurb.get()}</p>
            </div>
        </header>

        <OnboardingPanel form=onboarding />
        <ResultCardPanel card=result />
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
