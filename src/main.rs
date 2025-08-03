use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use moka::future::Cache;
use polars::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Instant};
use tower_http::{compression::CompressionLayer, services::ServeDir};

// Shared application state
#[derive(Clone)]
struct AppState {
    data: Arc<DataFrame>,
    cache: Cache<String, CachedResult>,
}

#[derive(Clone, Debug)]
struct CachedResult {
    data: Vec<u8>, // Pre-serialized JSON
    computed_at: Instant,
}

// API request/response types
#[derive(Deserialize, Debug)]
struct FilterParams {
    sex: Option<String>,
    equipment: Option<Vec<String>>,
    weight_class: Option<String>,
    squat: Option<f64>,
    bench: Option<f64>,
    deadlift: Option<f64>,
    bodyweight: Option<f64>,
    units: Option<String>,
    lift_type: Option<String>, // "squat", "bench", "deadlift", "total"
}

#[derive(Serialize, Deserialize)]
struct VisualizationResponse {
    histogram_data: HistogramData,
    scatter_data: ScatterData,
    wilks_histogram_data: HistogramData,
    wilks_scatter_data: ScatterData,
    user_percentile: Option<f64>,
    processing_time_ms: u64,
    total_records: usize,
}

#[derive(Serialize, Deserialize)]
struct HistogramData {
    values: Vec<f64>,
    counts: Vec<u32>,
    bins: Vec<f64>,
    min_val: f64,
    max_val: f64,
}

#[derive(Serialize, Deserialize)]
struct ScatterData {
    x: Vec<f64>, // bodyweight
    y: Vec<f64>, // lift values
    sex: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Iron Insights - High-Performance Powerlifting Analyzer...");
    
    let start = Instant::now();
    let data = tokio::task::spawn_blocking(|| load_and_preprocess_data()).await??;
    println!("📊 Data loaded in {:?}", start.elapsed());
    
    // Initialize cache with 1000 entries, 1-hour TTL
    let cache = Cache::builder()
        .max_capacity(1000)
        .time_to_live(std::time::Duration::from_secs(3600))
        .build();
    
    let state = AppState {
        data: Arc::new(data),
        cache,
    };
    
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/visualize", post(create_visualizations))
        .route("/api/stats", get(get_stats))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CompressionLayer::new())
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🌐 Server running on http://localhost:3000");
    println!("💡 Upload your powerlifting CSV to /data/openpowerlifting.csv to get started");
    
    axum::serve(listener, app).await?;
    Ok(())
}

fn load_and_preprocess_data() -> PolarsResult<DataFrame> {
    println!("📥 Loading powerlifting data...");
    
    // Look for CSV files with the proper naming convention
    let data_dir = std::path::Path::new("data");
    let csv_file = if data_dir.exists() {
        // Find the most recent openpowerlifting CSV file
        std::fs::read_dir(data_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let filename = path.file_name()?.to_str()?;
                
                // Check if it matches the pattern: openpowerlifting-YYYY-MM-DD-hash.csv
                if filename.starts_with("openpowerlifting-") && filename.ends_with(".csv") {
                    Some(path)
                } else {
                    None
                }
            })
            .max_by_key(|path| {
                // Sort by filename to get the most recent date
                path.file_name().unwrap_or_default().to_str().unwrap_or_default().to_string()
            })
    } else {
        None
    };
    
    let data_path = match csv_file {
        Some(path) => {
            println!("📂 Found powerlifting data: {:?}", path.file_name().unwrap());
            path
        }
        None => {
            println!("⚠️  No openpowerlifting CSV files found in data/ directory");
            println!("💡 Please download from: https://openpowerlifting.gitlab.io/opl-csv/bulk-csv.html");
            println!("🎯 Creating sample data for demo...");
            return create_sample_data();
        }
    };
    
    // Load real data using proper LazyCsvReader API for 0.49.1
    // Define schema with proper float types for lift columns
    let schema = Schema::from_iter([
        Field::new("Name".into(), DataType::String),
        Field::new("Sex".into(), DataType::String),
        Field::new("Equipment".into(), DataType::String),
        Field::new("BodyweightKg".into(), DataType::Float64),
        Field::new("Best3SquatKg".into(), DataType::Float64),
        Field::new("Best3BenchKg".into(), DataType::Float64),
        Field::new("Best3DeadliftKg".into(), DataType::Float64),
        Field::new("TotalKg".into(), DataType::Float64),
    ]);
    
    let df = LazyCsvReader::new(&data_path)
        .with_has_header(true)
        .with_separator(b',')
        .with_dtype_overwrite(Some(Arc::new(schema)))
        .with_infer_schema_length(Some(10000))  // Increase schema inference length
        .with_ignore_errors(true)  // Skip problematic rows
        .with_rechunk(true)  // Optimize memory layout
        .finish()?
        .select([
            col("Name"),
            col("Sex"),
            col("Equipment"), 
            col("BodyweightKg"),
            col("Best3SquatKg"),
            col("Best3BenchKg"), 
            col("Best3DeadliftKg"),
            col("TotalKg"),
        ])
        .filter(
            col("Best3SquatKg").gt(0)
                .or(col("Best3BenchKg").gt(0))
                .or(col("Best3DeadliftKg").gt(0))
        )
        .with_columns([
            // Pre-calculate weight classes
            calculate_weight_class_expr(),
            // Pre-calculate Wilks scores
            calculate_wilks_expr("Best3SquatKg", "SquatWilks"),
            calculate_wilks_expr("Best3BenchKg", "BenchWilks"), 
            calculate_wilks_expr("Best3DeadliftKg", "DeadliftWilks"),
            calculate_wilks_expr("TotalKg", "TotalWilks"),
        ])
        .collect()?;
    
    // Sample for performance - use 50,000 random records for web interface
    let df = if df.height() > 50000 {
        println!("📊 Sampling 50,000 records from {} total for better performance", df.height());
        df.sample_n_literal(50000, true, true, Some(42))?
    } else {
        df
    };
    
    println!("✅ Processed {} records", df.height());
    Ok(df)
}

