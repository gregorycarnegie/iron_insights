use std::collections::BTreeSet;
use web_sys::HtmlInputElement;

pub(super) fn unique(items: impl Iterator<Item = String>) -> Vec<String> {
    let mut set = BTreeSet::new();
    for item in items {
        set.insert(item);
    }
    set.into_iter().collect()
}

pub(super) fn pick_preferred(options: &[String], preferred: &str) -> String {
    if options.is_empty() {
        return String::new();
    }
    if let Some(v) = options.iter().find(|v| v.as_str() == preferred) {
        return v.clone();
    }
    options[0].clone()
}

pub(super) fn ipf_class_sort_key(class: &str) -> (u8, i32) {
    if let Some(prefix) = class.strip_suffix('+')
        && let Ok(v) = prefix.parse::<i32>()
    {
        return (1, v);
    }
    if let Ok(v) = class.parse::<i32>() {
        return (0, v);
    }
    (2, i32::MAX)
}

pub(super) fn metric_label(code: &str) -> &'static str {
    match code {
        "Dots" => "DOTS",
        "Wilks" => "Wilks",
        "GL" => "GL",
        _ => "Kg",
    }
}

pub(super) fn age_class_sort_key(class: &str) -> (u8, i32) {
    if class == "All Ages" {
        return (0, -1);
    }
    let start = class
        .split(['-', '+'])
        .next()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(i32::MAX);
    (1, start)
}

pub(super) fn parse_f32_input(ev: &web_sys::Event) -> f32 {
    leptos::prelude::event_target::<HtmlInputElement>(ev)
        .value()
        .parse::<f32>()
        .unwrap_or(0.0)
}
