// data.rs - Simplified with better error handling
use polars::prelude::*;
use polars::io::parquet::write::StatisticsOptions;
use std::sync::Arc;
use std::path::Path;
use crate::scoring::{calculate_dots_expr, calculate_weight_class_expr};

pub struct DataProcessor {
    sample_size: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            sample_size: 50000,
        }
    }
    
    pub fn with_sample_size(mut self, size: usize) -> Self {
        self.sample_size = size;
        self
    }

    pub fn load_and_preprocess_data(&self) -> PolarsResult<DataFrame> {
        println!("📥 Loading powerlifting data...");
        
        // Check for Parquet file first (fastest)
        if let Some(parquet_path) = self.find_parquet_file() {
            println!("🚀 Found Parquet file: {:?} - Loading at maximum speed!", parquet_path.file_name().unwrap());
            return self.load_parquet_data(&parquet_path);
        }
        
        // Fall back to CSV
        if let Some(csv_path) = self.find_csv_file() {
            println!("📂 Found CSV file: {:?} - Converting to Parquet for future speed boosts", csv_path.file_name().unwrap());
            let df = self.load_real_data(&csv_path)?;
            
            // Save as Parquet with matching filename
            if let Err(e) = self.save_as_parquet(&df, &csv_path) {
                println!("⚠️  Could not save Parquet cache: {}", e);
            } else {
                println!("💾 Saved Parquet cache for faster future loads");
            }
            
            return Ok(df);
        }
        
        // Generate sample data
        println!("⚠️  No data files found - generating sample data");
        self.create_sample_data()
    }
    
    pub async fn check_and_update_data(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        println!("🔍 Checking for OpenPowerlifting data updates...");
        
        let current_revision = self.get_current_revision();
        let latest_revision = self.fetch_latest_revision().await?;
        
        println!("📊 Current revision: {:?}", current_revision);
        println!("🌐 Latest revision: {}", latest_revision);
        
        if current_revision.as_ref() != Some(&latest_revision) {
            println!("📥 New data available! Downloading...");
            self.download_and_extract_data().await?;
            println!("✅ Data updated successfully!");
            return Ok(true);
        } else {
            println!("✅ Data is up to date!");
            return Ok(false);
        }
    }
    
    fn get_current_revision(&self) -> Option<String> {
        // Extract revision from current CSV filename
        if let Some(csv_path) = self.find_csv_file() {
            let filename = csv_path.file_name()?.to_str()?;
            // Extract revision from format: openpowerlifting-YYYY-MM-DD-REVISION.csv
            if let Some(revision_part) = filename.strip_prefix("openpowerlifting-")?.strip_suffix(".csv") {
                if let Some(revision) = revision_part.split('-').last() {
                    return Some(revision.to_string());
                }
            }
        }
        None
    }
    
    async fn fetch_latest_revision(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use scraper::{Html, Selector};
        
        let url = "https://openpowerlifting.gitlab.io/opl-csv/bulk-csv.html";
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        let html = response.text().await?;
        
        let document = Html::parse_document(&html);
        
        // Look for <li> elements that contain "Revision:" text
        let li_selector = Selector::parse("li").map_err(|e| format!("CSS selector error: {:?}", e))?;
        
        for li_element in document.select(&li_selector) {
            let text = li_element.text().collect::<Vec<_>>().join(" ");
            
            // Check if this <li> contains "Revision:"
            if text.trim().starts_with("Revision:") {
                // Look for a link within this <li> element
                let link_selector = Selector::parse("a").map_err(|e| format!("CSS selector error: {:?}", e))?;
                
                if let Some(link) = li_element.select(&link_selector).next() {
                    let revision = link.text().collect::<String>();
                    println!("🔍 Found revision in HTML: {}", revision);
                    return Ok(revision);
                }
            }
        }
        
        // If we can't find the revision, we should fail rather than generate one
        Err("Could not find revision number on OpenPowerlifting website. Please check the site structure or try again later.".into())
    }
    
    async fn download_and_extract_data(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;
        
        // First, get the actual revision from the website
        let revision = self.fetch_latest_revision().await?;
        
        let url = "https://openpowerlifting.gitlab.io/opl-csv/files/openpowerlifting-latest.zip";
        let client = reqwest::Client::new();
        
        println!("📥 Downloading: {}", url);
        let response = client.get(url).send().await?;
        let zip_data = response.bytes().await?;
        
        // Save zip temporarily
        std::fs::create_dir_all("data")?;
        let zip_path = "data/openpowerlifting-latest.zip";
        let mut zip_file = File::create(zip_path).await?;
        zip_file.write_all(&zip_data).await?;
        zip_file.sync_all().await?;
        drop(zip_file);
        
        // Extract CSV from zip using the actual revision
        self.extract_csv_from_zip(zip_path, &revision).await?;
        
        // Clean up zip file
        tokio::fs::remove_file(zip_path).await?;
        
        Ok(())
    }
    
    async fn extract_csv_from_zip(&self, zip_path: &str, revision: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use std::fs::File;
        use std::io::Read;
        use ::zip::read::ZipArchive;
        
        let file = File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        // Find the CSV file in the archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().ends_with(".csv") && file.name().contains("openpowerlifting") {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)?;
                
                // Generate filename with today's date and the actual revision from the website
                use chrono::Utc;
                let date_str = Utc::now().format("%Y-%m-%d").to_string();
                let csv_filename = format!("data/openpowerlifting-{}-{}.csv", date_str, revision);
                
                // Remove old files
                self.cleanup_old_data_files()?;
                
                // Write new CSV
                std::fs::write(&csv_filename, contents)?;
                println!("📁 Extracted to: {}", csv_filename);
                break;
            }
        }
        
        Ok(())
    }
    
    fn cleanup_old_data_files(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data_dir = Path::new("data");
        if !data_dir.exists() {
            return Ok(());
        }
        
        let entries = std::fs::read_dir(data_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("openpowerlifting-") && 
                   (filename.ends_with(".csv") || filename.ends_with(".parquet")) {
                    println!("🗑️  Removing old file: {}", filename);
                    std::fs::remove_file(&path)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn find_parquet_file(&self) -> Option<std::path::PathBuf> {
        let data_dir = std::path::Path::new("data");
        if !data_dir.exists() {
            return None;
        }
        
        std::fs::read_dir(data_dir)
            .ok()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let filename = path.file_name()?.to_str()?;
                
                // Look for openpowerlifting-*.parquet files
                if filename.starts_with("openpowerlifting-") && filename.ends_with(".parquet") {
                    Some(path)
                } else {
                    None
                }
            })
            .max_by_key(|path| {
                // Sort by filename to get the most recent date
                path.file_name().unwrap_or_default().to_str().unwrap_or_default().to_string()
            })
    }
    
    fn find_csv_file(&self) -> Option<std::path::PathBuf> {
        let data_dir = std::path::Path::new("data");
        if !data_dir.exists() {
            return None;
        }
        
        std::fs::read_dir(data_dir)
            .ok()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let filename = path.file_name()?.to_str()?;
                
                if filename.starts_with("openpowerlifting-") && filename.ends_with(".csv") {
                    Some(path)
                } else {
                    None
                }
            })
            .max_by_key(|path| {
                path.file_name().unwrap_or_default().to_str().unwrap_or_default().to_string()
            })
    }
    
    fn load_parquet_data(&self, path: &std::path::Path) -> PolarsResult<DataFrame> {
        let start = std::time::Instant::now();
        
        let df = LazyFrame::scan_parquet(path, ScanArgsParquet::default())?
            .collect()?;
        
        let df = self.apply_sampling(df)?;
        self.validate_dots_data(&df);
        
        println!("⚡ Loaded {} records from Parquet in {:?}", df.height(), start.elapsed());
        Ok(df)
    }
    
    fn load_real_data(&self, path: &std::path::Path) -> PolarsResult<DataFrame> {
        let start = std::time::Instant::now();
        
        println!("🔄 Reading CSV and calculating DOTS...");
        
        // Step 1: Load the basic data
        let schema = Schema::from_iter([
            Field::new("Name".into(), DataType::String),
            Field::new("Sex".into(), DataType::String),
            Field::new("Equipment".into(), DataType::String),
            Field::new("BodyweightKg".into(), DataType::Float32),
            Field::new("Best3SquatKg".into(), DataType::Float32),
            Field::new("Best3BenchKg".into(), DataType::Float32),
            Field::new("Best3DeadliftKg".into(), DataType::Float32),
            Field::new("TotalKg".into(), DataType::Float32),
        ]);
        
        let mut df = LazyCsvReader::new(path)
            .with_has_header(true)
            .with_separator(b',')
            .with_dtype_overwrite(Some(Arc::new(schema)))
            .with_infer_schema_length(Some(10000))
            .with_ignore_errors(true)
            .with_rechunk(true)
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
                col("Best3SquatKg").gt(0.0)
                    .and(col("Best3BenchKg").gt(0.0))
                    .and(col("Best3DeadliftKg").gt(0.0))
                    .and(col("BodyweightKg").gt(30.0))
                    .and(col("BodyweightKg").lt(300.0))
                    .and(col("TotalKg").gt(0.0))
            )
            .collect()?;

        println!("✅ Loaded {} records, now calculating DOTS scores...", df.height());

        // Step 2: Add weight class and DOTS calculations
        df = df
            .lazy()
            .with_columns([
                calculate_weight_class_expr(),
            ])
            .collect()?;

        println!("✅ Weight classes calculated, adding DOTS...");

        // Step 3: Add DOTS calculations one by one to catch errors
        df = df
            .lazy()
            .with_columns([
                calculate_dots_expr("Best3SquatKg", "SquatDOTS"),
            ])
            .collect()
            .map_err(|e| {
                println!("❌ Error calculating SquatDOTS: {}", e);
                e
            })?;

        println!("✅ SquatDOTS calculated");

        df = df
            .lazy()
            .with_columns([
                calculate_dots_expr("Best3BenchKg", "BenchDOTS"),
            ])
            .collect()
            .map_err(|e| {
                println!("❌ Error calculating BenchDOTS: {}", e);
                e
            })?;

        println!("✅ BenchDOTS calculated");

        df = df
            .lazy()
            .with_columns([
                calculate_dots_expr("Best3DeadliftKg", "DeadliftDOTS"),
            ])
            .collect()
            .map_err(|e| {
                println!("❌ Error calculating DeadliftDOTS: {}", e);
                e
            })?;

        println!("✅ DeadliftDOTS calculated");

        df = df
            .lazy()
            .with_columns([
                calculate_dots_expr("TotalKg", "TotalDOTS"),
            ])
            .collect()
            .map_err(|e| {
                println!("❌ Error calculating TotalDOTS: {}", e);
                e
            })?;

        println!("✅ TotalDOTS calculated");

        // Step 4: Filter out any invalid DOTS values
        df = df
            .lazy()
            .filter(
                col("SquatDOTS").is_finite()
                    .and(col("BenchDOTS").is_finite())
                    .and(col("DeadliftDOTS").is_finite())
                    .and(col("TotalDOTS").is_finite())
                    .and(col("SquatDOTS").gt(0.0))
                    .and(col("BenchDOTS").gt(0.0))
                    .and(col("DeadliftDOTS").gt(0.0))
                    .and(col("TotalDOTS").gt(0.0))
            )
            .collect()?;

        self.validate_dots_data(&df);
        let df = self.apply_sampling(df)?;
        
        println!("✅ Processed {} records from CSV in {:?}", df.height(), start.elapsed());
        Ok(df)
    }
    
    fn validate_dots_data(&self, df: &DataFrame) {
        println!("🔍 Validating DOTS data...");
        
        // Check each DOTS column
        for (lift_name, column_name) in [
            ("Squat", "SquatDOTS"),
            ("Bench", "BenchDOTS"),
            ("Deadlift", "DeadliftDOTS"),
            ("Total", "TotalDOTS"),
        ] {
            if let Ok(column) = df.column(column_name) {
                if let Ok(f32_series) = column.f32() {
                    let all_count = f32_series.len();
                    let null_count = f32_series.null_count();
                    let finite_count = f32_series.into_no_null_iter()
                        .filter(|&x| x.is_finite())
                        .count();
                    let positive_count = f32_series.into_no_null_iter()
                        .filter(|&x| x.is_finite() && x > 0.0)
                        .count();
                    let reasonable_count = f32_series.into_no_null_iter()
                        .filter(|&x| x.is_finite() && x > 0.0 && x < 1000.0)
                        .count();
                    
                    println!("📊 {} DOTS: {} total, {} null, {} finite, {} positive, {} reasonable",
                        lift_name, all_count, null_count, finite_count, positive_count, reasonable_count);
                    
                    if reasonable_count > 0 {
                        let values: Vec<f32> = f32_series.into_no_null_iter()
                            .filter(|&x| x.is_finite() && x > 0.0 && x < 1000.0)
                            .collect();
                        let avg = values.iter().sum::<f32>() / values.len() as f32;
                        let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
                        let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                        println!("📈 {} DOTS range: {:.1} - {:.1}, avg: {:.1}", 
                            lift_name, min, max, avg);
                    }
                } else {
                    println!("❌ {} DOTS column is not f32 type", lift_name);
                }
            } else {
                println!("❌ {} DOTS column not found", lift_name);
            }
        }
    }
    
    fn save_as_parquet(&self, df: &DataFrame, csv_path: &std::path::Path) -> PolarsResult<()> {
        std::fs::create_dir_all("data")?;
        
        let csv_filename = csv_path.file_stem().unwrap().to_str().unwrap();
        let parquet_path = format!("data/{}.parquet", csv_filename);
        
        let mut file = std::fs::File::create(&parquet_path)?;
        let mut df_mut = df.clone();
        ParquetWriter::new(&mut file)
            .with_compression(ParquetCompression::Snappy)
            .with_statistics(StatisticsOptions::default())
            .finish(&mut df_mut)?;
        
        println!("💾 Saved processed data to: {}", parquet_path);
        Ok(())
    }
    
    fn apply_sampling(&self, df: DataFrame) -> PolarsResult<DataFrame> {
        if df.height() > self.sample_size {
            println!("📊 Sampling {} records from {} total for performance", self.sample_size, df.height());
            df.sample_n_literal(self.sample_size, true, true, Some(42))
        } else {
            Ok(df)
        }
    }
    
    fn create_sample_data(&self) -> PolarsResult<DataFrame> {
        use rand::prelude::*;
        use rand::seq::SliceRandom;
        
        println!("🎯 Creating sample powerlifting data for demo...");
        
        let mut rng = rand::rng();
        let sample_size = 10000;
        
        let mut data = SampleDataBuilder::new(sample_size);
        
        for i in 0..sample_size {
            let sex = if rng.random_bool(0.7) { "M" } else { "F" };
            let eq = ["Raw", "Wraps", "Single-ply"].choose(&mut rng).unwrap();
            
            let lifter = data.generate_lifter(i, sex, eq, &mut rng);
            data.add_lifter(lifter);
        }
        
        let df = data.build()?;
        self.validate_dots_data(&df);
        
        println!("✅ Generated {} sample records with DOTS scoring", df.height());
        Ok(df)
    }

    pub fn convert_csv_to_parquet(&self, csv_path: &str, parquet_path: Option<&str>) -> PolarsResult<()> {
        let csv_path_buf = std::path::Path::new(csv_path);
        let output_path = if let Some(path) = parquet_path {
            path.to_string()
        } else {
            let csv_filename = csv_path_buf.file_stem().unwrap().to_str().unwrap();
            format!("{}.parquet", csv_filename)
        };
        
        println!("🔄 Converting {} to {}", csv_path, output_path);
        let start = std::time::Instant::now();
        
        let df = self.load_real_data(csv_path_buf)?;
        
        let mut file = std::fs::File::create(&output_path)?;
        let mut df_mut = df.clone();
        ParquetWriter::new(&mut file)
            .with_compression(ParquetCompression::Snappy)
            .with_statistics(StatisticsOptions::default())
            .finish(&mut df_mut)?;
        
        println!("✅ Conversion completed in {:?}", start.elapsed());
        println!("📊 Original records: {}", df.height());
        
        if let (Ok(csv_meta), Ok(parquet_meta)) = (
            std::fs::metadata(csv_path),
            std::fs::metadata(&output_path)
        ) {
            let compression_ratio = (1.0 - parquet_meta.len() as f64 / csv_meta.len() as f64) * 100.0;
            println!("💾 Size reduction: {:.1}% ({} MB → {} MB)", 
                compression_ratio,
                csv_meta.len() / 1_000_000,
                parquet_meta.len() / 1_000_000
            );
        }
        
        Ok(())
    }
}


