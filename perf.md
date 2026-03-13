# Performance Optimisation Findings

This is a corrected pass over the current code. Items below are split between:

- confirmed runtime hotspots or avoidable repeated work
- smaller allocation/planning cleanups
- places where the original note was directionally interesting but overstated

Tracking:

- Mark `Done` when a fix has been implemented.
- Mark `Unnecessary` when you decide the change is not worth doing.
- Leave both unchecked while the item is still open.

## Pipeline (`pipeline/src/`)

### HIGH PRIORITY

#### 1. Excessive String Allocations in Slice Key Generation

- [x] Done
- [ ] Unnecessary

**File:** `pipeline/src/bin/03_publish_data.rs` — Lines 300–401

Inside the per-row loop (`for i in 0..df.height()`), each accepted row converts borrowed `&str` values into owned `String`s and then clones those values across 8 roll-up keys. That creates dozens of string allocations per row before any histogram work happens.

```rust
let sex = sex.to_string();
let equipment = equipment.to_string();
let weight_class = weight_class.to_string();
let age_class = age_class.to_string();

let keys = [
    (sex.clone(), equipment.clone(), weight_class.clone(), age_class.clone()),
    (sex.clone(), "All".to_string(), weight_class.clone(), age_class.clone()),
    // ...
];
```

**Fix:** Keep slice keys borrowed for as long as `df` is alive, or intern repeated sentinels such as `"All"` / `"All Ages"` instead of allocating new `String`s every iteration.

---

#### 2. Repeated Parquet Scans for the Same `(tested, lift)` Input

- [x] Done
- [ ] Unnecessary

**File:** `pipeline/src/bin/03_publish_data.rs` — Lines 182–206, 270–274, 806–812

The current loop calls `publish_records_for_lift` once per metric. For `lift == "total"`, that means scanning and collecting the same Parquet file 4 times: `Kg`, `Dots`, `Wilks`, and `GL`.

```rust
for metric in metrics_for_lift(lift) {
    publish_records_for_lift(/* scans parquet each time */)?;
}
```

This is the real issue in that area. The combinations are not currently "independent" in a way that makes naive parallelisation safe, because each call mutates shared `shard_indices` and `trends_acc`.

**Fix:** Scan once per `(tested, lift)` and emit all metrics from the same materialised frame. Only consider parallelising after the shared accumulation/writing path has been refactored.

---

#### 3. `BTreeMap` for Slice Accumulation

- [x] Done
- [ ] Unnecessary

**File:** `pipeline/src/bin/03_publish_data.rs` — Line 300

```rust
let mut slices = BTreeMap::<(String, String, String, String), SliceAccumulator>::new();
```

Accumulation does not need sorted order, so `BTreeMap` is paying ordered-key comparison costs during the hottest write-heavy phase of the function.

**Fix:** Switch this accumulation map to `HashMap`/`FxHashMap`, then sort only at the serialization boundary if stable output order still matters there.

---

### MEDIUM PRIORITY

#### 4. Projection Push-Down Opportunity Before `.collect()`

- [x] Done
- [ ] Unnecessary

**File:** `pipeline/src/bin/03_publish_data.rs` — Lines 270–298

The problem here is not merely that the frame is collected eagerly. The bigger missed optimisation is that the scan is collected without first narrowing the column set, so Polars has to materialise more data than this function actually uses.

```rust
let df = LazyFrame::scan_parquet(...)
    .collect()?;
```

This function only reads:

- `Sex`
- `Equipment`
- `IpfWeightClass`
- `AgeClassBucket`
- `best_lift`
- `bodyweight_at_best`
- `date_at_best`

**Fix:** Add a `.select([...])` before `.collect()` so Parquet projection push-down can discard unused columns.

---

#### 5. Trend-Key `format!()` Allocations Inside the Row Loop

- [x] Done
- [ ] Unnecessary

**File:** `pipeline/src/bin/03_publish_data.rs` — Lines 326–341, 442–486

