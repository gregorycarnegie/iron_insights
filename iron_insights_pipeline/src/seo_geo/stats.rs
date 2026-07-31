use std::{fs, path::Path};

use iron_insights_core::{
    HistogramBin, parse_combined_bin, percentile_for_value, value_for_percentile,
};

use super::constants::LIFTS;

// Curated cohorts shown in the standards tables: raw, all ages, untested+tested
// combined ("all"), the weight classes most people search for.
const MEN_CLASSES: [&str; 7] = ["66", "74", "83", "93", "105", "120", "120_"];
const WOMEN_CLASSES: [&str; 7] = ["52", "57", "63", "69", "76", "84", "84_"];

pub(super) struct ClassRow {
    pub(super) wc: &'static str,
    /// (p50, p90) per lift, in LIFTS order; None when that bin is absent.
    pub(super) cells: [Option<(f32, f32)>; 4],
}

pub(super) struct Example {
    pub(super) sex: &'static str,
    pub(super) wc: &'static str,
    pub(super) lift: &'static str,
    pub(super) value: f32,
    pub(super) pct: f32,
}

pub(super) struct Stats {
    pub(super) men: Vec<ClassRow>,
    pub(super) women: Vec<ClassRow>,
    /// (lift, men_median, women_median, female/male %) in LIFTS order.
    pub(super) comparison: Vec<(&'static str, f32, f32, f32)>,
    pub(super) dots_m: Option<f32>,
    pub(super) dots_f: Option<f32>,
    pub(super) result_count: u64,
    pub(super) examples: Vec<Example>,
}

#[allow(clippy::too_many_arguments)]
fn read_hist(
    bins: &Path,
    sex: &str,
    equip: &str,
    wc: &str,
    age: &str,
    tested: &str,
    metric: &str,
    lift: &str,
) -> Option<HistogramBin> {
    let path = bins
        .join(sex)
        .join(equip)
        .join(wc)
        .join(age)
        .join(tested)
        .join(metric)
        .join(format!("{lift}.bin"));
    let bytes = fs::read(path).ok()?;
    parse_combined_bin(&bytes).map(|(hist, _heat)| hist)
}

pub(super) fn percentile_value(hist: &HistogramBin, q: f32) -> f32 {
    value_for_percentile(Some(hist), q).unwrap_or(0.0)
}

pub(super) fn value_percentile(hist: &HistogramBin, value: f32) -> f32 {
    percentile_for_value(Some(hist), value).map_or(0.0, |(pct, _, _)| 100.0 * pct)
}

pub(super) fn load_stats(bins: &Path) -> Option<Stats> {
    if !bins.is_dir() {
        return None;
    }

    let medians = |sex: &str, classes: &[&'static str]| -> Vec<ClassRow> {
        let mut rows = Vec::new();
        for &wc in classes {
            let mut cells: [Option<(f32, f32)>; 4] = [None; 4];
            let mut any = false;
            for (idx, &lift) in LIFTS.iter().enumerate() {
                if let Some(h) = read_hist(bins, sex, "raw", wc, "all_ages", "all", "kg", lift) {
                    cells[idx] = Some((percentile_value(&h, 0.5), percentile_value(&h, 0.9)));
                    any = true;
                }
            }
            if any {
                rows.push(ClassRow { wc, cells });
            }
        }
        rows
    };

    // Sex comparison: medians for the whole raw population of each sex.
    let mut comparison = Vec::new();
    for &lift in &LIFTS {
        let m = read_hist(bins, "m", "raw", "all", "all_ages", "all", "kg", lift);
        let f = read_hist(bins, "f", "raw", "all", "all_ages", "all", "kg", lift);
        if let (Some(m), Some(f)) = (m, f) {
            let mv = percentile_value(&m, 0.5);
            let fv = percentile_value(&f, 0.5);
            let ratio = if mv != 0.0 { 100.0 * fv / mv } else { 0.0 };
            comparison.push((lift, mv, fv, ratio));
        }
    }
    let dots = |sex: &str| {
        read_hist(bins, sex, "raw", "all", "all_ages", "all", "dots", "total")
            .map(|h| percentile_value(&h, 0.5))
    };

    // Precise dataset size: count individual competition lift results.
    let mut result_count = 0u64;
    for sex in ["m", "f"] {
        for lift in ["squat", "bench", "deadlift"] {
            if let Some(h) = read_hist(bins, sex, "all", "all", "all_ages", "all", "kg", lift) {
                result_count += u64::from(h.total);
            }
        }
    }

    // Worked percentile examples (raw, all ages) for the percentile page.
    let mut examples = Vec::new();
    for (sex, wc, lift, value) in [
        ("m", "83", "squat", 180.0f32),
        ("m", "83", "squat", 230.0),
        ("f", "63", "deadlift", 150.0),
    ] {
        if let Some(h) = read_hist(bins, sex, "raw", wc, "all_ages", "all", "kg", lift) {
            examples.push(Example {
                sex,
                wc,
                lift,
                value,
                pct: value_percentile(&h, value),
            });
        }
    }

    Some(Stats {
        men: medians("m", &MEN_CLASSES),
        women: medians("f", &WOMEN_CLASSES),
        comparison,
        dots_m: dots("m"),
        dots_f: dots("f"),
        result_count,
        examples,
    })
}
