//! ASCII visualization of observed vs expected Benford distributions.

use crate::analysis::AnalysisResult;

/// Generates an ASCII bar chart comparing observed vs expected digit distributions.
///
/// Each bar represents the observed proportion, with the expected (Benford)
/// proportion shown as a marker.
pub fn bar_chart(result: &AnalysisResult, width: usize) -> String {
    let width = width.clamp(20, 80);
    let mut out = String::new();

    out.push_str(&format!(
        "  {} Distribution — Observed vs Benford Expected\n\n",
        result.position.label()
    ));

    // Find max proportion for scaling
    let max_prop = result
        .digit_counts
        .iter()
        .map(|d| d.observed_proportion.max(d.expected_proportion))
        .fold(0.0f64, f64::max)
        .max(0.001);

    let bar_width = width.saturating_sub(18);

    out.push_str(&format!(
        "  {:>6}  {:<width$}  {:>8}  {:>8}\n",
        "Digit",
        "Distribution (observed=█ expected=|)",
        "Obs%",
        "Exp%",
        width = bar_width
    ));
    out.push_str(&format!(
        "  {:>6}  {:<width$}  {:>8}  {:>8}\n",
        "-----",
        "",
        "------",
        "------",
        width = bar_width
    ));

    for dc in &result.digit_counts {
        let obs_len = ((dc.observed_proportion / max_prop) * bar_width as f64).round() as usize;
        let exp_pos = ((dc.expected_proportion / max_prop) * bar_width as f64).round() as usize;

        let mut bar: Vec<char> = vec![' '; bar_width];
        for item in bar.iter_mut().take(obs_len.min(bar_width)) {
            *item = '█';
        }
        if exp_pos < bar_width && exp_pos > 0 {
            if bar[exp_pos - 1] == ' ' {
                bar[exp_pos - 1] = '|';
            } else {
                // overlap — mark with a different char
                bar[exp_pos - 1] = '╪';
            }
        }

        out.push_str(&format!(
            "  {:>6}  {}  {:>7.2}%  {:>7.2}%\n",
            dc.digit,
            bar.iter().collect::<String>(),
            dc.observed_proportion * 100.0,
            dc.expected_proportion * 100.0
        ));
    }

    out
}

/// Generates a summary visualization showing the key statistics.
pub fn summary_chart(result: &AnalysisResult) -> String {
    let mut out = String::new();

    out.push_str(&format!("  {} Analysis Summary\n", result.position.label()));
    out.push_str(&format!("  {}\n", "─".repeat(50)));
    out.push_str(&format!("  Sample size:    {}\n", result.sample_size));
    if result.skipped > 0 {
        out.push_str(&format!("  Skipped records: {}\n", result.skipped));
    }
    out.push_str(&format!(
        "  Risk score:      {:.1}/100  [{}]\n",
        result.risk_score,
        result.risk_level.label()
    ));
    out.push_str(&format!(
        "  MAD:             {:.6}  [{}]\n",
        result.mad.mad,
        result.mad.conformity.label()
    ));
    out.push_str(&format!(
        "  Chi-square:      {:.4} (df={}, p={:.6})  {}\n",
        result.chi_square.statistic,
        result.chi_square.degrees_of_freedom,
        result.chi_square.p_value,
        if result.chi_square.significant {
            "SIGNIFICANT"
        } else {
            "not significant"
        }
    ));
    out.push_str(&format!(
        "  KS statistic:    {:.6} (p={:.6})  {}\n",
        result.ks.statistic,
        result.ks.p_value,
        if result.ks.significant {
            "SIGNIFICANT"
        } else {
            "not significant"
        }
    ));

    out
}

/// Generates a deviation chart showing Z-statistics for each digit.
pub fn deviation_chart(result: &AnalysisResult) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "  {} — Per-Digit Deviations (Z-Statistics)\n\n",
        result.position.label()
    ));

    out.push_str("  Digit   Observed   Expected    Dev%       Z       Sig\n");
    out.push_str("  -----  ---------  ---------  -------  -------  ---\n");

    for dev in &result.deviations {
        let dev_pct = (dev.observed_proportion - dev.expected_proportion) * 100.0;
        let sig = if dev.significant { "***" } else { "" };
        out.push_str(&format!(
            "  {:>5}  {:>9}  {:>9.2}  {:>+7.2}%  {:>+7.3}  {}\n",
            dev.digit, dev.observed, dev.expected, dev_pct, dev.z_statistic, sig
        ));
    }

    out.push_str("\n  *** = significant deviation (|Z| > 1.96, p < 0.05)\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Dataset;
    use crate::digit::DigitPosition;

    fn make_result() -> AnalysisResult {
        let values: Vec<f64> = (1..=1000).map(|i| (i as f64).powi(3)).collect();
        let ds = Dataset {
            values,
            total_records: 1000,
            skipped: 0,
            source: "test".to_string(),
            column: None,
        };
        crate::analysis::analyze(&ds, DigitPosition::First).unwrap()
    }

    #[test]
    fn test_bar_chart_not_empty() {
        let result = make_result();
        let chart = bar_chart(&result, 60);
        assert!(chart.contains("First Digit"));
        assert!(chart.contains("Obs%"));
        assert!(chart.contains("Exp%"));
    }

    #[test]
    fn test_bar_chart_contains_all_digits() {
        let result = make_result();
        let chart = bar_chart(&result, 60);
        for d in 1..=9 {
            assert!(chart.contains(&format!("{:>6}", d)), "missing digit {}", d);
        }
    }

    #[test]
    fn test_summary_chart() {
        let result = make_result();
        let chart = summary_chart(&result);
        assert!(chart.contains("Sample size"));
        assert!(chart.contains("Risk score"));
        assert!(chart.contains("MAD"));
        assert!(chart.contains("Chi-square"));
        assert!(chart.contains("KS statistic"));
    }

    #[test]
    fn test_deviation_chart() {
        let result = make_result();
        let chart = deviation_chart(&result);
        assert!(chart.contains("Z-Statistics"));
        assert!(chart.contains("Sig"));
    }
}
