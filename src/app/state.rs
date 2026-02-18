use crate::config::AppConfig;
use crate::db::Database;
use crate::docker::DockerManager;
use crate::export;
use crate::gpu::{GpuHistory, GpuMonitor, GpuProcess, GpuStats};
use crate::models::{HyperParam, Metric, Run, RunStatus};
use crate::watcher::parser::parse_log_file;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

/// Which view/screen is active
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Splash,
    Menu,
    Dashboard,
    RunDetail,
    Compare,
    GpuMonitor,
    Settings,
    Help,
}

/// Input mode for text input
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

/// State for the Docker run dialog
#[derive(Debug, Clone)]
pub struct RunDialogState {
    pub image: String,
    pub command: String,
    pub output_dir: String,
    pub use_gpu: bool,
    pub active_field: usize, // 0=image, 1=command, 2=output_dir, 3=gpu toggle
    pub error_message: String,
}

impl RunDialogState {
    pub fn new(config: &AppConfig, next_run_number: usize) -> Self {
        let image = config
            .docker
            .as_ref()
            .map(|d| d.default_image.clone())
            .unwrap_or_else(|| "pytorch/pytorch:latest".into());

        let use_gpu = config
            .docker
            .as_ref()
            .map(|d| d.gpu)
            .unwrap_or(false);

        Self {
            image,
            command: "python train.py".into(),
            output_dir: format!("./experiments/run_{:03}", next_run_number),
            use_gpu,
            active_field: 1, // start on command field — image is usually correct
            error_message: String::new(),
        }
    }

    /// Get the active field's mutable value
    pub fn active_value_mut(&mut self) -> Option<&mut String> {
        match self.active_field {
            0 => Some(&mut self.image),
            1 => Some(&mut self.command),
            2 => Some(&mut self.output_dir),
            _ => None, // GPU field is a toggle, not text
        }
    }

    /// Cycle to next field
    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1) % 4;
    }

    /// Toggle GPU
    pub fn toggle_gpu(&mut self) {
        self.use_gpu = !self.use_gpu;
    }
}

/// Sub-view within run detail
#[derive(Debug, Clone, PartialEq)]
pub enum DetailSubView {
    Chart,
    Hyperparams,
    Logs,
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
    pub detail_sub_view: DetailSubView,
    pub container_logs: String,

    // Compare state
    pub compare_run_ids: Vec<i64>,
    pub compare_data: Vec<(String, Vec<Metric>)>,
    pub compare_metric_names: Vec<String>,
    pub compare_selected_metric: usize,

    // Search/Input
    pub input_mode: InputMode,
    pub search_query: String,
    pub filtered_runs: Vec<Run>,
    pub input_buffer: String,
    pub tag_list_selected: usize,

    // Run dialog
    pub run_dialog: Option<RunDialogState>,

    // GPU monitoring
    pub gpu_monitor: Option<GpuMonitor>,
    pub gpu_stats: Option<GpuStats>,
    pub gpu_history: GpuHistory,
    pub gpu_processes: Vec<GpuProcess>,
    pub last_gpu_poll: Instant,
    pub gpu_poll_interval_secs: u64,

    // Docker
    pub docker: Option<DockerManager>,
    pub docker_info: Option<crate::docker::DockerInfo>,

    // Help
    pub show_help: bool,

    // Menu
    pub menu_selected: usize,

    // Status bar
    pub status_message: String,
}

