--- Experiment Tracker schema v1

CREATE TABLE IF NOT EXISTS runs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'running' CHECK(status IN ('running', 'completed', 'failed', 'stopped')),
    log_path    TEXT NOT NULL UNIQUE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    notes       TEXT DEFAULT ''
);

CREATE TABLE IF NOT EXISTS hyperparams (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id  INTEGER NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    key     TEXT NOT NULL,
    value   TEXT NOT NULL,
    UNIQUE(run_id, key)
);

CREATE TABLE IF NOT EXISTS metrics (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id      INTEGER NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    epoch       INTEGER,
    step        INTEGER,
    value       REAL NOT NULL,
    recorded_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tags (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id  INTEGER NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    tag     TEXT NOT NULL,
    UNIQUE(run_id, tag)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_metrics_run_id ON metrics(run_id);
CREATE INDEX IF NOT EXISTS idx_metrics_name ON metrics(run_id, name);
CREATE INDEX IF NOT EXISTS idx_hyperparams_run_id ON hyperparams(run_id);
CREATE INDEX IF NOT EXISTS idx_tags_run_id ON tags(run_id);
CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);