Every row builds trend keys with `format!()`, even though most of the key is fixed for the current function call (`tested`, `lift`, `metric`).

**Fix:** Precompute the invariant pieces outside the row loop and only splice in the per-row fields that actually vary (`sex`, equipment roll-up).

---

### LOW PRIORITY

#### 6. Multi-Pass Histogram / Heatmap Builds

- [ ] Done
- [ ] Unnecessary

**File:** `pipeline/src/bin/03_publish_data.rs` — Lines 563–643

`build_histogram` and `build_heatmap` each do a bounds pass and then a fill pass. That is expected with the current fixed-bin design because bin edges must be known before the counts/grid can be allocated.

So the original "combine into a single pass" note was too simple. A true one-pass version would need either:

- temporary buffering plus a later fill step anyway, or
- a dynamically growing/rebucketing structure

**Fix:** Leave this alone unless profiling shows it matters. If it does, look for a design change rather than a mechanical loop fusion.

---

#### 7. Polars `Expr` Cloning in Weight-Class / Age-Class Derivation

- [ ] Done
- [ ] Unnecessary

**File:** `pipeline/src/bin/02_build_aggregates.rs` — Lines 172–250

There are many `bw.clone()` / `age.clone()` calls while building the Polars expressions, but this happens during expression construction, not inside the per-row publish loop.

That makes this much more of a planning-time/readability cleanup than a top runtime hotspot.

**Fix:** If desired, reduce the repetition for readability. Do not treat this as a priority optimisation without measurement.

---

#### 8. `normalize_version` Takes Owned `String`

- [x] Done
- [ ] Unnecessary

**File:** `pipeline/src/bin/03_publish_data.rs` — Lines 703–718

```rust
let normalized = normalize_version(meta.dataset_version.clone());
```

This is a small avoidable allocation.

**Fix:** Change the signature to `fn normalize_version(version: &str) -> String`.

---

#### 9. Path-to-String Conversions in Download Metadata

- [ ] Done
- [ ] Unnecessary

**File:** `pipeline/src/bin/01_download.rs` — Lines 41–58

`display().to_string()` here is not a meaningful performance issue. At most, this is a small cleanup/consistency point while assembling metadata strings.

**Fix:** Optional only. If touched, prefer a single consistent path-to-string approach such as `to_string_lossy().into_owned()`.

---

## App (`app/src/`)

### MEDIUM PRIORITY

#### 10. Repeated Full Scans in Selector Memos

- [x] Done
- [ ] Unnecessary

**File:** `app/src/webapp/selectors.rs` — Lines 6–164

The selector memos independently rescan `slice_rows` with overlapping predicates. That can turn one selection change into several linear passes over the same vector.

The original note was slightly too absolute: this is not literally "N full scans on every state change", but it can become several full scans on relevant selection changes.

**Fix:** If `slice_rows` is large enough for this to matter, pre-index by the common dimensions or derive multiple option lists from a shared filtered view.

---

#### 11. Histogram Totals Are Recomputed in Percentile Helpers

- [x] Done
- [ ] Unnecessary

**File:** `app/src/core.rs` — Lines 156–205

`percentile_for_value` and `value_for_percentile` each sum `hist.counts` to get the total, even though the histogram is immutable once parsed.

```rust
let total: u32 = hist.counts.iter().copied().sum();
```

**Fix:** Cache the total on a parsed histogram wrapper or store it on `HistogramBin` after parsing. If you want it on disk too, that becomes a format change rather than a local refactor.

---

#### 12. `histogram_diagnostics` Does 9 Separate Percentile Scans

- [x] Done
- [ ] Unnecessary

**File:** `app/src/core.rs` — Lines 216–275

`histogram_diagnostics` calls `value_for_percentile` 9 times (`p01` through `p99`), so it repeatedly walks the same histogram bins.

**Fix:** Batch the quantile lookups in one pass over `counts` if diagnostics becomes a measurable hotspot.

