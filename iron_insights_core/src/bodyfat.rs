/// Contains the calculated body fat percentage and the resulting fat and lean mass estimates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyfatResult {
    pub body_fat_pct: f32,
    pub lean_mass_kg: f32,
    pub fat_mass_kg: f32,
}

/// Seven skinfold measurements used by the Jackson-Pollock 7-site method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JacksonPollock7SiteSkinfolds {
    pub chest_mm: f32,
    pub midaxillary_mm: f32,
    pub tricep_mm: f32,
    pub subscapular_mm: f32,
    pub abdomen_mm: f32,
    pub suprailiac_mm: f32,
    pub thigh_mm: f32,
}

impl JacksonPollock7SiteSkinfolds {
    fn all_positive(self) -> bool {
        self.chest_mm > 0.0
            && self.midaxillary_mm > 0.0
            && self.tricep_mm > 0.0
            && self.subscapular_mm > 0.0
            && self.abdomen_mm > 0.0
            && self.suprailiac_mm > 0.0
            && self.thigh_mm > 0.0
    }

    fn sum(self) -> f32 {
        self.chest_mm
            + self.midaxillary_mm
            + self.tricep_mm
            + self.subscapular_mm
            + self.abdomen_mm
            + self.suprailiac_mm
            + self.thigh_mm
    }
}

/// Estimates body fat percentage for males using the US Navy method.
pub fn calc_bodyfat_male(
    height_cm: f32,
    weight_kg: f32,
    neck_cm: f32,
    waist_cm: f32,
) -> Option<BodyfatResult> {
    if height_cm <= 0.0 || neck_cm <= 0.0 || waist_cm <= neck_cm {
        return None;
    }
    let diff = waist_cm - neck_cm;
    let bf = 495.0
        / (1.0324 - 0.19077 * (diff as f64).log10() as f32
            + 0.15456 * (height_cm as f64).log10() as f32)
        - 450.0;
    let bf = bf.clamp(2.0, 60.0);
    let fat_mass = weight_kg * bf / 100.0;
    Some(BodyfatResult {
        body_fat_pct: bf,
        lean_mass_kg: weight_kg - fat_mass,
        fat_mass_kg: fat_mass,
    })
}

/// Estimates body fat percentage for females using the US Navy method.
pub fn calc_bodyfat_female(
    height_cm: f32,
    weight_kg: f32,
    neck_cm: f32,
    waist_cm: f32,
    hip_cm: f32,
) -> Option<BodyfatResult> {
    if height_cm <= 0.0 || neck_cm <= 0.0 {
        return None;
    }
    let diff = waist_cm + hip_cm - neck_cm;
    if diff <= 0.0 {
        return None;
    }
    let bf = 495.0
        / (1.29579 - 0.35004 * (diff as f64).log10() as f32
            + 0.22100 * (height_cm as f64).log10() as f32)
        - 450.0;
    let bf = bf.clamp(8.0, 60.0);
    let fat_mass = weight_kg * bf / 100.0;
    Some(BodyfatResult {
        body_fat_pct: bf,
        lean_mass_kg: weight_kg - fat_mass,
        fat_mass_kg: fat_mass,
    })
}

// ===== BODYFAT (ADDITIONAL METHODS) =====

/// Siri equation: converts body density (g/cc) to body fat percentage.
pub(crate) fn siri_bf_from_density(bd: f32) -> f32 {
    495.0 / bd - 450.0
}

fn bodyfat_min_for_sex(is_male: bool) -> f32 {
    if is_male { 2.0 } else { 8.0 }
}

fn make_bodyfat_result(bf_pct: f32, weight_kg: f32, is_male: bool) -> BodyfatResult {
    let bf = bf_pct.clamp(bodyfat_min_for_sex(is_male), 60.0);
    let fat_mass = weight_kg * bf / 100.0;
    BodyfatResult {
        body_fat_pct: bf,
        lean_mass_kg: weight_kg - fat_mass,
        fat_mass_kg: fat_mass,
    }
}

