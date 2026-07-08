//! Statistical goodness-of-fit tests for comparing observed vs Benford-expected distributions.
//!
//! Implements:
//! - Chi-square goodness-of-fit test (with p-value via the regularized incomplete gamma function)
//! - Kolmogorov-Smirnov one-sample test (with p-value approximation)
//! - Mean Absolute Deviation (MAD) — Nigrini's conformity metric
//! - Z-statistics for individual digit deviations

use crate::digit::DigitPosition;
use crate::distribution;

/// Result of a chi-square goodness-of-fit test.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChiSquareResult {
    /// The chi-square statistic.
    pub statistic: f64,
    /// Degrees of freedom.
    pub degrees_of_freedom: usize,
    /// The p-value (probability of observing this deviation under Benford's Law).
    pub p_value: f64,
    /// Whether the result is significant at alpha = 0.05.
    pub significant: bool,
    /// Critical value at alpha = 0.05 for the given degrees of freedom.
    pub critical_value_05: f64,
    /// Critical value at alpha = 0.01.
    pub critical_value_01: f64,
}

/// Result of a Kolmogorov-Smirnov test.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KsResult {
    /// The KS statistic (maximum absolute difference between CDFs).
    pub statistic: f64,
    /// The p-value.
    pub p_value: f64,
    /// Whether the result is significant at alpha = 0.05.
    pub significant: bool,
    /// Critical value at alpha = 0.05.
    pub critical_value_05: f64,
}

/// Nigrini's Mean Absolute Deviation (MAD) conformity assessment.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MadResult {
    /// The Mean Absolute Deviation value.
    pub mad: f64,
    /// Conformity level based on Nigrini's thresholds.
    pub conformity: ConformityLevel,
}

/// Nigrini's MAD conformity levels for the first-two digit test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConformityLevel {
    /// MAD < 0.006 — close conformity
    CloseConformity,
    /// 0.006 <= MAD < 0.012 — acceptable conformity
    AcceptableConformity,
    /// 0.012 <= MAD < 0.015 — marginally acceptable conformity
    MarginallyAcceptableConformity,
    /// MAD >= 0.015 — nonconformity
    Nonconformity,
}

impl ConformityLevel {
    pub fn label(&self) -> &'static str {
        match self {
            ConformityLevel::CloseConformity => "Close Conformity",
            ConformityLevel::AcceptableConformity => "Acceptable Conformity",
            ConformityLevel::MarginallyAcceptableConformity => "Marginally Acceptable",
            ConformityLevel::Nonconformity => "Nonconformity",
        }
    }
}

/// Per-digit deviation statistics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DigitDeviation {
    /// The digit value.
    pub digit: u32,
    /// Observed count.
    pub observed: usize,
    /// Expected count.
    pub expected: f64,
    /// Observed proportion.
    pub observed_proportion: f64,
    /// Expected proportion.
    pub expected_proportion: f64,
    /// Absolute deviation in proportion.
    pub deviation: f64,
    /// Z-statistic for this digit.
    pub z_statistic: f64,
    /// Whether this digit deviates significantly (|z| > 1.96).
    pub significant: bool,
}

/// Performs a chi-square goodness-of-fit test comparing observed counts to Benford expected.
///
/// `observed` is a vector of (digit, count) pairs. The expected counts are derived
/// from Benford's Law for the given position and total sample size.
pub fn chi_square_test(
    observed: &[(u32, usize)],
    position: DigitPosition,
) -> Result<ChiSquareResult, String> {
    if observed.is_empty() {
        return Err("observed data is empty".to_string());
    }

    let n: usize = observed.iter().map(|(_, c)| c).sum();
    if n == 0 {
        return Err("total count is zero".to_string());
    }

    let expected = distribution::expected_frequencies(position, n);
    let expected_map: std::collections::HashMap<u32, f64> = expected.iter().cloned().collect();

    let mut chi_sq = 0.0;
    for (digit, count) in observed {
        let exp = expected_map.get(digit).copied().unwrap_or(0.0);
        if exp > 0.0 {
            let diff = (*count as f64) - exp;
            chi_sq += (diff * diff) / exp;
        }
    }

    let df = observed.len() - 1;
    let p_value = chi_square_p_value(chi_sq, df);
    let crit_05 = chi_square_critical_value(df, 0.05);
    let crit_01 = chi_square_critical_value(df, 0.01);

    Ok(ChiSquareResult {
        statistic: chi_sq,
        degrees_of_freedom: df,
        p_value,
        significant: p_value < 0.05,
        critical_value_05: crit_05,
        critical_value_01: crit_01,
    })
}

