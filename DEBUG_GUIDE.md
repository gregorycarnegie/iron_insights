# 🔍 DOTS Debugging Guide

## Interactive Charts Features ✨

**New in v0.7.0**: Interactive crossfilter-style chart linking and export functionality!

- **Chart Interactions**: Click and drag to select data points, brush histogram ranges
- **Export Options**: High-quality PNG/SVG/JPEG exports, CSV data downloads
- **Reset**: Double-click any chart to clear all selections
- **Debugging**: Check browser console for crossfiltering setup messages

## Quick Fixes for Empty DOTS Charts

### 1. **Check Server Logs**

When you run `cargo run --release`, look for these debug messages:

```text
🔍 Filtered data: X records
📊 SquatDOTS: X valid values
📊 BenchDOTS: X valid values
📊 DeadliftDOTS: X valid values
📊 TotalDOTS: X valid values
```

If you see `0 valid values`, that's your problem!

### 2. **Enable Frontend Debug Mode**

1. Click "Toggle Debug Info" button in the web interface
2. Update the charts
3. Check if DOTS histogram/scatter show `0 values`

### 3. **Common Issues & Solutions**

#### Issue: "No DOTS data available"

**Cause**: DOTS calculations are producing NaN/infinite values
**Solutions**:

```bash
# Check your data
cargo run --bin debug_dots  # If you add the debug script

# Or examine sample data generation
cargo test test_sample_data_generation -- --nocapture
```

#### Issue: "Column not found" errors

**Cause**: DOTS columns not being created
**Solution**: Check your `scoring.rs` - make sure `DotsCoefficients` is public:

```rust
pub struct DotsCoefficients {
    pub a: f32,
    // ... etc
}
```

#### Issue: All DOTS values are 0 or NaN

**Cause**: Invalid bodyweight or lift values
**Solution**: Add better filtering in `data.rs`:

```rust
.filter(
    col("BodyweightKg").gt(30.0)
        .and(col("BodyweightKg").lt(300.0))
        .and(col("Best3SquatKg").gt(0.0))
        // etc...
)
```

### 4. **Test DOTS Calculation Manually**

Add this to your `main.rs` for quick testing:

```rust
use crate::scoring::calculate_dots_score;

println!("Test DOTS: {}", calculate_dots_score(180.0, 75.0)); // Should be ~310
```

### 5. **Check Your Data Pipeline**

Verify each step:

1. **Data Loading**: Are lift values > 0?
2. **DOTS Calculation**: Are bodyweights reasonable (30-300kg)?
3. **Filtering**: Are you filtering out all the data accidentally?
4. **Frontend**: Are arrays getting passed correctly?

### 6. **Sample Data Troubleshooting**

If using generated sample data, ensure:

```rust
// In SampleDataBuilder::generate_lifter()
let bodyweight = bodyweight_sample.clamp(40.0, 200.0); // Reasonable range
let squat = (bodyweight * sq_ratio * rng.random_range(0.7..1.3)).max(50.0); // Not zero
```

### 7. **Quick Debug Commands**

```bash
# 1. Run with debug output
RUST_LOG=debug cargo run --release

# 2. Test just the calculations
cargo test scoring::tests -- --nocapture

# 3. Check sample data generation
cargo test test_sample_data_generation -- --nocapture

# 4. Validate your real data (if using CSV)
cargo run convert your_data.csv  # This also validates
```

### 8. **Frontend Network Tab**

Open browser dev tools → Network tab → refresh page:

- Look for `/api/visualize` request
- Check the response - does it contain DOTS data?
- Example good response:

```json
{
  "dots_histogram_data": {
    "values": [312.5, 289.1, 445.2, ...],  // Should have values!
    "counts": [12, 15, 8, ...],
    "bins": [0, 10, 20, ...]
  }
}
```

### 9. **Data Validation Checklist**

✅ Bodyweights are between 30-300kg  
✅ Lift values are > 0  
✅ DOTS calculations produce finite numbers  
✅ No entire columns are filtered out  
✅ Frontend receives non-empty arrays  

### 10. **Emergency Sample Data Fix**

If nothing works, force sample data generation:

```rust
// In main.rs, temporarily comment out real data loading
// let data = data_processor.load_and_preprocess_data()

// Force sample data:
let data = data_processor.create_sample_data().unwrap();
```

## Still Not Working?

1. **Check the server terminal** for error messages
2. **Enable debug mode** in the frontend
3. **Look at browser console** for JavaScript errors
4. **Verify API response** in Network tab
5. **Test DOTS calculation** with known values

The most common issue is filtering out all data with strict validation rules. Try loosening the filters first, then tightening them once you see data flowing through.
