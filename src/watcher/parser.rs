use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::config::settings::ParserConfig;

/// A single parsed record from a log file
#[derive(Debug, Clone)]
pub struct ParsedRecord {
    pub epoch: Option<i64>,
    pub step: Option<i64>,
    pub metrics: HashMap<String, f64>,
}

/// A fully parsed log file
#[derive(Debug, Clone)]
pub struct ParsedLog {
    pub records: Vec<ParsedRecord>,
    pub hyperparams: HashMap<String, String>,
}

/// Detected file format
#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
    JsonLines,
    Csv,
}

/// Detect the format of a log file by extension and content
pub fn detect_format(path: &Path) -> Option<LogFormat> {
    // first try extension
    match path.extension().and_then(|e| e.to_str()) {
        Some("jsonl") | Some("ndjson") => return Some(LogFormat::JsonLines),
        Some("csv") => return Some(LogFormat::Csv),
        Some("json") => {
            // Could be a single JSON object or JSON lines
            if let Ok(content) = std::fs::read_to_string(path) {
                let first_line = content.lines().next().unwrap_or("");
                if first_line.trim_start().starts_with('{') {
                    return Some(LogFormat::JsonLines);
                }
            }
        }
        _ => {}
    }

    // Try content-based detection
    if let Ok(content) = std::fs::read_to_string(path) {
        let first_line = content.lines().next().unwrap_or("").trim();

        // if first line is valid JSON object, treat as JSONL
        if first_line.starts_with('{') && serde_json::from_str::<Value>(first_line).is_ok() {
            return Some(LogFormat::JsonLines);
        }

        // if first line has commas and looks like a header, treat as CSV
        if first_line.contains(',') && !first_line.starts_with('{') {
            return Some(LogFormat::Csv);
        }
    }

    None
}

/// Parse a log file, auto-detecting format
pub fn parse_log_file(path: &Path, config: &ParserConfig) -> Result<ParsedLog> {
    let format = match config.default_format.as_str() {
        "jsonl" => LogFormat::JsonLines,
        "csv" => LogFormat::Csv,
        _ => detect_format(path)
            .with_context(|| format!("Cannot detect format of: {}", path.display()))?,
    };

    match format {
        LogFormat::JsonLines => parse_jsonl(path, config),
        LogFormat::Csv => parse_csv(path, config),
    }
}

/// Parse a JSON Lines file
///
/// Expected format (one JSON object per line):
/// ```text
/// {"epoch": 1, "step": 100, "loss": 0.542, "accuracy": 0.73}
/// {"epoch": 2, "step": 200, "loss": 0.431, "accuracy": 0.81}
/// ```
fn parse_jsonl(path: &Path, config: &ParserConfig) -> Result<ParsedLog> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read: {}", path.display()))?;

    let mut records = Vec::new();
    let mut hyperparams = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let obj: Value = serde_json::from_str(line).with_context(|| {
            format!(
                "Invalid JSON at line {}: {}",
                line_num + 1,
                truncate(line, 50)
            )
        })?;

        let Value::Object(map) = &obj else {
            continue; // skip non-object lines
        };

        // check if this is a hyperparams line
        // Convention: if it has a "hyperparams" or "config" key, treat as hyperparams
        if let Some(hp) = map.get("hyperparams").or(map.get("config")) {
            if let Value::Object(hp_map) = hp {
                for (k, v) in hp_map {
                    hyperparams.insert(k.clone(), value_to_string(v));
                }
                continue;
            }
        }

        // parse a metric record
        let epoch = extract_int(&obj, &config.epoch_field);
        let step = extract_int(&obj, &config.step_field);

        let mut metrics = HashMap::new();

        // extract all numeric fields as metrics
        for (key, val) in map {
            // skip known non-metric fields
            if key == &config.epoch_field
                || key == &config.step_field
                || key == "timestamp"
                || key == "time"
                || key == "datetime"
                || key == "hyperparams"
                || key == "config"
            {
                continue;
            }

            if let Some(num) = val.as_f64() {
                metrics.insert(key.clone(), num);
            }
        }

        if !metrics.is_empty() {
            records.push(ParsedRecord {
                epoch,
                step,
                metrics,
            });
        }
    }

    Ok(ParsedLog {
        records,
        hyperparams,
    })
}

/// Parse a CSV file
///
/// Expected format:
/// ```text
/// epoch,step,loss,accuracy
/// 1,100,0.542,0.73
/// 2,200,0.431,0.81
/// ```
fn parse_csv(path: &Path, config: &ParserConfig) -> Result<ParsedLog> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read: {}", path.display()))?;

    let mut lines = content.lines();

    // first line is the header
    let header_line = lines.next().with_context(|| "CSV file is empty")?;

    let headers: Vec<&str> = header_line.split(',').map(|h| h.trim()).collect();

    if headers.is_empty() {
        anyhow::bail!("CSV has no columns");
    }

    // find columns
    let epoch_idx = headers.iter().position(|h| *h == config.epoch_field);
    let step_idx = headers.iter().position(|h| *h == config.step_field);

    // identify which columns are the metrics
    let skip_fields = [
        config.epoch_field.as_str(),
        config.step_field.as_str(),
        "timestamp",
        "time",
        "datetime",
    ];

    let metric_columns: Vec<(usize, &str)> = headers
        .iter()
        .enumerate()
        .filter(|(_, h)| !skip_fields.contains(h))
        .map(|(i, h)| (i, *h))
        .collect();

    let mut records = Vec::new();

    for (line_num, line) in lines.enumerate() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split(',').map(|f| f.trim()).collect();

        if fields.len() != headers.len() {
            eprintln!(
                "Warning: line {} has {} fields, expected {} - skipping",
                line_num + 2,
                fields.len(),
                headers.len()
            );
            continue;
        }

        let epoch = epoch_idx.and_then(|i| fields.get(i)?.parse::<i64>().ok());
        let step = step_idx.and_then(|i| fields.get(i)?.parse::<i64>().ok());

        let mut metrics = HashMap::new();

        for (col_idx, col_name) in &metric_columns {
            if let Some(field) = fields.get(*col_idx) {
                if let Ok(val) = field.parse::<f64>() {
                    metrics.insert(col_name.to_string(), val);
                }
            }
        }

        if !metrics.is_empty() {
            records.push(ParsedRecord {
                epoch,
                step,
                metrics,
            });
        }
    }

    Ok(ParsedLog {
        records,
        hyperparams: HashMap::new(),
    })
}

/// Helper: extract an integer from a JSON value by key
fn extract_int(obj: &Value, key: &str) -> Option<i64> {
    obj.get(key).and_then(|v| match v {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    })
}

/// Convert a JSON value to a string
fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
