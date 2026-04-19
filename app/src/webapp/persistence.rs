use super::AppPage;
use super::helpers::parse_query_f32;
use super::models::SavedUiState;
use leptos::prelude::*;

pub(super) struct UnitPrefCtx {
    pub(super) loaded: ReadSignal<bool>,
    pub(super) set_loaded: WriteSignal<bool>,
    pub(super) use_lbs: ReadSignal<bool>,
    pub(super) set_use_lbs: WriteSignal<bool>,
}

pub(super) fn setup_unit_pref_effects(ctx: UnitPrefCtx) {
    let UnitPrefCtx {
        loaded,
        set_loaded,
        use_lbs,
        set_use_lbs,
    } = ctx;

    Effect::new(move |_| {
        if loaded.get() {
            return;
        }
        let Some(w) = web_sys::window() else {
            return;
        };
        let Ok(Some(storage)) = w.local_storage() else {
            set_loaded.set(true);
            return;
        };
        if let Ok(Some(saved)) = storage.get_item("ironscale_units")
            && saved == "lb"
        {
            set_use_lbs.set(true);
        }
        set_loaded.set(true);
    });

    Effect::new(move |_| {
        if !loaded.get() {
            return;
        }
        let Some(w) = web_sys::window() else {
            return;
        };
        let Ok(Some(storage)) = w.local_storage() else {
            return;
        };
        let units = if use_lbs.get() { "lb" } else { "kg" };
        let _ = storage.set_item("ironscale_units", units);
    });
}

pub(super) struct QueryLoadCtx {
    pub(super) query_loaded: ReadSignal<bool>,
    pub(super) set_query_loaded: WriteSignal<bool>,
    pub(super) set_sex: WriteSignal<String>,
    pub(super) set_equip: WriteSignal<String>,
    pub(super) set_wc: WriteSignal<String>,
    pub(super) set_age: WriteSignal<String>,
    pub(super) set_tested: WriteSignal<String>,
    pub(super) set_lift: WriteSignal<String>,
    pub(super) set_metric: WriteSignal<String>,
    pub(super) squat: ReadSignal<f32>,
    pub(super) set_squat: WriteSignal<f32>,
    pub(super) bench: ReadSignal<f32>,
    pub(super) set_bench: WriteSignal<f32>,
    pub(super) deadlift: ReadSignal<f32>,
    pub(super) set_deadlift: WriteSignal<f32>,
    pub(super) bodyweight: ReadSignal<f32>,
    pub(super) set_bodyweight: WriteSignal<f32>,
    pub(super) set_squat_delta: WriteSignal<f32>,
    pub(super) set_bench_delta: WriteSignal<f32>,
    pub(super) set_deadlift_delta: WriteSignal<f32>,
    pub(super) set_lift_mult: WriteSignal<usize>,
    pub(super) set_bw_mult: WriteSignal<usize>,
    pub(super) set_calculated: WriteSignal<bool>,
}

pub(super) fn setup_query_load_effect(ctx: QueryLoadCtx) {
    let QueryLoadCtx {
        query_loaded,
        set_query_loaded,
        set_sex,
        set_equip,
        set_wc,
        set_age,
        set_tested,
        set_lift,
        set_metric,
        squat,
        set_squat,
        bench,
        set_bench,
        deadlift,
        set_deadlift,
        bodyweight,
        set_bodyweight,
        set_squat_delta,
        set_bench_delta,
        set_deadlift_delta,
        set_lift_mult,
        set_bw_mult,
        set_calculated,
    } = ctx;

    Effect::new(move |_| {
        if query_loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(search) = window.location().search() else {
            return;
        };
        if search.is_empty() {
            let mut restored_saved_state = false;
            if let Ok(Some(storage)) = window.local_storage()
                && let Ok(Some(raw)) = storage.get_item("ironscale_last_state")
                && let Ok(saved) = serde_json::from_str::<SavedUiState>(&raw)
            {
                restored_saved_state = true;
                set_sex.set(saved.sex);
                set_equip.set(saved.equip);
                set_wc.set(saved.wc);
                set_age.set(saved.age);
                set_tested.set(saved.tested);
                set_lift.set(saved.lift);
                set_metric.set(saved.metric);
                set_squat.set(saved.squat.clamp(0.0, 600.0));
                set_bench.set(saved.bench.clamp(0.0, 600.0));
                set_deadlift.set(saved.deadlift.clamp(0.0, 600.0));
                set_bodyweight.set(saved.bodyweight.clamp(35.0, 300.0));
                set_squat_delta.set(saved.squat_delta.clamp(-50.0, 50.0));
                set_bench_delta.set(saved.bench_delta.clamp(-50.0, 50.0));
                set_deadlift_delta.set(saved.deadlift_delta.clamp(-50.0, 50.0));
                set_lift_mult.set(saved.lift_mult.clamp(1, 4));
                set_bw_mult.set(saved.bw_mult.clamp(1, 5));
                set_calculated.set(saved.calculated);
            }
            if !restored_saved_state {
                set_calculated.set(true);
            }
            set_query_loaded.set(true);
            return;
        }
        let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) else {
            set_query_loaded.set(true);
            return;
        };
        if let Some(v) = params.get("sex") {
            set_sex.set(v);
        }
        if let Some(v) = params.get("equip") {
            set_equip.set(v);
        }
        if let Some(v) = params.get("wc") {
            set_wc.set(v);
        }
        if let Some(v) = params.get("age") {
            set_age.set(v);
        }
        if let Some(v) = params.get("tested") {
            set_tested.set(v);
        }
        if let Some(v) = params.get("lift") {
            set_lift.set(v);
        }
        if let Some(v) = params.get("metric") {
            set_metric.set(v);
        }
        set_squat.set(parse_query_f32(
            params.get("s"),
            squat.get_untracked(),
            0.0,
            600.0,
        ));
        set_bench.set(parse_query_f32(
            params.get("b"),
            bench.get_untracked(),
            0.0,
            600.0,
        ));
        set_deadlift.set(parse_query_f32(
            params.get("d"),
            deadlift.get_untracked(),
            0.0,
            600.0,
        ));
        set_bodyweight.set(parse_query_f32(
            params.get("bw"),
            bodyweight.get_untracked(),
            35.0,
            300.0,
        ));
        set_squat_delta.set(parse_query_f32(params.get("sd"), 0.0, -50.0, 50.0));
        set_bench_delta.set(parse_query_f32(params.get("bd"), 0.0, -50.0, 50.0));
        set_deadlift_delta.set(parse_query_f32(params.get("dd"), 0.0, -50.0, 50.0));
        if params.get("calc").as_deref() == Some("1") {
            set_calculated.set(true);
        }
        set_query_loaded.set(true);
    });
}

