// data.rs
use polars::prelude::*;
use std::sync::Arc;
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
        
        let csv_file = self.find_csv_file();
        
        match csv_file {
            Some(path) => {
                println!("📂 Found powerlifting data: {:?}", path.file_name().unwrap());
                self.load_real_data(&path)
            }
            None => {
                println!("⚠️  No openpowerlifting CSV files found in data/ directory");
                println!("💡 Please download from: https://openpowerlifting.gitlab.io/opl-csv/bulk-csv.html");
                println!("🎯 Creating sample data for demo...");
                self.create_sample_data()
            }
        }
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
    }
    
    fn load_real_data(&self, path: &std::path::Path) -> PolarsResult<DataFrame> {
        // Define schema with proper float types for lift columns
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
        
        let df = LazyCsvReader::new(path)
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
                col("Best3SquatKg").gt(0)
                    .and(col("Best3BenchKg").gt(0))
                    .and(col("Best3DeadliftKg").gt(0))
            )
            .filter(col("BodyweightKg").gt(0))
            .with_columns([
                calculate_weight_class_expr(),
                calculate_dots_expr("Best3SquatKg", "SquatDOTS"),
                calculate_dots_expr("Best3BenchKg", "BenchDOTS"), 
                calculate_dots_expr("Best3DeadliftKg", "DeadliftDOTS"),
                calculate_dots_expr("TotalKg", "TotalDOTS"),
            ])
            .collect()?;
        
        let df = self.apply_sampling(df)?;
        
        println!("✅ Processed {} records with DOTS scoring", df.height());
        Ok(df)
    }
    
    fn apply_sampling(&self, df: DataFrame) -> PolarsResult<DataFrame> {
        if df.height() > self.sample_size {
            println!("📊 Sampling {} records from {} total for better performance", self.sample_size, df.height());
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
        
        println!("✅ Generated {} sample records with DOTS scoring", df.height());
        Ok(df)
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
            (85.0, 15.0, 1.8, 1.3, 2.2) // Male averages
        } else {
            (65.0, 12.0, 1.4, 0.8, 1.8) // Female averages
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
        df! {
            "Name" => self.names,
            "Sex" => self.sexes,
            "Equipment" => self.equipment,
            "BodyweightKg" => self.bodyweights,
            "Best3SquatKg" => self.squats,
            "Best3BenchKg" => self.benches,
            "Best3DeadliftKg" => self.deadlifts,
            "TotalKg" => self.totals,
        }?
        .lazy()
        .with_columns([
            calculate_weight_class_expr(),
            calculate_dots_expr("Best3SquatKg", "SquatDOTS"),
            calculate_dots_expr("Best3BenchKg", "BenchDOTS"), 
            calculate_dots_expr("Best3DeadliftKg", "DeadliftDOTS"),
            calculate_dots_expr("TotalKg", "TotalDOTS"),
        ])
        .collect()
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