//! Data loading from multiple formats: CSV, JSON, JSONL, and plain text.
//!
//! Parses numeric data from files or stdin for Benford's Law analysis.
//! Handles malformed values gracefully by skipping them.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

/// A loaded dataset of numeric values.
#[derive(Debug, Clone)]
pub struct Dataset {
    /// The numeric values extracted (only valid positive numbers).
    pub values: Vec<f64>,
    /// Total lines/records read (including skipped).
    pub total_records: usize,
    /// Number of records skipped (non-numeric, zero, negative).
    pub skipped: usize,
    /// Source label (file path or "stdin").
    pub source: String,
    /// Optional column name if loaded from CSV/JSON.
    pub column: Option<String>,
}

impl Dataset {
    /// Returns the number of valid values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if there are no valid values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Loads numeric data from a file, auto-detecting the format.
/// If `column` is Some, only that column is extracted (for CSV/JSON).
pub fn load_file(path: &Path, column: Option<&str>) -> Result<Dataset, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let source = path.to_string_lossy().to_string();

    match ext.as_str() {
        "csv" | "tsv" => load_csv(path, column, &source),
        "json" => load_json_file(path, column, &source),
        "jsonl" | "ndjson" => load_jsonl_file(path, column, &source),
        _ => load_plain_text(path, column, &source), // try plain text
    }
}

/// Loads numeric data from stdin, treating it as plain text (one number per line).
pub fn load_stdin(column: Option<&str>) -> Result<Dataset, String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| format!("failed to read stdin: {}", e))?;
    parse_plain_text(&input, "stdin", column)
}

fn load_csv(path: &Path, column: Option<&str>, source: &str) -> Result<Dataset, String> {
    let file = File::open(path).map_err(|e| format!("failed to open {}: {}", source, e))?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(BufReader::new(file));

    let headers = reader
        .headers()
        .map_err(|e| format!("failed to read CSV headers: {}", e))?
        .clone();

    let col_index = match column {
        Some(col) => headers
            .iter()
            .position(|h| h == col)
            .ok_or_else(|| format!("column '{}' not found in CSV", col))?,
        None => 0, // default to first column
    };

    let mut values = Vec::new();
    let mut total = 0usize;
    let mut skipped = 0usize;

    for record in reader.records() {
        match record {
            Ok(r) => {
                total += 1;
                if let Some(field) = r.get(col_index) {
                    match parse_number(field) {
                        Some(v) => values.push(v),
                        None => skipped += 1,
                    }
                } else {
                    skipped += 1;
                }
            }
            Err(_) => skipped += 1,
        }
    }

    Ok(Dataset {
        values,
        total_records: total,
        skipped,
        source: source.to_string(),
        column: column.map(|s| s.to_string()),
    })
}

fn load_json_file(path: &Path, column: Option<&str>, source: &str) -> Result<Dataset, String> {
    let file = File::open(path).map_err(|e| format!("failed to open {}: {}", source, e))?;
    let reader = BufReader::new(file);
    let json: serde_json::Value =
        serde_json::from_reader(reader).map_err(|e| format!("failed to parse JSON: {}", e))?;

    let mut values = Vec::new();
    extract_numbers_from_json(&json, column, &mut values);

    let total = values.len();
    Ok(Dataset {
        values,
        total_records: total,
        skipped: 0,
        source: source.to_string(),
        column: column.map(|s| s.to_string()),
    })
}

fn load_jsonl_file(path: &Path, column: Option<&str>, source: &str) -> Result<Dataset, String> {
    let file = File::open(path).map_err(|e| format!("failed to open {}: {}", source, e))?;
    let reader = BufReader::new(file);
    let mut values = Vec::new();
    let mut total = 0usize;
    let mut skipped = 0usize;

    for line in reader.lines() {
        match line {
            Ok(l) if l.trim().is_empty() => continue,
            Ok(l) => {
                total += 1;
                match serde_json::from_str::<serde_json::Value>(&l) {
                    Ok(json) => {
                        let before = values.len();
                        extract_numbers_from_json(&json, column, &mut values);
                        if values.len() == before {
                            skipped += 1;
                        }
                    }
                    Err(_) => skipped += 1,
                }
            }
            Err(_) => skipped += 1,
        }
    }

    Ok(Dataset {
        values,
        total_records: total,
        skipped,
        source: source.to_string(),
        column: column.map(|s| s.to_string()),
    })
}

