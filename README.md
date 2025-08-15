# 🏋️ Iron Insights

## High-Performance Powerlifting Analytics with DOTS Scoring

A blazing-fast web application for analyzing powerlifting performance data using modern DOTS scoring. Built with Rust, Polars, and Axum for maximum performance and accuracy.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=WebAssembly&logoColor=white)
![Chart.js](https://img.shields.io/badge/chart.js-F5788D.svg?style=for-the-badge&logo=chart.js&logoColor=white)

### 🚀 **WebAssembly Integration**

- **⚡ Instant calculations** - Client-side DOTS calculations using compiled Rust
- **🎯 Strength classification** - Real-time strength level assessment (Beginner to World Class)
- **🔧 Consistent precision** - Same calculation logic on client and server
- **📱 Responsive UI** - Immediate feedback when users enter their lifts

## ✨ Features

### 🎯 **Modern DOTS Scoring**

- **DOTS (Dots Total)** - Modern strength scoring system
- **Gender-neutral formula** - Single coefficient set for all athletes
- **Accurate comparisons** - Normalized strength across bodyweight ranges
- **Industry standard** - Used in modern powerlifting competitions

### 📊 **Advanced Analytics**

- **Real-time visualizations** with interactive charts
- **Dual percentile system** - Raw weight vs DOTS comparisons
- **Performance filtering** by sex, equipment, and weight class
- **Scatter plot analysis** - Bodyweight vs performance relationships

### ⚡ **High Performance**

- **Rust-powered backend** - Memory-safe and lightning-fast
- **Polars data processing** - Optimized columnar operations
- **Intelligent caching** - Sub-second response times
- **Parallel processing** - Multi-core histogram generation

### 🎨 **Modern UI**

- **Responsive design** - Works on desktop and mobile
- **Interactive charts** - Powered by Plotly.js
- **Real-time updates** - Instant feedback on parameter changes
- **Professional styling** - Clean, modern interface

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- Git

### Installation

1. **Clone the repository**

   ```bash
   git clone https://github.com/yourusername/iron-insights.git
   cd iron-insights
   ```

2. **Build and run**

   ```bash
   cargo run --release
   ```

3. **Open your browser**

   ```text
   http://localhost:3000
   ```

### 📂 Data Setup (Optional)

For real data analysis, download the OpenPowerlifting dataset:

1. **Download CSV data**
   - Visit [OpenPowerlifting Bulk CSV](https://openpowerlifting.gitlab.io/opl-csv/bulk-csv.html)
   - Download the latest `openpowerlifting-YYYY-MM-DD-REVISION.csv` file

2. **Place in data directory**

   ```bash
   mkdir data
   mv openpowerlifting-*.csv data/
   ```

3. **Restart the application**
   - The app will automatically detect and load your data
   - Sample data is used if no CSV is found

## 🏗️ Architecture

```text
src/
├── main.rs           # Application entry point and server setup
├── config.rs         # Configuration management
├── models.rs         # Data structures and API types
├── data.rs           # Data loading and preprocessing
├── scoring.rs        # DOTS calculation engine
├── handlers.rs       # HTTP request handlers
├── cache.rs          # Caching layer implementation
├── filters.rs        # Data filtering logic
├── percentiles.rs    # Percentile calculations
├── viz.rs            # Data visualization utilities
├── share_card.rs     # Social sharing card generation
├── websocket.rs      # WebSocket real-time communication
├── arrow_utils.rs    # Apache Arrow utilities
├── debug_dots.rs     # DOTS calculation debugging
├── ui/               # Frontend UI components
│   ├── mod.rs        # UI module organization
│   └── components/   # Reusable UI components
│       ├── charts.rs     # Chart rendering
│       ├── controls.rs   # User input controls
│       ├── head.rs       # HTML head section
│       ├── header.rs     # Page header
│       ├── metrics.rs    # Performance metrics display
│       ├── realtime.rs   # Real-time updates
│       ├── scripts.rs    # JavaScript integration
│       ├── share_card.rs # Share card component
│       └── styles.rs     # CSS styling
└── wasm/             # WebAssembly module
    ├── Cargo.toml    # WASM-specific dependencies
    └── lib.rs        # WASM bindings for client-side calculations

static/
└── wasm/             # Compiled WebAssembly assets
    ├── iron_insights_wasm.js
    ├── iron_insights_wasm_bg.wasm
    └── *.d.ts        # TypeScript definitions
```

### 🧱 **Core Components**

- **Data Layer** - Apache Arrow/Polars-based processing with Parquet support
- **Scoring Engine** - Vectorized DOTS calculations (server + WASM)
- **Cache Layer** - Multi-tier caching with intelligent invalidation
- **Web Layer** - Axum async HTTP server with WebSocket support
- **Visualization** - Interactive charts with real-time updates
- **WebAssembly** - Client-side calculations for instant feedback
- **UI Components** - Modular, reusable frontend architecture

## 📊 DOTS Scoring

### DOTS Formula

```text
DOTS = Lift × (500 / (A + B×BW + C×BW² + D×BW³ + E×BW⁴))
```

**Coefficients:**

- A = -307.75076
- B = 24.0900756
- C = -0.1918759221
- D = 0.0007391293
- E = -0.000001093

## 🔧 Configuration

### Environment Variables

```bash
# Server configuration
IRON_INSIGHTS_PORT=3000
IRON_INSIGHTS_HOST=0.0.0.0

# Cache settings
CACHE_MAX_CAPACITY=1000
CACHE_TTL_SECONDS=3600

# Data processing
SAMPLE_SIZE=50000
HISTOGRAM_BINS=50
```

### Config File (`config.rs`)

```rust
pub struct AppConfig {
    pub cache_max_capacity: u64,
    pub cache_ttl_seconds: u64,
    pub sample_size: usize,
    pub histogram_bins: usize,
}
```

## 🧪 API Reference

### `POST /api/visualize`

Generate visualizations with filtering parameters.

**Request Body:**

```json
{
    "sex": "M",
    "lift_type": "squat",
    "bodyweight": 75.0,
    "squat": 180.0,
    "equipment": ["Raw"]
}
```

**Response:**

```json
{
    "histogram_data": { "values": [...], "bins": [...] },
    "scatter_data": { "x": [...], "y": [...], "sex": [...] },
    "dots_histogram_data": { "values": [...], "bins": [...] },
    "dots_scatter_data": { "x": [...], "y": [...], "sex": [...] },
    "user_percentile": 75.2,
    "user_dots_percentile": 78.5,
    "processing_time_ms": 45,
    "total_records": 48392
}
```

### `GET /api/stats`

Get application statistics.

**Response:**

```json
{
    "total_records": 48392,
    "cache_size": 127,
    "scoring_system": "DOTS",
    "uptime": "running"
}
```

## 🔬 Performance Benchmarks

| Operation | Time | Records |
|-----------|------|---------|
| **Data Loading** | ~2.5s | 2.2M records |
| **DOTS Calculation** | ~890ms | 2.2M records |
| **Histogram Generation** | ~23ms | 50K records |
| **Percentile Calculation** | ~8ms | 50K records |
| **API Response** | ~45ms | Full pipeline |

## Benchmarks on AMD Ryzen 7 5800X, 32GB RAM

### 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module tests
cargo test scoring::tests
```

#### Test Coverage

- ✅ DOTS calculation accuracy
- ✅ Weight class assignment
- ✅ Data loading pipeline
- ✅ API endpoint responses
- ✅ Caching behavior

## 🚀 Deployment

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/iron-insights /usr/local/bin/
EXPOSE 3000
CMD ["iron-insights"]
```

### Build Commands

```bash
# Development build
cargo build

# Production build
cargo build --release

# With optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## 🤝 Contributing

1. **Fork the repository**
2. **Create your feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run

# Format code
cargo fmt

# Lint code
cargo clippy
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[OpenPowerlifting](https://www.openpowerlifting.org/)** - Comprehensive powerlifting database
- **[DOTS Formula](https://en.wikipedia.org/wiki/Powerlifting#DOTS)** - Modern strength scoring system
- **[Polars](https://pola.rs/)** - Lightning-fast DataFrame library
- **[Axum](https://github.com/tokio-rs/axum)** - Ergonomic async web framework

## 📊 Data Sources

- **Primary**: OpenPowerlifting dataset (2.2M+ competition results)
- **Fallback**: Generated sample data for demonstration
- **Format**: CSV with standardized powerlifting meet data
- **Update Frequency**: Weekly from OpenPowerlifting

## 🔮 Roadmap

- [ ] **GLPoints scoring** - Alternative to DOTS
- [ ] **Historical trends** - Performance over time
- [ ] **Meet predictions** - ML-based performance forecasting
- [ ] **Mobile app** - Native iOS/Android versions
- [ ] **Federation analysis** - IPF vs USPA comparisons
- [ ] **Real-time updates** - WebSocket live data
- [ ] **Advanced filtering** - Age groups, drug testing
- [ ] **Export functionality** - PDF reports, CSV exports

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/gregorycarnegie/iron-insights/issues)
- **Discussions**: [GitHub Discussions](https://github.com/gregorycarnegie/iron-insights/discussions)

---

## Vibe coded with ❤️ and ⚡ by the Iron Insights team
