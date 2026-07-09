//! Core Benford's Law analysis engine.
//!
//! Combines digit extraction, expected distributions, and statistical tests
//! into a unified analysis result.

use crate::data::Dataset;
use crate::digit::{self, DigitPosition};
use crate::distribution;
use crate::statistics::{
    self, ChiSquareResult, ConformityLevel, DigitDeviation, KsResult, MadResult,
};

/// A complete Benford's Law analysis result for a single digit position.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisResult {
    /// Which digit position was analyzed.
    pub position: DigitPosition,
    /// Total number of valid values analyzed.
    pub sample_size: usize,
    /// Number of records skipped (invalid/non-positive).
    pub skipped: usize,
    /// Per-digit observed counts and expected values.
    pub digit_counts: Vec<DigitCount>,
    /// Chi-square goodness-of-fit result.
    pub chi_square: ChiSquareResult,
    /// Kolmogorov-Smirnov test result.
    pub ks: KsResult,
    /// Mean Absolute Deviation result.
    pub mad: MadResult,
    /// Per-digit deviation statistics with Z-scores.
    pub deviations: Vec<DigitDeviation>,
    /// Overall fraud/anomaly risk score (0-100, higher = more suspicious).
    pub risk_score: f64,
    /// Risk level classification.
    pub risk_level: RiskLevel,
}

/// Observed vs expected counts for a single digit value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DigitCount {
    /// The digit value.
    pub digit: u32,
    /// Observed count.
    pub observed: usize,
    /// Expected count (Benford).
    pub expected: f64,
    /// Observed proportion.
    pub observed_proportion: f64,
    /// Expected proportion (Benford).
    pub expected_proportion: f64,
}

/// Risk level classification based on combined statistical signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Low risk — data conforms well to Benford's Law.
    Low,
    /// Moderate risk — some deviations detected.
    Moderate,
    /// High risk — significant deviations, possible fraud/anomaly.
    High,
    /// Critical risk — extreme deviations, data likely manipulated.
    Critical,
}

impl RiskLevel {
    pub fn label(&self) -> &'static str {
        match self {
            RiskLevel::Low => "Low",
            RiskLevel::Moderate => "Moderate",
            RiskLevel::High => "High",
            RiskLevel::Critical => "Critical",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            RiskLevel::Low => "green",
            RiskLevel::Moderate => "yellow",
            RiskLevel::High => "red",
            RiskLevel::Critical => "bright_red",
        }
    }
}

/// Runs a complete Benford's Law analysis on a dataset for the given digit position.
pub fn analyze(dataset: &Dataset, position: DigitPosition) -> Result<AnalysisResult, String> {
    if dataset.is_empty() {
        return Err("no valid numeric values in dataset".to_string());
    }

    let digits = digit::extract_digits(&dataset.values, position);
    if digits.is_empty() {
        return Err("no valid digits could be extracted".to_string());
    }

    // Count observed digits
    let mut counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for d in &digits {
        *counts.entry(*d).or_insert(0) += 1;
    }

    // Build observed vector in position order
    let observed: Vec<(u32, usize)> = position
        .values()
        .into_iter()
        .map(|d| (d, counts.get(&d).copied().unwrap_or(0)))
        .collect();

    // Run statistical tests
    let chi_square = statistics::chi_square_test(&observed, position)?;
    let ks = statistics::ks_test(&observed, position)?;
    let mad = statistics::mad_test(&observed, position)?;
    let deviations = statistics::digit_deviations(&observed, position);

    // Build digit counts
    let n = digits.len();
    let n_f = n as f64;
    let expected_dist = distribution::expected_distribution(position);
    let digit_counts: Vec<DigitCount> = observed
        .iter()
        .zip(expected_dist.iter())
        .map(|((d, obs), (_, exp_prop))| DigitCount {
            digit: *d,
            observed: *obs,
            expected: exp_prop * n_f,
            observed_proportion: *obs as f64 / n_f,
            expected_proportion: *exp_prop,
        })
        .collect();

    // Compute risk score
    let (risk_score, risk_level) = compute_risk(&chi_square, &mad, &ks);

    Ok(AnalysisResult {
        position,
        sample_size: n,
        skipped: dataset.skipped,
        digit_counts,
        chi_square,
        ks,
        mad,
        deviations,
        risk_score,
        risk_level,
    })
}

/// Runs analysis across all standard digit positions and returns results for each.
pub fn analyze_all(dataset: &Dataset) -> Vec<AnalysisResult> {
    let positions = [
        DigitPosition::First,
        DigitPosition::Second,
        DigitPosition::Third,
        DigitPosition::FirstTwo,
        DigitPosition::FirstThree,
        DigitPosition::LastTwo,
    ];

    positions
        .iter()
        .filter_map(|pos| analyze(dataset, *pos).ok())
        .collect()
}