fn load_plain_text(path: &Path, _column: Option<&str>, source: &str) -> Result<Dataset, String> {
    let file = File::open(path).map_err(|e| format!("failed to open {}: {}", source, e))?;
    let mut content = String::new();
    BufReader::new(file)
        .read_to_string(&mut content)
        .map_err(|e| format!("failed to read {}: {}", source, e))?;
    parse_plain_text(&content, source, None)
}

/// Parses plain text input: one number per line (whitespace/comma separated also supported).
fn parse_plain_text(input: &str, source: &str, _column: Option<&str>) -> Result<Dataset, String> {
    let mut values = Vec::new();
    let mut total = 0usize;
    let mut skipped = 0usize;

    for line in input.lines() {
        for token in line.split(|c: char| c.is_whitespace() || c == ',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            total += 1;
            match parse_number(token) {
                Some(v) => values.push(v),
                None => skipped += 1,
            }
        }
    }

    Ok(Dataset {
        values,
        total_records: total,
        skipped,
        source: source.to_string(),
        column: None,
    })
}

/// Recursively extracts numbers from a JSON value.
/// If `column` is specified, only extracts values at that key.
fn extract_numbers_from_json(json: &serde_json::Value, column: Option<&str>, out: &mut Vec<f64>) {
    match json {
        serde_json::Value::Object(map) => {
            if let Some(col) = column {
                if let Some(v) = map.get(col) {
                    extract_numbers_from_json(v, None, out);
                }
            } else {
                for (_, v) in map {
                    extract_numbers_from_json(v, None, out);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                extract_numbers_from_json(v, column, out);
            }
        }
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.is_finite() && f > 0.0 {
                    out.push(f);
                }
            }
        }
        _ => {}
    }
}

