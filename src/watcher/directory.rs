use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

/// Minimum interval between events for the same file path
const DEBOUNCE_MS: u64 = 200;

/// Events the watcher sends to the main app
#[derive(Debug, Clone)]
pub enum WatchEvent {
    FileChanged(PathBuf), // new or modified log file was detected
    FileRemoved(PathBuf), // a file was removed
    Error(String),        // an error occurred in the watcher
}

/// Watches directories for experiment log files
pub struct DirectoryWatcher {
    _watcher: RecommendedWatcher,
    pub receiver: mpsc::Receiver<WatchEvent>,
}

impl DirectoryWatcher {
    /// Watch the given directories for log file changes
    pub fn new(watch_dirs: &[PathBuf]) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        let tx_clone = tx.clone();
        let last_events: Arc<Mutex<HashMap<PathBuf, Instant>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let debounce_state = last_events.clone();

        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                match result {
                    Ok(event) => {
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                for path in event.paths {
                                    if is_log_file(&path) {
                                        if should_emit(&debounce_state, &path) {
                                            let _ = tx_clone.send(WatchEvent::FileChanged(path));
                                        }
                                    }
                                }
                            }
                            EventKind::Remove(_) => {
                                for path in event.paths {
                                    if is_log_file(&path) {
                                        let _ = tx_clone.send(WatchEvent::FileRemoved(path));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        let _ = tx_clone.send(WatchEvent::Error(e.to_string()));
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        // Watch each directory
        for dir in watch_dirs {
            if dir.exists() {
                watcher.watch(dir, RecursiveMode::Recursive)?;
            } else {
                // create the directory if it does not exist
                std::fs::create_dir_all(dir)?;
                watcher.watch(dir, RecursiveMode::Recursive)?;
            }
        }

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }
    /// Non-Blocking check for new events
    pub fn try_recv(&self) -> Option<WatchEvent> {
        self.receiver.try_recv().ok()
    }

    /// Drain all pending events
    pub fn drain_events(&self) -> Vec<WatchEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }
        events
    }
}

/// sacn directories for existing log files (initil import)
pub fn scan_existing_files(watch_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for dir in watch_dirs {
        if !dir.exists() {
            continue;
        }
        scan_dir_recursive(dir, &mut files);
    }

    // sort by modificaiton time (newest first)
    files.sort_by(|a, b| {
        let time_a = std::fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let time_b = std::fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    files
}

fn scan_dir_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir_recursive(&path, files);
        } else if is_log_file(&path) {
            files.push(path);
        }
    }
}

/// Check if enough time has passed since the last event for this path (debounce)
fn should_emit(state: &Arc<Mutex<HashMap<PathBuf, Instant>>>, path: &Path) -> bool {
    let now = Instant::now();
    let debounce = Duration::from_millis(DEBOUNCE_MS);
    let mut map = match state.lock() {
        Ok(m) => m,
        Err(_) => return true, // poisoned mutex — emit anyway
    };

    if let Some(last) = map.get(path) {
        if now.duration_since(*last) < debounce {
            return false;
        }
    }
    map.insert(path.to_path_buf(), now);
    true
}

/// Check if a file is a supported log format
fn is_log_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("jsonl") | Some("ndjson") | Some("csv") | Some("json") | Some("log")
    )
}
