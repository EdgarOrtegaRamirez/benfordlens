//! CLI command definitions for BenfordLens.

use crate::analysis;
use crate::data::{self, Dataset};
use crate::digit::DigitPosition;
use crate::report::{self, OutputFormat};
use crate::visualize;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::PathBuf;

/// BenfordLens — Benford's Law analysis toolkit for fraud detection and data integrity.
#[derive(Parser, Debug)]
#[command(
    name = "benfordlens",
    version,
    about = "Benford's Law analysis toolkit for fraud detection, data integrity validation, and anomaly detection",
    long_about = "BenfordLens analyzes numeric datasets for conformity to Benford's Law (the first-digit law). \
                  It performs chi-square, Kolmogorov-Smirnov, and MAD goodness-of-fit tests, computes per-digit \
                  Z-statistics, generates a fraud/anomaly risk score, and produces reports in text, JSON, \
                  markdown, and HTML formats."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a full Benford's Law analysis on a data file.
    Analyze {
        /// Input file (CSV, JSON, JSONL, or plain text). Use "-" for stdin.
        #[arg(short, long)]
        input: String,

        /// Column name to analyze (CSV/JSON). Defaults to first column.
        #[arg(short, long)]
        column: Option<String>,

        /// Digit position to analyze.
        #[arg(short, long, value_enum, default_value = "first")]
        position: PositionArg,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "text")]
        format: FormatArg,

        /// Output file (defaults to stdout).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Analyze all digit positions and generate a combined report.
    AnalyzeAll {
        /// Input file (CSV, JSON, JSONL, or plain text). Use "-" for stdin.
        #[arg(short, long)]
        input: String,

        /// Column name to analyze (CSV/JSON).
        #[arg(short, long)]
        column: Option<String>,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "text")]
        format: FormatArg,

        /// Output file (defaults to stdout).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show the digit distribution chart for a specific position.
    Digits {
        /// Input file.
        #[arg(short, long)]
        input: String,

        /// Column name.
        #[arg(short, long)]
        column: Option<String>,

        /// Digit position.
        #[arg(short, long, value_enum, default_value = "first")]
        position: PositionArg,
    },

    /// Run goodness-of-fit tests (chi-square, KS, MAD).
    Test {
        /// Input file.
        #[arg(short, long)]
        input: String,

        /// Column name.
        #[arg(short, long)]
        column: Option<String>,

        /// Digit position.
        #[arg(short, long, value_enum, default_value = "first")]
        position: PositionArg,
    },

    /// Compute the fraud/anomaly risk score.
    Score {
        /// Input file.
        #[arg(short, long)]
        input: String,

        /// Column name.
        #[arg(short, long)]
        column: Option<String>,

        /// Digit position.
        #[arg(short, long, value_enum, default_value = "first")]
        position: PositionArg,
    },

    /// Visualize the observed vs expected distribution.
    Viz {
        /// Input file.
        #[arg(short, long)]
        input: String,

        /// Column name.
        #[arg(short, long)]
        column: Option<String>,

        /// Digit position.
        #[arg(short, long, value_enum, default_value = "first")]
        position: PositionArg,
    },

    /// Show Benford's Law reference information (expected distributions).
    Info {
        /// Digit position to show.
        #[arg(short, long, value_enum, default_value = "first")]
        position: PositionArg,
    },

    /// Generate sample data that follows Benford's Law (for testing/demo).
    Sample {
        /// Number of values to generate.
        #[arg(short, long, default_value = "1000")]
        count: usize,

        /// Output file (defaults to stdout).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// CLI enum for digit positions.
#[derive(Clone, Debug, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionArg {
    First,
    Second,
    Third,
    FirstTwo,
    FirstThree,
    LastTwo,
}

impl PositionArg {
    pub fn to_position(&self) -> DigitPosition {
        match self {
            PositionArg::First => DigitPosition::First,
            PositionArg::Second => DigitPosition::Second,
            PositionArg::Third => DigitPosition::Third,
            PositionArg::FirstTwo => DigitPosition::FirstTwo,
            PositionArg::FirstThree => DigitPosition::FirstThree,
            PositionArg::LastTwo => DigitPosition::LastTwo,
        }
    }
}

/// CLI enum for output formats.
#[derive(Clone, Debug, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatArg {
    Text,
    Json,
    Markdown,
    Html,
}

impl FormatArg {
    pub fn to_format(&self) -> OutputFormat {
        match self {
            FormatArg::Text => OutputFormat::Text,
            FormatArg::Json => OutputFormat::Json,
            FormatArg::Markdown => OutputFormat::Markdown,
            FormatArg::Html => OutputFormat::Html,
        }
    }
}

