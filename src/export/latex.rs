use crate::models::{HyperParam, Metric};

/// Export a single run's metrics as a LaTeX table
pub fn export_run_latex(
    run_name: &str,
    hyperparams: &[HyperParam],
    latest_metrics: &[(String, f64)],
) -> String {
    let mut tex = String::new();

    tex.push_str(&format!("% Experiment: {}\n", escape_latex(run_name)));
    tex.push_str(&format!(
        "% Generated on {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    // Hyperparameters table
    if !hyperparams.is_empty() {
        tex.push_str("\\begin{table}[h]\n");
        tex.push_str("\\centering\n");
        tex.push_str(&format!(
            "\\caption{{Hyperparameters for {}}}\n",
            escape_latex(run_name)
        ));
        tex.push_str(&format!("\\label{{tab:{}-params}}\n", slug(run_name)));
        tex.push_str("\\begin{tabular}{ll}\n");
        tex.push_str("\\toprule\n");
        tex.push_str("Parameter & Value \\\\\n");
        tex.push_str("\\midrule\n");

        for hp in hyperparams {
            tex.push_str(&format!(
                "{} & {} \\\\\n",
                escape_latex(&hp.key),
                escape_latex(&hp.value)
            ));
        }

        tex.push_str("\\bottomrule\n");
        tex.push_str("\\end{tabular}\n");
        tex.push_str("\\end{table}\n\n");
    }

    // Metrics table
    if !latest_metrics.is_empty() {
        tex.push_str("\\begin{table}[h]\n");
        tex.push_str("\\centering\n");
        tex.push_str(&format!(
            "\\caption{{Results for {}}}\n",
            escape_latex(run_name)
        ));
        tex.push_str(&format!("\\label{{tab:{}-results}}\n", slug(run_name)));
        tex.push_str("\\begin{tabular}{lr}\n");
        tex.push_str("\\toprule\n");
        tex.push_str("Metric & Value \\\\\n");
        tex.push_str("\\midrule\n");

        for (name, value) in latest_metrics {
            tex.push_str(&format!("{} & {:.4} \\\\\n", escape_latex(name), value));
        }

        tex.push_str("\\bottomrule\n");
        tex.push_str("\\end{tabular}\n");
        tex.push_str("\\end{table}\n");
    }

    tex
}

/// Export a comparison table as LaTeX
pub fn export_compare_latex(runs_data: &[(String, Vec<Metric>)]) -> String {
    let mut tex = String::new();

    tex.push_str("% Experiment Comparison\n");
    tex.push_str(&format!(
        "% Generated on {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    // Collect all unique metric names
    let mut all_metric_names: Vec<String> = Vec::new();
    for (_, metrics) in runs_data {
        for m in metrics {
            if !all_metric_names.contains(&m.name) {
                all_metric_names.push(m.name.clone());
            }
        }
    }

    let run_names: Vec<&str> = runs_data.iter().map(|(n, _)| n.as_str()).collect();
    let num_runs = run_names.len();

    // Column spec: l for metric name, r for each run value
    let col_spec = format!("l{}", "r".repeat(num_runs));

    tex.push_str("\\begin{table}[h]\n");
    tex.push_str("\\centering\n");
    tex.push_str("\\caption{Experiment Comparison}\n");
    tex.push_str("\\label{tab:comparison}\n");
    tex.push_str(&format!("\\begin{{tabular}}{{{}}}\n", col_spec));
    tex.push_str("\\toprule\n");

    // Header row
    tex.push_str("Metric");
    for name in &run_names {
        tex.push_str(&format!(" & {}", escape_latex(name)));
    }
    tex.push_str(" \\\\\n");
    tex.push_str("\\midrule\n");

    // Data rows
    for metric_name in &all_metric_names {
        let values: Vec<Option<f64>> = runs_data
            .iter()
            .map(|(_, metrics)| {
                metrics
                    .iter()
                    .filter(|m| &m.name == metric_name)
                    .last()
                    .map(|m| m.value)
            })
            .collect();

        // Determine best value
        let is_lower_better = metric_name.contains("loss") || metric_name.contains("error");
        let best = if is_lower_better {
            values
                .iter()
                .filter_map(|v| *v)
                .fold(f64::INFINITY, f64::min)
        } else {
            values
                .iter()
                .filter_map(|v| *v)
                .fold(f64::NEG_INFINITY, f64::max)
        };

        tex.push_str(&escape_latex(metric_name));
        for val in &values {
            match val {
                Some(v) => {
                    let is_best = (*v - best).abs() < 1e-10;
                    if is_best {
                        tex.push_str(&format!(" & \\textbf{{{:.4}}}", v));
                    } else {
                        tex.push_str(&format!(" & {:.4}", v));
                    }
                }
                None => tex.push_str(" & ---"),
            }
        }
        tex.push_str(" \\\\\n");
    }

    tex.push_str("\\bottomrule\n");
    tex.push_str("\\end{tabular}\n");
    tex.push_str("\\end{table}\n");

    tex
}

/// Escape special LaTeX characters
fn escape_latex(s: &str) -> String {
    s.replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}

/// Convert a string to a URL/label-safe slug
fn slug(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .replace("--", "-")
}