fn create_sample_data() -> PolarsResult<DataFrame> {
    use rand::prelude::*;
    use rand::seq::SliceRandom;
    use rand::rng;
    use rand_distr::Normal;
    
    println!("🎯 Creating sample powerlifting data for demo...");
    
    let mut rng = rng();
    let sample_size = 10000;
    
    // Generate realistic powerlifting data
    let mut names = Vec::new();
    let mut sexes = Vec::new();
    let mut equipment = Vec::new();
    let mut bodyweights = Vec::new();
    let mut squats = Vec::new();
    let mut benches = Vec::new();
    let mut deadlifts = Vec::new();
    let mut totals = Vec::new();
    
    for i in 0..sample_size {
        let sex = if rng.random_bool(0.7) { "M" } else { "F" };
        let eq = ["Raw", "Wraps", "Single-ply"].choose(&mut rng).unwrap();
        
        // Generate realistic bodyweights and lifts based on sex
        let (bw_mean, bw_std, sq_ratio, bp_ratio, dl_ratio) = if sex == "M" {
            (85.0, 15.0, 1.8, 1.3, 2.2) // Male averages
        } else {
            (65.0, 12.0, 1.4, 0.8, 1.8) // Female averages
        };
        
        let bodyweight_sample: f64 = Normal::new(bw_mean, bw_std).unwrap().sample(&mut rng);
        let bodyweight = bodyweight_sample.clamp(40.0, 200.0);
        let squat: f64 = (bodyweight * sq_ratio * rng.random_range(0.7..1.3)).max(50.0);
        let bench: f64 = (bodyweight * bp_ratio * rng.random_range(0.7..1.3)).max(30.0);
        let deadlift: f64 = (bodyweight * dl_ratio * rng.random_range(0.7..1.3)).max(60.0);
        let total = squat + bench + deadlift;
        
        names.push(format!("Lifter{}", i));
        sexes.push(sex.to_string());
        equipment.push(eq.to_string());
        bodyweights.push(bodyweight);
        squats.push(squat);
        benches.push(bench);
        deadlifts.push(deadlift);
        totals.push(total);
    }
    
    let df = df! {
        "Name" => names,
        "Sex" => sexes,
        "Equipment" => equipment,
        "BodyweightKg" => bodyweights,
        "Best3SquatKg" => squats,
        "Best3BenchKg" => benches,
        "Best3DeadliftKg" => deadlifts,
        "TotalKg" => totals,
    }?
    .lazy()
    .with_columns([
        calculate_weight_class_expr(),
        calculate_wilks_expr("Best3SquatKg", "SquatWilks"),
        calculate_wilks_expr("Best3BenchKg", "BenchWilks"), 
        calculate_wilks_expr("Best3DeadliftKg", "DeadliftWilks"),
        calculate_wilks_expr("TotalKg", "TotalWilks"),
    ])
    .collect()?;
    
    println!("✅ Generated {} sample records", df.height());
    Ok(df)
}

