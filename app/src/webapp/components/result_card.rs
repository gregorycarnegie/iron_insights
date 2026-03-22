use super::ResultCardSections;
use crate::webapp::helpers::kg_to_display;
use crate::webapp::share::{ShareImagePayload, download_share_png};
use crate::webapp::ui::lift_label;
use gloo_timers::callback::Timeout;
use js_sys::Math;
use leptos::prelude::*;

const GAUGE_CIRCUMFERENCE: f32 = 282.743_35;

#[derive(Clone)]
struct ConfettiPiece {
    id: usize,
    style: String,
}

fn tier_slug(tier: Option<&str>) -> &'static str {
    match tier.unwrap_or("Novice") {
        "Legend" => "legend",
        "Elite" => "elite",
        "Advanced" => "advanced",
        "Intermediate" => "intermediate",
        _ => "novice",
    }
}

fn confetti_count(tier: Option<&str>) -> usize {
    match tier.unwrap_or("Novice") {
        "Legend" => 140,
        "Elite" => 110,
        _ => 60,
    }
}

fn random_between(min: f64, max: f64) -> f64 {
    min + (max - min) * Math::random()
}

fn build_confetti(count: usize) -> Vec<ConfettiPiece> {
    const PALETTE: [&str; 6] = [
        "#c8f135", "#4ecdc4", "#ffd700", "#ff6b8a", "#e040fb", "#f0efe8",
    ];

    (0..count)
        .map(|id| {
            let x = random_between(-280.0, 280.0);
            let y = random_between(-240.0, 180.0);
            let rotation = random_between(-520.0, 520.0);
            let duration = random_between(0.9, 1.55);
            let delay = random_between(0.0, 0.14);
            let size = random_between(6.0, 13.0);
            let color = PALETTE[id % PALETTE.len()];
            ConfettiPiece {
                id,
                style: format!(
                    "--x:{x:.1}px;--y:{y:.1}px;--rot:{rotation:.1}deg;--duration:{duration:.2}s;--delay:{delay:.2}s;--size:{size:.1}px;--color:{color};"
                ),
            }
        })
        .collect()
}

