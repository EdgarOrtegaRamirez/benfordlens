//! Report generation in multiple formats: text, JSON, markdown, HTML.

use crate::analysis::AnalysisResult;
use crate::data::Dataset;

/// Output format for reports.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// Plain text report.
    Text,
    /// JSON structured report.
    Json,
    /// Markdown report.
    Markdown,
    /// HTML report.
    Html,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Text => "txt",
            OutputFormat::Json => "json",
            OutputFormat::Markdown => "md",
            OutputFormat::Html => "html",
        }
    }
}

/// Generates a report for a single analysis result.
pub fn generate_report(result: &AnalysisResult, dataset: &Dataset, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => text_report(result, dataset),
        OutputFormat::Json => json_report(result, dataset),
        OutputFormat::Markdown => markdown_report(result, dataset),
        OutputFormat::Html => html_report(result, dataset),
    }
}

/// Generates a combined report for multiple analysis results (all digit positions).
pub fn generate_combined_report(
    results: &[AnalysisResult],
    dataset: &Dataset,
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Text => {
            let mut out = String::new();
            out.push_str(&text_header(dataset));
            for r in results {
                out.push('\n');
                out.push_str(&crate::visualize::summary_chart(r));
                out.push('\n');
                out.push_str(&crate::visualize::bar_chart(r, 70));
                out.push('\n');
            }
            out
        }
        OutputFormat::Json => {
            let report = serde_json::json!({
                "dataset": {
                    "source": dataset.source,
                    "column": dataset.column,
                    "total_records": dataset.total_records,
                    "valid_values": dataset.values.len(),
                    "skipped": dataset.skipped,
                },
                "analyses": results,
            });
            serde_json::to_string_pretty(&report).unwrap_or_else(|e| format!("JSON error: {}", e))
        }
        OutputFormat::Markdown => {
            let mut out = String::new();
            out.push_str(&markdown_header(dataset));
            for r in results {
                out.push_str(&markdown_section(r));
            }
            out
        }
        OutputFormat::Html => {
            let mut out = String::new();
            out.push_str(&html_header(dataset));
            for r in results {
                out.push_str(&html_section(r));
            }
            out.push_str("</body>\n</html>\n");
            out
        }
    }
}

fn text_report(result: &AnalysisResult, dataset: &Dataset) -> String {
    let mut out = String::new();
    out.push_str(&text_header(dataset));
    out.push('\n');
    out.push_str(&crate::visualize::summary_chart(result));
    out.push('\n');
    out.push_str(&crate::visualize::bar_chart(result, 70));
    out.push('\n');
    out.push_str(&crate::visualize::deviation_chart(result));
    out
}

fn text_header(dataset: &Dataset) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════════════════════╗\n");
    out.push_str("║              BenfordLens Analysis Report                ║\n");
    out.push_str("╚══════════════════════════════════════════════════════════╝\n\n");
    out.push_str(&format!("  Source:          {}\n", dataset.source));
    if let Some(ref col) = dataset.column {
        out.push_str(&format!("  Column:          {}\n", col));
    }
    out.push_str(&format!("  Total records:   {}\n", dataset.total_records));
    out.push_str(&format!("  Valid values:    {}\n", dataset.values.len()));
    if dataset.skipped > 0 {
        out.push_str(&format!("  Skipped (invalid): {}\n", dataset.skipped));
    }
    out
}

fn json_report(result: &AnalysisResult, dataset: &Dataset) -> String {
    let report = serde_json::json!({
        "dataset": {
            "source": dataset.source,
            "column": dataset.column,
            "total_records": dataset.total_records,
            "valid_values": dataset.values.len(),
            "skipped": dataset.skipped,
        },
        "analysis": result,
    });
    serde_json::to_string_pretty(&report).unwrap_or_else(|e| format!("JSON error: {}", e))
}

fn markdown_report(result: &AnalysisResult, dataset: &Dataset) -> String {
    let mut out = String::new();
    out.push_str(&markdown_header(dataset));
    out.push_str(&markdown_section(result));
    out
}

