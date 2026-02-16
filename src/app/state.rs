use crate::config::AppConfig;
use crate::db::Database;
use crate::docker::{ContainerState, DockerInfo, DockerManager};
use crate::gpu::{GpuHistory, GpuMonitor, GpuProcess, GpuStats};
use crate::models::{HyperParam, Metric, Run, RunStatus};
use crate::watcher::parser::parse_log_file;
use anyhow::Result;
use std::path::Path;
use std::time::{Duration, Instant};

/// Which view/screen is active
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Dashboard,
    RunDetail,
    Compare,
    GpuMonitor,
    Help,
}

/// Input mode for search/text input and popup routing
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    TagInput,
    TagList,
    NotesInput,
    DeleteConfirm,
    RunDialog,
}

/// Core application state
pub struct App {
    pub config: AppConfig,
    pub db: Database,

    // Navigation
    pub current_view: View,
    pub previous_view: Option<View>,
    pub should_quit: bool,

    // Dashboard state
    pub runs: Vec<Run>,
    pub selected_run_index: usize,
    pub selected_tab: usize,
    pub tab_titles: Vec<String>,

    // Run detail state
    pub current_run: Option<Run>,
    pub current_metrics: Vec<Metric>,
    pub current_metric_names: Vec<String>,
    pub selected_metric_index: usize,
    pub current_tags: Vec<String>,
    pub current_latest_metrics: Vec<(String, f64)>,
    pub current_hyperparams: Vec<HyperParam>,

    // GPU monitoring
    pub gpu_monitor: Option<GpuMonitor>,
    pub gpu_stats: Option<GpuStats>,
    pub gpu_history: GpuHistory,
    pub gpu_processes: Vec<GpuProcess>,
    pub last_gpu_poll: Option<Instant>,

    // Docker
    pub docker: Option<DockerManager>,
    pub docker_info: Option<DockerInfo>,

    // Compare state
    pub compare_run_ids: Vec<i64>,
    pub compare_data: Vec<(Run, Vec<Metric>)>,
    pub compare_metric_names: Vec<String>,
    pub compare_selected_metric: usize,

    // Tag list popup
    pub tag_list_selected: usize,

    // Search/Input
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub search_query: String,
    pub filtered_runs: Vec<Run>,

    // Help
    pub show_help: bool,

    // Status bar
    pub status_message: String,
}

impl App {
    pub fn new(config: AppConfig, db: Database) -> Result<Self> {
        let runs = db.get_all_runs().unwrap_or_default();

        Ok(Self {
            config,
            db,

            current_view: View::Dashboard,
            previous_view: None,
            should_quit: false,

            runs: runs.clone(),
            selected_run_index: 0,
            selected_tab: 0,
            tab_titles: vec![
                "Dashboard".into(),
                "Detail".into(),
                "Compare".into(),
            ],

            current_run: None,
            current_metrics: Vec::new(),
            current_metric_names: Vec::new(),
            selected_metric_index: 0,
            current_tags: Vec::new(),
            current_latest_metrics: Vec::new(),
            current_hyperparams: Vec::new(),

            gpu_monitor: GpuMonitor::new(),
            gpu_stats: None,
            gpu_history: GpuHistory::new(120), // ~2 minutes at 1s polling
            gpu_processes: Vec::new(),
            last_gpu_poll: None,

            docker: DockerManager::new(),
            docker_info: None,

            compare_run_ids: Vec::new(),
            compare_data: Vec::new(),
            compare_metric_names: Vec::new(),
            compare_selected_metric: 0,

            tag_list_selected: 0,

            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            search_query: String::new(),
            filtered_runs: runs,

            show_help: false,

            status_message: "Ready".into(),
        })
    }

    // ─── Data Import (Day 2) ──────────────────────────────