/// Estimates body fat percentage using the YMCA (Wallace-Ross) method.
///
/// The formula is imperial-native (waist in inches, weight in pounds); inputs
/// are converted internally so callers can stay metric.
pub fn calc_bodyfat_ymca(weight_kg: f32, waist_cm: f32, is_male: bool) -> Option<BodyfatResult> {
    if weight_kg <= 0.0 || waist_cm <= 0.0 {
        return None;
    }
    let weight_lb = weight_kg * 2.204_622_5;
    let waist_in = waist_cm / 2.54;
    let intercept = if is_male { -98.42 } else { -76.76 };
    let bf = (intercept + 4.15 * waist_in - 0.082 * weight_lb) / weight_lb * 100.0;
    Some(make_bodyfat_result(bf, weight_kg, is_male))
}

/// Estimates body fat percentage using the Jackson-Pollock 3-site skinfold method.
///
/// Site ordering is sex-specific:
/// - Male: `site_a` = chest, `site_b` = abdomen, `site_c` = thigh
/// - Female: `site_a` = tricep, `site_b` = suprailiac, `site_c` = thigh
pub fn calc_bodyfat_jp3(
    age_years: f32,
    weight_kg: f32,
    is_male: bool,
    site_a_mm: f32,
    site_b_mm: f32,
    site_c_mm: f32,
) -> Option<BodyfatResult> {
    if age_years <= 0.0
        || age_years > 120.0
        || weight_kg <= 0.0
        || site_a_mm <= 0.0
        || site_b_mm <= 0.0
        || site_c_mm <= 0.0
    {
        return None;
    }
    let sum = site_a_mm + site_b_mm + site_c_mm;
    let bd = if is_male {
        1.109_38 - 0.000_826_7 * sum + 0.000_001_6 * sum * sum - 0.000_257_4 * age_years
    } else {
        1.099_492_1 - 0.000_992_9 * sum + 0.000_002_3 * sum * sum - 0.000_139_2 * age_years
    };
    if bd <= 0.0 {
        return None;
    }
    Some(make_bodyfat_result(
        siri_bf_from_density(bd),
        weight_kg,
        is_male,
    ))
}

/// Estimates body fat percentage using the Jackson-Pollock 7-site skinfold method.
pub fn calc_bodyfat_jp7(
    age_years: f32,
    weight_kg: f32,
    is_male: bool,
    skinfolds: JacksonPollock7SiteSkinfolds,
) -> Option<BodyfatResult> {
    if age_years <= 0.0 || age_years > 120.0 || weight_kg <= 0.0 || !skinfolds.all_positive() {
        return None;
    }
    let sum = skinfolds.sum();
    let bd = if is_male {
        1.112 - 0.000_434_99 * sum + 0.000_000_55 * sum * sum - 0.000_288_26 * age_years
    } else {
        1.097 - 0.000_469_71 * sum + 0.000_000_56 * sum * sum - 0.000_128_28 * age_years
    };
    if bd <= 0.0 {
        return None;
    }
    Some(make_bodyfat_result(
        siri_bf_from_density(bd),
        weight_kg,
        is_male,
    ))
}

/// Returns a descriptive body fat category based on percentage and sex.
///
/// ```
/// use iron_insights_core::bodyfat_category;
/// assert_eq!(bodyfat_category(4.0, true),  "Essential");
/// assert_eq!(bodyfat_category(8.0, true),  "Elite Athlete");
/// assert_eq!(bodyfat_category(22.0, true), "Average");
/// assert_eq!(bodyfat_category(12.0, false), "Essential");
/// assert_eq!(bodyfat_category(35.0, false), "Obese");
/// ```
pub fn bodyfat_category(pct: f32, is_male: bool) -> &'static str {
    if is_male {
        if pct < 6.0 {
            "Essential"
        } else if pct < 11.0 {
            "Elite Athlete"
        } else if pct < 15.0 {
            "Athlete"
        } else if pct < 20.0 {
            "Fitness"
        } else if pct < 25.0 {
            "Average"
        } else {
            "Obese"
        }
    } else if pct < 14.0 {
        "Essential"
    } else if pct < 18.0 {
        "Elite Athlete"
    } else if pct < 22.0 {
        "Athlete"
    } else if pct < 26.0 {
        "Fitness"
    } else if pct < 32.0 {
        "Average"
    } else {
        "Obese"
    }
}