/// Runs the CLI command.
pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Analyze {
            input,
            column,
            position,
            format,
            output,
        } => run_analyze(
            &input,
            column.as_deref(),
            position.to_position(),
            format.to_format(),
            output,
        ),
        Commands::AnalyzeAll {
            input,
            column,
            format,
            output,
        } => run_analyze_all(&input, column.as_deref(), format.to_format(), output),
        Commands::Digits {
            input,
            column,
            position,
        } => run_digits(&input, column.as_deref(), position.to_position()),
        Commands::Test {
            input,
            column,
            position,
        } => run_test(&input, column.as_deref(), position.to_position()),
        Commands::Score {
            input,
            column,
            position,
        } => run_score(&input, column.as_deref(), position.to_position()),
        Commands::Viz {
            input,
            column,
            position,
        } => run_viz(&input, column.as_deref(), position.to_position()),
        Commands::Info { position } => run_info(position.to_position()),
        Commands::Sample { count, output } => run_sample(count, output),
    }
}

fn load_dataset(input: &str, column: Option<&str>) -> Result<Dataset> {
    if input == "-" {
        data::load_stdin(column).map_err(anyhow::Error::msg)
    } else {
        data::load_file(&PathBuf::from(input), column).map_err(anyhow::Error::msg)
    }
}

fn write_output(content: &str, output: &Option<PathBuf>) -> Result<()> {
    match output {
        Some(path) => {
            std::fs::write(path, content)
                .map_err(|e| anyhow::anyhow!("failed to write output file: {}", e))?;
            eprintln!("Report written to {}", path.display());
        }
        None => {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            handle
                .write_all(content.as_bytes())
                .map_err(|e| anyhow::anyhow!("failed to write to stdout: {}", e))?;
        }
    }
    Ok(())
}

fn run_analyze(
    input: &str,
    column: Option<&str>,
    position: DigitPosition,
    format: OutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    let dataset = load_dataset(input, column)?;
    if dataset.is_empty() {
        anyhow::bail!("no valid numeric values found in input");
    }
    let result = analysis::analyze(&dataset, position).map_err(anyhow::Error::msg)?;
    let report = report::generate_report(&result, &dataset, format);
    write_output(&report, &output)
}

fn run_analyze_all(
    input: &str,
    column: Option<&str>,
    format: OutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    let dataset = load_dataset(input, column)?;
    if dataset.is_empty() {
        anyhow::bail!("no valid numeric values found in input");
    }
    let results = analysis::analyze_all(&dataset);
    let report = report::generate_combined_report(&results, &dataset, format);
    write_output(&report, &output)
}

fn run_digits(input: &str, column: Option<&str>, position: DigitPosition) -> Result<()> {
    let dataset = load_dataset(input, column)?;
    if dataset.is_empty() {
        anyhow::bail!("no valid numeric values found in input");
    }
    let result = analysis::analyze(&dataset, position).map_err(anyhow::Error::msg)?;
    let chart = visualize::bar_chart(&result, 70);
    println!("{}", chart);
    Ok(())
}

fn run_test(input: &str, column: Option<&str>, position: DigitPosition) -> Result<()> {
    let dataset = load_dataset(input, column)?;
    if dataset.is_empty() {
        anyhow::bail!("no valid numeric values found in input");
    }
    let result = analysis::analyze(&dataset, position).map_err(anyhow::Error::msg)?;
    println!("{}", visualize::summary_chart(&result));
    println!();
    println!("{}", visualize::deviation_chart(&result));
    Ok(())
}

fn run_score(input: &str, column: Option<&str>, position: DigitPosition) -> Result<()> {
    let dataset = load_dataset(input, column)?;
    if dataset.is_empty() {
        anyhow::bail!("no valid numeric values found in input");
    }
    let result = analysis::analyze(&dataset, position).map_err(anyhow::Error::msg)?;
    println!(
        "Risk Score: {:.1}/100  [{}]",
        result.risk_score,
        result.risk_level.label()
    );
    println!(
        "MAD:        {:.6}  [{}]",
        result.mad.mad,
        result.mad.conformity.label()
    );
    println!(
        "Chi-square: {:.4} (df={}, p={:.6})  {}",
        result.chi_square.statistic,
        result.chi_square.degrees_of_freedom,
        result.chi_square.p_value,
        if result.chi_square.significant {
            "SIGNIFICANT"
        } else {
            "not significant"
        }
    );
    println!(
        "KS:         {:.6} (p={:.6})  {}",
        result.ks.statistic,
        result.ks.p_value,
        if result.ks.significant {
            "SIGNIFICANT"
        } else {
            "not significant"
        }
    );
    Ok(())
}

