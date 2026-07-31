use super::{
    models::{RootIndex, SliceRow},
    ui::{age_class_sort_key, ipf_class_sort_key, unique},
};
use crate::core::parse_shard_key;
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
                    parse_shard_key(k)
                        .and_then(|(sx, eq)| if sx == s { Some(eq.to_string()) } else { None })
                }))
            })
            .unwrap_or_default()
    })
}

/// The rows of one shard, kept in `SliceKey` order. The cohort dropdowns cascade
/// by filtering this list on demand — a shard is a few thousand rows, so the
/// scan is cheaper than the parallel lookup maps it replaced.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct SliceSelectorIndex {
    rows: Vec<SliceRow>,
}

impl SliceSelectorIndex {
    fn from_rows(rows: Vec<SliceRow>) -> Self {
        Self { rows }
    }

    pub(super) fn wc_options(&self) -> Vec<String> {
        let mut classes = unique(self.rows.iter().map(|r| r.key.wc.clone()));
        classes.sort_by_key(|wc| ipf_class_sort_key(wc));
        classes
    }

    pub(super) fn age_options(&self, wc: &str) -> Vec<String> {
        let mut ages = unique(
            self.rows
                .iter()
                .filter(|r| r.key.wc == wc)
                .map(|r| r.key.age.clone()),
        );
        ages.sort_by_key(|age| age_class_sort_key(age));
        ages
    }

    pub(super) fn tested_options(&self, wc: &str, age: &str) -> Vec<String> {
        unique(
            self.rows
                .iter()
                .filter(|r| r.key.wc == wc && r.key.age == age)
                .map(|r| r.key.tested.clone()),
        )
    }

    pub(super) fn lift_options(&self, wc: &str, age: &str, tested: &str) -> Vec<String> {
        unique(
            self.rows
                .iter()
                .filter(|r| r.key.wc == wc && r.key.age == age && r.key.tested == tested)
                .map(|r| r.key.lift.clone()),
        )
    }

    pub(super) fn metric_options(
        &self,
        wc: &str,
        age: &str,
        tested: &str,
        lift: &str,
    ) -> Vec<String> {
        unique(
            self.rows
                .iter()
                .filter(|r| {
                    r.key.wc == wc
                        && r.key.age == age
                        && r.key.tested == tested
                        && r.key.lift == lift
                })
                .map(|r| r.key.metric.clone()),
        )
    }

    pub(super) fn current_row(
        &self,
        wc: &str,
        age: &str,
        tested: &str,
        lift: &str,
        metric: &str,
    ) -> Option<SliceRow> {
        self.rows
            .iter()
            .find(|r| {
                r.key.wc == wc
                    && r.key.age == age
                    && r.key.tested == tested
                    && r.key.lift == lift
                    && r.key.metric == metric
            })
            .cloned()
    }

    /// Rows to fall back on when the exact cohort has no slice: narrowest match
    /// first, widening to every row rather than returning nothing.
    pub(super) fn candidate_rows(&self, tested: &str, lift: &str, metric: &str) -> Vec<SliceRow> {
        let matching = |with_metric: bool| -> Vec<SliceRow> {
            self.rows
                .iter()
                .filter(|r| {
                    r.key.tested == tested
                        && r.key.lift == lift
                        && (!with_metric || r.key.metric == metric)
                })
                .cloned()
                .collect()
        };

        let exact = matching(true);
        if !exact.is_empty() {
            return exact;
        }
        let any_metric = matching(false);
        if any_metric.is_empty() {
            self.rows.clone()
        } else {
            any_metric
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::SliceKey, webapp::models::SliceIndexEntry};
    use wasm_bindgen_test::wasm_bindgen_test;

    fn row(wc: &str, age: &str, tested: &str, lift: &str, metric: &str) -> SliceRow {
        SliceRow {
            key: SliceKey {
                sex: "M".to_string(),
                equip: "Raw".to_string(),
                wc: wc.to_string(),
                age: age.to_string(),
                tested: tested.to_string(),
                lift: lift.to_string(),
                metric: metric.to_string(),
                metric_explicit: true,
            },
            entry: SliceIndexEntry {
                bin: format!("{wc}-{age}-{tested}-{lift}-{metric}.bin"),
                inline: String::new(),
                summary: None,
            },
        }
    }

    #[wasm_bindgen_test]
    fn selector_index_cascades_wc_age_tested_lift_metric() {
        let index = SliceSelectorIndex::from_rows(vec![
            row("All", "All Ages", "All", "T", "Kg"),
            row("83", "All Ages", "All", "T", "Kg"),
            row("83", "24-34", "All", "T", "Kg"),
            row("83", "24-34", "Tested", "S", "Kg"),
            row("83", "24-34", "Tested", "T", "Dots"),
            row("93", "All Ages", "All", "T", "Kg"),
        ]);

        let snapshot = format!(
            "wc={:?}; age83={:?}; tested83_24={:?}; lifts83_24_tested={:?}; metrics83_24_tested_t={:?}",
            index.wc_options(),
            index.age_options("83"),
            index.tested_options("83", "24-34"),
            index.lift_options("83", "24-34", "Tested"),
            index.metric_options("83", "24-34", "Tested", "T"),
        );

        assert_eq!(
            snapshot,
            "wc=[\"All\", \"83\", \"93\"]; age83=[\"All Ages\", \"24-34\"]; tested83_24=[\"All\", \"Tested\"]; lifts83_24_tested=[\"S\", \"T\"]; metrics83_24_tested_t=[\"Dots\"]",
        );

        let exact = index
            .current_row("83", "24-34", "Tested", "T", "Dots")
            .expect("exact row should resolve");
        assert_eq!(exact.entry.bin, "83-24-34-Tested-T-Dots.bin");

        let fallback = index.candidate_rows("Tested", "T", "Kg");
        assert_eq!(fallback.len(), 1);
        assert_eq!(fallback[0].key.wc, "83");
        assert_eq!(fallback[0].key.age, "24-34");
    }
}
