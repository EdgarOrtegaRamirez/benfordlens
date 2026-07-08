//! Digit extraction utilities for Benford's Law analysis.
//!
//! Extracts leading digits, trailing digits, and digit sequences from numeric values.
//! Benford's Law applies to the significand (mantissa) of positive numbers in base 10.

use std::fmt;

/// Which digit position to analyze.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigitPosition {
    /// First (leading) digit: 1-9
    First,
    /// Second digit: 0-9
    Second,
    /// Third digit: 0-9
    Third,
    /// First two digits combined: 10-99
    FirstTwo,
    /// First three digits combined: 100-999
    FirstThree,
    /// Last two digits: 00-99 (used for detecting rounded/fabricated data)
    LastTwo,
}

impl DigitPosition {
    /// Returns the human-readable label for this position.
    pub fn label(&self) -> &'static str {
        match self {
            DigitPosition::First => "First Digit",
            DigitPosition::Second => "Second Digit",
            DigitPosition::Third => "Third Digit",
            DigitPosition::FirstTwo => "First-Two Digits",
            DigitPosition::FirstThree => "First-Three Digits",
            DigitPosition::LastTwo => "Last-Two Digits",
        }
    }

    /// Returns the valid range of digit values for this position (min, max inclusive).
    pub fn range(&self) -> (u32, u32) {
        match self {
            DigitPosition::First => (1, 9),
            DigitPosition::Second => (0, 9),
            DigitPosition::Third => (0, 9),
            DigitPosition::FirstTwo => (10, 99),
            DigitPosition::FirstThree => (100, 999),
            DigitPosition::LastTwo => (0, 99),
        }
    }

    /// Returns all valid digit values for this position as a Vec.
    pub fn values(&self) -> Vec<u32> {
        let (min, max) = self.range();
        (min..=max).collect()
    }

    /// Number of categories for this position.
    pub fn category_count(&self) -> usize {
        let (min, max) = self.range();
        (max - min + 1) as usize
    }
}

impl fmt::Display for DigitPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Extracts the digit value for the given position from a positive number.
///
/// Returns `None` if the number is zero, negative, infinite, or NaN,
/// or if the number doesn't have enough digits for the requested position.
///
/// # Examples
/// ```
/// use benfordlens::digit::{extract_digit, DigitPosition};
///
/// assert_eq!(extract_digit(12345.0, DigitPosition::First), Some(1));
/// assert_eq!(extract_digit(12345.0, DigitPosition::Second), Some(2));
/// assert_eq!(extract_digit(12345.0, DigitPosition::FirstTwo), Some(12));
/// assert_eq!(extract_digit(0.00456, DigitPosition::First), Some(4));
/// assert_eq!(extract_digit(0.0, DigitPosition::First), None);
/// ```
pub fn extract_digit(value: f64, position: DigitPosition) -> Option<u32> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }

    match position {
        DigitPosition::First => first_digit(value),
        DigitPosition::Second => second_digit(value),
        DigitPosition::Third => third_digit(value),
        DigitPosition::FirstTwo => first_two_digits(value),
        DigitPosition::FirstThree => first_three_digits(value),
        DigitPosition::LastTwo => last_two_digits(value),
    }
}

/// Extracts the first (leading) digit (1-9) from a positive number.
fn first_digit(value: f64) -> Option<u32> {
    let mantissa = significand(value)?;
    Some(mantissa as u32)
}

/// Extracts the second digit (0-9) from a positive number.
fn second_digit(value: f64) -> Option<u32> {
    let mantissa = significand(value)?;
    let scaled = mantissa * 10.0;
    Some(((scaled + 1e-6) as u32) % 10)
}

/// Extracts the third digit (0-9) from a positive number.
fn third_digit(value: f64) -> Option<u32> {
    let mantissa = significand(value)?;
    let scaled = mantissa * 100.0;
    Some(((scaled + 1e-6) as u32) % 10)
}

/// Extracts the first two digits (10-99) from a positive number.
fn first_two_digits(value: f64) -> Option<u32> {
    let mantissa = significand(value)?;
    Some(((mantissa * 10.0) + 1e-6) as u32)
}

/// Extracts the first three digits (100-999) from a positive number.
fn first_three_digits(value: f64) -> Option<u32> {
    let mantissa = significand(value)?;
    Some(((mantissa * 100.0) + 1e-6) as u32)
}

/// Extracts the last two significant digits (00-99) from a positive number.
/// This strips the integer part and looks at the last two digits before the decimal.
fn last_two_digits(value: f64) -> Option<u32> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    // Get the integer representation, then take last two digits
    let int_part = value.abs().trunc() as u64;
    if int_part == 0 {
        // For numbers < 1, use the significand approach
        let mantissa = significand(value)?;
        // Scale to integer and take last two
        let scaled = (mantissa * 1e15) as u64;
        Some((scaled % 100) as u32)
    } else {
        Some((int_part % 100) as u32)
    }
}

/// Computes the significand (mantissa) of a positive number such that
/// 1 <= mantissa < 10. This is the core of Benford's Law analysis.
///
/// Uses logarithm: mantissa = 10^(log10(n) - floor(log10(n)))
fn significand(value: f64) -> Option<f64> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let log = value.abs().log10();
    let frac = log - log.floor();
    let mantissa = 10f64.powf(frac);
    // Guard against floating point edge cases where mantissa rounds to 10.0
    let mantissa = if mantissa >= 10.0 {
        mantissa / 10.0
    } else {
        mantissa
    };
    // Guard against mantissa < 1.0 due to floating point
    let mantissa = if mantissa < 1.0 {
        mantissa * 10.0
    } else {
        mantissa
    };
    Some(mantissa)
}

