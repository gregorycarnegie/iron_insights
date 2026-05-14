#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SliceKey {
    pub sex: String,
    pub equip: String,
    pub wc: String,
    pub age: String,
    pub tested: String,
    pub lift: String,
    pub metric: String,
    pub metric_explicit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceEntryPaths {
    pub meta: String,
    pub bin: String,
}

/// Parses a pipe-separated slice key string into a [`SliceKey`].
///
/// Fields are order-independent. `metric` defaults to `"Kg"` when omitted.
/// Returns `None` if any required field (`sex`, `equip`, `wc`, `age`, `tested`, `lift`)
/// is missing or if a segment has no `=` separator.
///
/// ```
/// use iron_insights_core::parse_slice_key;
/// let key = parse_slice_key("sex=M|equip=Raw|wc=93|age=Open|tested=Yes|lift=T").unwrap();
/// assert_eq!(key.sex, "M");
/// assert_eq!(key.metric, "Kg"); // default when omitted
/// assert!(parse_slice_key("sex=M|equip=Raw").is_none()); // missing required fields
/// ```
pub fn parse_slice_key(raw: &str) -> Option<SliceKey> {
    let mut sex = None;
    let mut equip = None;
    let mut wc = None;
    let mut age = None;
    let mut tested = None;
    let mut lift = None;
    let mut metric = None;
    let mut metric_explicit = false;

    for part in raw.split('|') {
        let (k, v) = parse_key_part(part)?;
        match k {
            "sex" => sex = Some(v.to_string()),
            "equip" => equip = Some(v.to_string()),
            "wc" => wc = Some(v.to_string()),
            "age" => age = Some(v.to_string()),
            "tested" => tested = Some(v.to_string()),
            "lift" => lift = Some(v.to_string()),
            "metric" => {
                metric = Some(v.to_string());
                metric_explicit = true;
            }
            _ => {}
        }
    }

    Some(SliceKey {
        sex: sex?,
        equip: equip?,
        wc: wc?,
        age: age?,
        tested: tested?,
        lift: lift?,
        metric: metric.unwrap_or_else(|| "Kg".to_string()),
        metric_explicit,
    })
}

/// Extracts `(sex, equip)` from a pipe-separated key string.
///
/// Returns `None` if either field is absent.
///
/// ```
/// use iron_insights_core::parse_shard_key;
/// let (sex, equip) = parse_shard_key("sex=F|equip=Raw|wc=63").unwrap();
/// assert_eq!(sex, "F");
/// assert_eq!(equip, "Raw");
/// assert!(parse_shard_key("sex=M").is_none()); // missing equip
/// ```
pub fn parse_shard_key(raw: &str) -> Option<(&str, &str)> {
    let mut sex = None;
    let mut equip = None;

    for part in raw.split('|') {
        let (k, v) = parse_key_part(part)?;
        match k {
            "sex" => sex = Some(v),
            "equip" => equip = Some(v),
            _ => {}
        }
    }

    Some((sex?, equip?))
}

pub fn entry_paths_from_slice_key(raw: &str) -> Option<(SliceKey, SliceEntryPaths)> {
    let key = parse_slice_key(raw)?;
    let base = payload_base_path_from_slice_key(&key)?;

    Some((
        key,
        SliceEntryPaths {
            meta: format!("meta/{base}.json"),
            bin: format!("bin/{base}.bin"),
        },
    ))
}

fn payload_base_path_from_slice_key(key: &SliceKey) -> Option<String> {
    let sex_slug = slug(&key.sex);
    let equip_slug = slug(&key.equip);
    let wc_slug = slug(&key.wc);
    let age_slug = slug(&key.age);
    let lift_name = lift_name_from_code(&key.lift)?;
    let tested_dir = tested_dir_from_bucket(&key.tested);

    if key.metric_explicit {
        let metric_dir = slug(&key.metric);
        Some(format!(
            "{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested_dir}/{metric_dir}/{lift_name}"
        ))
    } else {
        Some(format!(
            "{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested_dir}/{lift_name}"
        ))
    }
}

fn parse_key_part(part: &str) -> Option<(&str, &str)> {
    let (key, value) = part.split_once('=')?;
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some((key, value))
}

fn slug(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' => c.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' | '-' => c,
            _ => '_',
        })
        .collect()
}

fn lift_name_from_code(code: &str) -> Option<&'static str> {
    match code {
        "S" => Some("squat"),
        "B" => Some("bench"),
        "D" => Some("deadlift"),
        "T" => Some("total"),
        _ => None,
    }
}

fn tested_dir_from_bucket(bucket: &str) -> String {
    if bucket.eq_ignore_ascii_case("yes") {
        "tested".to_string()
    } else {
        slug(bucket)
    }
}

#[cfg(test)]
mod tests {
    use super::{entry_paths_from_slice_key, parse_shard_key, parse_slice_key};
    use proptest::prelude::*;

    #[test]
    fn parse_slice_key_defaults_metric_to_kg() {
        let key = parse_slice_key("sex=M|equip=Raw|wc=93|age=Open|tested=Yes|lift=T").expect("key");

        assert_eq!(key.sex, "M");
        assert_eq!(key.equip, "Raw");
        assert_eq!(key.wc, "93");
        assert_eq!(key.age, "Open");
        assert_eq!(key.tested, "Yes");
        assert_eq!(key.lift, "T");
        assert_eq!(key.metric, "Kg");
        assert!(!key.metric_explicit);
    }

    #[test]
    fn parse_slice_key_rejects_malformed_segment() {
        assert!(
            parse_slice_key("sex=M|equip=Raw|broken|wc=93|age=Open|tested=Yes|lift=T").is_none()
        );
    }

