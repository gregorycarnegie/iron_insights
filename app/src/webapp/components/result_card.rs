use crate::webapp::share::download_share_png;
use crate::webapp::ui::lift_label;
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
#[component]
pub(in crate::webapp) fn ResultCardPanel(
    calculated: ReadSignal<bool>,
    percentile: Memo<Option<(f32, usize, u32)>>,
    rank_tier: Memo<Option<&'static str>>,
    load_error: ReadSignal<Option<String>>,
    unavailable_reason: Memo<Option<String>>,
    show_share: ReadSignal<bool>,
    set_show_share: WriteSignal<bool>,
    share_url: Memo<Option<String>>,
    share_status: ReadSignal<Option<String>>,
    set_share_status: WriteSignal<Option<String>>,
    share_handle: ReadSignal<String>,
    set_share_handle: WriteSignal<String>,
    bodyweight: ReadSignal<f32>,
    squat: ReadSignal<f32>,
    bench: ReadSignal<f32>,
    deadlift: ReadSignal<f32>,
    lift: ReadSignal<String>,
) -> impl IntoView {
    view! {
        <section class="panel result-card">
            <h2>"Result"</h2>
            <Show
                when=move || calculated.get()
                fallback=move || view! { <p class="muted">"Press Calculate my ranking to load your headline result."</p> }
            >
                <p class="big">
                    {move || match percentile.get() {
                        Some((pct, _, _)) => format!("You are stronger than {:.1}% of lifters", pct * 100.0),
                        None => unavailable_reason
                            .get()
                            .unwrap_or_else(|| {
                                load_error
                                    .get()
                                    .map(|err| format!("Comparison data unavailable: {err}"))
                                    .unwrap_or_else(|| "No matching group found for these filters.".to_string())
                            }),
                    }}
                </p>
                <p class="topline">
                    {move || match percentile.get() {
                        Some((pct, _, total)) => format!("Top {:.1}% | Compared against {} lifters", (1.0 - pct).max(0.0) * 100.0, total),
                        None => "Top bracket unavailable.".to_string(),
                    }}
                </p>
                <p class="tier">
                    {move || match rank_tier.get() {
                        Some(tier) => format!("Strength level: {}", tier),
                        None => "Strength level: unavailable".to_string(),
                    }}
                </p>
                <p class="muted">"Higher is stronger."</p>
                <div class="share-row">
                    <button
                        type="button"
                        class="secondary"
                        on:click=move |_| set_show_share.update(|open| *open = !*open)
                    >
                        "Share my ranking"
                    </button>
                    <button
                        type="button"
                        class="secondary"
                        on:click=move |_| {
                            let Some(url) = share_url.get() else {
                                set_share_status.set(Some("Unable to generate share link.".to_string()));
                                return;
                            };
                            let Some(window) = web_sys::window() else {
                                set_share_status.set(Some("Clipboard unavailable.".to_string()));
                                return;
                            };
                            let clipboard = window.navigator().clipboard();
                            let _ = clipboard.write_text(&url);
                            set_share_status.set(Some("Link copied.".to_string()));
                        }
                    >
                        "Copy link"
                    </button>
                    <button
                        type="button"
                        class="secondary"
                        on:click=move |_| {
                            let Some(url) = share_url.get() else {
                                set_share_status.set(Some("Unable to generate challenge link.".to_string()));
                                return;
                            };
                            let challenge_url = if url.contains('?') {
                                format!("{url}&challenge=1")
                            } else {
                                format!("{url}?challenge=1")
                            };
                            let Some(window) = web_sys::window() else {
                                set_share_status.set(Some("Clipboard unavailable.".to_string()));
                                return;
                            };
                            let clipboard = window.navigator().clipboard();
                            let _ = clipboard.write_text(&challenge_url);
                            set_share_status.set(Some("Challenge link copied.".to_string()));
                        }
                    >
                        "Challenge a friend"
                    </button>
                </div>
                <Show when=move || show_share.get()>
                    <div class="share-card">
                        <label>
                            "Name / handle (optional)"
                            <input
                                type="text"
                                placeholder="@lifter"
                                prop:value=move || share_handle.get()
                                on:input=move |ev| set_share_handle.set(event_target_value(&ev))
                            />
                        </label>
                        <button
                            type="button"
                            class="secondary"
                            on:click=move |_| {
                                let Some((pct, _, _)) = percentile.get() else {
                                    set_share_status.set(Some("Calculate first to generate an image.".to_string()));
                                    return;
                                };
                                let tier = rank_tier.get().unwrap_or("Unknown");
                                let result = download_share_png(
                                    &share_handle.get(),
                                    bodyweight.get(),
                                    squat.get(),
                                    bench.get(),
                                    deadlift.get(),
                                    lift_label(&lift.get()),
                                    pct,
                                    tier,
                                );
                                match result {
                                    Ok(()) => set_share_status.set(Some("PNG downloaded.".to_string())),
                                    Err(err) => set_share_status.set(Some(err)),
                                }
                            }
                        >
                            "Download PNG"
                        </button>
                    </div>
                </Show>
                <Show when=move || share_status.get().is_some()>
                    <p class="muted">{move || share_status.get().unwrap_or_default()}</p>
                </Show>
            </Show>
        </section>
    }
}
