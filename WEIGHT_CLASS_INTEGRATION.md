# Weight Class Filtering Integration

## Overview

Weight class filtering has been successfully integrated into the Iron Insights analytics platform. Users can now filter data by specific powerlifting weight classes using the weight class dropdown in the UI.

## Implementation Details

### Frontend Changes

#### UI Components

- **Dropdown Element**: `select#weightClass` with men's and women's weight class options
- **JavaScript State**: Added `currentWeightClass` variable to track selected weight class
- **Event Handling**: Added `setupWeightClassFilter()` function to handle dropdown changes
- **Parameter Passing**: Weight class is now included in API requests to `/api/visualize`

#### Weight Class Options

**Men's Classes**: 59kg, 66kg, 74kg, 83kg, 93kg, 105kg, 120kg, 120+kg
**Women's Classes**: 47kg, 52kg, 57kg, 63kg, 69kg, 76kg, 84kg, 84+kg

### Backend Changes

#### Data Processing (Polars)

- **Filter Logic**: Updated `apply_filters()` in `filters.rs`
- **Format Conversion**: Dropdown values (e.g., "74") → database format (e.g., "74kg")
- **Plus Classes**: Handles "120+" → "120kg+" conversion
- **Database Column**: Filters on `WeightClassKg` column

#### DuckDB Analytics

- **Weight Distribution**: Added `weight_class` parameter to `calculate_weight_distribution()`
- **Competitive Analysis**: Added `weight_class` parameter to `analyze_competitive_position()`
- **SQL Filtering**: Dynamic WHERE clause injection for weight class filtering
- **Format Consistency**: Same dropdown → database format conversion as Polars

#### API Endpoints

- **Existing Endpoints**: `/api/visualize`, `/api/visualize-arrow` now support weight_class parameter
- **DuckDB Endpoints**: All `/api/*-duckdb` endpoints updated with weight class support
- **Parameter Structure**: `weight_class` is optional in `FilterParams`

## Usage Examples

### Frontend JavaScript

```javascript
// Weight class state management
let currentWeightClass = 'All';

// Event handling
document.getElementById('weightClass').addEventListener('change', function(e) {
    currentWeightClass = e.target.value;
    updateCharts(); // Triggers new API call with weight class filter
});

// API request parameters
const params = {
    sex: currentSex,
    lift_type: currentLiftType,
    equipment: currentEquipment,
    weight_class: currentWeightClass !== 'All' ? currentWeightClass : null
};
```

### Backend Filtering (Polars)

```rust
// Filter application in filters.rs
if let Some(weight_class) = &params.weight_class {
    if weight_class != "All" {
        let db_weight_class = if weight_class.ends_with('+') {
            format!("{}kg+", weight_class.trim_end_matches('+'))
        } else {
            format!("{}kg", weight_class)
        };
        lf = lf.filter(col("WeightClassKg").eq(lit(db_weight_class.as_str())));
    }
}
```

### DuckDB Analytics

```rust
// Weight class filtering in DuckDB queries
let weight_class_filter = if let Some(wc) = weight_class {
    let db_weight_class = if wc.ends_with('+') {
        format!("{}kg+", wc.trim_end_matches('+'))
    } else {
        format!("{}kg", wc)
    };
    format!("AND WeightClassKg = '{}'", db_weight_class)
} else {
    String::new()
};
```

## Data Flow

1. **User Selection**: User selects weight class from dropdown (e.g., "74")
2. **State Update**: JavaScript updates `currentWeightClass = "74"`
3. **API Request**: `updateCharts()` sends request with `weight_class: "74"`
4. **Backend Processing**:
   - Polars: Converts "74" → "74kg" and filters `WeightClassKg = "74kg"`
   - DuckDB: Same conversion and SQL WHERE clause injection
5. **Response**: Filtered data returned showing only 74kg athletes
6. **UI Update**: Charts and visualizations update with filtered data

## Weight Class Format Mapping

| Dropdown Value | Database Value | Description |
|---------------|----------------|-------------|
| `"59"`        | `"59kg"`       | 59kg weight class |
| `"74"`        | `"74kg"`       | 74kg weight class |
| `"120"`       | `"120kg"`      | 120kg weight class |
| `"120+"`      | `"120kg+"`     | 120kg+ (superheavyweight) |
| `"84+"`       | `"84kg+"`      | Women's 84kg+ class |
| `"All"`       | `null`         | No filtering (all classes) |

## Testing

### Manual Testing Steps

1. **Load Analytics Page**: Navigate to `/analytics`
2. **Select Weight Class**: Choose specific weight class from dropdown
3. **Verify Filtering**: Check that data visualizations update
4. **Test Multiple Classes**: Try different weight classes
5. **Test "All Classes"**: Verify no filtering when "All" is selected

### API Testing

```bash
# Test Polars endpoint with weight class
curl -X POST http://localhost:3000/api/visualize \
  -H "Content-Type: application/json" \
  -d '{"sex":"M","lift_type":"squat","equipment":["Raw"],"weight_class":"74"}'

# Test DuckDB endpoint with weight class
curl -X POST http://localhost:3000/api/weight-distribution-duckdb \
  -H "Content-Type: application/json" \
  -d '{"sex":"M","lift_type":"squat","equipment":["Raw"],"weight_class":"74"}'
```

### Expected Behavior

- **Filtered Results**: Only athletes in selected weight class appear in charts
- **Performance**: Filtering should reduce dataset size and improve query speed
- **Consistency**: Both Polars and DuckDB endpoints return consistent filtered results
- **Error Handling**: Invalid weight classes are handled gracefully

## Performance Impact

### Positive Effects

- **Reduced Dataset Size**: Filtering by weight class significantly reduces data volume
- **Faster Queries**: Smaller datasets result in faster chart generation
- **More Relevant Comparisons**: Users compare against similar-sized athletes

### Considerations

- **Index Usage**: Weight class filtering benefits from indexing on `WeightClassKg`
- **Cache Effectiveness**: Filtered results create more cache entries but improve hit rates
- **Memory Usage**: Smaller filtered datasets reduce memory pressure

## Future Enhancements

### Potential Improvements

1. **Multi-Class Selection**: Allow selecting multiple weight classes simultaneously
2. **Weight Class Recommendations**: Suggest appropriate weight class based on user's bodyweight
3. **Class Transitions**: Show historical data across weight class changes
4. **Relative Rankings**: Compare performance within weight class vs. overall

### Integration Opportunities

1. **Share Cards**: Include weight class information in generated share cards
2. **1RM Calculator**: Factor weight class into strength standards
3. **Competitive Analysis**: Enhanced weight class-specific percentile calculations

## Troubleshooting

### Common Issues

1. **No Data**: Selected weight class may have insufficient data
2. **Format Mismatch**: Ensure dropdown values match expected format
3. **Cache Issues**: Weight class changes should invalidate relevant cache entries

### Debug Steps

1. **Check Console**: JavaScript logs weight class changes
2. **Verify API**: Confirm weight_class parameter in network requests
3. **Backend Logs**: DuckDB/Polars filtering logs show applied filters
4. **Database Query**: Verify `WeightClassKg` column values in dataset
