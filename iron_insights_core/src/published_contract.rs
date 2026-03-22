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
    pub hist: String,
    pub heat: String,
}

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
            hist: format!("hist/{base}.bin"),
            heat: format!("heat/{base}.bin"),
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
        assert_eq!(paths.hist, "hist/m/raw/all/all_ages/tested/total.bin");
        assert_eq!(paths.heat, "heat/m/raw/all/all_ages/tested/total.bin");
    }

    #[test]
    fn entry_paths_from_slice_key_includes_metric_directory_when_explicit() {
        let (key, paths) = entry_paths_from_slice_key(
            "sex=F|equip=Raw|wc=63|age=Open|tested=All|lift=B|metric=Lb",
        )
        .expect("paths");

        assert!(key.metric_explicit);
        assert_eq!(paths.meta, "meta/f/raw/63/open/all/lb/bench.json");
        assert_eq!(paths.hist, "hist/f/raw/63/open/all/lb/bench.bin");
        assert_eq!(paths.heat, "heat/f/raw/63/open/all/lb/bench.bin");
    }
}
