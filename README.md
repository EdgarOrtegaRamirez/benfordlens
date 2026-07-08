# BenfordLens

> Benford's Law analysis toolkit for fraud detection, data integrity validation, and anomaly detection.

Benford's Law states that in many naturally-occurring collections of numbers, the leading digit follows a predictable distribution: **1** appears ~30.1% of the time, while **9** appears only ~4.6%. Deviations from this distribution can indicate fraud, fabricated data, or anomalous datasets.

BenfordLens provides a comprehensive toolkit for analyzing numeric data against Benford's Law using multiple statistical tests, visualizations, and multi-format reporting.

## Features

- **Multiple digit positions**: First, second, third, first-two, first-three, and last-two digits
- **Three goodness-of-fit tests**: Chi-square, Kolmogorov-Smirnov, Mean Absolute Deviation (MAD)
- **Per-digit Z-statistics**: Identify which specific digits deviate significantly
- **Composite risk scoring**: 0-100 score with risk levels (Low, Moderate, High, Critical)
- **ASCII visualization**: Bar charts comparing observed vs expected distributions
- **Multi-format reports**: Text, JSON, Markdown, HTML
- **Multi-format input**: CSV, JSON, JSONL, plain text files, and stdin
- **Zero ML dependencies**: Pure Rust statistical implementation
- **CLI + library**: Use as a command-line tool or embed as a library

## Installation

### From Source

```bash
cargo install --path .
```

### From Crate (when published)

```bash
cargo add benfordlens
```

### Docker

```bash
docker build -t benfordlens .
docker run --rm -v $(pwd):/data benfordlens benfordlens analyze --input /data/your_data.csv
```

## Quick Start

### Analyze a CSV file

```bash
benfordlens analyze --input data.csv --column amount
```

### Analyze with JSON output

```bash
benfordlens analyze --input data.csv --column amount --format json --output report.json
```

### Analyze all digit positions

```bash
benfordlens analyze-all --input data.csv --column amount --format markdown
```

### Use as a library

```rust
use benfordlens::{analysis, data, DigitPosition};

let dataset = data::load_file(std::path::Path::new("data.csv"), Some("amount"))
    .expect("failed to load");
let result = analysis::analyze(&dataset, DigitPosition::First)
    .expect("analysis failed");

println!("Risk score: {:.1}/100 ({})", result.risk_score, result.risk_level.label());
```

## Command Reference

| Command | Description |
|---------|-------------|
| `analyze` | Run full analysis on a data file |
| `analyze-all` | Analyze all digit positions |
| `digits` | Show digit distribution chart for a position |
| `test` | Run goodness-of-fit tests only |
| `score` | Compute fraud/anomaly risk score |
| `viz` | Visualize observed vs expected distribution |
| `info` | Show Benford's Law reference information |
| `sample` | Generate sample Benford-conforming data |

### Flags

| Flag | Description |
|------|-------------|
| `--input <file>` | Input data file (required) |
| `--column <name>` | Column name for CSV/JSON (default: first numeric column) |
| `--position <pos>` | Digit position: first, second, third, first-two, first-three, last-two |
| `--format <fmt>` | Output format: text, json, md, html (default: text) |
| `--output <file>` | Write output to file (default: stdout) |

## Input Formats

### CSV

```csv
id,amount,description
1,120.50,Payment received
2,345.75,Invoice #456
3,1200.00,Wire transfer
```

### JSON

```json
[100, 200, 300, 456.78]
```

Or with named fields:

```json
[{"amount": 100}, {"amount": 200}, {"name": "x"}]
```

### JSONL

```jsonl
{"value": 100}
{"value": 200}
{"value": 300}
```

### Plain Text

```
123
456
789
```

## Understanding Results

### Risk Score

| Score | Level | Interpretation |
|-------|-------|----------------|
| 0–25 | Low | Data conforms well to Benford's Law |
| 26–50 | Moderate | Minor deviations, may be normal variation |
| 51–75 | High | Significant deviations, further investigation recommended |
| 76–100 | Critical | Strong evidence of non-conformity (potential fraud) |

### MAD Conformity Levels

| MAD | Level |
|-----|-------|
| < 0.006 | Close conformity |
| 0.006–0.012 | Acceptable conformity |
| 0.012–0.015 | Marginally acceptable |
| > 0.015 | Nonconformity |

### When Benford's Law Applies

Benford's Law works best with:
- Data spanning **multiple orders of magnitude** (e.g., 0.01 to 10,000)
- **Naturally occurring** numbers (populations, prices, river lengths)
- **Growth-based** data (compounding interest, population growth)
- **Measurements** without arbitrary limits

Benford's Law does **NOT** apply to:
- **Assigned numbers** (invoice numbers, zip codes, phone numbers)
- **Human-determined** numbers (IDs, serial numbers)
- **Constrained ranges** (scores out of 100, percentages 0-100%)
- **Uniform distributions** (random lottery numbers)

## Architecture

```
benfordlens/
├── src/
│   ├── digit.rs        # Digit extraction (significand, mantissa)
│   ├── distribution.rs  # Benford's Law expected probabilities
│   ├── statistics.rs    # Chi-square, KS, MAD, Z-statistics
│   ├── data.rs          # Multi-format data loaders
│   ├── analysis.rs      # Core analysis engine
│   ├── visualize.rs     # ASCII bar chart visualization
│   ├── report.rs        # Text/JSON/MD/HTML report generation
│   ├── cli.rs           # CLI command handling (clap)
│   ├── lib.rs           # Public API
│   └── main.rs          # CLI binary entrypoint
├── tests/
│   └── fixtures/        # Test data files
├── Cargo.toml
├── .github/workflows/ci.yml
└── README.md
```

## Testing

```bash
cargo test            # Run all tests
cargo test --lib      # Unit tests only
cargo test --doc      # Doc tests only
```

## License

MIT License — see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Add tests for new functionality
4. Ensure all tests pass (`cargo test`)
5. Submit a pull request

## Acknowledgments

- Benford, F. (1938). *The Law of Anomalous Numbers*. Proceedings of the American Philosophical Society.
- Statistical methods adapted from Nigrini, M. J. (2012). *Benford's Law: Applications for Forensic Accounting, Auditing, and Fraud Detection*.
