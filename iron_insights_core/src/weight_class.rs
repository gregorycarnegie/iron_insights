/// IPF men's weight class boundaries and their canonical string labels.
///
/// Each entry is `(upper_bound_kg_exclusive, label)`. The final entry uses
/// `f32::INFINITY` to represent the open-ended top class (`"120+"`).
pub const IPF_MALE_WEIGHT_CLASSES: &[(f32, &str)] = &[
    (53.0, "53"),
    (59.0, "59"),
    (66.0, "66"),
    (74.0, "74"),
    (83.0, "83"),
    (93.0, "93"),
    (105.0, "105"),
    (120.0, "120"),
    (f32::INFINITY, "120+"),
];

/// IPF women's weight class boundaries and their canonical string labels.
///
/// Each entry is `(upper_bound_kg_exclusive, label)`. The final entry uses
/// `f32::INFINITY` to represent the open-ended top class (`"84+"`).
pub const IPF_FEMALE_WEIGHT_CLASSES: &[(f32, &str)] = &[
    (43.0, "43"),
    (47.0, "47"),
    (52.0, "52"),
    (57.0, "57"),
    (63.0, "63"),
    (69.0, "69"),
    (76.0, "76"),
    (84.0, "84"),
    (f32::INFINITY, "84+"),
];

/// Returns the IPF weight class label for a given bodyweight (kg) and sex (`"M"` or `"F"`).
///
/// Returns `None` if the sex string is not `"M"` or `"F"`.
///
/// ```
/// use iron_insights_core::ipf_weight_class;
/// assert_eq!(ipf_weight_class(80.0, "M"), Some("83"));
/// assert_eq!(ipf_weight_class(83.0, "M"), Some("83")); // inclusive upper bound
/// assert_eq!(ipf_weight_class(57.0, "F"), Some("57"));
/// assert_eq!(ipf_weight_class(80.0, "X"), None);        // unknown sex
/// ```
pub fn ipf_weight_class(bodyweight_kg: f32, sex: &str) -> Option<&'static str> {
    let classes = match sex {
        "M" => IPF_MALE_WEIGHT_CLASSES,
        "F" => IPF_FEMALE_WEIGHT_CLASSES,
        _ => return None,
    };
    classes
        .iter()
        .find(|(upper, _)| bodyweight_kg <= *upper)
        .map(|(_, label)| *label)
}
