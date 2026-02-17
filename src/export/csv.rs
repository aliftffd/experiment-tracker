use crate::models::Metric;

/// Export metrics for a single run as CSV
pub fn export_run_csv(run_name: &str, metrics: &[Metric]) -> String {
    let mut csv = String::new();

    csv.push_str("run,epoch,step,metric,value,recorded_at\n");

    for m in metrics {
        csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            escape_csv(run_name),
            m.epoch.map(|e| e.to_string()).unwrap_or_default(),
            m.step.map(|s| s.to_string()).unwrap_or_default(),
            escape_csv(&m.name),
            m.value,
            m.recorded_at.format("%Y-%m-%d %H:%M:%S"),
        ));
    }

    csv
}

/// Export metrics for multiple runs as CSV (for compare view)
pub fn export_compare_csv(runs_data: &[(String, Vec<Metric>)]) -> String {
    let mut csv = String::new();

    csv.push_str("run,epoch,step,metric,value,recorded_at\n");

    for (run_name, metrics) in runs_data {
        for m in metrics {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                escape_csv(run_name),
                m.epoch.map(|e| e.to_string()).unwrap_or_default(),
                m.step.map(|s| s.to_string()).unwrap_or_default(),
                escape_csv(&m.name),
                m.value,
                m.recorded_at.format("%Y-%m-%d %H:%M:%S"),
            ));
        }
    }

    csv
}

/// Export a summary table as CSV (latest metrics per run)
pub fn export_summary_csv(runs_data: &[(String, Vec<Metric>)]) -> String {
    let mut csv = String::new();

    // Collect all unique metric names
    let mut all_metric_names: Vec<String> = Vec::new();
    for (_, metrics) in runs_data {
        for m in metrics {
            if !all_metric_names.contains(&m.name) {
                all_metric_names.push(m.name.clone());
            }
        }
    }

    // Header: run, metric1, metric2, ...
    csv.push_str("run");
    for name in &all_metric_names {
        csv.push_str(&format!(",{}", escape_csv(name)));
    }
    csv.push('\n');

    // One row per run
    for (run_name, metrics) in runs_data {
        csv.push_str(&escape_csv(run_name));
        for metric_name in &all_metric_names {
            let latest = metrics
                .iter()
                .filter(|m| &m.name == metric_name)
                .last()
                .map(|m| m.value);
            match latest {
                Some(v) => csv.push_str(&format!(",{}", v)),
                None => csv.push(','),
            }
        }
        csv.push('\n');
    }

    csv
}

/// Escape a CSV field (wrap in quotes if it contains comma, quote, or newline)
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