---

#### 13. Duplicate Trend Memos

- [x] Done
- [ ] Unnecessary

**File:** `app/src/webapp/components/trends.rs` — Lines 43–138

Several memos call `trend_points.get()` and recompute related ranges/summaries independently:

- `total_path`
- `total_ticks`
- `p50_path`
- `p90_path`
- `pct_ticks`
- summary memos based on first/last points

**Fix:** If trend rendering shows up in profiles, consolidate the derived chart state into one memo-backed struct.

---

### LOW PRIORITY

#### 14. `BTreeSet` in `unique()` Is Not a Free `HashSet` Swap

- [ ] Done
- [ ] Unnecessary

**File:** `app/src/webapp/ui.rs` — Lines 1–10

The earlier recommendation to replace `BTreeSet` with `HashSet` was incomplete. `unique()` currently returns values in sorted order, and several callers rely on that deterministic output without re-sorting afterwards.

**Fix:** Keep the current behaviour unless you also reintroduce sorting after deduplication. This is not a drop-in optimisation.

---

#### 15. `age_label()` Returns Owned `String` for Mostly Static Values

- [ ] Done
- [ ] Unnecessary

**File:** `app/src/webapp/ui.rs` — Lines 54–74

Most branches allocate a fresh `String` from a literal. This is real, but minor.

**Fix:** Return `Cow<'static, str>` if you want to remove those literal allocations without making the dynamic branches awkward.

---

#### 16. SVG `format!()` / `.to_string()` Allocations on Render

- [ ] Done
- [ ] Unnecessary

**File:** `app/src/webapp/charts.rs` — Lines 6–64

The histogram SVG renderer creates many short-lived `String`s for numeric attributes during render.

This is a small allocation hotspot, not a major architectural problem.

**Fix:** Prefer numeric attributes where Leptos supports them, or precompute stable labels/positions when it improves clarity.

---

#### 17. Trends and Heatmap Data Are Already Lazy-Loaded at the Page Level

- [ ] Done
- [ ] Unnecessary

**File:** `app/src/webapp/mod.rs` — Lines 312–337, 545–556  
**File:** `app/src/webapp/state.rs` — Lines 152–203, 205–310

The original note was inaccurate. The current app already gates:

- trends fetches behind `nerds_page_active`
- histogram/heatmap fetches behind `calculated`
- heatmap fetches behind `nerds_page_active`

So this is not an unconditional eager-load problem today.

**Fix:** If needed, the remaining improvement is panel-level lazy loading within the Stats for Nerds page, not basic request gating.

---

#### 18. `slug()` Allocates on Every Call

- [ ] Done
- [ ] Unnecessary

**File:** `app/src/webapp/slices.rs` — Lines 91–99

`slug()` builds a new `String` each time a slice path is derived. This is minor and likely only relevant if slice-key parsing becomes a measured bottleneck.

**Fix:** Memoize derived paths or cache slugged fields on the parsed key only if profiling justifies it.

---

#### 19. `rebin_2d` Already Uses a Row-Major Source Access Pattern

- [ ] Done
- [ ] Unnecessary

**File:** `app/src/core.rs` — Lines 482–506

The earlier cache-locality warning was incorrect. The current loop walks `grid` in row-major order:

```rust
for y in 0..height {
    for x in 0..width {
        let src = y * width + x;
        // ...
    }
}
```

So there is no obvious column-major read bug here.

**Fix:** Leave this alone unless a real profile shows `rebin_2d` dominating large-grid workloads.

---

#### 20. Scoring Functions Could Be Table-Driven, but They Are Already Small

- [ ] Done
- [ ] Unnecessary

**File:** `app/src/core.rs` — Lines 413–470

`dots_points`, `wilks_points`, and `goodlift_points` re-match on sex/equipment and evaluate their formulas per call. That is real work, but the functions are compact and easy to read.

**Fix:** Only move to coefficient tables if these functions show up as hot in measurement.