/// Parses a string token into a positive f64. Returns None for non-numeric, zero, or negative.
/// Handles common formats: integers, decimals, scientific notation, currency symbols, commas.
pub fn parse_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // Strip currency symbols and common prefixes
    let s = s
        .trim_start_matches('$')
        .trim_start_matches('€')
        .trim_start_matches('£')
        .trim_start_matches('+')
        .trim();
    // Remove thousands separators (commas)
    let s = s.replace(',', "");
    // Remove trailing percent signs
    let s = s.trim_end_matches('%');

    if s.is_empty() {
        return None;
    }

    match s.parse::<f64>() {
        Ok(v) if v.is_finite() && v > 0.0 => Some(v),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_number_basic() {
        assert_eq!(parse_number("123"), Some(123.0));
        assert_eq!(parse_number("3.14"), Some(3.14));
        assert_eq!(parse_number("0.001"), Some(0.001));
        assert_eq!(parse_number("1e5"), Some(100000.0));
    }

    #[test]
    fn test_parse_number_currency() {
        assert_eq!(parse_number("$123.45"), Some(123.45));
        assert_eq!(parse_number("€1000"), Some(1000.0));
        assert_eq!(parse_number("£42"), Some(42.0));
    }

    #[test]
    fn test_parse_number_thousands() {
        assert_eq!(parse_number("1,234,567"), Some(1234567.0));
        assert_eq!(parse_number("$1,234.56"), Some(1234.56));
    }

    #[test]
    fn test_parse_number_invalid() {
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number("0"), None);
        assert_eq!(parse_number("-5"), None);
        assert_eq!(parse_number("NaN"), None);
        assert_eq!(parse_number("inf"), None);
    }

    #[test]
    fn test_parse_number_percent() {
        assert_eq!(parse_number("50%"), Some(50.0));
        assert_eq!(parse_number("3.14%"), Some(3.14));
    }

    #[test]
    fn test_parse_plain_text() {
        let input = "123\n456\n789\nabc\n0\n-1\n";
        let ds = parse_plain_text(input, "test", None).unwrap();
        assert_eq!(ds.values, vec![123.0, 456.0, 789.0]);
        assert_eq!(ds.total_records, 6);
        assert_eq!(ds.skipped, 3);
    }

    #[test]
    fn test_parse_plain_text_csv_line() {
        let input = "100, 200, 300\n400 500 600\n";
        let ds = parse_plain_text(input, "test", None).unwrap();
        assert_eq!(ds.values, vec![100.0, 200.0, 300.0, 400.0, 500.0, 600.0]);
    }

    #[test]
    fn test_load_csv_file() {
        let mut tmp = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        writeln!(tmp, "name,value").unwrap();
        writeln!(tmp, "a,100").unwrap();
        writeln!(tmp, "b,200").unwrap();
        writeln!(tmp, "c,invalid").unwrap();
        writeln!(tmp, "d,300").unwrap();
        let _ = tmp.flush();

        let ds = load_file(tmp.path(), Some("value")).unwrap();
        assert_eq!(ds.values, vec![100.0, 200.0, 300.0]);
        assert_eq!(ds.total_records, 4);
        assert_eq!(ds.skipped, 1);
    }

    #[test]
    fn test_load_csv_default_column() {
        let mut tmp = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        writeln!(tmp, "value").unwrap();
        writeln!(tmp, "100").unwrap();
        writeln!(tmp, "200").unwrap();
        writeln!(tmp, "300").unwrap();
        let _ = tmp.flush();

        let ds = load_file(tmp.path(), None).unwrap();
        assert_eq!(ds.values, vec![100.0, 200.0, 300.0]);
    }

    #[test]
    fn test_load_json_file() {
        let mut tmp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(tmp, "[100, 200, 300, 0, -5, \"abc\"]").unwrap();
        let _ = tmp.flush();

        let ds = load_file(tmp.path(), None).unwrap();
        assert_eq!(ds.values, vec![100.0, 200.0, 300.0]);
    }

    #[test]
    fn test_load_json_object_with_column() {
        let mut tmp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(
            tmp,
            "[{{\"amount\": 100}}, {{\"amount\": 200}}, {{\"name\": \"x\"}}]"
        )
        .unwrap();
        let _ = tmp.flush();

        let ds = load_file(tmp.path(), Some("amount")).unwrap();
        assert_eq!(ds.values, vec![100.0, 200.0]);
    }

    #[test]
    fn test_load_jsonl_file() {
        let mut tmp = tempfile::Builder::new()
            .suffix(".jsonl")
            .tempfile()
            .unwrap();
        writeln!(tmp, "{{\"v\": 100}}").unwrap();
        writeln!(tmp, "{{\"v\": 200}}").unwrap();
        writeln!(tmp, "{{\"v\": 300}}").unwrap();
        let _ = tmp.flush();

        let ds = load_file(tmp.path(), Some("v")).unwrap();
        assert_eq!(ds.values, vec![100.0, 200.0, 300.0]);
    }

    #[test]
    fn test_load_plain_text_file() {
        let mut tmp = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
        writeln!(tmp, "123").unwrap();
        writeln!(tmp, "456").unwrap();
        writeln!(tmp, "789").unwrap();
        let _ = tmp.flush();

        let ds = load_file(tmp.path(), None).unwrap();
        assert_eq!(ds.values, vec![123.0, 456.0, 789.0]);
    }

    #[test]
    fn test_dataset_methods() {
        let ds = Dataset {
            values: vec![1.0, 2.0, 3.0],
            total_records: 3,
            skipped: 0,
            source: "test".to_string(),
            column: None,
        };
        assert_eq!(ds.len(), 3);
        assert!(!ds.is_empty());
    }

    #[test]
    fn test_load_missing_file() {
        let result = load_file(Path::new("/nonexistent/file.csv"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_missing_column() {
        let mut tmp = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        writeln!(tmp, "a,b").unwrap();
        writeln!(tmp, "1,2").unwrap();
        let _ = tmp.flush();
        let result = load_file(tmp.path(), Some("nonexistent"));
        assert!(result.is_err());
    }
}
