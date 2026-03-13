use super::slices::parse_shard_key;
use super::ui::{age_class_sort_key, ipf_class_sort_key, unique};
use super::{RootIndex, SliceRow};
use leptos::prelude::*;
use std::collections::{BTreeSet, HashMap};

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
                    parse_shard_key(k)
                        .and_then(|(sx, eq)| if sx == s { Some(eq.to_string()) } else { None })
                }))
            })
            .unwrap_or_default()
    })
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct SliceSelectorIndex {
    all_rows: Vec<SliceRow>,
    weight_classes: Vec<String>,
    ages_by_wc: HashMap<String, Vec<String>>,
    tested_by_wc_age: HashMap<String, Vec<String>>,
    lifts_by_wc_age_tested: HashMap<String, Vec<String>>,
    metrics_by_wc_age_tested_lift: HashMap<String, Vec<String>>,
    exact_rows: HashMap<String, SliceRow>,
    rows_by_tested_lift_metric: HashMap<String, Vec<SliceRow>>,
    rows_by_tested_lift: HashMap<String, Vec<SliceRow>>,
}

impl SliceSelectorIndex {
    fn from_rows(rows: Vec<SliceRow>) -> Self {
        let mut all_rows = Vec::with_capacity(rows.len());
        let mut weight_classes = BTreeSet::new();
        let mut ages_by_wc = HashMap::<String, BTreeSet<String>>::new();
        let mut tested_by_wc_age = HashMap::<String, BTreeSet<String>>::new();
        let mut lifts_by_wc_age_tested = HashMap::<String, BTreeSet<String>>::new();
        let mut metrics_by_wc_age_tested_lift = HashMap::<String, BTreeSet<String>>::new();
        let mut exact_rows = HashMap::<String, SliceRow>::new();
        let mut rows_by_tested_lift_metric = HashMap::<String, Vec<SliceRow>>::new();
        let mut rows_by_tested_lift = HashMap::<String, Vec<SliceRow>>::new();

        for row in rows {
            let wc = row.key.wc.clone();
            let age = row.key.age.clone();
            let tested = row.key.tested.clone();
            let lift = row.key.lift.clone();
            let metric = row.key.metric.clone();

            weight_classes.insert(wc.clone());
            ages_by_wc
                .entry(wc.clone())
                .or_default()
                .insert(age.clone());
            tested_by_wc_age
                .entry(selector_key(&[&wc, &age]))
                .or_default()
                .insert(tested.clone());
            lifts_by_wc_age_tested
                .entry(selector_key(&[&wc, &age, &tested]))
                .or_default()
                .insert(lift.clone());
            metrics_by_wc_age_tested_lift
                .entry(selector_key(&[&wc, &age, &tested, &lift]))
                .or_default()
                .insert(metric.clone());

            let row_clone = row.clone();
            exact_rows.insert(
                selector_key(&[&wc, &age, &tested, &lift, &metric]),
                row_clone.clone(),
            );
            rows_by_tested_lift_metric
                .entry(selector_key(&[&tested, &lift, &metric]))
                .or_default()
                .push(row_clone.clone());
            rows_by_tested_lift
                .entry(selector_key(&[&tested, &lift]))
                .or_default()
                .push(row_clone.clone());
            all_rows.push(row_clone);
        }

        let mut weight_classes: Vec<_> = weight_classes.into_iter().collect();
        weight_classes.sort_by_key(|wc| ipf_class_sort_key(wc));

        let ages_by_wc = ages_by_wc
            .into_iter()
            .map(|(wc, ages)| {
                let mut ages: Vec<_> = ages.into_iter().collect();
                ages.sort_by_key(|age| age_class_sort_key(age));
                (wc, ages)
            })
            .collect();

        let tested_by_wc_age = tested_by_wc_age
            .into_iter()
            .map(|(key, values)| (key, values.into_iter().collect()))
            .collect();
        let lifts_by_wc_age_tested = lifts_by_wc_age_tested
            .into_iter()
            .map(|(key, values)| (key, values.into_iter().collect()))
            .collect();
        let metrics_by_wc_age_tested_lift = metrics_by_wc_age_tested_lift
            .into_iter()
            .map(|(key, values)| (key, values.into_iter().collect()))
            .collect();

        Self {
            all_rows,
            weight_classes,
            ages_by_wc,
            tested_by_wc_age,
            lifts_by_wc_age_tested,
            metrics_by_wc_age_tested_lift,
            exact_rows,
            rows_by_tested_lift_metric,
            rows_by_tested_lift,
        }
    }

