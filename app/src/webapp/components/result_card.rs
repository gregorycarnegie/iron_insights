use super::ResultCardSections;
use crate::webapp::share::{ShareImagePayload, download_share_png};
use crate::webapp::ui::lift_label;
use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn ResultCardPanel(card: ResultCardSections) -> impl IntoView {
    let ResultCardSections {
        status,
        share,
        lifts,
    } = card;
    view! {
        <section class="panel result-card">
            <h2>"Result"</h2>
            <Show
                when=move || status.calculated.get()
                fallback=move || view! { <p class="muted">"Press Calculate my ranking to load your headline result."</p> }
            >
                <p class="big">
                    {move || match status.percentile.get() {
                        Some((pct, _, _)) => format!("You are stronger than {:.1}% of lifters", pct * 100.0),
                        None => status.unavailable_reason
                            .get()
                            .unwrap_or_else(|| {
                                status.load_error
                                    .get()
                                    .map(|err| format!("Comparison data unavailable: {err}"))
                                    .unwrap_or_else(|| "No matching group found for these filters.".to_string())
                            }),
                    }}
                </p>
                <p class="topline">
                    {move || match status.percentile.get() {
                        Some((pct, _, total)) => format!("Top {:.1}% | Compared against {} lifters", (1.0 - pct).max(0.0) * 100.0, total),
                        None => "Top bracket unavailable.".to_string(),
                    }}
                </p>
                <p class="tier">
                    {move || match status.rank_tier.get() {
                        Some(tier) => format!("Strength level: {}", tier),
                        None => "Strength level: unavailable".to_string(),
                    }}
                </p>
                <p class="muted">"Higher is stronger."</p>
                <div class="share-row">
                    <button
                        type="button"
                        class="secondary"
                        on:click=move |_| share.set_show_share.update(|open| *open = !*open)
                    >
                        "Share my ranking"
                    </button>
                    <button
                        type="button"
                        class="secondary"
                        on:click=move |_| {
                            let Some(url) = share.share_url.get() else {
                                share.set_share_status.set(Some("Unable to generate share link.".to_string()));
                                return;
                            };
                            let Some(window) = web_sys::window() else {
                                share.set_share_status.set(Some("Clipboard unavailable.".to_string()));
                                return;
                            };
                            let clipboard = window.navigator().clipboard();
                            let _ = clipboard.write_text(&url);
                            share.set_share_status.set(Some("Link copied.".to_string()));
                        }
                    >
                        "Copy link"
                    </button>
                    <button
                        type="button"
                        class="secondary"
                        on:click=move |_| {
                            let Some(url) = share.share_url.get() else {
                                share.set_share_status.set(Some("Unable to generate challenge link.".to_string()));
                                return;
                            };
                            let challenge_url = if url.contains('?') {
                                format!("{url}&challenge=1")
                            } else {
                                format!("{url}?challenge=1")
                            };
                            let Some(window) = web_sys::window() else {
                                share.set_share_status.set(Some("Clipboard unavailable.".to_string()));
                                return;
                            };
                            let clipboard = window.navigator().clipboard();
                            let _ = clipboard.write_text(&challenge_url);
                            share.set_share_status.set(Some("Challenge link copied.".to_string()));
                        }
                    >
                        "Challenge a friend"
                    </button>
                </div>
                <Show when=move || share.show_share.get()>
                    <div class="share-card">
                        <label>
                            "Name / handle (optional)"
                            <input
                                type="text"
                                placeholder="@lifter"
                                prop:value=move || share.share_handle.get()
                                on:input=move |ev| share.set_share_handle.set(event_target_value(&ev))
                            />
                        </label>
                        <button
                            type="button"
                            class="secondary"
                            on:click=move |_| {
                                let Some((pct, _, _)) = status.percentile.get() else {
                                    share.set_share_status.set(Some("Calculate first to generate an image.".to_string()));
                                    return;
                                };
                                let tier = status.rank_tier.get().unwrap_or("Unknown");
                                let result = download_share_png(ShareImagePayload {
                                    handle: &share.share_handle.get(),
                                    bodyweight: lifts.bodyweight.get(),
                                    squat: lifts.squat.get(),
                                    bench: lifts.bench.get(),
                                    deadlift: lifts.deadlift.get(),
                                    lift_focus: lift_label(&lifts.lift.get()),
                                    percentile: pct,
                                    tier,
                                });
                                match result {
                                    Ok(()) => share.set_share_status.set(Some("PNG downloaded.".to_string())),
                                    Err(err) => share.set_share_status.set(Some(err)),
                                }
                            }
                        >
                            "Download PNG"
                        </button>
                    </div>
                </Show>
                <Show when=move || share.share_status.get().is_some()>
                    <p class="muted">{move || share.share_status.get().unwrap_or_default()}</p>
                </Show>
            </Show>
        </section>
    }
}
