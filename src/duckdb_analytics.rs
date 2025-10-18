// src/duckdb_analytics.rs - In-process SQL analytics engine for complex queries
use crate::models::{RankingEntry, RankingsParams, RankingsResponse};
use duckdb::{Connection, Error as DuckError, Result as DuckResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tracing::{info, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileData {
    pub sex: String,
    pub equipment: String,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrengthDistribution {
    pub lift_type: String,
    pub sex: String,
    pub equipment: String,
    pub weight_class: Option<String>,
    pub bin_start: f64,
    pub bin_end: f64,
    pub count: i64,
    pub avg_dots: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveAnalysis {
    pub sex: String,
    pub equipment: String,
    pub lift_type: String,
    pub user_lift: f64,
    pub user_bodyweight: f64,
    pub percentile: f64,
    pub rank: i64,
    pub total_competitors: i64,
    pub dots_score: f64,
}

pub struct DuckDBAnalytics {
    // Mutex protects against concurrent access since DuckDB Connection is not Sync
    conn: Mutex<Connection>,
    // Query result cache: cache_key -> (timestamp, serialized_result)
    #[allow(dead_code)]
    query_cache: Arc<Mutex<HashMap<String, (SystemTime, Vec<u8>)>>>,
}

impl DuckDBAnalytics {
    /// Initialize DuckDB with Parquet data source
    #[instrument(skip_all)]
    pub fn from_parquet<P: AsRef<Path>>(parquet_path: P) -> DuckResult<Self> {
        let requested_path = parquet_path.as_ref();
        let resolved_path = resolve_parquet_path(requested_path)?;

        if resolved_path != requested_path {
            info!(
                "Requested DuckDB parquet '{}' not found. Using latest match '{}'.",
                requested_path.display(),
                resolved_path.display()
            );
        }

        info!(
            "Initializing DuckDB analytics engine with: {}",
            resolved_path.display()
        );

        let conn = Connection::open_in_memory()?;

        // Configure DuckDB for optimal performance
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        // Set optimal pragmas for performance
        conn.execute(&format!("PRAGMA threads = {}", cpu_count), [])?;
        conn.execute("PRAGMA enable_object_cache = true", [])?;
        conn.execute("PRAGMA memory_limit = '4GB'", [])?; // Lower from 8GB for better stability
        conn.execute("PRAGMA max_memory = '4GB'", [])?;
        conn.execute("PRAGMA temp_directory = './tmp/duckdb'", [])?; // Use local temp directory

        // Enable query optimization hints
        conn.execute("PRAGMA preserve_insertion_order = false", [])?; // Allow reordering for optimization

        // Create view from Parquet file (safe path handling)
        let safe_path = resolved_path.to_string_lossy().replace('\'', "''");
        conn.execute(
            &format!(
                "CREATE VIEW lifts AS SELECT * FROM read_parquet('{}')",
                safe_path
            ),
            [],
        )?;

        info!("DuckDB analytics engine initialized successfully");
        Ok(Self {
            conn: Mutex::new(conn),
            query_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get cached query result if available and not expired (5 minutes TTL)
    #[allow(dead_code)]
    fn get_cached_query(&self, cache_key: &str) -> Option<Vec<u8>> {
        let cache = self.query_cache.lock().ok()?;
        if let Some((timestamp, data)) = cache.get(cache_key) {
            // Cache for 5 minutes (300 seconds)
            if timestamp.elapsed().ok()?.as_secs() < 300 {
                info!("Cache hit for query: {}", cache_key);
                return Some(data.clone());
            }
        }
        None
    }

    /// Cache query result with timestamp
    #[allow(dead_code)]
    fn cache_query_result(&self, cache_key: String, data: Vec<u8>) {
        if let Ok(mut cache) = self.query_cache.lock() {
            cache.insert(cache_key, (SystemTime::now(), data));

            // Limit cache size to 100 entries
            if cache.len() > 100 {
                // Remove oldest entries (first 20)
                let mut entries: Vec<_> = cache.iter().map(|(k, (t, _))| (k.clone(), *t)).collect();
                entries.sort_by_key(|(_, time)| *time);
                for (key, _) in entries.iter().take(20) {
                    cache.remove(key);
                }
                info!("Cache cleanup: removed 20 oldest entries");
            }
        }
    }

    /// Calculate DOTS percentiles grouped by sex and equipment
    #[instrument(skip(self))]
    pub fn calculate_dots_percentiles(&self) -> DuckResult<Vec<PercentileData>> {
        info!("Calculating DOTS percentiles with DuckDB");

        let query = r#"
            SELECT
                Sex,
                Equipment,
                quantile_cont(TotalDOTS, 0.25) AS p25,
                quantile_cont(TotalDOTS, 0.50) AS p50,
                quantile_cont(TotalDOTS, 0.75) AS p75,
                quantile_cont(TotalDOTS, 0.90) AS p90,
                quantile_cont(TotalDOTS, 0.95) AS p95,
                quantile_cont(TotalDOTS, 0.99) AS p99,
                COUNT(*) AS count
            FROM lifts
            WHERE TotalDOTS IS NOT NULL
                AND TotalDOTS > 0
                AND TotalDOTS < 1000  -- Filter outliers
            GROUP BY Sex, Equipment
            ORDER BY Sex, Equipment
        "#;

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            Ok(PercentileData {
                sex: row.get::<_, String>(0)?,
                equipment: row.get::<_, String>(1)?,
                p25: row.get::<_, f64>(2)?,
                p50: row.get::<_, f64>(3)?,
                p75: row.get::<_, f64>(4)?,
                p90: row.get::<_, f64>(5)?,
                p95: row.get::<_, f64>(6)?,
                p99: row.get::<_, f64>(7)?,
                count: row.get::<_, i64>(8)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        info!("Calculated {} percentile groups", results.len());
        Ok(results)
    }

    /// Calculate weight distribution bins for histogram visualization
    #[instrument(skip(self))]
    pub fn calculate_weight_distribution(
        &self,
        lift_type: &str,
        sex: &str,
        equipment: &[String],
        bin_count: usize,
        weight_class: Option<&str>,
    ) -> DuckResult<Vec<StrengthDistribution>> {
        info!("Calculating weight distribution for {lift_type}");

        let equipment_filter = equipment
            .iter()
            .map(|e| format!("'{}'", e.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ");

        let lift_column = match lift_type {
            "squat" => "Best3SquatKg",
            "bench" => "Best3BenchKg",
            "deadlift" => "Best3DeadliftKg",
            "total" => "TotalKg",
            _ => {
                return Err(duckdb::Error::InvalidColumnName(
                    "Invalid lift type".to_string(),
                ));
            }
        };

        // Build weight class filter clause
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

        let query = format!(
            r#"
            WITH filtered_data AS (
                SELECT
                    {lift_column} as lift_weight,
                    TotalDOTS,
                    BodyweightKg,
                    WeightClassKg
                FROM lifts
                WHERE Sex = ?
                    AND Equipment IN ({equipment_filter})
                    AND {lift_column} IS NOT NULL
                    AND {lift_column} > 0
                    AND TotalDOTS IS NOT NULL
                    AND TotalDOTS > 0
                    {weight_class_filter}
            ),
            stats AS (
                SELECT
                    MIN(lift_weight) as min_weight,
                    MAX(lift_weight) as max_weight
                FROM filtered_data
            ),
            bins AS (
                SELECT
                    generate_series(0, {bin_count} - 1) as bin_idx,
                    min_weight + (max_weight - min_weight) * bin_idx / {bin_count} as bin_start,
                    min_weight + (max_weight - min_weight) * (bin_idx + 1) / {bin_count} as bin_end
                FROM stats
            )
            SELECT
                ? as lift_type,
                ? as sex,
                ? as equipment,
                NULL as weight_class,
                b.bin_start,
                b.bin_end,
                COUNT(fd.lift_weight) as count,
                COALESCE(AVG(fd.TotalDOTS), 0) as avg_dots
            FROM bins b
            LEFT JOIN filtered_data fd ON fd.lift_weight >= b.bin_start AND fd.lift_weight < b.bin_end
            GROUP BY b.bin_idx, b.bin_start, b.bin_end
            ORDER BY b.bin_start
        "#
        );

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;
        let equipment_str = equipment.join(",");
        let rows = stmt.query_map([sex, lift_type, sex, &equipment_str], |row| {
            Ok(StrengthDistribution {
                lift_type: row.get::<_, String>(0)?,
                sex: row.get::<_, String>(1)?,
                equipment: row.get::<_, String>(2)?,
                weight_class: row.get::<_, Option<String>>(3)?,
                bin_start: row.get::<_, f64>(4)?,
                bin_end: row.get::<_, f64>(5)?,
                count: row.get::<_, i64>(6)?,
                avg_dots: row.get::<_, f64>(7)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        info!("Generated {} distribution bins", results.len());
        Ok(results)
    }

    /// Analyze competitive position for a specific lift
    #[instrument(skip(self))]
    pub fn analyze_competitive_position(
        &self,
        lift_type: &str,
        user_lift: f64,
        user_bodyweight: f64,
        sex: &str,
        equipment: &[String],
        weight_class: Option<&str>,
    ) -> DuckResult<CompetitiveAnalysis> {
        info!("Analyzing competitive position for {user_lift}kg {lift_type}");

        let equipment_filter = equipment
            .iter()
            .map(|e| format!("'{}'", e.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ");

        let lift_column = match lift_type {
            "squat" => "Best3SquatKg",
            "bench" => "Best3BenchKg",
            "deadlift" => "Best3DeadliftKg",
            "total" => "TotalKg",
            _ => {
                return Err(duckdb::Error::InvalidColumnName(
                    "Invalid lift type".to_string(),
                ));
            }
        };

        // Build weight class filter clause
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

        let query = format!(
            r#"
            WITH filtered_lifts AS (
                SELECT
                    {lift_column} as lift_weight,
                    TotalDOTS,
                    BodyweightKg,
                    WeightClassKg
                FROM lifts
                WHERE Sex = ?
                    AND Equipment IN ({equipment_filter})
                    AND {lift_column} IS NOT NULL
                    AND {lift_column} > 0
                    AND TotalDOTS IS NOT NULL
                    {weight_class_filter}
            ),
            user_rank AS (
                SELECT
                    COUNT(*) FILTER (WHERE lift_weight < ?) as below_user,
                    COUNT(*) as total_competitors
                FROM filtered_lifts
            ),
            user_dots AS (
                SELECT
                    -- Simplified DOTS calculation (you may want to use your existing WASM function)
                    ? * 500 / (? * 1.0) as estimated_dots
            )
            SELECT
                ? as sex,
                ? as equipment,
                ? as lift_type,
                ? as user_lift,
                ? as user_bodyweight,
                (below_user * 100.0 / GREATEST(total_competitors, 1)) as percentile,
                below_user + 1 as rank,
                total_competitors,
                estimated_dots as dots_score
            FROM user_rank, user_dots
        "#
        );

        let equipment_str = equipment.join(",");
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;
        let row = stmt.query_row(
            [
                sex,
                &user_lift.to_string(),
                &user_lift.to_string(),
                &user_bodyweight.to_string(),
                sex,
                &equipment_str,
                lift_type,
                &user_lift.to_string(),
                &user_bodyweight.to_string(),
            ],
            |row| {
                Ok(CompetitiveAnalysis {
                    sex: row.get::<_, String>(0)?,
                    equipment: row.get::<_, String>(1)?,
                    lift_type: row.get::<_, String>(2)?,
                    user_lift: row.get::<_, f64>(3)?,
                    user_bodyweight: row.get::<_, f64>(4)?,
                    percentile: row.get::<_, f64>(5)?,
                    rank: row.get::<_, i64>(6)?,
                    total_competitors: row.get::<_, i64>(7)?,
                    dots_score: row.get::<_, f64>(8)?,
                })
            },
        )?;

        info!(
            "Competitive analysis completed: {:.1}% percentile",
            row.percentile
        );
        Ok(row)
    }

    /// Get summary statistics for caching
    #[instrument(skip(self))]
    pub fn get_summary_stats(&self) -> DuckResult<serde_json::Value> {
        let query = r#"
            SELECT
                COUNT(*) as total_records,
                COUNT(DISTINCT Name) as unique_lifters,
                AVG(TotalDOTS) as avg_dots,
                quantile_cont(TotalDOTS, 0.95) as top_5_percent_dots,
                MIN(Date) as earliest_date,
                MAX(Date) as latest_date
            FROM lifts
            WHERE TotalDOTS IS NOT NULL AND TotalDOTS > 0
        "#;

        let conn = self.conn.lock().unwrap();
        let row = conn.query_row(query, [], |row| {
            Ok(serde_json::json!({
                "total_records": row.get::<_, i64>(0)?,
                "unique_lifters": row.get::<_, i64>(1)?,
                "avg_dots": row.get::<_, f64>(2)?,
                "top_5_percent_dots": row.get::<_, f64>(3)?,
                "earliest_date": row.get::<_, String>(4)?,
                "latest_date": row.get::<_, String>(5)?
            }))
        })?;

        Ok(row)
    }

    /// Get rankings from full dataset with pagination and filtering
    #[instrument(skip(self))]
    pub fn get_rankings_from_full_dataset(
        &self,
        params: &RankingsParams,
    ) -> DuckResult<RankingsResponse> {
        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(100).min(1000); // Cap at 1000 records per page
        let sort_by = params.sort_by.as_deref().unwrap_or("dots");
        let offset = (page - 1) * per_page;

        info!(
            "Getting rankings page {} (size: {}) sorted by {}",
            page, per_page, sort_by
        );

        // Build WHERE clause conditions
        let mut where_conditions = vec![
            "TotalDOTS IS NOT NULL".to_string(),
            "TotalDOTS > 0".to_string(),
            "TotalDOTS < 1000".to_string(), // Filter outliers
            "Best3SquatKg > 0".to_string(),
            "Best3BenchKg > 0".to_string(),
            "Best3DeadliftKg > 0".to_string(),
            "TotalKg > 0".to_string(),
        ];

        let mut params_vec = Vec::new();

        if let Some(sex) = &params.sex {
            where_conditions.push("Sex = ?".to_string());
            params_vec.push(sex.clone());
        }

        if let Some(equipment) = &params.equipment {
            where_conditions.push("Equipment = ?".to_string());
            params_vec.push(equipment.clone());
        }

        if let Some(weight_class) = &params.weight_class {
            // Convert UI weight class format to DB format
            let db_weight_class = if weight_class.ends_with('+') {
                format!("{}kg+", weight_class.trim_end_matches('+'))
            } else {
                format!("{}kg", weight_class)
            };
            where_conditions.push("WeightClassKg = ?".to_string());
            params_vec.push(db_weight_class);
        }

        if let Some(federation) = &params.federation {
            where_conditions.push("Federation = ?".to_string());
            params_vec.push(federation.to_uppercase());
        }

        if let Some(year) = params.year {
            where_conditions.push("EXTRACT(YEAR FROM Date) = ?".to_string());
            params_vec.push(year.to_string());
        }

        let where_clause = where_conditions.join(" AND ");

        // Determine ORDER BY clause
        let order_column = match sort_by {
            "total" => "TotalKg",
            "squat" => "Best3SquatKg",
            "bench" => "Best3BenchKg",
            "deadlift" => "Best3DeadliftKg",
            _ => "TotalDOTS", // Default to DOTS
        };

        // Main query with ranking
        let query = format!(
            r#"
            WITH ranked_lifts AS (
                SELECT
                    ROW_NUMBER() OVER (ORDER BY {} DESC) as rank,
                    Name,
                    Federation,
                    Date::VARCHAR as Date,
                    Sex,
                    Equipment,
                    WeightClassKg,
                    BodyweightKg,
                    Best3SquatKg,
                    Best3BenchKg,
                    Best3DeadliftKg,
                    TotalKg,
                    TotalDOTS,
                    COUNT(*) OVER() as total_count
                FROM lifts
                WHERE {}
            )
            SELECT
                rank,
                Name,
                Federation,
                Date,
                Sex,
                Equipment,
                WeightClassKg,
                BodyweightKg,
                Best3SquatKg,
                Best3BenchKg,
                Best3DeadliftKg,
                TotalKg,
                TotalDOTS,
                total_count
            FROM ranked_lifts
            WHERE rank > ? AND rank <= ?
            ORDER BY rank
        "#,
            order_column, where_clause
        );

        // Add pagination parameters
        params_vec.push(offset.to_string());
        params_vec.push((offset + per_page).to_string());

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;

        let param_refs: Vec<&dyn duckdb::ToSql> =
            params_vec.iter().map(|s| s as &dyn duckdb::ToSql).collect();
        let rows = stmt.query_map(&param_refs[..], |row| {
            let total_count: i64 = row.get("total_count")?;

            Ok((
                RankingEntry {
                    rank: row.get::<_, i64>("rank")? as u32,
                    name: row.get("Name")?,
                    federation: row.get("Federation")?,
                    date: row.get("Date")?,
                    sex: row.get("Sex")?,
                    equipment: row.get("Equipment")?,
                    weight_class: row
                        .get::<_, String>("WeightClassKg")?
                        .trim_end_matches("kg")
                        .to_string(),
                    bodyweight: row.get::<_, f64>("BodyweightKg")? as f32,
                    squat: row.get::<_, f64>("Best3SquatKg")? as f32,
                    bench: row.get::<_, f64>("Best3BenchKg")? as f32,
                    deadlift: row.get::<_, f64>("Best3DeadliftKg")? as f32,
                    total: row.get::<_, f64>("TotalKg")? as f32,
                    dots: row.get::<_, f64>("TotalDOTS")? as f32,
                },
                total_count as u32,
            ))
        })?;

        let mut entries = Vec::new();
        let mut total_count = 0u32;

        for row in rows {
            let (entry, count) = row?;
            total_count = count;
            entries.push(entry);
        }

        let total_pages = if total_count > 0 {
            (total_count + per_page - 1) / per_page
        } else {
            1
        };

        let response = RankingsResponse {
            entries,
            total_count,
            page,
            per_page,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        };

        info!(
            "Retrieved {} rankings entries (page {}/{})",
            response.entries.len(),
            page,
            total_pages
        );
        Ok(response)
    }

    /// Close the connection (called automatically when dropped)
    #[allow(dead_code)]
    pub fn close(self) -> DuckResult<()> {
        // Connection automatically closes when dropped
        info!("DuckDB analytics engine closed");
        Ok(())
    }
}

fn resolve_parquet_path(requested: &Path) -> Result<PathBuf, DuckError> {
    if requested.exists() {
        return Ok(requested.to_path_buf());
    }

    let parent = requested.parent().unwrap_or_else(|| Path::new("."));
    let prefix = requested.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    let entries =
        fs::read_dir(parent).map_err(|_| DuckError::InvalidPath(requested.to_path_buf()))?;
    let mut newest: Option<(SystemTime, PathBuf)> = None;

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.extension().and_then(|ext| ext.to_str()) != Some("parquet") {
            continue;
        }

        let name = entry_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if !prefix.is_empty() && !name.starts_with(prefix) {
            continue;
        }

        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let replace = newest
            .as_ref()
            .map(|(time, _)| modified > *time)
            .unwrap_or(true);

        if replace {
            newest = Some((modified, entry_path));
        }
    }

    newest
        .map(|(_, path)| path)
        .ok_or_else(|| DuckError::InvalidPath(requested.to_path_buf()))
}
