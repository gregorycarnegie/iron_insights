use super::{
    CrossSexChartsPanel, CrossSexPanel, DistributionControlsPanel, MenVsWomenPageSections,
    OnboardingPanel, ProgressPanel,
};
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn MenVsWomenPage(page: MenVsWomenPageSections) -> impl IntoView {
    let MenVsWomenPageSections {
        dataset_blurb,
        selection_summary,
        onboarding,
        controls,
        comparison_loading,
        comparison,
        heat_loading,
        heat_error,
        male_hist,
        female_hist,
        male_heat,
        female_heat,
        hist_x_label,
        user_lift,
        bodyweight,
        use_lbs,
        unit_label,
        progress,
    } = page;

    view! {
        <header class="hero">
            <p class="hero-tag">"// cross-sex comparison"</p>
            <h1>
                "Men vs"
                <br />
                <span>"Women"</span>
            </h1>
            <p class="intro">
                "Compare the same input against aligned male and female cohorts, then inspect how both distributions overlap."
            </p>
            <div class="hero-badges">
                <p class="data-badge">
                    <strong>{move || dataset_blurb.get()}</strong>
                </p>
                <p class="data-badge">{move || selection_summary.get()}</p>
            </div>
        </header>

        <OnboardingPanel form=onboarding />

        <section class="nerd-section">
            <CrossSexPanel
                loading=comparison_loading
                comparison=comparison
                use_lbs=use_lbs
                unit_label=unit_label
            />
            <DistributionControlsPanel
                lift_mult=controls.lift_mult
                set_lift_mult=controls.set_lift_mult
                bw_mult=controls.bw_mult
                set_bw_mult=controls.set_bw_mult
            />
            <CrossSexChartsPanel
                hist_loading=comparison_loading
                comparison=comparison
                male_hist=male_hist
                female_hist=female_hist
                heat_loading=heat_loading
                heat_error=heat_error
                male_heat=male_heat
                female_heat=female_heat
                hist_x_label=hist_x_label
                user_lift=user_lift
                bodyweight=bodyweight
                lift_mult=controls.lift_mult
                bw_mult=controls.bw_mult
            />
            <ProgressPanel tracking=progress />
        </section>
    }
}