impl App {
    pub fn new(config: AppConfig, db: Database) -> Result<Self> {
        let runs = db.get_all_runs().unwrap_or_default();

        let gpu_monitor = GpuMonitor::new();
        let gpu_poll_interval = config.gpu.as_ref()
            .map(|g| g.poll_interval_secs)
            .unwrap_or(2);

        // Poll once immediately so splash shows real GPU info on first render
        let initial_gpu_stats = gpu_monitor.as_ref().and_then(|m| m.poll_stats().ok());

        let docker = DockerManager::new();
        let docker_info = docker.as_ref().map(|d| d.check_health());

        Ok(Self {
            config,
            db,

            current_view: View::Splash,
            previous_view: None,
            should_quit: false,

            runs: runs.clone(),
            selected_run_index: 0,
            selected_tab: 0,
            tab_titles: vec![
                "Dashboard".into(),
                "Detail".into(),
                "Compare".into(),
                "GPU".into(),
            ],

            current_run: None,
            current_metrics: Vec::new(),
            current_metric_names: Vec::new(),
            selected_metric_index: 0,
            current_tags: Vec::new(),
            current_latest_metrics: Vec::new(),
            current_hyperparams: Vec::new(),
            detail_sub_view: DetailSubView::Chart,
            container_logs: String::new(),

            compare_run_ids: Vec::new(),
            compare_data: Vec::new(),
            compare_metric_names: Vec::new(),
            compare_selected_metric: 0,

            input_mode: InputMode::Normal,
            search_query: String::new(),
            filtered_runs: runs,
            input_buffer: String::new(),
            tag_list_selected: 0,

            run_dialog: None,

            gpu_monitor,
            gpu_stats: initial_gpu_stats,
            gpu_history: GpuHistory::new(300),
            gpu_processes: Vec::new(),
            last_gpu_poll: Instant::now(),
            gpu_poll_interval_secs: gpu_poll_interval,

            docker,
            docker_info,

            show_help: false,

            menu_selected: 0,

            status_message: "Ready".into(),
        })
    }

    // ─── Export ───────────────────────────────────────────

