use experiment_tracker::db::Database;
use experiment_tracker::models::RunStatus;

#[test]
fn test_insert_and_get_run() {
    let db = Database::open_memory().unwrap();
    let run = db.insert_run("test-run", "/tmp/test.jsonl").unwrap();

    assert_eq!(run.name, "test-run");
    assert_eq!(run.log_path, "/tmp/test.jsonl");
    assert_eq!(run.status, RunStatus::Running);
}

#[test]
fn test_insert_and_get_metrics() {
    let db = Database::open_memory().unwrap();
    let run = db.insert_run("test-run", "/tmp/test.jsonl").unwrap();

    db.insert_metric(run.id, "loss", Some(1), Some(100), 0.5)
        .unwrap();
    db.insert_metric(run.id, "loss", Some(2), Some(200), 0.3)
        .unwrap();
    db.insert_metric(run.id, "accuracy", Some(1), Some(100), 0.8)
        .unwrap();

    let metrics = db.get_metrics_for_run(run.id).unwrap();
    assert_eq!(metrics.len(), 3);

    let names = db.get_metric_names(run.id).unwrap();
    assert_eq!(names, vec!["accuracy", "loss"]);

    let latest = db.get_latest_metrics(run.id).unwrap();
    assert_eq!(latest.len(), 2);
}

#[test]
fn test_tags() {
    let db = Database::open_memory().unwrap();
    let run = db.insert_run("test-run", "/tmp/test.jsonl").unwrap();

    db.add_tag(run.id, "baseline").unwrap();
    db.add_tag(run.id, "transformer").unwrap();

    let tags = db.get_tags_for_run(run.id).unwrap();
    assert_eq!(tags.len(), 2);

    db.remove_tag(run.id, "baseline").unwrap();
    let tags = db.get_tags_for_run(run.id).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].tag, "transformer");
}

#[test]
fn test_delete_run_cascades() {
    let db = Database::open_memory().unwrap();
    let run = db.insert_run("test-run", "/tmp/test.jsonl").unwrap();

    db.insert_metric(run.id, "loss", Some(1), None, 0.5)
        .unwrap();
    db.add_tag(run.id, "test").unwrap();

    db.delete_run(run.id).unwrap();

    let metrics = db.get_metrics_for_run(run.id).unwrap();
    assert!(metrics.is_empty());

    let tags = db.get_tags_for_run(run.id).unwrap();
    assert!(tags.is_empty());
}

#[test]
fn test_update_run_status() {
    let db = Database::open_memory().unwrap();
    let run = db.insert_run("test-run", "/tmp/test.jsonl").unwrap();
    assert_eq!(run.status, RunStatus::Running);

    db.update_run_status(run.id, &RunStatus::Completed).unwrap();
    let updated = db.get_run(run.id).unwrap();
    assert_eq!(updated.status, RunStatus::Completed);
}

#[test]
fn test_insert_hyperparams_batch() {
    let db = Database::open_memory().unwrap();
    let run = db.insert_run("test-run", "/tmp/test.jsonl").unwrap();

    let params = vec![
        ("lr", "0.001"),
        ("batch_size", "64"),
        ("epochs", "50"),
    ];
    db.insert_hyperparams_batch(run.id, &params).unwrap();

    let hp = db.get_hyperparams_for_run(run.id).unwrap();
    assert_eq!(hp.len(), 3);

    // Verify values
    let lr = hp.iter().find(|h| h.key == "lr").unwrap();
    assert_eq!(lr.value, "0.001");

    // Test upsert behavior (INSERT OR REPLACE)
    let updated_params = vec![("lr", "0.01")];
    db.insert_hyperparams_batch(run.id, &updated_params).unwrap();

    let hp = db.get_hyperparams_for_run(run.id).unwrap();
    let lr = hp.iter().find(|h| h.key == "lr").unwrap();
    assert_eq!(lr.value, "0.01");
}

#[test]
fn test_get_runs_batch() {
    let db = Database::open_memory().unwrap();
    let run1 = db.insert_run("run-1", "/tmp/1.jsonl").unwrap();
    let run2 = db.insert_run("run-2", "/tmp/2.jsonl").unwrap();
    let _run3 = db.insert_run("run-3", "/tmp/3.jsonl").unwrap();

    // Fetch only run1 and run2
    let runs = db.get_runs_batch(&[run1.id, run2.id]).unwrap();
    assert_eq!(runs.len(), 2);

    let names: Vec<&str> = runs.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"run-1"));
    assert!(names.contains(&"run-2"));

    // Empty input returns empty result
    let empty = db.get_runs_batch(&[]).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn test_large_metrics_batch() {
    let db = Database::open_memory().unwrap();
    let run = db.insert_run("big-run", "/tmp/big.jsonl").unwrap();

    let batch: Vec<(i64, &str, Option<i64>, Option<i64>, f64)> = (0..1000)
        .map(|i| (run.id, "loss", Some(i as i64), Some(i as i64 * 10), i as f64 * 0.001))
        .collect();

    db.insert_metrics_batch(&batch).unwrap();

    let count = db.get_metric_count(run.id).unwrap();
    assert_eq!(count, 1000);
}
