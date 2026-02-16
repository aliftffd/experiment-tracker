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