fn calculate_wilks_expr(lift_col: &str, output_col: &str) -> Expr {
    let male_coeffs = [47.46178854, 8.472061379, 0.07369410346, -0.001395833811, 7.07665973070743e-06, -1.20804336482315e-08];
    let female_coeffs = [-125.4255398, 13.71219419, -0.03307250631, -0.001050400051, 9.38773881462799e-06, -2.3334613884954e-08];
    
    // Vectorized Wilks calculation - simplified to avoid complex expressions
    when(col("Sex").eq(lit("M")))
        .then(
            col(lift_col) * lit(600.0) / 
            (lit(male_coeffs[0]) + 
             lit(male_coeffs[1]) * col("BodyweightKg") +
             lit(male_coeffs[2]) * col("BodyweightKg").pow(2))
        )
        .when(col("Sex").eq(lit("F")))
        .then(
            col(lift_col) * lit(600.0) /
            (lit(female_coeffs[0]) + 
             lit(female_coeffs[1]) * col("BodyweightKg") +
             lit(female_coeffs[2]) * col("BodyweightKg").pow(2))
        )
        .otherwise(lit(0.0))
        .alias(output_col)
}

fn calculate_weight_class_expr() -> Expr {
    when(col("Sex").eq(lit("M")))
        .then(
            when(col("BodyweightKg").lt_eq(59.0)).then(lit("59kg"))
            .when(col("BodyweightKg").lt_eq(66.0)).then(lit("66kg"))
            .when(col("BodyweightKg").lt_eq(74.0)).then(lit("74kg"))
            .when(col("BodyweightKg").lt_eq(83.0)).then(lit("83kg"))
            .when(col("BodyweightKg").lt_eq(93.0)).then(lit("93kg"))
            .when(col("BodyweightKg").lt_eq(105.0)).then(lit("105kg"))
            .when(col("BodyweightKg").lt_eq(120.0)).then(lit("120kg"))
            .otherwise(lit("120kg+"))
        )
        .otherwise(
            when(col("BodyweightKg").lt_eq(47.0)).then(lit("47kg"))
            .when(col("BodyweightKg").lt_eq(52.0)).then(lit("52kg"))
            .when(col("BodyweightKg").lt_eq(57.0)).then(lit("57kg"))
            .when(col("BodyweightKg").lt_eq(63.0)).then(lit("63kg"))
            .when(col("BodyweightKg").lt_eq(69.0)).then(lit("69kg"))
            .when(col("BodyweightKg").lt_eq(76.0)).then(lit("76kg"))
            .when(col("BodyweightKg").lt_eq(84.0)).then(lit("84kg"))
            .otherwise(lit("84kg+"))
        )
        .alias("WeightClassKg")
}

