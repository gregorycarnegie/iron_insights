use super::SliceIndexEntry;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SliceKey {
    pub(super) sex: String,
    pub(super) equip: String,
    pub(super) wc: String,
    pub(super) age: String,
    pub(super) tested: String,
    pub(super) lift: String,
    pub(super) metric: String,
}

pub(super) fn parse_slice_key(raw: &str) -> Option<SliceKey> {
    let mut sex = None;
    let mut equip = None;
    let mut wc = None;
    let mut age = None;
    let mut tested = None;
    let mut lift = None;
    let mut metric = None;

    for part in raw.split('|') {
        let (k, v) = part.split_once('=')?;
        match k {
            "sex" => sex = Some(v.to_string()),
            "equip" => equip = Some(v.to_string()),
            "wc" => wc = Some(v.to_string()),
            "age" => age = Some(v.to_string()),
            "tested" => tested = Some(v.to_string()),
            "lift" => lift = Some(v.to_string()),
            "metric" => metric = Some(v.to_string()),
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
    })
}

pub(super) fn parse_shard_key(raw: &str) -> Option<(&str, &str)> {
    let mut sex = None;
    let mut equip = None;
    for part in raw.split('|') {
        let (k, v) = part.split_once('=')?;
        match k {
            "sex" => sex = Some(v),
            "equip" => equip = Some(v),
            _ => {}
        }
    }
    Some((sex?, equip?))
}

pub(super) fn entry_from_slice_key(raw: &str) -> Option<(SliceKey, SliceIndexEntry)> {
    let key = parse_slice_key(raw)?;
    let sex_slug = slug(&key.sex);
    let equip_slug = slug(&key.equip);
    let wc_slug = slug(&key.wc);
    let age_slug = slug(&key.age);
    let lift_name = lift_name_from_code(&key.lift)?;
    let tested_dir = tested_dir_from_bucket(&key.tested);
    let has_metric = raw.split('|').any(|part| part.starts_with("metric="));
    let base = if has_metric {
        let metric_dir = slug(&key.metric);
        format!("{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested_dir}/{metric_dir}/{lift_name}")
    } else {
        format!("{sex_slug}/{equip_slug}/{wc_slug}/{age_slug}/{tested_dir}/{lift_name}")
    };

    Some((
        key,
        SliceIndexEntry {
            meta: format!("meta/{base}.json"),
            hist: format!("hist/{base}.bin"),
            heat: format!("heat/{base}.bin"),
        },
    ))
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
