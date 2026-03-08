use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn FaqPanel() -> impl IntoView {
    view! {
        <section class="panel faq">
            <h2>"FAQ"</h2>
            <details>
                <summary>"What does percentile mean?"</summary>
                <p>
                    "Percentile shows the share of comparable lifters you outperform. Higher percentile means stronger relative ranking."
                </p>
            </details>
            <details>
                <summary>"Where does the data come from?"</summary>
                <p>
                    "Data is loaded from the bundled competition dataset version shown at the top of the page."
                </p>
            </details>
            <details>
                <summary>"Why does equipment matter?"</summary>
                <p>
                    "Different equipment changes performance. Filtering by equipment gives fairer comparisons."
                </p>
            </details>
            <p class="muted">
                <a href="./landing/faq.html">"Read full FAQ"</a>
                " | "
                <a href="./landing/methodology.html">"Methodology notes"</a>
            </p>
        </section>
    }
}