async fn create_visualizations(
    State(state): State<AppState>,
    Json(params): Json<FilterParams>,
) -> Result<Json<VisualizationResponse>, StatusCode> {
    let start = Instant::now();
    
    // Generate cache key
    let cache_key = format!("{:?}", params);
    
    // Check cache first
    if let Some(cached) = state.cache.get(&cache_key).await {
        if cached.computed_at.elapsed().as_secs() < 300 { // 5-minute cache
            let response: VisualizationResponse = 
                serde_json::from_slice(&cached.data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(Json(response));
        }
    }
    
    // Apply filters
    let filtered_data = apply_filters_fast(&state.data, &params);
    
    // Determine which lift to visualize (default to squat)
    let lift_type = params.lift_type.as_deref().unwrap_or("squat");
    
    // Generate visualizations
    let histogram_data = create_histogram_data(&filtered_data, lift_type, false);
    let scatter_data = create_scatter_data(&filtered_data, lift_type, false);
    let wilks_histogram_data = create_histogram_data(&filtered_data, lift_type, true);
    let wilks_scatter_data = create_scatter_data(&filtered_data, lift_type, true);
    let user_percentile = calculate_user_percentile(&filtered_data, &params);
    
    let response = VisualizationResponse {
        histogram_data,
        scatter_data,
        wilks_histogram_data,
        wilks_scatter_data,
        user_percentile,
        processing_time_ms: start.elapsed().as_millis() as u64,
        total_records: filtered_data.height(),
    };
    
    // Cache the result
    let cached_data = serde_json::to_vec(&response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.cache.insert(cache_key, CachedResult {
        data: cached_data,
        computed_at: Instant::now(),
    }).await;
    
    Ok(Json(response))
}

fn apply_filters_fast(data: &DataFrame, params: &FilterParams) -> DataFrame {
    let mut df = data.clone().lazy();
    
    // Apply filters using Polars' optimized expressions
    if let Some(sex) = &params.sex {
        if sex != "All" {
            df = df.filter(col("Sex").eq(lit(sex.clone())));
        }
    }
    
    if let Some(equipment) = &params.equipment {
        if !equipment.is_empty() {
            let eq_filter = equipment.iter()
                .map(|eq| col("Equipment").eq(lit(eq.clone())))
                .reduce(|acc, expr| acc.or(expr))
                .unwrap();
            df = df.filter(eq_filter);
        }
    }
    
    df.collect().unwrap_or_else(|_| data.clone())
}

fn create_histogram_data(data: &DataFrame, lift_type: &str, use_wilks: bool) -> HistogramData {
    let column = if use_wilks {
        match lift_type {
            "squat" => "SquatWilks",
            "bench" => "BenchWilks", 
            "deadlift" => "DeadliftWilks",
            "total" => "TotalWilks",
            _ => "SquatWilks",
        }
    } else {
        match lift_type {
            "squat" => "Best3SquatKg",
            "bench" => "Best3BenchKg",
            "deadlift" => "Best3DeadliftKg", 
            "total" => "TotalKg",
            _ => "Best3SquatKg",
        }
    };
    
    let values: Vec<f64> = data.column(column)
        .map(|col| {
            col.f64()
                .map(|s| s.into_no_null_iter().filter(|&x| x > 0.0).collect())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    
    if values.is_empty() {
        return HistogramData { 
            values: vec![], 
            counts: vec![], 
            bins: vec![],
            min_val: 0.0,
            max_val: 0.0,
        };
    }
    
    // Fast parallel histogram calculation
    let (min_val, max_val) = values.par_iter()
        .fold(|| (f64::INFINITY, f64::NEG_INFINITY), |acc, &x| (acc.0.min(x), acc.1.max(x)))
        .reduce(|| (f64::INFINITY, f64::NEG_INFINITY), |a, b| (a.0.min(b.0), a.1.max(b.1)));
    
    let num_bins = 50;
    let bin_width = (max_val - min_val) / num_bins as f64;
    let mut bins = vec![0u32; num_bins];
    
    // Sequential binning for correctness
    for &val in &values {
        let bin_idx = ((val - min_val) / bin_width).floor() as usize;
        let bin_idx = bin_idx.min(num_bins - 1);
        bins[bin_idx] += 1;
    }
    
    let bin_edges: Vec<f64> = (0..=num_bins)
        .map(|i| min_val + i as f64 * bin_width)
        .collect();
    
    HistogramData {
        values,
        counts: bins,
        bins: bin_edges,
        min_val,
        max_val,
    }
}

fn create_scatter_data(data: &DataFrame, lift_type: &str, use_wilks: bool) -> ScatterData {
    let y_column = if use_wilks {
        match lift_type {
            "squat" => "SquatWilks",
            "bench" => "BenchWilks",
            "deadlift" => "DeadliftWilks", 
            "total" => "TotalWilks",
            _ => "SquatWilks",
        }
    } else {
        match lift_type {
            "squat" => "Best3SquatKg",
            "bench" => "Best3BenchKg",
            "deadlift" => "Best3DeadliftKg",
            "total" => "TotalKg", 
            _ => "Best3SquatKg",
        }
    };
    
    let bodyweight: Vec<f64> = data.column("BodyweightKg")
        .map(|col| col.f64().map(|s| s.into_no_null_iter().collect()).unwrap_or_default())
        .unwrap_or_default();
    
    let y_values: Vec<f64> = data.column(y_column)
        .map(|col| col.f64().map(|s| s.into_no_null_iter().collect()).unwrap_or_default())
        .unwrap_or_default();
    
    let sex: Vec<String> = data.column("Sex")
        .map(|col| {
            col.str()
                .map(|s| s.into_no_null_iter().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    
    ScatterData {
        x: bodyweight,
        y: y_values,
        sex,
    }
}

fn calculate_user_percentile(_data: &DataFrame, _params: &FilterParams) -> Option<f64> {
    // Placeholder - implement percentile calculation
    Some(75.0)
}

async fn serve_index() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Iron Insights - Powerlifting Analytics</title>
        <script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
        <style>
            body { 
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; 
                margin: 0; 
                padding: 20px; 
                background: #f5f5f5; 
            }
            .container { 
                max-width: 1200px; 
                margin: 0 auto; 
                background: white; 
                border-radius: 8px; 
                padding: 20px; 
                box-shadow: 0 2px 10px rgba(0,0,0,0.1); 
            }
            .header { 
                text-align: center; 
                margin-bottom: 30px; 
            }
            .controls { 
                display: grid; 
                grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); 
                gap: 15px; 
                margin-bottom: 30px; 
            }
            .chart { 
                margin: 20px 0; 
                height: 400px; 
            }
            button { 
                background: #007bff; 
                color: white; 
                border: none; 
                padding: 10px 20px; 
                border-radius: 5px; 
                cursor: pointer; 
            }
            input, select { 
                padding: 8px; 
                border: 1px solid #ddd; 
                border-radius: 4px; 
            }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="header">
                <h1>🏋️ Iron Insights</h1>
                <p>High-Performance Powerlifting Analytics</p>
            </div>
            
            <div class="controls">
                <div>
                    <label>Sex:</label>
                    <select id="sex">
                        <option value="M">Male</option>
                        <option value="F">Female</option>
                        <option value="All">All</option>
                    </select>
                </div>
                <div>
                    <label>Lift Type:</label>
                    <select id="liftType">
                        <option value="squat">Squat</option>
                        <option value="bench">Bench Press</option>
                        <option value="deadlift">Deadlift</option>
                        <option value="total">Total</option>
                    </select>
                </div>
                <div>
                    <label>Your Bodyweight (kg):</label>
                    <input type="number" id="bodyweight" placeholder="75" step="0.1">
                </div>
                <div>
                    <label>Your Lift (kg):</label>
                    <input type="number" id="userLift" placeholder="150" step="0.5">
                </div>
                <div>
                    <button onclick="updateCharts()">Update Charts</button>
                </div>
            </div>
            
            <div id="histogram" class="chart"></div>
            <div id="scatter" class="chart"></div>
            <div id="wilksHistogram" class="chart"></div>
            <div id="wilksScatter" class="chart"></div>
            
            <div id="stats"></div>
        </div>
        
        <script>
            async function updateCharts() {
                const params = {
                    sex: document.getElementById('sex').value,
                    lift_type: document.getElementById('liftType').value,
                    bodyweight: parseFloat(document.getElementById('bodyweight').value) || null,
                    squat: document.getElementById('liftType').value === 'squat' ? parseFloat(document.getElementById('userLift').value) : null,
                    bench: document.getElementById('liftType').value === 'bench' ? parseFloat(document.getElementById('userLift').value) : null,
                    deadlift: document.getElementById('liftType').value === 'deadlift' ? parseFloat(document.getElementById('userLift').value) : null,
                    equipment: ["Raw"]
                };
                
                try {
                    const response = await fetch('/api/visualize', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(params)
                    });
                    
                    const data = await response.json();
                    
                    // Create histogram
                    Plotly.newPlot('histogram', [{
                        x: data.histogram_data.values,
                        type: 'histogram',
                        nbinsx: 50,
                        name: 'Distribution'
                    }], {
                        title: `${params.lift_type.charAt(0).toUpperCase() + params.lift_type.slice(1)} Distribution`,
                        xaxis: { title: 'Weight (kg)' },
                        yaxis: { title: 'Frequency' }
                    });
                    
                    // Create scatter plot
                    Plotly.newPlot('scatter', [{
                        x: data.scatter_data.x,
                        y: data.scatter_data.y,
                        mode: 'markers',
                        type: 'scatter',
                        marker: { 
                            size: 4, 
                            opacity: 0.6,
                            color: data.scatter_data.sex.map(s => s === 'M' ? 'blue' : 'pink')
                        },
                        name: 'Lifters'
                    }], {
                        title: `${params.lift_type.charAt(0).toUpperCase() + params.lift_type.slice(1)} vs Bodyweight`,
                        xaxis: { title: 'Bodyweight (kg)' },
                        yaxis: { title: 'Weight (kg)' }
                    });
                    
                    // Show stats
                    document.getElementById('stats').innerHTML = `
                        <h3>Statistics</h3>
                        <p>Processing time: ${data.processing_time_ms}ms</p>
                        <p>Total records: ${data.total_records.toLocaleString()}</p>
                        <p>Your percentile: ${data.user_percentile || 'N/A'}</p>
                    `;
                    
                } catch (error) {
                    console.error('Error:', error);
                    document.getElementById('stats').innerHTML = '<p>Error loading data</p>';
                }
            }
            
            // Load initial data
            updateCharts();
        </script>
    </body>
    </html>
    "#)
}

async fn get_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "total_records": state.data.height(),
        "cache_size": state.cache.entry_count(),
        "uptime": "running"
    }))
}