/// Extracts digits from a slice of f64 values, filtering out invalid entries.
/// Returns a vector of (digit_value, count) would be done by the analysis module;
/// this returns the raw digit values.
pub fn extract_digits(values: &[f64], position: DigitPosition) -> Vec<u32> {
    values
        .iter()
        .filter_map(|&v| extract_digit(v, position))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_digit_whole_numbers() {
        assert_eq!(first_digit(1.0), Some(1));
        assert_eq!(first_digit(9.0), Some(9));
        assert_eq!(first_digit(10.0), Some(1));
        assert_eq!(first_digit(99.0), Some(9));
        assert_eq!(first_digit(100.0), Some(1));
        assert_eq!(first_digit(23456.0), Some(2));
        assert_eq!(first_digit(7890.0), Some(7));
    }

    #[test]
    fn test_first_digit_decimals() {
        assert_eq!(first_digit(0.1), Some(1));
        assert_eq!(first_digit(0.01), Some(1));
        assert_eq!(first_digit(0.234), Some(2));
        assert_eq!(first_digit(0.00567), Some(5));
        assert_eq!(first_digit(3.14159), Some(3));
    }

    #[test]
    fn test_first_digit_invalid() {
        assert_eq!(first_digit(0.0), None);
        assert_eq!(first_digit(-1.0), None);
        assert_eq!(first_digit(f64::NAN), None);
        assert_eq!(first_digit(f64::INFINITY), None);
        assert_eq!(first_digit(f64::NEG_INFINITY), None);
    }

    #[test]
    fn test_second_digit() {
        assert_eq!(second_digit(12.0), Some(2));
        assert_eq!(second_digit(234.0), Some(3));
        assert_eq!(second_digit(100.0), Some(0));
        assert_eq!(second_digit(98765.0), Some(8));
        assert_eq!(second_digit(0.0456), Some(5));
    }

    #[test]
    fn test_third_digit() {
        assert_eq!(third_digit(123.0), Some(3));
        assert_eq!(third_digit(2345.0), Some(4));
        assert_eq!(third_digit(1000.0), Some(0));
        assert_eq!(third_digit(0.00567), Some(7));
    }

    #[test]
    fn test_first_two_digits() {
        assert_eq!(first_two_digits(12.0), Some(12));
        assert_eq!(first_two_digits(234.0), Some(23));
        assert_eq!(first_two_digits(100.0), Some(10));
        assert_eq!(first_two_digits(98765.0), Some(98));
        assert_eq!(first_two_digits(0.0456), Some(45));
        assert_eq!(first_two_digits(1.0), Some(10));
    }

    #[test]
    fn test_first_three_digits() {
        assert_eq!(first_three_digits(123.0), Some(123));
        assert_eq!(first_three_digits(2345.0), Some(234));
        assert_eq!(first_three_digits(1000.0), Some(100));
        assert_eq!(first_three_digits(0.00567), Some(567));
    }

    #[test]
    fn test_last_two_digits() {
        assert_eq!(last_two_digits(12345.0), Some(45));
        assert_eq!(last_two_digits(100.0), Some(0));
        assert_eq!(last_two_digits(7.0), Some(7));
        assert_eq!(last_two_digits(123.0), Some(23));
    }

    #[test]
    fn test_extract_digit_dispatch() {
        assert_eq!(extract_digit(234.0, DigitPosition::First), Some(2));
        assert_eq!(extract_digit(234.0, DigitPosition::Second), Some(3));
        assert_eq!(extract_digit(234.0, DigitPosition::Third), Some(4));
        assert_eq!(extract_digit(234.0, DigitPosition::FirstTwo), Some(23));
        assert_eq!(extract_digit(234.0, DigitPosition::FirstThree), Some(234));
    }

    #[test]
    fn test_digit_position_range() {
        assert_eq!(DigitPosition::First.range(), (1, 9));
        assert_eq!(DigitPosition::Second.range(), (0, 9));
        assert_eq!(DigitPosition::FirstTwo.range(), (10, 99));
        assert_eq!(DigitPosition::FirstThree.range(), (100, 999));
        assert_eq!(DigitPosition::LastTwo.range(), (0, 99));
    }

    #[test]
    fn test_digit_position_category_count() {
        assert_eq!(DigitPosition::First.category_count(), 9);
        assert_eq!(DigitPosition::Second.category_count(), 10);
        assert_eq!(DigitPosition::FirstTwo.category_count(), 90);
        assert_eq!(DigitPosition::FirstThree.category_count(), 900);
    }

    #[test]
    fn test_extract_digits_batch() {
        let values = vec![12.0, 34.0, 56.0, 0.0, -1.0, 78.0];
        let digits = extract_digits(&values, DigitPosition::First);
        assert_eq!(digits, vec![1, 3, 5, 7]);
    }

    #[test]
    fn test_significand_range() {
        // significand should always be in [1, 10)
        for v in [1.0, 5.0, 10.0, 100.0, 0.001, 999.0, 12345.0] {
            let m = significand(v).unwrap();
            assert!(
                m >= 1.0 && m < 10.0,
                "significand of {} is {} (out of range)",
                v,
                m
            );
        }
    }
}
