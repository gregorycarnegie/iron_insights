use crate::webapp::models::SliceIndexEntry;

pub(super) use crate::core::{SliceKey, parse_shard_key, parse_slice_key};

pub(super) fn entry_from_slice_key(raw: &str) -> Option<(SliceKey, SliceIndexEntry)> {
    let (key, paths) = crate::core::entry_paths_from_slice_key(raw)?;

    Some((
        key,
        SliceIndexEntry {
            meta: paths.meta,
            hist: paths.hist,
            heat: paths.heat,
            summary: None,
        },
    ))
}