fn html_report(result: &AnalysisResult, dataset: &Dataset) -> String {
    let mut out = String::new();
    out.push_str(&html_header(dataset));
    out.push_str(&html_section(result));
    out.push_str("</body>\n</html>\n");
    out
}

fn markdown_header(dataset: &Dataset) -> String {
    let mut out = String::new();
    out.push_str("# BenfordLens Analysis Report\n\n");
    out.push_str(&format!("- **Source:** {}\n", dataset.source));
    if let Some(ref col) = dataset.column {
        out.push_str(&format!("- **Column:** {}\n", col));
    }
    out.push_str(&format!("- **Total records:** {}\n", dataset.total_records));
    out.push_str(&format!("- **Valid values:** {}\n", dataset.values.len()));
    if dataset.skipped > 0 {
        out.push_str(&format!("- **Skipped (invalid):** {}\n", dataset.skipped));
    }
    out.push('\n');
    out
}

fn markdown_section(r: &AnalysisResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("## {} Analysis\n\n", r.position.label()));
    out.push_str("| Metric | Value |\n|--------|-------|\n");
    out.push_str(&format!("| Sample size | {} |\n", r.sample_size));
    out.push_str(&format!(
        "| Risk score | {:.1}/100 ({}) |\n",
        r.risk_score,
        r.risk_level.label()
    ));
    out.push_str(&format!(
        "| MAD | {:.6} ({}) |\n",
        r.mad.mad,
        r.mad.conformity.label()
    ));
    out.push_str(&format!(
        "| Chi-square | {:.4} (df={}, p={:.6}) |\n",
        r.chi_square.statistic, r.chi_square.degrees_of_freedom, r.chi_square.p_value
    ));
    out.push_str(&format!(
        "| KS statistic | {:.6} (p={:.6}) |\n\n",
        r.ks.statistic, r.ks.p_value
    ));

    out.push_str("| Digit | Observed | Expected | Obs% | Exp% | Z-stat | Sig |\n");
    out.push_str("|-------|----------|----------|------|------|--------|-----|\n");
    for dev in &r.deviations {
        let sig = if dev.significant { "✓" } else { "" };
        out.push_str(&format!(
            "| {} | {} | {:.2} | {:.2}% | {:.2}% | {:+.3} | {} |\n",
            dev.digit,
            dev.observed,
            dev.expected,
            dev.observed_proportion * 100.0,
            dev.expected_proportion * 100.0,
            dev.z_statistic,
            sig
        ));
    }
    out.push('\n');
    out
}