    /// Import a log file: parse it, create/update run, insert metrics
    pub fn import_log_file(&mut self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let run_name = derive_run_name(path);

        // Parse the log file
        let parsed = parse_log_file(path, &self.config.parser)?;

        // Check if this run already exists
        let run = if let Some(existing) = self.db.get_run_by_path(&path_str)? {
            existing
        } else {
            self.db.insert_run(&run_name, &path_str)?
        };

        // Get current metric count to determine where new data starts
        let existing_count = self.db.get_metric_count(run.id)? as usize;

        // Only insert records we haven't seen yet
        if parsed.records.len() > existing_count {
            let new_records = &parsed.records[existing_count..];

            // Flatten: each record may have multiple metrics (loss, acc, etc.)
            let batch: Vec<(i64, &str, Option<i64>, Option<i64>, f64)> = new_records
                .iter()
                .flat_map(|record| {
                    record.metrics.iter().map(move |(name, value)| {
                        (run.id, name.as_str(), record.epoch, record.step, *value)
                    })
                })
                .collect();

            if !batch.is_empty() {
                self.db.insert_metrics_batch(&batch)?;
                self.set_status(format!(
                    "Imported {} metrics from {}",
                    batch.len(),
                    run_name
                ));
            }
        }

        // Import hyperparameters (upsert — replace if key already exists)
        for (key, value) in &parsed.hyperparams {
            self.db.conn.execute(
                "INSERT OR REPLACE INTO hyperparams (run_id, key, value) VALUES (?1, ?2, ?3)",
                rusqlite::params![run.id, key, value],
            )?;
        }

        // Refresh the UI
        self.refresh_runs()?;

        Ok(())
    }

    /// Scan watch directories and import all existing log files
    pub fn import_existing_files(&mut self) -> Result<usize> {
        let watch_dirs = self.config.resolved_watch_dirs();
        let files = crate::watcher::scan_existing_files(&watch_dirs);

        let mut imported = 0;
        for file in &files {
            match self.import_log_file(file) {
                Ok(_) => imported += 1,
                Err(e) => {
                    // Don't fail the whole scan if one file is bad
                    eprintln!("Warning: failed to import {}: {}", file.display(), e);
                }
            }
        }

        if imported > 0 {
            self.set_status(format!("Imported {} log files", imported));
        }

        Ok(imported)
    }

    // ─── Navigation ──────────────────────────────────────

    /// Refresh runs from the database
    pub fn refresh_runs(&mut self) -> Result<()> {
        self.runs = self.db.get_all_runs()?;
        self.apply_search_filter();

        if self.selected_run_index >= self.visible_runs().len() {
            self.selected_run_index = self.visible_runs().len().saturating_sub(1);
        }

        Ok(())
    }

    pub fn visible_runs(&self) -> &Vec<Run> {
        if self.search_query.is_empty() {
            &self.runs
        } else {
            &self.filtered_runs
        }
    }

    pub fn selected_run(&self) -> Option<&Run> {
        self.visible_runs().get(self.selected_run_index)
    }

    pub fn load_run_detail(&mut self, run_id: i64) -> Result<()> {
        self.current_run = Some(self.db.get_run(run_id)?);
        self.current_metrics = self.db.get_metrics_for_run(run_id)?;
        self.current_metric_names = self.db.get_metric_names(run_id)?;
        self.current_latest_metrics = self.db.get_latest_metrics(run_id)?;
        self.current_hyperparams = self.db.get_hyperparams_for_run(run_id)?;
        self.selected_metric_index = 0;

        let tags = self.db.get_tags_for_run(run_id)?;
        self.current_tags = tags.into_iter().map(|t| t.tag).collect();

        Ok(())
    }

    pub fn navigate_to(&mut self, view: View) {
        self.previous_view = Some(self.current_view.clone());
        self.current_view = view;
    }

    pub fn go_back(&mut self) {
        if let Some(prev) = self.previous_view.take() {
            self.current_view = prev;
        } else {
            self.current_view = View::Dashboard;
        }
    }

    // ─── Search ──────────────────────────────────────────

    fn apply_search_filter(&mut self) {
        let query = self.search_query.to_lowercase();
        self.filtered_runs = self
            .runs
            .iter()
            .filter(|r| {
                r.name.to_lowercase().contains(&query)
                    || r.status.to_string().to_lowercase().contains(&query)
                    || r.notes.to_lowercase().contains(&query)
            })
            .cloned()
            .collect();
    }

    pub fn update_search(&mut self, query: String) {
        self.search_query = query;
        self.apply_search_filter();
        self.selected_run_index = 0;
    }