/// Performs a Kolmogorov-Smirnov one-sample test against Benford's expected CDF.
pub fn ks_test(observed: &[(u32, usize)], position: DigitPosition) -> Result<KsResult, String> {
    if observed.is_empty() {
        return Err("observed data is empty".to_string());
    }

    let n: usize = observed.iter().map(|(_, c)| c).sum();
    if n == 0 {
        return Err("total count is zero".to_string());
    }

    // Build observed CDF and expected CDF
    let values = position.values();
    let expected_dist = distribution::expected_distribution(position);
    let expected_map: std::collections::HashMap<u32, f64> = expected_dist.iter().cloned().collect();
    let observed_map: std::collections::HashMap<u32, usize> = observed.iter().cloned().collect();

    let n_f = n as f64;
    let mut obs_cdf = 0.0;
    let mut exp_cdf = 0.0;
    let mut max_diff = 0.0;

    for d in &values {
        let obs_count = observed_map.get(d).copied().unwrap_or(0);
        obs_cdf += obs_count as f64 / n_f;
        exp_cdf += expected_map.get(d).copied().unwrap_or(0.0);
        let diff = (obs_cdf - exp_cdf).abs();
        if diff > max_diff {
            max_diff = diff;
        }
    }

    let crit_05 = 1.36 / (n_f).sqrt();
    let p_value = ks_p_value(max_diff, n_f);

    Ok(KsResult {
        statistic: max_diff,
        p_value,
        significant: max_diff > crit_05,
        critical_value_05: crit_05,
    })
}

/// Computes Nigrini's Mean Absolute Deviation (MAD) and conformity level.
///
/// MAD = sum(|observed_proportion - expected_proportion|) / number_of_categories
pub fn mad_test(observed: &[(u32, usize)], position: DigitPosition) -> Result<MadResult, String> {
    if observed.is_empty() {
        return Err("observed data is empty".to_string());
    }

    let n: usize = observed.iter().map(|(_, c)| c).sum();
    if n == 0 {
        return Err("total count is zero".to_string());
    }

    let expected_dist = distribution::expected_distribution(position);
    let expected_map: std::collections::HashMap<u32, f64> = expected_dist.iter().cloned().collect();
    let observed_map: std::collections::HashMap<u32, usize> = observed.iter().cloned().collect();

    let n_f = n as f64;
    let mut total_dev = 0.0;
    let mut count = 0;

    for (d, _) in &expected_dist {
        let obs_prop = observed_map.get(d).copied().unwrap_or(0) as f64 / n_f;
        let exp_prop = expected_map.get(d).copied().unwrap_or(0.0);
        total_dev += (obs_prop - exp_prop).abs();
        count += 1;
    }

    let mad = total_dev / count as f64;
    let conformity = classify_mad(mad);

    Ok(MadResult { mad, conformity })
}

/// Classifies a MAD value into Nigrini's conformity levels (first-two digit thresholds).
pub fn classify_mad(mad: f64) -> ConformityLevel {
    if mad < 0.006 {
        ConformityLevel::CloseConformity
    } else if mad < 0.012 {
        ConformityLevel::AcceptableConformity
    } else if mad < 0.015 {
        ConformityLevel::MarginallyAcceptableConformity
    } else {
        ConformityLevel::Nonconformity
    }
}

/// Computes per-digit deviations and Z-statistics.
pub fn digit_deviations(observed: &[(u32, usize)], position: DigitPosition) -> Vec<DigitDeviation> {
    let n: usize = observed.iter().map(|(_, c)| c).sum();
    let n_f = n as f64;
    let expected_dist = distribution::expected_distribution(position);
    let observed_map: std::collections::HashMap<u32, usize> = observed.iter().cloned().collect();

    let mut result = Vec::new();
    for (d, exp_prop) in &expected_dist {
        let obs_count = observed_map.get(d).copied().unwrap_or(0);
        let obs_prop = obs_count as f64 / n_f;
        let exp_count = exp_prop * n_f;
        let dev = (obs_prop - exp_prop).abs();

        // Z-statistic: (observed - expected) / sqrt(expected * (1 - p))
        let z = if exp_count > 0.0 && n_f > 0.0 {
            let variance = exp_count * (1.0 - exp_prop);
            if variance > 0.0 {
                ((obs_count as f64) - exp_count) / variance.sqrt()
            } else {
                0.0
            }
        } else {
            0.0
        };

        result.push(DigitDeviation {
            digit: *d,
            observed: obs_count,
            expected: exp_count,
            observed_proportion: obs_prop,
            expected_proportion: *exp_prop,
            deviation: dev,
            z_statistic: z,
            significant: z.abs() > 1.96,
        });
    }
    result
}