struct SampleDataBuilder {
    names: Vec<String>,
    sexes: Vec<String>,
    equipment: Vec<String>,
    bodyweights: Vec<f32>,
    squats: Vec<f32>,
    benches: Vec<f32>,
    deadlifts: Vec<f32>,
    totals: Vec<f32>,
}

impl SampleDataBuilder {
    fn new(capacity: usize) -> Self {
        Self {
            names: Vec::with_capacity(capacity),
            sexes: Vec::with_capacity(capacity),
            equipment: Vec::with_capacity(capacity),
            bodyweights: Vec::with_capacity(capacity),
            squats: Vec::with_capacity(capacity),
            benches: Vec::with_capacity(capacity),
            deadlifts: Vec::with_capacity(capacity),
            totals: Vec::with_capacity(capacity),
        }
    }
    
    fn generate_lifter(&self, id: usize, sex: &str, equipment: &str, rng: &mut impl rand::Rng) -> SampleLifter {
        use rand_distr::{Normal, Distribution};
        
        let (bw_mean, bw_std, sq_ratio, bp_ratio, dl_ratio) = if sex == "M" {
            (85.0, 15.0, 1.8, 1.3, 2.2)
        } else {
            (65.0, 12.0, 1.4, 0.8, 1.8)
        };
        
        let bodyweight_sample: f32 = Normal::new(bw_mean, bw_std).unwrap().sample(rng);
        let bodyweight = bodyweight_sample.clamp(40.0, 200.0);
        let squat: f32 = (bodyweight * sq_ratio * rng.random_range(0.7..1.3)).max(50.0);
        let bench: f32 = (bodyweight * bp_ratio * rng.random_range(0.7..1.3)).max(30.0);
        let deadlift: f32 = (bodyweight * dl_ratio * rng.random_range(0.7..1.3)).max(60.0);
        let total = squat + bench + deadlift;
        
        SampleLifter {
            name: format!("Lifter{}", id),
            sex: sex.to_string(),
            equipment: equipment.to_string(),
            bodyweight,
            squat,
            bench,
            deadlift,
            total,
        }
    }
    