/// Computes a composite risk score (0-100) from multiple statistical signals.
///
/// The score combines:
/// - Chi-square p-value (lower p = higher risk)
/// - MAD conformity (higher MAD = higher risk)
/// - KS statistic (higher = higher risk)
/// - Number of significantly deviating digits
fn compute_risk(chi: &ChiSquareResult, mad: &MadResult, ks: &KsResult) -> (f64, RiskLevel) {
    // Chi-square contribution (0-40 points): -log10(p_value) scaled
    let chi_contrib = if chi.p_value <= 0.0 {
        40.0
    } else if chi.p_value >= 1.0 {
        0.0
    } else {
        let neg_log = -chi.p_value.log10();
        (neg_log * 10.0).clamp(0.0, 40.0)
    };

    // MAD contribution (0-30 points): based on conformity level
    let mad_contrib = match mad.conformity {
        ConformityLevel::CloseConformity => 0.0,
        ConformityLevel::AcceptableConformity => 10.0,
        ConformityLevel::MarginallyAcceptableConformity => 20.0,
        ConformityLevel::Nonconformity => 30.0,
    };

    // KS contribution (0-30 points): ratio of statistic to critical value
    let ks_contrib = if ks.critical_value_05 > 0.0 {
        let ratio = ks.statistic / ks.critical_value_05;
        (ratio * 15.0).clamp(0.0, 30.0)
    } else {
        0.0
    };

    let score = (chi_contrib + mad_contrib + ks_contrib).clamp(0.0, 100.0);
    let level = if score < 20.0 {
        RiskLevel::Low
    } else if score < 45.0 {
        RiskLevel::Moderate
    } else if score < 75.0 {
        RiskLevel::High
    } else {
        RiskLevel::Critical
    };

    (score, level)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dataset(values: Vec<f64>) -> Dataset {
        Dataset {
            values,
            total_records: 0,
            skipped: 0,
            source: "test".to_string(),
            column: None,
        }
    }

    #[test]
    fn test_analyze_empty() {
        let ds = make_dataset(vec![]);
        let result = analyze(&ds, DigitPosition::First);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_benford_conforming_data() {
        // Generate data that follows Benford's Law using log-uniform distribution.
        // When log10(x) is uniform, first digits follow Benford's Law.
        let mut values = Vec::new();
        let mut seed = 42u64;
        for _ in 0..10000 {
            // Simple xorshift
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let u = ((seed & 0xFFFFFFFF) as f64) / (u32::MAX as f64);
            let v = 10f64.powf(u * 10.0 - 5.0); // range 1e-5 to 1e5
            values.push(v);
        }
        let ds = make_dataset(values);
        let result = analyze(&ds, DigitPosition::First).unwrap();
        assert_eq!(result.position, DigitPosition::First);
        assert!(result.sample_size > 0);
        // Log-uniform data should follow Benford's Law closely
        assert!(
            result.risk_score < 40.0,
            "risk_score = {}",
            result.risk_score
        );
    }

    #[test]
    fn test_analyze_non_benford_data() {
        // Uniform random-ish data (1-9 evenly) violates Benford
        let values: Vec<f64> = (1..=10000).map(|i| (i % 9 + 1) as f64 * 100.0).collect();
        let ds = make_dataset(values);
        let result = analyze(&ds, DigitPosition::First).unwrap();
        assert!(
            result.risk_score > 40.0,
            "risk_score = {}",
            result.risk_score
        );
    }

    #[test]
    fn test_analyze_all_positions() {
        let values: Vec<f64> = (1..=1000).map(|i| (i as f64) * 7.3).collect();
        let ds = make_dataset(values);
        let results = analyze_all(&ds);
        assert!(results.len() >= 5);
    }

    #[test]
    fn test_risk_level_classification() {
        assert_eq!(RiskLevel::Low.label(), "Low");
        assert_eq!(RiskLevel::Moderate.label(), "Moderate");
        assert_eq!(RiskLevel::High.label(), "High");
        assert_eq!(RiskLevel::Critical.label(), "Critical");
    }

    #[test]
    fn test_digit_counts_structure() {
        let ds = make_dataset(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
        let result = analyze(&ds, DigitPosition::First).unwrap();
        assert_eq!(result.digit_counts.len(), 9); // first digit has 9 categories
        assert_eq!(result.digit_counts[0].digit, 1);
        assert_eq!(result.digit_counts[0].observed, 1); // 10 -> first digit 1
    }
}