/// Computes the chi-square p-value using the regularized lower incomplete gamma function.
/// P(x; k) = gamma_lower(k/2, x/2) / Gamma(k/2)
fn chi_square_p_value(x: f64, df: usize) -> f64 {
    if df == 0 {
        return 1.0;
    }
    let k = df as f64 / 2.0;
    let p = regularized_lower_gamma(k, x / 2.0);
    // p-value = 1 - CDF = 1 - P(x; k)
    (1.0 - p).max(0.0).min(1.0)
}

/// Regularized lower incomplete gamma function: P(a, x) = gamma_lower(a, x) / Gamma(a).
/// Uses the series expansion for x < a+1 and continued fraction for x >= a+1.
fn regularized_lower_gamma(a: f64, x: f64) -> f64 {
    if x < 0.0 || a <= 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        return 0.0;
    }

    let log_gamma_a = log_gamma(a);

    if x < a + 1.0 {
        // Series expansion
        let term = x.ln() * a - x - log_gamma_a;
        let mut sum = 1.0 / a;
        let mut term_val = 1.0 / a;
        for _ in 0..200 {
            term_val *= x / (a + sum);
            sum += term_val;
            if term_val.abs() < sum.abs() * 1e-15 {
                break;
            }
        }
        sum * term.exp()
    } else {
        // Continued fraction (Lentz's algorithm) for upper gamma, then complement
        let upper = upper_incomplete_gamma(a, x, log_gamma_a);
        1.0 - upper
    }
}