#[component]
pub(in crate::webapp) fn ResultCardPanel(card: ResultCardSections) -> impl IntoView {
    let ResultCardSections {
        status,
        share,
        lifts,
    } = card;
    let calculated = status.calculated;
    let percentile = status.percentile;
    let rank_tier = status.rank_tier;
    let reveal_tick = status.reveal_tick;
    let load_error = status.load_error;
    let unavailable_reason = status.unavailable_reason;

    let show_share = share.show_share;
    let set_show_share = share.set_show_share;
    let share_url = share.share_url;
    let share_status = share.share_status;
    let set_share_status = share.set_share_status;
    let share_handle = share.share_handle;
    let set_share_handle = share.set_share_handle;

    let bodyweight = lifts.bodyweight;
    let squat = lifts.squat;
    let bench = lifts.bench;
    let deadlift = lifts.deadlift;
    let lift = lifts.lift;
    let use_lbs = lifts.use_lbs;
    let unit_label = lifts.unit_label;

    let (revealed_tick, set_revealed_tick) = signal(0u64);
    let (active_reveal_tick, set_active_reveal_tick) = signal(0u64);
    let (flash_active, set_flash_active) = signal(false);
    let (celebrating, set_celebrating) = signal(false);
    let (badge_impact, set_badge_impact) = signal(false);
    let (confetti, set_confetti) = signal(Vec::<ConfettiPiece>::new());

    Effect::new(move |_| {
        let tick = reveal_tick.get();
        let ready = calculated.get() && percentile.get().is_some();
        if !ready || tick == 0 || tick <= revealed_tick.get_untracked() {
            return;
        }

        set_revealed_tick.set(tick);
        set_active_reveal_tick.set(tick);
        set_flash_active.set(true);
        set_celebrating.set(true);
        set_badge_impact.set(true);
        set_confetti.set(build_confetti(confetti_count(rank_tier.get())));

        let active_reveal_tick = active_reveal_tick;
        let set_flash_active = set_flash_active;
        Timeout::new(240, move || {
            if active_reveal_tick.get_untracked() == tick {
                set_flash_active.set(false);
            }
        })
        .forget();

        let active_reveal_tick = active_reveal_tick;
        let set_celebrating = set_celebrating;
        Timeout::new(620, move || {
            if active_reveal_tick.get_untracked() == tick {
                set_celebrating.set(false);
            }
        })
        .forget();

        let active_reveal_tick = active_reveal_tick;
        let set_badge_impact = set_badge_impact;
        Timeout::new(760, move || {
            if active_reveal_tick.get_untracked() == tick {
                set_badge_impact.set(false);
            }
        })
        .forget();

        let active_reveal_tick = active_reveal_tick;
        let set_confetti = set_confetti;
        Timeout::new(1700, move || {
            if active_reveal_tick.get_untracked() == tick {
                set_confetti.set(Vec::new());
            }
        })
        .forget();
    });

    view! {
        <section class=move || format!("panel result-card result-card--{}", tier_slug(rank_tier.get()))>
            <div class="screen-flash" class:fire=move || flash_active.get()></div>
            <div class="panel-titlebar">
                <div>
                    <h2>"Result"</h2>
                    <p class="muted">
                        "Your percentile, tier, and share-ready result card."
                    </p>
                </div>
            </div>
            <Show
                when=move || calculated.get()
                fallback=move || {
                    view! {
                        <div class="result-waiting">
                            <p class="result-empty-title">"Ready when you are."</p>
                            <p class="muted">"Press Calculate my ranking to reveal your percentile result."</p>
                        </div>
                    }
                }
            >
                {move || match percentile.get() {
                    Some((pct, rank, total)) => {
                        let percentile_value = pct * 100.0;
                        let top_value = (100.0 - percentile_value).max(0.0);
                        let tier_label = rank_tier.get().unwrap_or("Novice");
                        let total_lift = squat.get() + bench.get() + deadlift.get();
                        let gauge_offset = GAUGE_CIRCUMFERENCE * (1.0 - pct.clamp(0.0, 1.0));
                        view! {
                            <div class="result-block" class:celebrating=move || celebrating.get()>
                                <div class="result-confetti" aria-hidden="true">
                                    <For each=move || confetti.get() key=|piece| piece.id let:piece>
                                        <span class="confetti-piece" style=piece.style.clone()></span>
                                    </For>
                                </div>
                                <div class="result-top">
                                    <div class="gauge-wrap" aria-hidden="true">
                                        <svg class="gauge-svg" viewBox="0 0 100 100">
                                            <circle class="gauge-bg" cx="50" cy="50" r="45"></circle>
                                            <circle
                                                class="gauge-track-glow"
                                                cx="50"
                                                cy="50"
                                                r="45"
                                                stroke-dasharray=GAUGE_CIRCUMFERENCE.to_string()
                                                stroke-dashoffset=move || gauge_offset.to_string()
                                            ></circle>
                                            <circle
                                                class="gauge-fill"
                                                cx="50"
                                                cy="50"
                                                r="45"
                                                stroke-dasharray=GAUGE_CIRCUMFERENCE.to_string()
                                                stroke-dashoffset=move || gauge_offset.to_string()
                                            ></circle>
                                        </svg>
                                        <div class="gauge-center">
                                            <div class="gauge-value">{format!("{percentile_value:.1}%")}</div>
                                            <div class="gauge-label">"Percentile"</div>
                                        </div>
                                    </div>
                                    <div class="result-info">
                                        <div
                                            class=move || format!("level-badge {}", tier_slug(Some(tier_label)))
                                            class:rank-up=move || badge_impact.get()
                                        >
                                            <span class="tier-name">{tier_label}</span>
                                            <span class="tier-shine"></span>
                                        </div>
                                        <p class="result-desc">
                                            "You are stronger than "
                                            <strong>{format!("{percentile_value:.1}%")}</strong>
                                            " of lifters in the dataset."
                                        </p>
                                        <p class="result-note">
                                            {format!("Estimated rank: {} / {} lifters in this cohort.", rank, total)}
                                        </p>
                                        <div class="result-chips">
                                            <div class="result-chip">
                                                "Rank "
                                                <strong>{format!("Top {top_value:.1}%")}</strong>
                                            </div>
                                            <div class="result-chip">
                                                "Total "
                                                <strong>
                                                    {format!(
                                                        "{:.1} {}",
                                                        kg_to_display(total_lift, use_lbs.get()),
                                                        unit_label.get()
                                                    )}
                                                </strong>
                                            </div>
                                            <div class="result-chip">
                                                "Bodyweight "
                                                <strong>
                                                    {format!(
                                                        "{:.1} {}",
                                                        kg_to_display(bodyweight.get(), use_lbs.get()),
                                                        unit_label.get()
                                                    )}
                                                </strong>
                                            </div>
                                            <div class="result-chip">
                                                "Focus "
                                                <strong>{lift_label(&lift.get())}</strong>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div class="bar-wrap">
                                    <div class="bar-track">
                                        <div class="bar-fill" style=move || format!("width:{percentile_value:.1}%")></div>
                                    </div>
                                    <div class="bar-labels">
                                        <span>"0th"</span>
                                        <span class="current">{format!("{percentile_value:.1} / 100")}</span>
                                        <span>"100th"</span>
                                    </div>
                                </div>
                                <div class="action-row">
                                    <button
                                        type="button"
                                        class="action-btn"
                                        aria-controls="share-options"
                                        aria-expanded=move || show_share.get().to_string()
                                        on:click=move |_| set_show_share.update(|open| *open = !*open)
                                    >
                                        {move || if show_share.get() { "Hide share options" } else { "Share result" }}
                                    </button>
                                    <button
                                        type="button"
                                        class="action-btn"
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
                                        class="action-btn"
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
                                        "Challenge link"
                                    </button>
                                </div>
                                <Show when=move || show_share.get()>
                                    <div id="share-options" class="share-card">
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
                                                let tier = rank_tier.get().unwrap_or("Unknown");
                                                let result = download_share_png(ShareImagePayload {
                                                    handle: &share_handle.get(),
                                                    bodyweight: bodyweight.get(),
                                                    squat: squat.get(),
                                                    bench: bench.get(),
                                                    deadlift: deadlift.get(),
                                                    lift_focus: lift_label(&lift.get()),
                                                    percentile: pct,
                                                    tier,
                                                });
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
                                    <p class="muted" role="status" aria-live="polite" aria-atomic="true">
                                        {move || share_status.get().unwrap_or_default()}
                                    </p>
                                </Show>
                            </div>
                        }
                        .into_any()
                    }
                    None => {
                        view! {
                            <div class="result-waiting result-unavailable">
                                <p class="result-empty-title">"Result unavailable"</p>
                                <p class="muted">
                                    {unavailable_reason
                                        .get()
                                        .unwrap_or_else(|| {
                                            load_error
                                                .get()
                                                .map(|err| format!("Comparison data unavailable: {err}"))
                                                .unwrap_or_else(|| "No matching group found for these filters.".to_string())
                                        })}
                                </p>
                            </div>
                        }
                        .into_any()
                    }
                }}
            </Show>
        </section>
    }
}
