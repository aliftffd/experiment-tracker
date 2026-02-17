use crate::models::{HyperParam, Metric, Run};

/// Export a single run summary as markdown
pub fn export_run_markdown(
    run: &Run,
    metrics: &[Metric],
    hyperparams: &[HyperParam],
    tags: &[String],
    latest_metrics: &[(String, f64)],
) -> String {
    let mut md = String::new();

    // title
    md.push_str(&format!("# {} \n\n", run.name));

    // status and the metadata
    md.push_str(&format!(
        "- **Status:** {} {}\n",
        run.status.symbol(),
        run.status
    ));
    md.push_str(&format!(
        "- **Created:** {}\n",
        run.created_at.format("%Y-%m-%d %H:%M:%S")
    ));
    md.push_str(&format!(
        "- **Updated:** {}\n",
        run.updated_at.format("%Y-%m-%d %H:%M:%S")
    ));
    md.push_str(&format!("- **Log path:** `{}`\n", run.log_path));

    if !tags.is_empty() {
        let tag_str: Vec<String> = tags.iter().map(|t| format!("`{}`", t)).collect();
        md.push_str(&format!("- **Tags:** {} \n", tag_str.join(", ")));
    }

    if !run.notes.is_empty() {
        md.push_str(&format!("\n## Notes\n\n{}\n", run.notes));
    }

    // hyperparameter table
    if !hyperparams.is_empty() {
        md.push_str("\n## Hyperparameters\n\n");
        md.push_str("| Parameter | Value |\n");
        md.push_str("|-----------|-------|\n");
        for hp in hyperparams {
            md.push_str(&format!("| {} | {} |\n", hp.key, hp.value));
        }
    }

    // final metrics table
    if !latest_metrics.is_empty() {
        md.push_str("\n## Final Metrics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        for (name, value) in latest_metrics {
            md.push_str(&format!("| {} | {:.6} |\n", name, value));
        }
    }

    // Training Summary
    if !metrics.is_empty() {
        let total_epochs = metrics.iter().filter_map(|m| m.epoch).max().unwrap_or(0);
        let total_steps = metrics.iter().filter_map(|m| m.step).max().unwrap_or(0);

        let unique_metrics: Vec<String> = {
            let mut names: Vec<String> = metrics.iter().map(|m| m.name.clone()).collect();
            names.sort();
            names.dedup();
            names
        };

        md.push_str("\n## Training Summary \n\n");
        md.push_str(&format!("- **Total Epoch:** {}\n", total_epochs));
        if total_steps > 0 {
            md.push_str(&format!("- **Total Steps:** {}\n", total_steps));
        }
        md.push_str(&format!(
            "- **Metrics Tracked:** {}\n",
            unique_metrics.join(", ")
        ));
        md.push_str(&format!("- **Total data points:** {}\n", metrics.len()));
    }

    md.push_str(&format!(
        "\n---\n*Exported by experiment-tracker on {}*\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    md
}

/// export a comparison of multiple runs as markdown
pub fn export_compare_markdown(runs_data: &[(String, Vec<Metric>)]) -> String {
    let mut md = String::new();

    md.push_str("# Experiment Comparison \n\n");
    md.push_str(&format!(
        "*Generated on {}*\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    // let collect all unique metrics
    let mut all_metric_names: Vec<String> = Vec::new();
    for (_, metrics) in runs_data {
        for m in metrics {
            if !all_metric_names.contains(&m.name) {
                all_metric_names.push(m.name.clone());
            }
        }
    }

    // build a header
    let run_names: Vec<&str> = runs_data.iter().map(|(n, _)| n.as_str()).collect();

    md.push_str("| Metric |");
    for name in &run_names {
        md.push_str(&format!(" {} |", name));
    }
    md.push('\n');

    md.push_str("|--------|");
    for _ in &run_names {
        md.push_str("--------|");
    }
    md.push('\n');

    // build rows
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

        md.push_str(&format!("| {} |", metric_name));
        for val in &values {
            match val {
                Some(v) => {
                    let is_best = (*v - best).abs() < 1e-30;
                    if is_best {
                        md.push_str(&format!(" **{:.6}** |", v));
                    } else {
                        md.push_str(&format!(" {:.6} |", v));
                    }
                }
                None => md.push_str(" - |"),
            }
        }
        md.push('\n');
    }

    // run summaries
    md.push_str("\n## Run Details\n\n");
    for (name, metrics) in runs_data {
        let total_epochs = metrics.iter().filter_map(|m| m.epoch).max().unwrap_or(0);
        md.push_str(&format!(
            "- **{}:** {} epochs, {} data points\n",
            name,
            total_epochs,
            metrics.len()
        ));
    }

    md
}
