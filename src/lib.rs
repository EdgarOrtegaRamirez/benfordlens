//! # BenfordLens — Benford's Law Analysis Toolkit
//!
//! A comprehensive Rust library and CLI for analyzing numeric datasets for
//! conformity to Benford's Law (the first-digit law).
//!
//! ## Overview
//!
//! Benford's Law states that in many naturally-occurring collections of numbers,
//! the leading digit is likely to be small: 1 appears ~30.1% of the time, while
//! 9 appears only ~4.6% of the time. Deviations from this distribution can indicate
//! fraud, fabricated data, or anomalous datasets.
//!
//! ## Features
//!
//! - **Digit distributions**: first, second, third, first-two, first-three, last-two digits
//! - **Goodness-of-fit tests**: chi-square, Kolmogorov-Smirnov, Mean Absolute Deviation (MAD)
//! - **Per-digit Z-statistics**: identify which specific digits deviate significantly
//! - **Risk scoring**: composite 0-100 fraud/anomaly score with risk levels
//! - **ASCII visualization**: bar charts comparing observed vs expected distributions
//! - **Multi-format reports**: text, JSON, markdown, HTML
//! - **Multi-format input**: CSV, JSON, JSONL, plain text, stdin
//! - **Zero ML dependencies**: pure Rust statistical implementation
//!
//! ## Quick Start
//!
//! ```no_run
//! use benfordlens::{analysis, data, digit::DigitPosition};
//!
//! let dataset = data::load_file(std::path::Path::new("data.csv"), Some("amount"))
//!     .expect("failed to load");
//! let result = analysis::analyze(&dataset, DigitPosition::First)
//!     .expect("analysis failed");
//! println!("Risk score: {:.1}/100 ({})", result.risk_score, result.risk_level.label());
//! ```

pub mod analysis;
pub mod cli;
pub mod data;
pub mod digit;
pub mod distribution;
pub mod report;
pub mod statistics;
pub mod visualize;

pub use analysis::{AnalysisResult, RiskLevel};
pub use data::{load_file, load_stdin, Dataset};
pub use digit::{extract_digit, extract_digits, DigitPosition};
pub use distribution::{expected_distribution, expected_probability};
pub use statistics::{ChiSquareResult, ConformityLevel, KsResult, MadResult};
