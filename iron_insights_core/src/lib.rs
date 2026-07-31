mod binary;
mod bodyfat;
mod histogram;
mod one_rep_max;
mod plates;
mod published_contract;
mod rebin;
mod scoring;
#[cfg(test)]
mod tests;
mod tiers;
mod unit_conversion;
mod weight_class;

pub use binary::{
    BINARY_FORMAT_VERSION, COMBINED_MAGIC, HEATMAP_MAGIC, HISTOGRAM_MAGIC, HeatmapBin,
    HistogramBin, decode_counts, encode_counts, parse_combined_bin,
};
pub use bodyfat::{
    BodyfatResult, JacksonPollock7SiteSkinfolds, bodyfat_category, calc_bodyfat_female,
    calc_bodyfat_jp3, calc_bodyfat_jp7, calc_bodyfat_male, calc_bodyfat_ymca,
};
pub use histogram::{
    equivalent_value_for_same_percentile, percentile_for_value, value_for_percentile,
};
pub use one_rep_max::calc_1rm;
pub use plates::{IPF_PLATES_KG, plates_per_side};
pub use published_contract::{SliceKey, parse_shard_key, parse_slice_key, slug};
pub use rebin::{rebin_1d, rebin_2d};
pub use scoring::{dots_points, goodlift_points, wilks_points};
pub use tiers::tier_for_percentile;
pub use unit_conversion::KG_PER_LB;
pub use weight_class::{IPF_FEMALE_WEIGHT_CLASSES, IPF_MALE_WEIGHT_CLASSES, ipf_weight_class};
