use experiment_tracker::db::Database;
use experiment_tracker::export::{csv, latex, markdown};
use experiment_tracker::models::RunStatus;

fn setup_test_data() -> (Database, i64) {
    let db = Database::open_memory().unwrap();

    let run = db.insert_run("test-run", "/tmp/test.jsonl").unwrap();
    db.update_run_status(run.id, &RunStatus::Completed).unwrap();
    db.update_run_notes(run.id, "Test notes for export")
        .unwrap();
    db.add_tag(run.id, "baseline").unwrap();
    db.add_tag(run.id, "test").unwrap();

    // Add hyperparams
    db.conn
        .execute(
            "INSERT INTO hyperparams (run_id, key, value) VALUES (?1, ?2, ?3)",
            rusqlite::params![run.id, "lr", "0.001"],
        )
        .unwrap();
    db.conn
        .execute(
            "INSERT INTO hyperparams (run_id, key, value) VALUES (?1, ?2, ?3)",
            rusqlite::params![run.id, "batch_size", "64"],
        )
        .unwrap();

    // Add metrics
    for epoch in 0..5 {
        let loss = 1.0 - (epoch as f64 * 0.15);
        let acc = 0.5 + (epoch as f64 * 0.1);
        db.insert_metric(run.id, "loss", Some(epoch), Some(epoch * 100), loss)
            .unwrap();
        db.insert_metric(run.id, "accuracy", Some(epoch), Some(epoch * 100), acc)
            .unwrap();
    }

    (db, run.id)
}

#[test]
fn test_markdown_export_contains_run_name() {
    let (db, run_id) = setup_test_data();
    let run = db.get_run(run_id).unwrap();
    let metrics = db.get_metrics_for_run(run_id).unwrap();
    let hyperparams = db.get_hyperparams_for_run(run_id).unwrap();
    let latest = db.get_latest_metrics(run_id).unwrap();
    let tags = vec!["baseline".to_string(), "test".to_string()];

    let md = markdown::export_run_markdown(&run, &metrics, &hyperparams, &tags, &latest);

    assert!(md.contains("# test-run"));
    assert!(md.contains("completed"));
    assert!(md.contains("`baseline`"));
    assert!(md.contains("Test notes for export"));
    assert!(md.contains("lr"));
    assert!(md.contains("0.001"));
    assert!(md.contains("loss"));
    assert!(md.contains("accuracy"));
}

#[test]
fn test_markdown_compare_highlights_best() {
    let (db, run_id) = setup_test_data();
    let metrics1 = db.get_metrics_for_run(run_id).unwrap();

    // Create a second run with worse metrics
    let run2 = db.insert_run("worse-run", "/tmp/test2.jsonl").unwrap();
    for epoch in 0..5 {
        let loss = 2.0 - (epoch as f64 * 0.1);
        let acc = 0.3 + (epoch as f64 * 0.05);
        db.insert_metric(run2.id, "loss", Some(epoch), Some(epoch * 100), loss)
            .unwrap();
        db.insert_metric(run2.id, "accuracy", Some(epoch), Some(epoch * 100), acc)
            .unwrap();
    }
    let metrics2 = db.get_metrics_for_run(run2.id).unwrap();

    let runs_data = vec![
        ("test-run".to_string(), metrics1),
        ("worse-run".to_string(), metrics2),
    ];

    let md = markdown::export_compare_markdown(&runs_data);

    assert!(md.contains("# Experiment Comparison"));
    assert!(md.contains("test-run"));
    assert!(md.contains("worse-run"));
    // Best values should be bold
    assert!(md.contains("**"));
}

#[test]
fn test_csv_export_has_header() {
    let (db, run_id) = setup_test_data();
    let metrics = db.get_metrics_for_run(run_id).unwrap();

    let csv_output = csv::export_run_csv("test-run", &metrics);

    let lines: Vec<&str> = csv_output.lines().collect();
    assert_eq!(lines[0], "run,epoch,step,metric,value,recorded_at");
    assert!(lines.len() > 1); // has data rows
    assert!(lines[1].starts_with("test-run,"));
}

#[test]
fn test_csv_export_correct_row_count() {
    let (db, run_id) = setup_test_data();
    let metrics = db.get_metrics_for_run(run_id).unwrap();

    let csv_output = csv::export_run_csv("test-run", &metrics);

    let lines: Vec<&str> = csv_output.lines().collect();
    // 1 header + 10 data rows (5 epochs * 2 metrics each)
    assert_eq!(lines.len(), 11);
}

#[test]
fn test_csv_summary_pivots_correctly() {
    let (db, run_id) = setup_test_data();
    let metrics = db.get_metrics_for_run(run_id).unwrap();

    let runs_data = vec![("test-run".to_string(), metrics)];

    let csv_output = csv::export_summary_csv(&runs_data);

    let lines: Vec<&str> = csv_output.lines().collect();
    // Header should have run + metric columns
    assert!(lines[0].contains("run"));
    assert!(lines[0].contains("loss") || lines[0].contains("accuracy"));
    // One data row for the run
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_latex_export_has_tabular() {
    let (db, run_id) = setup_test_data();
    let hyperparams = db.get_hyperparams_for_run(run_id).unwrap();
    let latest = db.get_latest_metrics(run_id).unwrap();

    let tex = latex::export_run_latex("test-run", &hyperparams, &latest);

    assert!(tex.contains("\\begin{tabular}"));
    assert!(tex.contains("\\end{tabular}"));
    assert!(tex.contains("\\toprule"));
    assert!(tex.contains("\\bottomrule"));
    assert!(tex.contains("lr"));
    assert!(tex.contains("0.001"));
}

#[test]
fn test_latex_escapes_special_chars() {
    let (db, _) = setup_test_data();

    // Create a run with special characters in name
    let run = db
        .insert_run("test_run #1 & friends", "/tmp/special.jsonl")
        .unwrap();
    db.insert_metric(run.id, "loss", Some(1), None, 0.5)
        .unwrap();
    let latest = db.get_latest_metrics(run.id).unwrap();

    let tex = latex::export_run_latex("test_run #1 & friends", &[], &latest);

    // Special chars should be escaped
    assert!(tex.contains("test\\_run \\#1 \\& friends"));
    assert!(!tex.contains("test_run #1 & friends")); // raw version should NOT appear in tabular
}

#[test]
fn test_latex_compare_bolds_best() {
    let (db, run_id) = setup_test_data();
    let metrics1 = db.get_metrics_for_run(run_id).unwrap();

    let run2 = db.insert_run("run2", "/tmp/r2.jsonl").unwrap();
    db.insert_metric(run2.id, "loss", Some(1), None, 0.9)
        .unwrap();
    db.insert_metric(run2.id, "accuracy", Some(1), None, 0.6)
        .unwrap();
    let metrics2 = db.get_metrics_for_run(run2.id).unwrap();

    let runs_data = vec![
        ("test-run".to_string(), metrics1),
        ("run2".to_string(), metrics2),
    ];

    let tex = latex::export_compare_latex(&runs_data);

    assert!(tex.contains("\\textbf{"));
    assert!(tex.contains("\\begin{tabular}"));
}
