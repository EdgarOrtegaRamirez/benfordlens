//! Benford's Law expected probability distributions.
//!
//! Benford's Law states that for many naturally-occurring collections of numbers,
//! the leading digit `d` (1-9) occurs with probability `P(d) = log10(1 + 1/d)`.
//!
//! This generalizes to:
//! - First digit: P(d) = log10(1 + 1/d), for d in 1..=9
//! - Second digit: P(d) = sum over k=1..=9 of log10(1 + 1/(10k + d)), for d in 0..=9
//! - First-two digits: P(d) = log10(1 + 1/d), for d in 10..=99
//! - First-three digits: P(d) = log10(1 + 1/d), for d in 100..=999
//! - Last-two digits: uniform 1/100 (for comparison against fabricated data)

use crate::digit::DigitPosition;

/// Computes the Benford expected probability for a single digit value at the given position.
///
/// # Examples
/// ```
/// use benfordlens::digit::DigitPosition;
/// use benfordlens::distribution::expected_probability;
///
/// // First digit 1 should be ~0.30103
/// let p = expected_probability(DigitPosition::First, 1);
/// assert!((p - 0.30103).abs() < 0.0001);
///
/// // First digit 9 should be ~0.04576
/// let p = expected_probability(DigitPosition::First, 9);
/// assert!((p - 0.04576).abs() < 0.0001);
/// ```
pub fn expected_probability(position: DigitPosition, digit: u32) -> f64 {
    match position {
        DigitPosition::First => log10_1_plus_inv(digit as f64),
        DigitPosition::Second => second_digit_probability(digit),
        DigitPosition::Third => third_digit_probability(digit),
        DigitPosition::FirstTwo => log10_1_plus_inv(digit as f64),
        DigitPosition::FirstThree => log10_1_plus_inv(digit as f64),
        DigitPosition::LastTwo => 0.01, // uniform
    }
}

/// Computes the full expected probability distribution for a digit position.
/// Returns a vector of (digit_value, probability) pairs.
pub fn expected_distribution(position: DigitPosition) -> Vec<(u32, f64)> {
    position
        .values()
        .into_iter()
        .map(|d| (d, expected_probability(position, d)))
        .collect()
}

/// Computes the expected frequency (count) for a digit position given a sample size.
pub fn expected_frequencies(position: DigitPosition, n: usize) -> Vec<(u32, f64)> {
    let n_f = n as f64;
    expected_distribution(position)
        .into_iter()
        .map(|(d, p)| (d, p * n_f))
        .collect()
}

/// Returns the expected probability for the second digit (0-9).
/// P(d2) = sum_{k=1}^{9} log10(1 + 1/(10k + d2))
fn second_digit_probability(d2: u32) -> f64 {
    (1..=9)
        .map(|k| log10_1_plus_inv((10 * k + d2) as f64))
        .sum()
}

/// Returns the expected probability for the third digit (0-9).
/// P(d3) = sum over first-two-digit prefixes of log10(1 + 1/(100*prefix_fraction + d3))
/// More precisely: P(d3) = sum_{k=10}^{99} log10(1 + 1/(10k + d3))
fn third_digit_probability(d3: u32) -> f64 {
    (10..=99)
        .map(|k| log10_1_plus_inv((10 * k + d3) as f64))
        .sum()
}

/// Computes log10(1 + 1/x) — the core Benford formula.
fn log10_1_plus_inv(x: f64) -> f64 {
    (1.0 + 1.0 / x).log10()
}

/// Verifies that a distribution sums to ~1.0 (sanity check).
pub fn distribution_sum(position: DigitPosition) -> f64 {
    expected_distribution(position).iter().map(|(_, p)| p).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_digit_probabilities() {
        let dist = expected_distribution(DigitPosition::First);
        // Known Benford values
        let expected = [
            (1, 0.30103),
            (2, 0.17609),
            (3, 0.12494),
            (4, 0.09691),
            (5, 0.07918),
            (6, 0.06695),
            (7, 0.05799),
            (8, 0.05115),
            (9, 0.04576),
        ];
        for (digit, exp) in expected {
            let actual = expected_probability(DigitPosition::First, digit);
            assert!(
                (actual - exp).abs() < 0.0001,
                "first digit {}: expected {}, got {}",
                digit,
                exp,
                actual
            );
        }
        // Should sum to 1.0
        let sum: f64 = dist.iter().map(|(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 0.0001, "sum = {}", sum);
    }

    #[test]
    fn test_first_digit_monotonic_decreasing() {
        let dist = expected_distribution(DigitPosition::First);
        for i in 1..dist.len() {
            assert!(
                dist[i].1 < dist[i - 1].1,
                "digit {} should have lower prob than digit {}",
                dist[i].0,
                dist[i - 1].0
            );
        }
    }

    #[test]
    fn test_second_digit_sums_to_one() {
        let sum = distribution_sum(DigitPosition::Second);
        assert!((sum - 1.0).abs() < 0.0001, "second digit sum = {}", sum);
    }

    #[test]
    fn test_second_digit_values() {
        // Second digit 0 should be ~0.11968, 9 should be ~0.08500
        let p0 = expected_probability(DigitPosition::Second, 0);
        let p9 = expected_probability(DigitPosition::Second, 9);
        assert!((p0 - 0.11968).abs() < 0.001, "p0 = {}", p0);
        assert!((p9 - 0.08500).abs() < 0.001, "p9 = {}", p9);
    }

    #[test]
    fn test_third_digit_sums_to_one() {
        let sum = distribution_sum(DigitPosition::Third);
        assert!((sum - 1.0).abs() < 0.001, "third digit sum = {}", sum);
    }

    #[test]
    fn test_first_two_digits_sums_to_one() {
        let sum = distribution_sum(DigitPosition::FirstTwo);
        assert!((sum - 1.0).abs() < 0.001, "first-two sum = {}", sum);
    }

    #[test]
    fn test_first_two_digits_values() {
        // First-two 10 should be ~0.04139, 99 should be ~0.00436
        let p10 = expected_probability(DigitPosition::FirstTwo, 10);
        let p99 = expected_probability(DigitPosition::FirstTwo, 99);
        assert!((p10 - 0.04139).abs() < 0.0001, "p10 = {}", p10);
        assert!((p99 - 0.00437).abs() < 0.0001, "p99 = {}", p99);
    }

    #[test]
    fn test_first_three_digits_sums_to_one() {
        let sum = distribution_sum(DigitPosition::FirstThree);
        assert!((sum - 1.0).abs() < 0.001, "first-three sum = {}", sum);
    }

    #[test]
    fn test_last_two_digits_uniform() {
        for d in 0..=99 {
            assert!(
                (expected_probability(DigitPosition::LastTwo, d) - 0.01).abs() < 1e-10,
                "last two digit {} not uniform",
                d
            );
        }
        let sum = distribution_sum(DigitPosition::LastTwo);
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_expected_frequencies() {
        let freqs = expected_frequencies(DigitPosition::First, 1000);
        // digit 1 should be ~301
        assert!((freqs[0].1 - 301.03).abs() < 0.1);
        // digit 9 should be ~46
        assert!((freqs[8].1 - 45.76).abs() < 0.1);
    }
}