/// Upper incomplete gamma via continued fraction.
fn upper_incomplete_gamma(a: f64, x: f64, log_gamma_a: f64) -> f64 {
    let tiny = 1e-300;
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / tiny;
    let mut d = 1.0 / b;
    let mut h = d;
    for i in 1..=200 {
        let an = -(i as f64) * (i as f64 - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < tiny {
            d = tiny;
        }
        c = b + an / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() < 1e-15 {
            break;
        }
    }
    let term = a.ln() * a - a - x.ln() - log_gamma_a;
    term.exp() * h
}

/// Lanczos approximation of log(Gamma(x)).
fn log_gamma(x: f64) -> f64 {
    if x < 0.5 {
        // Reflection formula: Gamma(x)Gamma(1-x) = pi / sin(pi*x)
        return ((std::f64::consts::PI / x).sin().abs()).ln() - log_gamma(1.0 - x);
    }
    // Lanczos coefficients (g=7)
    const COEFF: [f64; 9] = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    let z = x - 1.0;
    let mut result = COEFF[0];
    for (i, c) in COEFF.iter().enumerate().skip(1) {
        result += c / (z + i as f64);
    }
    let t = z + 7.5;
    0.9189385332046727 + t.ln() * (z + 0.5) - t + result.ln()
}

/// Chi-square critical value lookup table for common alpha levels and df.
/// Falls back to the Wilson-Hilferty approximation for values not in the table.
fn chi_square_critical_value(df: usize, alpha: f64) -> f64 {
    // Lookup table for alpha = 0.05
    let crit_05: &[f64] = &[
        0.0, 3.841, 5.991, 7.815, 9.488, 11.070, 12.592, 14.067, 15.507, 16.919, 18.307, 19.675,
        21.026, 22.362, 23.685, 24.996, 26.296, 27.587, 28.869, 30.144, 31.410, 32.671, 33.924,
        35.172, 36.415, 37.652, 38.885, 40.113, 41.337, 42.557, 43.773, 44.985, 46.194, 47.400,
        48.602, 49.802, 50.998, 52.192, 53.384, 54.572, 55.758, 56.942, 58.124, 59.304, 60.481,
        61.656, 62.830, 64.001, 65.171, 66.339, 67.505, 68.669, 69.832, 70.993, 72.153, 73.311,
        74.468, 75.624, 76.778, 77.931, 79.082, 80.232, 81.381, 82.529, 83.676, 84.821, 85.965,
        87.108, 88.250, 89.391, 90.531, 91.670, 92.808, 93.945, 95.081, 96.217, 97.351, 98.484,
        99.617, 100.749, 101.879, 103.010, 104.139, 105.268, 106.397, 107.525, 108.651, 109.777,
    ];
    let crit_01: &[f64] = &[
        0.0, 6.635, 9.210, 11.345, 13.277, 15.086, 16.812, 18.475, 20.090, 21.666, 23.209, 24.725,
        26.217, 27.688, 29.141, 30.578, 32.000, 33.409, 34.805, 36.191, 37.566, 38.932, 40.289,
        41.638, 42.980, 44.314, 45.642, 46.963, 48.278, 49.588, 50.892, 52.191, 53.486, 54.776,
        56.061, 57.342, 58.619, 59.893, 61.162, 62.428, 63.691, 64.950, 66.206, 67.459, 68.710,
        69.957, 71.201, 72.443, 73.683, 74.919, 76.154, 77.386, 78.616, 79.843, 81.069, 82.292,
        83.513, 84.733, 85.950, 87.166, 88.379, 89.591, 90.802, 92.010, 93.217, 94.422, 95.626,
        96.828, 98.028, 99.228, 100.425, 101.621, 102.816, 104.010, 105.202, 106.393, 107.583,
        108.771, 109.958, 111.144, 112.329, 113.512, 114.695, 115.876, 117.057, 118.236, 119.414,
    ];

    let table = if (alpha - 0.05).abs() < 1e-6 {
        crit_05
    } else {
        crit_01
    };

    if df < table.len() {
        table[df]
    } else {
        // Wilson-Hilferty approximation: x = df * (1 - 2/(9df) + z * sqrt(2/(9df)))^3
        let z = if (alpha - 0.05).abs() < 1e-6 {
            1.6449
        } else {
            2.3263
        };
        let df_f = df as f64;
        let p = 1.0 - 2.0 / (9.0 * df_f);
        let q = z * (2.0 / (9.0 * df_f)).sqrt();
        df_f * (p + q).powi(3)
    }
}

/// KS p-value approximation using the asymptotic formula.
fn ks_p_value(d: f64, n: f64) -> f64 {
    let en = n.sqrt();
    let lambda = (en + 0.12 + 0.11 / en) * d;
    // Use the asymptotic series for Q_KS(lambda) = 2 * sum((-1)^(j-1) * exp(-2 * j^2 * lambda^2))
    let mut p = 0.0;
    for j in 1..=100 {
        let sign = if j % 2 == 1 { 1.0 } else { -1.0 };
        let term = sign * (-2.0 * (j as f64).powi(2) * lambda * lambda).exp();
        p += term;
        if term.abs() < 1e-12 {
            break;
        }
    }
    let p = 2.0 * p;
    p.max(0.0).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_observed(digit_counts: &[(u32, usize)]) -> Vec<(u32, usize)> {
        digit_counts.to_vec()
    }

    #[test]
    fn test_chi_square_perfect_conformity() {
        // Generate data that perfectly matches Benford's Law for first digit
        let n = 10000;
        let expected = distribution::expected_frequencies(DigitPosition::First, n);
        let observed: Vec<(u32, usize)> = expected
            .iter()
            .map(|(d, c)| (*d, c.round() as usize))
            .collect();
        let result = chi_square_test(&observed, DigitPosition::First).unwrap();
        assert!(result.statistic < 1.0, "chi_sq = {}", result.statistic);
        assert!(result.p_value > 0.95, "p_value = {}", result.p_value);
        assert!(!result.significant);
    }

    #[test]
    fn test_chi_square_extreme_nonconformity() {
        // All values start with 9 — extreme violation
        let observed = vec![
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 0),
            (6, 0),
            (7, 0),
            (8, 0),
            (9, 1000),
        ];
        let result = chi_square_test(&observed, DigitPosition::First).unwrap();
        assert!(result.statistic > 1000.0);
        assert!(result.p_value < 0.001);
        assert!(result.significant);
    }

    #[test]
    fn test_chi_square_empty() {
        let observed: Vec<(u32, usize)> = vec![];
        let result = chi_square_test(&observed, DigitPosition::First);
        assert!(result.is_err());
    }

    #[test]
    fn test_chi_square_zero_count() {
        let observed = vec![(1, 0), (2, 0)];
        let result = chi_square_test(&observed, DigitPosition::First);
        assert!(result.is_err());
    }

    #[test]
    fn test_ks_perfect_conformity() {
        let n = 10000;
        let expected = distribution::expected_frequencies(DigitPosition::First, n);
        let observed: Vec<(u32, usize)> = expected
            .iter()
            .map(|(d, c)| (*d, c.round() as usize))
            .collect();
        let result = ks_test(&observed, DigitPosition::First).unwrap();
        assert!(result.statistic < 0.05, "ks = {}", result.statistic);
        assert!(!result.significant);
    }

    #[test]
    fn test_mad_close_conformity() {
        let n = 10000;
        let expected = distribution::expected_frequencies(DigitPosition::First, n);
        let observed: Vec<(u32, usize)> = expected
            .iter()
            .map(|(d, c)| (*d, c.round() as usize))
            .collect();
        let result = mad_test(&observed, DigitPosition::First).unwrap();
        assert!(result.mad < 0.006, "mad = {}", result.mad);
        assert_eq!(result.conformity, ConformityLevel::CloseConformity);
    }

    #[test]
    fn test_mad_nonconformity() {
        let observed = vec![(9, 1000)];
        let result = mad_test(&observed, DigitPosition::First).unwrap();
        assert!(result.mad > 0.015, "mad = {}", result.mad);
        assert_eq!(result.conformity, ConformityLevel::Nonconformity);
    }

    #[test]
    fn test_classify_mad_thresholds() {
        assert_eq!(classify_mad(0.001), ConformityLevel::CloseConformity);
        assert_eq!(classify_mad(0.006), ConformityLevel::AcceptableConformity);
        assert_eq!(
            classify_mad(0.012),
            ConformityLevel::MarginallyAcceptableConformity
        );
        assert_eq!(classify_mad(0.015), ConformityLevel::Nonconformity);
        assert_eq!(classify_mad(0.1), ConformityLevel::Nonconformity);
    }

    #[test]
    fn test_digit_deviations() {
        let observed = make_observed(&[
            (1, 300),
            (2, 200),
            (3, 100),
            (4, 100),
            (5, 100),
            (6, 50),
            (7, 50),
            (8, 50),
            (9, 50),
        ]);
        let devs = digit_deviations(&observed, DigitPosition::First);
        assert_eq!(devs.len(), 9);
        assert_eq!(devs[0].digit, 1);
        assert_eq!(devs[0].observed, 300);
    }

    #[test]
    fn test_chi_square_critical_values() {
        // df=1, alpha=0.05 -> 3.841
        assert!((chi_square_critical_value(1, 0.05) - 3.841).abs() < 0.01);
        // df=9, alpha=0.05 -> 16.919
        assert!((chi_square_critical_value(9, 0.05) - 16.919).abs() < 0.01);
        // df=1, alpha=0.01 -> 6.635
        assert!((chi_square_critical_value(1, 0.01) - 6.635).abs() < 0.01);
    }

    #[test]
    fn test_chi_square_critical_value_large_df() {
        // Large df should use Wilson-Hilferty approximation
        let v = chi_square_critical_value(200, 0.05);
        assert!(v > 200.0 && v < 260.0, "v = {}", v);
    }

    #[test]
    fn test_log_gamma() {
        // Gamma(0.5) = sqrt(pi), log = 0.5724
        assert!((log_gamma(0.5) - 0.5724).abs() < 0.001);
        // Gamma(1) = 1, log = 0
        assert!((log_gamma(1.0) - 0.0).abs() < 0.001);
        // Gamma(2) = 1, log = 0
        assert!((log_gamma(2.0) - 0.0).abs() < 0.001);
        // Gamma(5) = 24, log(24) = 3.178
        assert!((log_gamma(5.0) - 3.178).abs() < 0.001);
    }

    #[test]
    fn test_chi_square_p_value_known() {
        // chi-square = 3.841, df=1 -> p ~ 0.05
        let p = chi_square_p_value(3.841, 1);
        assert!((p - 0.05).abs() < 0.01, "p = {}", p);
        // chi-square = 0.0, df=1 -> p = 1.0
        let p = chi_square_p_value(0.0, 1);
        assert!((p - 1.0).abs() < 0.001, "p = {}", p);
    }
}