pub(super) struct HashNavCtx {
    pub(super) page_loaded: ReadSignal<bool>,
    pub(super) set_page_loaded: WriteSignal<bool>,
    pub(super) active_page: ReadSignal<AppPage>,
    pub(super) set_active_page: WriteSignal<AppPage>,
}

pub(super) fn setup_hash_nav_effects(ctx: HashNavCtx) {
    let HashNavCtx {
        page_loaded,
        set_page_loaded,
        active_page,
        set_active_page,
    } = ctx;

    Effect::new(move |_| {
        if page_loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        if let Ok(hash) = window.location().hash() {
            let p = if hash.eq_ignore_ascii_case("#nerds") {
                AppPage::Nerds
            } else if hash.eq_ignore_ascii_case("#men-vs-women") {
                AppPage::MenVsWomen
            } else if hash.eq_ignore_ascii_case("#1rm") {
                AppPage::OneRm
            } else if hash.eq_ignore_ascii_case("#plate-calc") {
                AppPage::PlateCalc
            } else if hash.eq_ignore_ascii_case("#bodyfat") {
                AppPage::Bodyfat
            } else {
                AppPage::Ranking
            };
            set_active_page.set(p);
        }
        set_page_loaded.set(true);
    });

    Effect::new(move |_| {
        if !page_loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let _ = window.location().set_hash(active_page.get().hash());
    });
}

pub(super) struct StatePersistCtx {
    pub(super) query_loaded: ReadSignal<bool>,
    pub(super) sex: ReadSignal<String>,
    pub(super) equip: ReadSignal<String>,
    pub(super) wc: ReadSignal<String>,
    pub(super) age: ReadSignal<String>,
    pub(super) tested: ReadSignal<String>,
    pub(super) lift: ReadSignal<String>,
    pub(super) metric: ReadSignal<String>,
    pub(super) squat: ReadSignal<f32>,
    pub(super) bench: ReadSignal<f32>,
    pub(super) deadlift: ReadSignal<f32>,
    pub(super) bodyweight: ReadSignal<f32>,
    pub(super) squat_delta: ReadSignal<f32>,
    pub(super) bench_delta: ReadSignal<f32>,
    pub(super) deadlift_delta: ReadSignal<f32>,
    pub(super) lift_mult: ReadSignal<usize>,
    pub(super) bw_mult: ReadSignal<usize>,
    pub(super) calculated: ReadSignal<bool>,
}

pub(super) fn setup_state_persist_effect(ctx: StatePersistCtx) {
    let StatePersistCtx {
        query_loaded,
        sex,
        equip,
        wc,
        age,
        tested,
        lift,
        metric,
        squat,
        bench,
        deadlift,
        bodyweight,
        squat_delta,
        bench_delta,
        deadlift_delta,
        lift_mult,
        bw_mult,
        calculated,
    } = ctx;

    Effect::new(move |_| {
        if !query_loaded.get() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(Some(storage)) = window.local_storage() else {
            return;
        };
        let snapshot = SavedUiState {
            sex: sex.get(),
            equip: equip.get(),
            wc: wc.get(),
            age: age.get(),
            tested: tested.get(),
            lift: lift.get(),
            metric: metric.get(),
            squat: squat.get(),
            bench: bench.get(),
            deadlift: deadlift.get(),
            bodyweight: bodyweight.get(),
            squat_delta: squat_delta.get(),
            bench_delta: bench_delta.get(),
            deadlift_delta: deadlift_delta.get(),
            lift_mult: lift_mult.get(),
            bw_mult: bw_mult.get(),
            share_handle: String::new(),
            calculated: calculated.get(),
        };
        if let Ok(raw) = serde_json::to_string(&snapshot) {
            let _ = storage.set_item("ironscale_last_state", &raw);
        }
    });
}
