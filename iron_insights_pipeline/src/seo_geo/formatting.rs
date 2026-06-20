const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub(super) fn human_date(iso: &str) -> String {
    let parts: Vec<&str> = iso.split('-').collect();
    let [y, m, d] = parts[..] else {
        return iso.to_string();
    };
    let (Ok(month), Ok(day)) = (m.parse::<usize>(), d.parse::<u32>()) else {
        return iso.to_string();
    };
    if (1..=12).contains(&month) {
        format!("{day} {} {y}", MONTHS[month - 1])
    } else {
        iso.to_string()
    }
}

pub(super) fn group_thousands(n: u64) -> String {
    let digits = n.to_string();
    let bytes = digits.as_bytes();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}
