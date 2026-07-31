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

fn parse_key_part(part: &str) -> Option<(&str, &str)> {
    let (key, value) = part.split_once('=')?;
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some((key, value))
}

/// Lowercases ASCII letters and replaces every other non-`[a-z0-9-]` character
/// with `_`, producing the path segments used throughout the published layout.
///
/// ```
/// use iron_insights_core::slug;
/// assert_eq!(slug("All Ages"), "all_ages");
/// assert_eq!(slug("120+"), "120_");
/// assert_eq!(slug("Single-ply"), "single-ply");
/// ```
pub fn slug(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' => c.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' | '-' => c,
            _ => '_',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_shard_key, parse_slice_key};
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
        fn slug_only_emits_path_safe_characters(raw in field_value()) {
            let slugged = super::slug(&raw);

            prop_assert!(slugged.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'));
            prop_assert_eq!(slugged.chars().count(), raw.chars().count());
        }
    }
}
