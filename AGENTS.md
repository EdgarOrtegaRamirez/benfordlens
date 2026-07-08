# BenfordLens

BenfordLens is a comprehensive Benford's Law analysis toolkit for fraud detection, data integrity validation, and anomaly detection. It provides statistical tests (chi-square, KS, MAD), per-digit Z-statistics, risk scoring, and multi-format reporting.

## For AI Agents

This project is a Rust CLI and library for analyzing numeric datasets against Benford's Law.

### Key Files
- `src/lib.rs` — Public API
- `src/cli.rs` — CLI commands (clap)
- `src/analysis.rs` — Core analysis engine
- `src/statistics.rs` — Chi-square, KS, MAD, Z-statistics
- `src/digit.rs` — Digit extraction from numbers
- `src/distribution.rs` — Benford's expected probabilities
- `src/data.rs` — Multi-format data loaders (CSV, JSON, JSONL, text)
- `src/report.rs` — Text/JSON/MD/HTML report generation
- `src/visualize.rs` — ASCII bar chart visualization

### Building
```bash
cargo build --release
cargo test
```

### Usage
```bash
benfordlens analyze --input data.csv --column amount
benfordlens analyze --input data.csv --column amount --format json --output report.json
benfordlens analyze-all --input data.csv --column amount --format markdown
```

### API
```rust
use benfordlens::{analysis, data, DigitPosition};
let dataset = data::load_file(Path::new("data.csv"), Some("amount"))?;
let result = analysis::analyze(&dataset, DigitPosition::First)?;
println!("Risk: {:.1}/100", result.risk_score);
```