fn html_header(dataset: &Dataset) -> String {
    let mut out = String::new();
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    out.push_str("<meta charset=\"UTF-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    out.push_str("<title>BenfordLens Analysis Report</title>\n");
    out.push_str("<style>\n");
    out.push_str("body { font-family: -apple-system, sans-serif; max-width: 900px; margin: 2em auto; padding: 0 1em; }\n");
    out.push_str("table { border-collapse: collapse; width: 100%; margin: 1em 0; }\n");
    out.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
    out.push_str("th { background: #f4f4f4; }\n");
    out.push_str(".risk-low { color: green; }\n");
    out.push_str(".risk-moderate { color: orange; }\n");
    out.push_str(".risk-high { color: red; }\n");
    out.push_str(".risk-critical { color: darkred; font-weight: bold; }\n");
    out.push_str(".sig { background: #ffdddd; }\n");
    out.push_str("</style>\n</head>\n<body>\n");
    out.push_str("<h1>BenfordLens Analysis Report</h1>\n");
    out.push_str(&format!(
        "<p><strong>Source:</strong> {}</p>\n",
        html_escape(&dataset.source)
    ));
    if let Some(ref col) = dataset.column {
        out.push_str(&format!(
            "<p><strong>Column:</strong> {}</p>\n",
            html_escape(col)
        ));
    }
    out.push_str(&format!(
        "<p><strong>Total records:</strong> {} | <strong>Valid values:</strong> {}",
        dataset.total_records,
        dataset.values.len()
    ));
    if dataset.skipped > 0 {
        out.push_str(&format!(" | <strong>Skipped:</strong> {}", dataset.skipped));
    }
    out.push_str("</p>\n");
    out
}

fn html_section(r: &AnalysisResult) -> String {
    let risk_class = format!("risk-{}", r.risk_level.label().to_lowercase());
    let mut out = String::new();
    out.push_str(&format!("<h2>{} Analysis</h2>\n", r.position.label()));
    out.push_str(&format!(
        "<p><strong>Sample size:</strong> {} | <strong>Risk:</strong> <span class=\"{}\">{:.1}/100 ({})</span></p>\n",
        r.sample_size, risk_class, r.risk_score, r.risk_level.label()
    ));
    out.push_str(&format!(
        "<p><strong>MAD:</strong> {:.6} ({}) | <strong>Chi-square:</strong> {:.4} (p={:.6}) | <strong>KS:</strong> {:.6} (p={:.6})</p>\n",
        r.mad.mad,
        r.mad.conformity.label(),
        r.chi_square.statistic,
        r.chi_square.p_value,
        r.ks.statistic,
        r.ks.p_value
    ));
    out.push_str("<table>\n<thead><tr><th>Digit</th><th>Observed</th><th>Expected</th><th>Obs%</th><th>Exp%</th><th>Z-stat</th><th>Sig</th></tr></thead>\n<tbody>\n");
    for dev in &r.deviations {
        let sig_class = if dev.significant {
            " class=\"sig\""
        } else {
            ""
        };
        let sig = if dev.significant { "✓" } else { "" };
        out.push_str(&format!(
            "<tr{}><td>{}</td><td>{}</td><td>{:.2}</td><td>{:.2}%</td><td>{:.2}%</td><td>{:+.3}</td><td>{}</td></tr>\n",
            sig_class, dev.digit, dev.observed, dev.expected,
            dev.observed_proportion * 100.0, dev.expected_proportion * 100.0,
            dev.z_statistic, sig
        ));
    }
    out.push_str("</tbody>\n</table>\n");
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Dataset;
    use crate::digit::DigitPosition;

    fn make_result() -> (AnalysisResult, Dataset) {
        let values: Vec<f64> = (1..=1000).map(|i| (i as f64).powi(3)).collect();
        let ds = Dataset {
            values,
            total_records: 1000,
            skipped: 0,
            source: "test.csv".to_string(),
            column: Some("amount".to_string()),
        };
        let result = crate::analysis::analyze(&ds, DigitPosition::First).unwrap();
        (result, ds)
    }

    #[test]
    fn test_text_report() {
        let (result, ds) = make_result();
        let report = generate_report(&result, &ds, OutputFormat::Text);
        assert!(report.contains("BenfordLens"));
        assert!(report.contains("First Digit"));
        assert!(report.contains("Risk score"));
    }

    #[test]
    fn test_json_report() {
        let (result, ds) = make_result();
        let report = generate_report(&result, &ds, OutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert!(parsed["analysis"].is_object());
        assert!(parsed["dataset"].is_object());
    }

    #[test]
    fn test_markdown_report() {
        let (result, ds) = make_result();
        let report = generate_report(&result, &ds, OutputFormat::Markdown);
        assert!(report.contains("# BenfordLens"));
        assert!(report.contains("## First Digit Analysis"));
        assert!(report.contains("| Digit |"));
    }

    #[test]
    fn test_html_report() {
        let (result, ds) = make_result();
        let report = generate_report(&result, &ds, OutputFormat::Html);
        assert!(report.contains("<!DOCTYPE html>"));
        assert!(report.contains("<table>"));
        assert!(report.contains("</html>"));
    }

    #[test]
    fn test_combined_report() {
        let (result, ds) = make_result();
        let results = vec![result];
        let report = generate_combined_report(&results, &ds, OutputFormat::Text);
        assert!(report.contains("BenfordLens"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("a&b<c>d\"e"), "a&amp;b&lt;c&gt;d&quot;e");
    }

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Text.extension(), "txt");
        assert_eq!(OutputFormat::Json.extension(), "json");
        assert_eq!(OutputFormat::Markdown.extension(), "md");
        assert_eq!(OutputFormat::Html.extension(), "html");
    }
}
