use super::ProgressSections;
use crate::webapp::helpers::kg_to_display;
use crate::webapp::models::SavedSnapshot;
use crate::webapp::ui::lift_label;
use js_sys::Date;
use leptos::prelude::*;
use std::collections::HashMap;

const SNAPSHOT_STORAGE_KEY: &str = "iron_insights_snapshots_v1";
const MAX_SNAPSHOTS: usize = 120;

fn now_unix_secs() -> u64 {
    (Date::now() / 1_000.0).floor() as u64
}

#[component]
pub(in crate::webapp) fn ProgressPanel(tracking: ProgressSections) -> impl IntoView {
    let ProgressSections {
        result,
        selection,
        lifts,
        display,
    } = tracking;
    let (snapshots, set_snapshots) = signal(Vec::<SavedSnapshot>::new());
    let (import_text, set_import_text) = signal(String::new());
    let (status, set_status) = signal(None::<String>);
    let (loaded, set_loaded) = signal(false);

    Effect::new(move |_| {
        if loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            set_loaded.set(true);
            return;
        };
        let Ok(Some(storage)) = window.local_storage() else {
            set_loaded.set(true);
            return;
        };
        if let Ok(Some(raw)) = storage.get_item(SNAPSHOT_STORAGE_KEY)
            && let Ok(saved) = serde_json::from_str::<Vec<SavedSnapshot>>(&raw)
        {
            set_snapshots.set(saved);
        }
        set_loaded.set(true);
    });

    let movement_rows = Memo::new(move |_| {
        let mut ordered = snapshots.get();
        ordered.sort_by_key(|s| s.saved_at_secs);

        let mut prev_by_series: HashMap<String, f32> = HashMap::new();
        let mut rows: Vec<(SavedSnapshot, Option<f32>)> = Vec::with_capacity(ordered.len());
        for snap in ordered {
            let series_key = format!("{}|{}", snap.lift, snap.metric);
            let delta = prev_by_series
                .get(&series_key)
                .copied()
                .map(|prev| snap.percentile - prev);
            prev_by_series.insert(series_key, snap.percentile);
            rows.push((snap, delta));
        }
        rows.reverse();
        rows
    });

    view! {
        <section class="panel progress-panel">
            <h2>"Progress Tracking"</h2>
            <p class="muted">
                "Save snapshots locally, track percentile changes over time, and export/import JSON."
            </p>
            <div class="control-row">
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| {
                        let Some((pct, rank, total)) = result.percentile.get() else {
                            set_status.set(Some("Calculate first to save a snapshot.".to_string()));
                            return;
                        };
                        if !result.calculated.get() {
                            set_status.set(Some("Calculate first to save a snapshot.".to_string()));
                            return;
                        }
                        let snapshot = SavedSnapshot {
                            saved_at_secs: now_unix_secs(),
                            percentile: pct,
                            rank,
                            total_lifters: total,
                            sex: selection.sex.get(),
                            equip: selection.equip.get(),
                            wc: selection.wc.get(),
                            age: selection.age.get(),
                            tested: selection.tested.get(),
                            lift: selection.lift.get(),
                            metric: selection.metric.get(),
                            squat: lifts.squat.get(),
                            bench: lifts.bench.get(),
                            deadlift: lifts.deadlift.get(),
                            bodyweight: lifts.bodyweight.get(),
                        };

                        set_snapshots.update(|rows| {
                            rows.push(snapshot);
                            if rows.len() > MAX_SNAPSHOTS {
                                let trim = rows.len() - MAX_SNAPSHOTS;
                                rows.drain(0..trim);
                            }
                            if let Some(window) = web_sys::window()
                                && let Ok(Some(storage)) = window.local_storage()
                                && let Ok(raw) = serde_json::to_string(rows)
                            {
                                let _ = storage.set_item(SNAPSHOT_STORAGE_KEY, &raw);
                            }
                        });
                        set_status.set(Some("Snapshot saved locally.".to_string()));
                    }
                >
                    "Save snapshot"
                </button>
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| {
                        let rows = snapshots.get_untracked();
                        if rows.is_empty() {
                            set_status.set(Some("No snapshots to export.".to_string()));
                            return;
                        }
                        match serde_json::to_string_pretty(&rows) {
                            Ok(raw) => {
                                set_import_text.set(raw.clone());
                                if let Some(window) = web_sys::window() {
                                    let clipboard = window.navigator().clipboard();
                                    let _ = clipboard.write_text(&raw);
                                }
                                set_status.set(Some("Snapshot JSON exported to textbox (and copied if clipboard is available).".to_string()));
                            }
                            Err(_) => set_status.set(Some("Failed to export snapshot JSON.".to_string())),
                        }
                    }
                >
                    "Export JSON"
                </button>
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| {
                        let raw = import_text.get();
                        let parsed = serde_json::from_str::<Vec<SavedSnapshot>>(&raw);
                        let Ok(mut imported) = parsed else {
                            set_status.set(Some("Import failed: invalid JSON format.".to_string()));
                            return;
                        };
                        imported.sort_by_key(|s| s.saved_at_secs);
                        if imported.len() > MAX_SNAPSHOTS {
                            let trim = imported.len() - MAX_SNAPSHOTS;
                            imported.drain(0..trim);
                        }
                        if let Some(window) = web_sys::window()
                            && let Ok(Some(storage)) = window.local_storage()
                            && let Ok(serialized) = serde_json::to_string(&imported)
                        {
                            let _ = storage.set_item(SNAPSHOT_STORAGE_KEY, &serialized);
                        }
                        set_snapshots.set(imported);
                        set_status.set(Some("Snapshots imported.".to_string()));
                    }
                >
                    "Import JSON"
                </button>
            </div>
            <label class="json-box">
                <span>"Snapshot JSON"</span>
                <textarea
                    rows="6"
                    prop:value=move || import_text.get()
                    on:input=move |ev| set_import_text.set(event_target_value(&ev))
                    placeholder="Paste exported snapshot JSON here and click Import JSON."
                ></textarea>
            </label>
            <Show when=move || status.get().is_some()>
                <p class="muted" role="status" aria-live="polite" aria-atomic="true">
                    {move || status.get().unwrap_or_default()}
                </p>
            </Show>
            <div class="progress-list">
                <Show
                    when=move || !movement_rows.get().is_empty()
                    fallback=move || view! { <p class="muted">"No snapshots yet."</p> }
                >
                    <For
                        each=move || movement_rows.get()
                        key=|(snap, _)| format!("{}-{}-{}-{}", snap.saved_at_secs, snap.lift, snap.metric, snap.rank)
                        let:row
                    >
                        {{
                            let snap = row.0.clone();
                            let delta = row.1;
                            let lift_name = lift_label(&snap.lift).to_string();
                            let metric_name = snap.metric.clone();
                            view! {
                                <div class="progress-item">
                                    <p>
                                        <strong>{format!("{:.1}%", snap.percentile * 100.0)}</strong>
                                        " · "
                                        {format!(
                                            "{} / {} · {} {}",
                                            snap.rank,
                                            snap.total_lifters,
                                            lift_name,
                                            metric_name
                                        )}
                                    </p>
                                    <p class="muted">
                                        {match delta {
                                            Some(d) => format!("Change from previous {} {} snapshot: {:+.2} pts", lift_name, metric_name, d * 100.0),
                                            None => "First snapshot in this lift/metric series.".to_string(),
                                        }}
                                    </p>
                                    <p class="muted">
                                        {move || format!(
                                            "S {:.1} / B {:.1} / D {:.1} · BW {:.1} {} · saved {}",
                                            kg_to_display(snap.squat, display.use_lbs.get()),
                                            kg_to_display(snap.bench, display.use_lbs.get()),
                                            kg_to_display(snap.deadlift, display.use_lbs.get()),
                                            kg_to_display(snap.bodyweight, display.use_lbs.get()),
                                            display.unit_label.get(),
                                            snap.saved_at_secs
                                        )}
                                    </p>
                                </div>
                            }
                        }}
                    </For>
                </Show>
            </div>
        </section>
    }
}