    pub(super) fn wc_options(&self) -> Vec<String> {
        self.weight_classes.clone()
    }

    pub(super) fn age_options(&self, wc: &str) -> Vec<String> {
        self.ages_by_wc.get(wc).cloned().unwrap_or_default()
    }

    pub(super) fn tested_options(&self, wc: &str, age: &str) -> Vec<String> {
        self.tested_by_wc_age
            .get(&selector_key(&[wc, age]))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn lift_options(&self, wc: &str, age: &str, tested: &str) -> Vec<String> {
        self.lifts_by_wc_age_tested
            .get(&selector_key(&[wc, age, tested]))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn metric_options(
        &self,
        wc: &str,
        age: &str,
        tested: &str,
        lift: &str,
    ) -> Vec<String> {
        self.metrics_by_wc_age_tested_lift
            .get(&selector_key(&[wc, age, tested, lift]))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn current_row(
        &self,
        wc: &str,
        age: &str,
        tested: &str,
        lift: &str,
        metric: &str,
    ) -> Option<SliceRow> {
        self.exact_rows
            .get(&selector_key(&[wc, age, tested, lift, metric]))
            .cloned()
    }

    pub(super) fn candidate_rows(&self, tested: &str, lift: &str, metric: &str) -> Vec<SliceRow> {
        self.rows_by_tested_lift_metric
            .get(&selector_key(&[tested, lift, metric]))
            .cloned()
            .or_else(|| {
                self.rows_by_tested_lift
                    .get(&selector_key(&[tested, lift]))
                    .cloned()
            })
            .unwrap_or_else(|| self.all_rows.clone())
    }
}

pub(super) fn slice_selector_index(
    slice_rows: ReadSignal<Vec<SliceRow>>,
) -> Memo<SliceSelectorIndex> {
    Memo::new(move |_| SliceSelectorIndex::from_rows(slice_rows.get()))
}

pub(super) fn tested_options(
    selector_index: Memo<SliceSelectorIndex>,
    wc: ReadSignal<String>,
    age: ReadSignal<String>,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let w = wc.get();
        let a = age.get();
        selector_index.with(|index| index.tested_options(&w, &a))
    })
}

pub(super) fn wc_options(selector_index: Memo<SliceSelectorIndex>) -> Memo<Vec<String>> {
    Memo::new(move |_| selector_index.with(SliceSelectorIndex::wc_options))
}

pub(super) fn age_options(
    selector_index: Memo<SliceSelectorIndex>,
    wc: ReadSignal<String>,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let w = wc.get();
        selector_index.with(|index| index.age_options(&w))
    })
}

pub(super) fn lift_options(
    selector_index: Memo<SliceSelectorIndex>,
    wc: ReadSignal<String>,
    age: ReadSignal<String>,
    tested: ReadSignal<String>,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let w = wc.get();
        let a = age.get();
        let t = tested.get();
        selector_index.with(|index| index.lift_options(&w, &a, &t))
    })
}

pub(super) fn metric_options(
    selector_index: Memo<SliceSelectorIndex>,
    wc: ReadSignal<String>,
    age: ReadSignal<String>,
    tested: ReadSignal<String>,
    lift: ReadSignal<String>,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let w = wc.get();
        let a = age.get();
        let t = tested.get();
        let l = lift.get();
        selector_index.with(|index| index.metric_options(&w, &a, &t, &l))
    })
}

fn selector_key(parts: &[&str]) -> String {
    let mut key = String::with_capacity(
        parts.iter().map(|part| part.len()).sum::<usize>() + parts.len().saturating_sub(1),
    );
    for (idx, part) in parts.iter().enumerate() {
        if idx > 0 {
            key.push('\u{1f}');
        }
        key.push_str(part);
    }
    key
}