    fn add_lifter(&mut self, lifter: SampleLifter) {
        self.names.push(lifter.name);
        self.sexes.push(lifter.sex);
        self.equipment.push(lifter.equipment);
        self.bodyweights.push(lifter.bodyweight);
        self.squats.push(lifter.squat);
        self.benches.push(lifter.bench);
        self.deadlifts.push(lifter.deadlift);
        self.totals.push(lifter.total);
    }
    
    fn build(self) -> PolarsResult<DataFrame> {
        let df = df! {
            "Name" => self.names,
            "Sex" => self.sexes,
            "Equipment" => self.equipment,
            "BodyweightKg" => self.bodyweights,
            "Best3SquatKg" => self.squats,
            "Best3BenchKg" => self.benches,
            "Best3DeadliftKg" => self.deadlifts,
            "TotalKg" => self.totals,
        }?;

        // Add calculations step by step
        let df = df
            .lazy()
            .with_columns([calculate_weight_class_expr()])
            .collect()?;

        let df = df
            .lazy()
            .with_columns([
                calculate_dots_expr("Best3SquatKg", "SquatDOTS"),
                calculate_dots_expr("Best3BenchKg", "BenchDOTS"), 
                calculate_dots_expr("Best3DeadliftKg", "DeadliftDOTS"),
                calculate_dots_expr("TotalKg", "TotalDOTS"),
            ])
            .collect()?;

        Ok(df)
    }
}

struct SampleLifter {
    name: String,
    sex: String,
    equipment: String,
    bodyweight: f32,
    squat: f32,
    bench: f32,
    deadlift: f32,
    total: f32,
}