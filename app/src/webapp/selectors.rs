use super::slices::parse_shard_key;
use super::ui::{age_class_sort_key, ipf_class_sort_key, unique};
use super::{RootIndex, SliceRow};
use leptos::prelude::*;

pub(super) fn sex_options(root_index: ReadSignal<Option<RootIndex>>) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        root_index
            .get()
            .map(|root| {
                unique(
                    root.shards
                        .keys()
                        .filter_map(|k| parse_shard_key(k).map(|(s, _)| s.to_string())),
                )
            })
            .unwrap_or_default()
    })
}

pub(super) fn equip_options(
    root_index: ReadSignal<Option<RootIndex>>,
    sex: ReadSignal<String>,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let s = sex.get();
        root_index
            .get()
            .map(|root| {
                unique(root.shards.keys().filter_map(|k| {
                    parse_shard_key(k).and_then(|(sx, eq)| if sx == s { Some(eq.to_string()) } else { None })
                }))
            })
            .unwrap_or_default()
    })
}

pub(super) fn tested_options(
    slice_rows: ReadSignal<Vec<SliceRow>>,
    sex: ReadSignal<String>,
    equip: ReadSignal<String>,
    wc: ReadSignal<String>,
    age: ReadSignal<String>,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let w = wc.get();
        let a = age.get();
        unique(
            slice_rows
                .get()
                .iter()
                .filter(|r| r.key.sex == s && r.key.equip == e && r.key.wc == w && r.key.age == a)
                .map(|r| r.key.tested.clone()),
        )
    })
}

pub(super) fn wc_options(
    slice_rows: ReadSignal<Vec<SliceRow>>,
    sex: ReadSignal<String>,
    equip: ReadSignal<String>,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let mut classes = unique(
            slice_rows
                .get()
                .iter()
                .filter(|r| r.key.sex == s && r.key.equip == e)
                .map(|r| r.key.wc.clone()),
        );
        classes.sort_by_key(|c| ipf_class_sort_key(c));
        classes
    })
}

pub(super) fn age_options(
    slice_rows: ReadSignal<Vec<SliceRow>>,
    sex: ReadSignal<String>,
    equip: ReadSignal<String>,
    wc: ReadSignal<String>,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let w = wc.get();
        let mut classes = unique(
            slice_rows
                .get()
                .iter()
                .filter(|r| r.key.sex == s && r.key.equip == e && r.key.wc == w)
                .map(|r| r.key.age.clone()),
        );
        classes.sort_by_key(|c| age_class_sort_key(c));
        classes
    })
}

pub(super) fn lift_options(
    slice_rows: ReadSignal<Vec<SliceRow>>,
    sex: ReadSignal<String>,
    equip: ReadSignal<String>,
    wc: ReadSignal<String>,
    age: ReadSignal<String>,
    tested: ReadSignal<String>,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let w = wc.get();
        let a = age.get();
        let t = tested.get();
        unique(
            slice_rows
                .get()
                .iter()
                .filter(|r| {
                    r.key.sex == s
                        && r.key.equip == e
                        && r.key.wc == w
                        && r.key.age == a
                        && r.key.tested == t
                })
                .map(|r| r.key.lift.clone()),
        )
    })
}

pub(super) fn metric_options(
    slice_rows: ReadSignal<Vec<SliceRow>>,
    sex: ReadSignal<String>,
    equip: ReadSignal<String>,
    wc: ReadSignal<String>,
    age: ReadSignal<String>,
    tested: ReadSignal<String>,
    lift: ReadSignal<String>,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let s = sex.get();
        let e = equip.get();
        let w = wc.get();
        let a = age.get();
        let t = tested.get();
        let l = lift.get();
        unique(
            slice_rows
                .get()
                .iter()
                .filter(|r| {
                    r.key.sex == s
                        && r.key.equip == e
                        && r.key.wc == w
                        && r.key.age == a
                        && r.key.tested == t
                        && r.key.lift == l
                })
                .map(|r| r.key.metric.clone()),
        )
    })
}
