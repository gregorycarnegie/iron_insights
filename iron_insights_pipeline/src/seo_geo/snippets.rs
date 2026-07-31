use super::{
    constants::LIFT_LABELS,
    stats::{ClassRow, Stats},
};

pub(super) fn wc_label(wc: &str) -> String {
    match wc.strip_suffix('_') {
        Some(prefix) => format!("{prefix} kg+"),
        None => format!("{wc} kg"),
    }
}

fn lift_label(lift: &str) -> &'static str {
    LIFT_LABELS
        .iter()
        .find(|(k, _)| *k == lift)
        .map(|(_, v)| *v)
        .unwrap_or("")
}

fn standards_table(rows: &[ClassRow], sex_label: &str) -> String {
    let mut body = String::new();
    for row in rows {
        body.push_str(&format!("<tr><td>{}</td>", wc_label(row.wc)));
        for cell in &row.cells {
            match cell {
                Some((p50, p90)) => body.push_str(&format!("<td>{p50:.0} / {p90:.0}</td>")),
                None => body.push_str("<td>&mdash;</td>"),
            }
        }
        body.push_str("</tr>");
    }
    format!(
        "<p><strong>{sex_label}, raw, all ages</strong> &mdash; median (p50) / \
strong-lifter (p90), kilograms:</p><table><thead><tr><th>Weight class</th>\
<th>Squat</th><th>Bench</th><th>Deadlift</th><th>Total</th></tr></thead>\
<tbody>{body}</tbody></table>"
    )
}

pub(super) fn standards_tables_html(stats: &Stats) -> String {
    if stats.men.is_empty() {
        return String::new();
    }
    format!(
        "{}{}",
        standards_table(&stats.men, "Men"),
        standards_table(&stats.women, "Women")
    )
}

pub(super) fn comparison_table_html(stats: &Stats) -> String {
    if stats.comparison.is_empty() {
        return String::new();
    }
    let mut rows = String::new();
    for (lift, m, f, ratio) in &stats.comparison {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{m:.0} kg</td><td>{f:.0} kg</td><td>{ratio:.0}%</td></tr>",
            lift_label(lift)
        ));
    }
    let table = format!(
        "<table><thead><tr><th>Lift</th><th>Men (median)</th><th>Women (median)</th>\
<th>Female &divide; male</th></tr></thead><tbody>{rows}</tbody></table>"
    );
    let note = match (stats.dots_m, stats.dots_f) {
        (Some(m), Some(f)) => {
            let pct = if m != 0.0 { 100.0 * f / m } else { 0.0 };
            format!(
                "<p>Raw kilograms understate women's relative strength because the median \
female lifter is lighter. On bodyweight-adjusted <strong>DOTS</strong> points the gap \
narrows sharply: the median male total scores {m:.0} DOTS versus {f:.0} for women \
&mdash; about {pct:.0}%.</p>"
            )
        }
        _ => String::new(),
    };
    format!(
        "{table}<p>Across the whole raw population the gap is widest in the bench press \
and narrowest in the deadlift.</p>{note}"
    )
}

pub(super) fn ordinal(n: i64) -> String {
    let suffix = if (11..=13).contains(&(n % 100)) {
        "th"
    } else {
        match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{n}{suffix}")
}

pub(super) fn percentile_examples_html(stats: &Stats) -> String {
    if stats.examples.is_empty() {
        return String::new();
    }
    let mut items = String::new();
    for ex in &stats.examples {
        let sex_word = if ex.sex == "m" { "male" } else { "female" };
        items.push_str(&format!(
            "<li>A <strong>{} {sex_word}</strong> lifter (raw, all ages) with a {:.0} kg {} \
sits at roughly the <strong>{} percentile</strong> of that cohort.</li>",
            wc_label(ex.wc),
            ex.value,
            ex.lift,
            ordinal(ex.pct.round() as i64),
        ));
    }
    format!("<p>Worked examples from the live data:</p><ul>{items}</ul>")
}
