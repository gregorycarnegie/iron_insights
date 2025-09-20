# DuckDB Integration for Iron Insights

## Overview

DuckDB has been integrated as a complementary analytics engine alongside the existing Polars/Arrow stack. This provides powerful SQL capabilities for complex analytical queries while maintaining the high-performance data processing pipeline.

## Architecture

### Hybrid Analytics Stack

- **Source of Truth**: Parquet files (unchanged)
- **DuckDB**: Complex SQL analytics (percentiles, window functions, multi-way joins)
- **Polars**: Fast transforms, Arrow I/O, existing optimized compute paths
- **Caching**: Shared caching layer for both engines

### Threading Model

- DuckDB connections are protected by `Mutex<Connection>` for thread safety
- Operations are executed in `spawn_blocking` to avoid blocking the async runtime
- Each query gets a lock on the connection for execution

## New API Endpoints

### 1. Percentile Calculations

```duckdb
GET /api/percentiles-duckdb
```

Returns DOTS percentiles grouped by sex and equipment using DuckDB's `quantile_cont()` function.

**Response:**

```json
{
  "percentiles": [
    {
      "sex": "M",
      "equipment": "Raw",
      "p25": 145.2,
      "p50": 185.7,
      "p75": 234.8,
      "p90": 289.4,
      "p95": 325.1,
      "p99": 412.3,
      "count": 15420
    }
  ],
  "generated_at": "2025-01-15T10:30:00Z",
  "engine": "duckdb"
}
```

### 2. Weight Distribution Analysis

```duckdb
POST /api/weight-distribution-duckdb
```

Calculate histogram bins for lift weight distributions with advanced SQL windowing.

**Request:**

```json
{
  "lift_type": "squat",
  "sex": "M",
  "equipment": ["Raw"],
  "bin_count": 50
}
```

### 3. Competitive Analysis

```duckdb
POST /api/competitive-analysis-duckdb
```

Analyze competitive position using ranking and percentile calculations.

**Request:**

```json
{
  "lift_type": "total",
  "squat": 180,
  "bench": 120,
  "deadlift": 220,
  "bodyweight": 80,
  "sex": "M",
  "equipment": ["Raw"]
}
```

### 4. Summary Statistics

```duckdb
GET /api/summary-stats-duckdb
```

Get aggregated statistics using DuckDB's analytical functions.

## Performance Optimizations

### DuckDB Configuration

```sql
PRAGMA threads = {cpu_count};          -- Use available CPU cores
PRAGMA enable_object_cache = true;     -- Cache query plans
PRAGMA memory_limit = '8GB';           -- Memory allocation
```

### Query Patterns

- **Quantiles**: `quantile_cont(column, percentile)` instead of `PERCENTILE_CONT(...) WITHIN GROUP`
- **Binning**: `generate_series()` for histogram bins
- **Filtering**: Early WHERE clauses to reduce data volume
- **Aggregation**: Grouped calculations for multiple metrics

## Usage Examples

### Calculate Strength Percentiles

```rust
use duckdb_analytics::DuckDBAnalytics;

let analytics = DuckDBAnalytics::from_parquet("data/openpowerlifting.parquet")?;
let percentiles = analytics.calculate_dots_percentiles()?;

for p in percentiles {
    println!("{} {} - P50: {:.1}", p.sex, p.equipment, p.p50);
}
```

### Competitive Position Analysis

```rust
let analysis = analytics.analyze_competitive_position(
    "total",
    520.0,  // user lift
    80.0,   // bodyweight
    "M",
    &["Raw".to_string()]
)?;

println!("You rank {}th out of {} lifters ({:.1}%)",
    analysis.rank, analysis.total_competitors, analysis.percentile);
```

## Benefits

### Complex Analytics

- **Multi-dimensional percentiles**: Grouped by sex, equipment, weight class
- **Window functions**: Ranking, running totals, moving averages
- **Advanced aggregations**: Histogram calculations, distribution analysis

### Performance

- **In-memory processing**: Direct Parquet reading without DataFrame conversion
- **Columnar execution**: Optimized for analytical workloads
- **Multi-core utilization**: Automatic parallelization

### SQL Expressiveness

- **Readable queries**: Complex logic expressed in standard SQL
- **Maintainability**: Easier to modify analytical calculations
- **Flexibility**: Ad-hoc analysis capabilities

## Migration Strategy

### Gradual Adoption

1. **Phase 1**: New endpoints using DuckDB (current)
2. **Phase 2**: Migrate complex Polars operations to DuckDB
3. **Phase 3**: Hybrid optimization based on query characteristics

### Backwards Compatibility

- All existing Polars/Arrow endpoints remain functional
- Caching layer shared between both engines
- WebSocket and real-time features unchanged

## Monitoring

### Logging

- Query execution times logged with `tracing`
- DuckDB initialization and errors tracked
- Performance metrics for cache hit/miss ratios

### Error Handling

- Graceful fallback if DuckDB initialization fails
- Thread-safe error reporting with proper HTTP status codes
- Service availability indicators in API responses

## Future Enhancements

### Potential Optimizations

- **Persistent connections**: Investigate connection pooling
- **Materialized views**: Cache frequently-used calculations
- **Index optimization**: Add covering indexes for filtered queries
- **Streaming results**: Large result set pagination

### Advanced Analytics

- **Time-series analysis**: Historical strength progression
- **Correlation analysis**: Equipment vs. performance relationships
- **Outlier detection**: Statistical anomaly identification
- **Predictive modeling**: Performance trend analysis

## Configuration

### Environment Variables

```bash
# DuckDB memory limit (optional)
DUCKDB_MEMORY_LIMIT=8GB

# Thread count (optional, defaults to all cores)
DUCKDB_THREADS=8

# Enable query logging
DUCKDB_QUERY_LOG=true
```

### Parquet File Path

The DuckDB integration expects the Parquet file at:

```text
data/openpowerlifting.parquet
```

This path is configurable during `DuckDBAnalytics::from_parquet()` initialization.

## Troubleshooting

### Common Issues

#### "Must have at least 1 thread!" Error

- **Cause**: DuckDB thread configuration issue
- **Solution**: The code now auto-detects CPU count and sets threads appropriately
- **Fallback**: Defaults to 4 threads if detection fails

#### "DuckDB not available" in API responses

- **Cause**: DuckDB initialization failed during startup
- **Behavior**: Application continues with Polars-only functionality
- **Check**: Server logs will show DuckDB initialization warnings

#### Slow First Query

- **Expected**: DuckDB compiles queries on first execution
- **Solution**: Subsequent queries with similar patterns are much faster
- **Optimization**: Object cache is enabled to persist query plans

### Performance Tips

1. **Memory Allocation**: Adjust memory limit based on available RAM
2. **Thread Count**: Monitor CPU utilization during heavy analytical workloads
3. **Query Optimization**: Use WHERE clauses early to filter data
4. **Result Pagination**: Consider implementing pagination for large result sets