    #[test]
    fn parse_slice_key_rejects_empty_values() {
        assert!(parse_slice_key("sex=M|equip=Raw|wc=|age=Open|tested=Yes|lift=T").is_none());
    }

    #[test]
    fn parse_shard_key_extracts_sex_and_equipment() {
        let (sex, equip) =
            parse_shard_key("equip=Raw|sex=F|ignored=value").expect("shard should parse");

        assert_eq!(sex, "F");
        assert_eq!(equip, "Raw");
    }

    #[test]
    fn entry_paths_from_slice_key_omits_metric_directory_for_legacy_keys() {
        let (_, paths) =
            entry_paths_from_slice_key("sex=M|equip=Raw|wc=All|age=All Ages|tested=Yes|lift=T")
                .expect("paths");

        assert_eq!(paths.meta, "meta/m/raw/all/all_ages/tested/total.json");
        assert_eq!(paths.bin, "bin/m/raw/all/all_ages/tested/total.bin");
    }

    #[test]
    fn entry_paths_from_slice_key_includes_metric_directory_when_explicit() {
        let (key, paths) = entry_paths_from_slice_key(
            "sex=F|equip=Raw|wc=63|age=Open|tested=All|lift=B|metric=Lb",
        )
        .expect("paths");

        assert!(key.metric_explicit);
        assert_eq!(paths.meta, "meta/f/raw/63/open/all/lb/bench.json");
        assert_eq!(paths.bin, "bin/f/raw/63/open/all/lb/bench.bin");
    }

    fn field_value() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::sample::select(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 _./:+-"
                    .chars()
                    .collect::<Vec<_>>(),
            ),
            1..25,
        )
        .prop_map(|chars| chars.into_iter().collect::<String>())
    }

    fn lift_code() -> impl Strategy<Value = &'static str> {
        prop::sample::select(vec!["S", "B", "D", "T"])
    }

    fn invalid_lift_code() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::sample::select(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-"
                    .chars()
                    .collect::<Vec<_>>(),
            ),
            1..13,
        )
        .prop_map(|chars| chars.into_iter().collect::<String>())
        .prop_filter("reserved valid lift code", |code| {
            !matches!(code.as_str(), "S" | "B" | "D" | "T")
        })
    }

    fn required_segments(
        sex: &str,
        equip: &str,
        wc: &str,
        age: &str,
        tested: &str,
        lift: &str,
    ) -> Vec<String> {
        vec![
            format!("sex={sex}"),
            format!("equip={equip}"),
            format!("wc={wc}"),
            format!("age={age}"),
            format!("tested={tested}"),
            format!("lift={lift}"),
        ]
    }

    proptest! {
        #[test]
        fn parse_slice_key_is_order_independent_for_complete_keys(
            sex in field_value(),
            equip in field_value(),
            wc in field_value(),
            age in field_value(),
            tested in field_value(),
            lift in lift_code(),
            metric in prop::option::of(field_value()),
        ) {
            let mut segments = required_segments(&sex, &equip, &wc, &age, &tested, lift);
            if let Some(metric) = &metric {
                segments.push(format!("metric={metric}"));
            }

            let forward = segments.join("|");
            segments.reverse();
            let reversed = segments.join("|");

            prop_assert_eq!(parse_slice_key(&forward), parse_slice_key(&reversed));
        }

        #[test]
        fn parse_slice_key_rejects_missing_required_fields(
            sex in field_value(),
            equip in field_value(),
            wc in field_value(),
            age in field_value(),
            tested in field_value(),
            lift in lift_code(),
            missing_index in 0usize..6,
        ) {
            let mut segments = required_segments(&sex, &equip, &wc, &age, &tested, lift);
            segments.remove(missing_index);

            prop_assert!(parse_slice_key(&segments.join("|")).is_none());
        }

        #[test]
        fn entry_paths_keep_meta_and_bin_on_same_payload_base(
            sex in field_value(),
            equip in field_value(),
            wc in field_value(),
            age in field_value(),
            tested in field_value(),
            lift in lift_code(),
            metric in prop::option::of(field_value()),
        ) {
            let mut segments = required_segments(&sex, &equip, &wc, &age, &tested, lift);
            let metric_explicit = metric.is_some();
            if let Some(metric) = &metric {
                segments.push(format!("metric={metric}"));
            }

            let (_, paths) = entry_paths_from_slice_key(&segments.join("|")).expect("paths");
            let meta_base = paths
                .meta
                .strip_prefix("meta/")
                .and_then(|path| path.strip_suffix(".json"))
                .expect("meta path shape");
            let bin_base = paths
                .bin
                .strip_prefix("bin/")
                .and_then(|path| path.strip_suffix(".bin"))
                .expect("bin path shape");

            prop_assert_eq!(meta_base, bin_base);
            prop_assert_eq!(meta_base.split('/').count(), if metric_explicit { 7 } else { 6 });
            prop_assert!(!meta_base.contains('|'));
            prop_assert!(!meta_base.contains('\\'));
        }

        #[test]
        fn entry_paths_reject_unknown_lift_codes(
            sex in field_value(),
            equip in field_value(),
            wc in field_value(),
            age in field_value(),
            tested in field_value(),
            lift in invalid_lift_code(),
        ) {
            let raw = required_segments(&sex, &equip, &wc, &age, &tested, &lift).join("|");

            prop_assert!(parse_slice_key(&raw).is_some());
            prop_assert!(entry_paths_from_slice_key(&raw).is_none());
        }
    }
}