fn run_viz(input: &str, column: Option<&str>, position: DigitPosition) -> Result<()> {
    let dataset = load_dataset(input, column)?;
    if dataset.is_empty() {
        anyhow::bail!("no valid numeric values found in input");
    }
    let result = analysis::analyze(&dataset, position).map_err(anyhow::Error::msg)?;
    println!("{}", visualize::bar_chart(&result, 70));
    Ok(())
}

fn run_info(position: DigitPosition) -> Result<()> {
    println!("Benford's Law — {} Distribution", position.label());
    println!("{}\n", "─".repeat(50));
    println!("Benford's Law states that for many naturally-occurring collections of");
    println!("numbers, the leading digit d occurs with probability:");
    println!();
    if matches!(
        position,
        DigitPosition::First | DigitPosition::FirstTwo | DigitPosition::FirstThree
    ) {
        println!("    P(d) = log10(1 + 1/d)");
    } else if matches!(position, DigitPosition::Second | DigitPosition::Third) {
        println!("    P(d) = sum over prefixes of log10(1 + 1/(prefix*10 + d))");
    } else {
        println!("    P(d) = 1/100  (uniform — for detecting fabricated/rounded data)");
    }
    println!();
    println!("Expected distribution:\n");
    println!(
        "  {:>6}  {:>10}  {:>10}",
        "Digit", "Probability", "Percentage"
    );
    println!(
        "  {:>6}  {:>10}  {:>10}",
        "-----", "-----------", "----------"
    );
    for (d, p) in crate::distribution::expected_distribution(position) {
        println!("  {:>6}  {:>10.6}  {:>9.4}%", d, p, p * 100.0);
    }
    let sum: f64 = crate::distribution::expected_distribution(position)
        .iter()
        .map(|(_, p)| p)
        .sum();
    println!("  {:>6}  {:>10.6}  {:>9.4}%", "Total", sum, sum * 100.0);
    println!();
    println!("Applications: fraud detection, forensic accounting, data integrity");
    println!("validation, election analysis, tax audit screening, scientific");
    println!("data integrity checking.");
    Ok(())
}

fn run_sample(count: usize, output: Option<PathBuf>) -> Result<()> {
    // Generate data following Benford's Law using the inverse CDF method.
    // For first digit d: CDF(d) = sum_{k=1}^{d} log10(1+1/k) = log10(1 + d)
    use std::time::Instant;
    let start = Instant::now();
    let mut rng = SimpleRng::new(start.elapsed().as_nanos() as u64 ^ 0x9E3779B97F4A7C15);
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let u = rng.next_f64();
        // Inverse CDF: first digit d where log10(1+d) >= u, i.e., d = ceil(10^u - 1)
        let d = (10f64.powf(u) - 1.0).ceil() as u32;
        let d = d.clamp(1, 9);
        // Add random mantissa for variety
        let mantissa = d as f64 + rng.next_f64();
        let exponent = (rng.next_u32() % 6) as i32 - 2; // 10^-2 to 10^3
        let value = mantissa * 10f64.powi(exponent);
        values.push(value);
    }

    let content = values
        .iter()
        .map(|v| format!("{:.4}", v))
        .collect::<Vec<_>>()
        .join("\n");
    write_output(&(content + "\n"), &output)
}

/// Simple xorshift RNG for deterministic-enough sample generation.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state & 0xFFFFFFFF) as u32
    }

    fn next_f64(&mut self) -> f64 {
        let hi = self.next_u32() as u64;
        let lo = self.next_u32() as u64;
        let bits = (hi << 32) | lo;
        // Map to [0, 1)
        (bits as f64) / (u64::MAX as f64 + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_arg_conversion() {
        assert!(matches!(
            PositionArg::First.to_position(),
            DigitPosition::First
        ));
        assert!(matches!(
            PositionArg::Second.to_position(),
            DigitPosition::Second
        ));
        assert!(matches!(
            PositionArg::FirstTwo.to_position(),
            DigitPosition::FirstTwo
        ));
    }

    #[test]
    fn test_format_arg_conversion() {
        assert!(matches!(FormatArg::Text.to_format(), OutputFormat::Text));
        assert!(matches!(FormatArg::Json.to_format(), OutputFormat::Json));
        assert!(matches!(FormatArg::Html.to_format(), OutputFormat::Html));
    }

    #[test]
    fn test_simple_rng_range() {
        let mut rng = SimpleRng::new(42);
        for _ in 0..1000 {
            let v = rng.next_f64();
            assert!(v >= 0.0 && v < 1.0, "rng value out of range: {}", v);
        }
    }

    #[test]
    fn test_simple_rng_deterministic() {
        let mut a = SimpleRng::new(123);
        let mut b = SimpleRng::new(123);
        for _ in 0..10 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }
}