    // ─── Helpers ─────────────────────────────────────────

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    pub fn move_up(&mut self) {
        if self.selected_run_index > 0 {
            self.selected_run_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let max = self.visible_runs().len().saturating_sub(1);
        if self.selected_run_index < max {
            self.selected_run_index += 1;
        }
    }

    pub fn cycle_metric(&mut self) {
        if !self.current_metric_names.is_empty() {
            self.selected_metric_index =
                (self.selected_metric_index + 1) % self.current_metric_names.len();
        }
    }

    // ─── GPU Monitoring ───────────────────────────────────

    /// Poll GPU stats only if enough time has passed since the last poll
    pub fn poll_gpu_if_needed(&mut self, interval: Duration) {
        let should_poll = match self.last_gpu_poll {
            Some(last) => last.elapsed() >= interval,
            None => true,
        };

        if !should_poll {
            return;
        }

        if let Some(monitor) = &self.gpu_monitor {
            if let Ok(stats) = monitor.poll_stats() {
                self.gpu_history.push(stats.clone());
                self.gpu_stats = Some(stats);
            }
            if let Ok(procs) = monitor.poll_processes() {
                self.gpu_processes = procs;
            }
            self.last_gpu_poll = Some(Instant::now());
        }
    }

    // ─── Docker ───────────────────────────────────────────

    /// Poll Docker containers and update run statuses for finished containers
    pub fn poll_docker(&mut self) {
        let updates = if let Some(mgr) = &mut self.docker {
            mgr.poll_containers()
        } else {
            return;
        };

        for (run_id, state) in updates {
            let new_status = match &state {
                ContainerState::Exited(0) => RunStatus::Completed,
                ContainerState::Exited(_) => RunStatus::Failed,
                ContainerState::Failed(_) => RunStatus::Failed,
                _ => continue,
            };

            if self.db.update_run_status(run_id, &new_status).is_ok() {
                self.set_status(format!(
                    "Run {} finished: {}",
                    run_id,
                    new_status
                ));
            }
        }

        let _ = self.refresh_runs();
    }

    // ─── Compare ──────────────────────────────────────────

    /// Check if a run is marked for comparison
    pub fn is_selected_for_compare(&self, run_id: i64) -> bool {
        self.compare_run_ids.contains(&run_id)
    }

    /// Toggle a run in the compare set (max 5 runs)
    pub fn toggle_compare(&mut self, run_id: i64) {
        if let Some(pos) = self.compare_run_ids.iter().position(|&id| id == run_id) {
            self.compare_run_ids.remove(pos);
            self.compare_data.retain(|(r, _)| r.id != run_id);
        } else if self.compare_run_ids.len() < 5 {
            self.compare_run_ids.push(run_id);
            if let Ok(run) = self.db.get_run(run_id) {
                let metrics = self.db.get_metrics_for_run(run_id).unwrap_or_default();
                self.compare_data.push((run, metrics));
            }
        } else {
            self.set_status("Compare limit: max 5 runs");
        }
    }

    /// Load all compare data and compute the union of metric names
    pub fn load_compare_data(&mut self) -> Result<()> {
        self.compare_data.clear();
        self.compare_metric_names.clear();

        let mut all_names = std::collections::BTreeSet::new();

        for &run_id in &self.compare_run_ids.clone() {
            let run = self.db.get_run(run_id)?;
            let metrics = self.db.get_metrics_for_run(run_id)?;
            let names = self.db.get_metric_names(run_id)?;
            for name in &names {
                all_names.insert(name.clone());
            }
            self.compare_data.push((run, metrics));
        }

        self.compare_metric_names = all_names.into_iter().collect();
        self.compare_selected_metric = 0;

        Ok(())
    }

    /// Cycle through metrics in compare view
    pub fn cycle_compare_metric(&mut self) {
        if !self.compare_metric_names.is_empty() {
            self.compare_selected_metric =
                (self.compare_selected_metric + 1) % self.compare_metric_names.len();
        }
    }
}

/// Derive a human-readable run name from a file path
///
/// If the filename is generic (like "metrics.jsonl"), use the parent dir name.
/// Otherwise use the file stem.
///
/// Examples:
///   "./experiments/run_001/metrics.jsonl" → "run_001"
///   "./experiments/my_training.csv" → "my_training"
fn derive_run_name(path: &Path) -> String {
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let generic_names = ["metrics", "log", "train", "training", "output", "results"];

    if generic_names.contains(&file_stem.to_lowercase().as_str()) {
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(file_stem)
            .to_string()
    } else {
        file_stem.to_string()
    }
}