    /// Export current run as markdown
    pub fn export_current_run_markdown(&self) -> Result<String> {
        let run = self.current_run.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No run selected"))?;

        let base_dir = export::export_base_dir(&self.config.resolved_watch_dirs());

        let content = export::markdown::export_run_markdown(
            run,
            &self.current_metrics,
            &self.current_hyperparams,
            &self.current_tags,
            &self.current_latest_metrics,
        );

        let path = export::write_export(&base_dir, &run.name, "md", &content)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// Export current run metrics as CSV
    pub fn export_current_run_csv(&self) -> Result<String> {
        let run = self.current_run.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No run selected"))?;

        let base_dir = export::export_base_dir(&self.config.resolved_watch_dirs());

        let content = export::csv::export_run_csv(&run.name, &self.current_metrics);
        let path = export::write_export(&base_dir, &run.name, "csv", &content)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// Export current run as LaTeX
    pub fn export_current_run_latex(&self) -> Result<String> {
        let run = self.current_run.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No run selected"))?;

        let base_dir = export::export_base_dir(&self.config.resolved_watch_dirs());

        let content = export::latex::export_run_latex(
            &run.name,
            &self.current_hyperparams,
            &self.current_latest_metrics,
        );

        let path = export::write_export(&base_dir, &run.name, "tex", &content)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// Export compare view as markdown
    pub fn export_compare_markdown(&self) -> Result<String> {
        let base_dir = export::export_base_dir(&self.config.resolved_watch_dirs());
        let content = export::markdown::export_compare_markdown(&self.compare_data);
        let path = export::write_compare_export(&base_dir, "md", &content)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// Export compare view as CSV
    pub fn export_compare_csv(&self) -> Result<String> {
        let base_dir = export::export_base_dir(&self.config.resolved_watch_dirs());
        let content = export::csv::export_summary_csv(&self.compare_data);
        let path = export::write_compare_export(&base_dir, "csv", &content)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// Export compare view as LaTeX
    pub fn export_compare_latex(&self) -> Result<String> {
        let base_dir = export::export_base_dir(&self.config.resolved_watch_dirs());
        let content = export::latex::export_compare_latex(&self.compare_data);
        let path = export::write_compare_export(&base_dir, "tex", &content)?;
        Ok(path.to_string_lossy().to_string())
    }

    // ─── Docker Run Dialog ───────────────────────────────

    /// Open the run dialog with defaults
    pub fn open_run_dialog(&mut self) {
        let next_num = self.runs.len() + 1;
        self.run_dialog = Some(RunDialogState::new(&self.config, next_num));
        self.input_mode = InputMode::RunDialog;
    }

    /// Execute the Docker run from dialog state
    pub fn execute_docker_run(&mut self) -> Result<()> {
        let dialog = self.run_dialog.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No run dialog open"))?
            .clone();

        // Pre-run checks
        let docker = self.docker.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Docker not available"))?;

        if let Some(info) = &self.docker_info {
            if !info.running {
                anyhow::bail!("Docker daemon is not running");
            }
            if dialog.use_gpu && !info.gpu_support {
                anyhow::bail!("Docker GPU support not available. Install nvidia-container-toolkit.");
            }
        }

        // VRAM check
        if dialog.use_gpu {
            if let Some(stats) = &self.gpu_stats {
                let vram_threshold = self.config.gpu.as_ref()
                    .map(|g| g.vram_critical as f32)
                    .unwrap_or(95.0);
                if stats.vram_percent() > vram_threshold {
                    anyhow::bail!(
                        "GPU VRAM is {:.0}% full ({}/{} MB). Risk of OOM.",
                        stats.vram_percent(),
                        stats.vram_used_mb,
                        stats.vram_total_mb
                    );
                }
            }
        }

        // Create output directory
        let output_path = crate::platform::expand_path(&dialog.output_dir);
        std::fs::create_dir_all(&output_path)?;

        // Create run in database
        let run_name = derive_run_name(&output_path);
        let log_path = output_path.join("metrics.jsonl");
        let run = self.db.insert_run(&run_name, &log_path.to_string_lossy())?;

        // Get container workdir
        let container_workdir = self.config.docker.as_ref()
            .map(|d| d.container_workdir.clone())
            .unwrap_or_else(|| "/workspace/output".into());

        // Launch container
        let docker = self.docker.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Docker not available"))?;

        let container_id = docker.run_container(
            run.id,
            &dialog.image,
            &dialog.command,
            &output_path.to_string_lossy(),
            &container_workdir,
            dialog.use_gpu,
            &HashMap::new(),
        )?;

        self.set_status(format!(
            "Started container {} for {}",
            &container_id[..12.min(container_id.len())],
            run_name
        ));

        // Refresh and clean up dialog
        self.refresh_runs()?;
        self.run_dialog = None;
        self.input_mode = InputMode::Normal;

        Ok(())
    }

    /// Update container logs for current run
    pub fn refresh_container_logs(&mut self) {
        if let Some(run) = &self.current_run {
            if let Some(docker) = &self.docker {
                if docker.is_running(run.id) {
                    if let Ok(logs) = docker.get_logs(run.id, 50) {
                        self.container_logs = logs;
                    }
                }
            }
        }
    }

    /// Cycle detail sub-view
    pub fn cycle_detail_sub_view(&mut self) {
        self.detail_sub_view = match self.detail_sub_view {
            DetailSubView::Chart => DetailSubView::Hyperparams,
            DetailSubView::Hyperparams => DetailSubView::Logs,
            DetailSubView::Logs => DetailSubView::Chart,
        };

        // Refresh logs when switching to logs view
        if self.detail_sub_view == DetailSubView::Logs {
            self.refresh_container_logs();
        }
    }

    // ─── GPU ─────────────────────────────────────────────

    pub fn poll_gpu_if_needed(&mut self) {
        let elapsed = self.last_gpu_poll.elapsed().as_secs();
        if elapsed < self.gpu_poll_interval_secs {
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
        }

        self.last_gpu_poll = Instant::now();
    }

    // ─── Docker ──────────────────────────────────────────

    pub fn poll_docker(&mut self) {
        if let Some(docker) = &mut self.docker {
            let updates = docker.poll_containers();
            for (run_id, state) in updates {
                let new_status = match &state {
                    crate::docker::ContainerState::Exited(0) => RunStatus::Completed,
                    crate::docker::ContainerState::Exited(_) => RunStatus::Failed,
                    crate::docker::ContainerState::Failed(_) => RunStatus::Failed,
                    _ => continue,
                };

                if self.db.update_run_status(run_id, &new_status).is_ok() {
                    let status_msg = match &state {
                        crate::docker::ContainerState::Exited(code) => {
                            format!("Container exited (code {})", code)
                        }
                        crate::docker::ContainerState::Failed(msg) => {
                            format!("Container failed: {}", msg)
                        }
                        _ => "Container stopped".to_string(),
                    };
                    self.set_status(status_msg);
                    let _ = self.refresh_runs();
                }
            }
        }
    }

    // ─── Compare ─────────────────────────────────────────

    pub fn toggle_compare(&mut self, run_id: i64) {
        if self.compare_run_ids.contains(&run_id) {
            self.compare_run_ids.retain(|&id| id != run_id);
        } else {
            if self.compare_run_ids.len() >= 5 {
                self.set_status("Maximum 5 runs for comparison");
                return;
            }
            self.compare_run_ids.push(run_id);
        }
        let count = self.compare_run_ids.len();
        self.set_status(format!("{} run(s) selected for comparison", count));
    }

    pub fn load_compare_data(&mut self) -> Result<()> {
        self.compare_data.clear();
        self.compare_metric_names.clear();

        for &run_id in &self.compare_run_ids.clone() {
            let run = self.db.get_run(run_id)?;
            let metrics = self.db.get_metrics_for_run(run_id)?;

            for m in &metrics {
                if !self.compare_metric_names.contains(&m.name) {
                    self.compare_metric_names.push(m.name.clone());
                }
            }

            self.compare_data.push((run.name, metrics));
        }

        self.compare_selected_metric = 0;
        Ok(())
    }

    pub fn is_selected_for_compare(&self, run_id: i64) -> bool {
        self.compare_run_ids.contains(&run_id)
    }

    // ─── Data Import ─────────────────────────────────────

    pub fn import_log_file(&mut self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let run_name = derive_run_name(path);

        let parsed = parse_log_file(path, &self.config.parser)?;

        let run = if let Some(existing) = self.db.get_run_by_path(&path_str)? {
            existing
        } else {
            self.db.insert_run(&run_name, &path_str)?
        };

        let existing_count = self.db.get_metric_count(run.id)? as usize;

        if parsed.records.len() > existing_count {
            let new_records = &parsed.records[existing_count..];

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

        for (key, value) in &parsed.hyperparams {
            self.db.conn.execute(
                "INSERT OR REPLACE INTO hyperparams (run_id, key, value) VALUES (?1, ?2, ?3)",
                rusqlite::params![run.id, key, value],
            )?;
        }

        self.refresh_runs()?;
        Ok(())
    }

    pub fn import_existing_files(&mut self) -> Result<usize> {
        let watch_dirs = self.config.resolved_watch_dirs();
        let files = crate::watcher::scan_existing_files(&watch_dirs);

        let mut imported = 0;
        for file in &files {
            match self.import_log_file(file) {
                Ok(_) => imported += 1,
                Err(e) => {
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

    pub fn refresh_runs(&mut self) -> Result<()> {
        self.runs = self.db.get_all_runs()?;
        self.apply_search_filter();
        if self.selected_run_index >= self.visible_runs().len() {
            self.selected_run_index = self.visible_runs().len().saturating_sub(1);
        }
        Ok(())
    }

    pub fn visible_runs(&self) -> &Vec<Run> {
        if self.search_query.is_empty() { &self.runs } else { &self.filtered_runs }
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
        self.detail_sub_view = DetailSubView::Chart;

        let tags = self.db.get_tags_for_run(run_id)?;
        self.current_tags = tags.into_iter().map(|t| t.tag).collect();

        // Load container logs if active
        self.refresh_container_logs();

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

    fn apply_search_filter(&mut self) {
        let query = self.search_query.to_lowercase();
        self.filtered_runs = self.runs.iter()
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

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    pub fn db_file_size(&self) -> String {
        let db_path = self.config.resolved_db_path();
        match std::fs::metadata(&db_path) {
            Ok(meta) => {
                let bytes = meta.len();
                if bytes < 1024 {
                    format!("{} B", bytes)
                } else if bytes < 1024 * 1024 {
                    format!("{:.1} KB", bytes as f64 / 1024.0)
                } else {
                    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                }
            }
            Err(_) => "unknown".to_string(),
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_run_index > 0 { self.selected_run_index -= 1; }
    }

    pub fn move_down(&mut self) {
        let max = self.visible_runs().len().saturating_sub(1);
        if self.selected_run_index < max { self.selected_run_index += 1; }
    }

    pub fn cycle_metric(&mut self) {
        if !self.current_metric_names.is_empty() {
            self.selected_metric_index =
                (self.selected_metric_index + 1) % self.current_metric_names.len();
        }
    }

    pub fn cycle_compare_metric(&mut self) {
        if !self.compare_metric_names.is_empty() {
            self.compare_selected_metric =
                (self.compare_selected_metric + 1) % self.compare_metric_names.len();
        }
    }
}

fn derive_run_name(path: &Path) -> String {
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
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
