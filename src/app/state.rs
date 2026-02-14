use crate::config::AppConfig;
use crate::db::Database;
use crate::models::{Metric, Run};
use anyhow::Result;

/// which view/screen is active
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Dashboard,
    RunDetail,
    Compare,
    Help,
}

/// input mode for search/ text input
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    TagInput,
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

    // Run Detail state
    pub current_run: Option<Run>,
    pub current_metrics: Vec<Metric>,
    pub current_metric_names: Vec<String>,
    pub selected_metric_index: usize,
    pub current_tags: Vec<String>,
    pub current_latest_metrics: Vec<(String, f64)>,

    // compare state
    pub compare_runs: Vec<i64>,

    // search or Input
    pub input_mode: InputMode,
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
            tab_titles: vec!["Dashboard".into(), "Detail".into(), "Compare".into()],

            current_run: None,
            current_metrics: Vec::new(),
            current_metric_names: Vec::new(),
            selected_metric_index: 0,
            current_tags: Vec::new(),
            current_latest_metrics: Vec::new(),

            compare_runs: Vec::new(),

            input_mode: InputMode::Normal,
            search_query: String::new(),
            filtered_runs: runs,

            show_help: false,

            status_message: "Ready".into(),
        })
    }

    /// Set the status bar message
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    /// refresh runs from the Database
    pub fn refresh_runs(&mut self) -> Result<()> {
        self.runs = self.db.get_all_runs()?;
        self.apply_search_filter();

        // keep selection in bounds
        if self.selected_run_index >= self.visible_runs().len() {
            self.selected_run_index = self.visible_runs().len().saturating_sub(1);
        }

        Ok(())
    }

    /// Get the currently visible_runs (filtered to all )
    pub fn visible_runs(&self) -> &Vec<Run> {
        if self.search_query.is_empty() {
            &self.runs
        } else {
            &self.filtered_runs
        }
    }

    /// get the currently selected run
    pub fn selected_run(&self) -> Option<&Run> {
        self.visible_runs().get(self.selected_run_index)
    }

    /// load detail data for selected run
    pub fn load_run_detail(&mut self, run_id: i64) -> Result<()> {
        self.current_run = Some(self.db.get_run(run_id)?);
        self.current_metrics = self.db.get_metrics_for_run(run_id)?;
        self.current_metric_names = self.db.get_metric_names(run_id)?;
        self.current_latest_metrics = self.db.get_latest_metrics(run_id)?;
        self.selected_metric_index = 0;

        let tags = self.db.get_tags_for_run(run_id)?;
        self.current_tags = tags.into_iter().map(|t| t.tag).collect();

        Ok(())
    }

    /// navigate to a view
    pub fn navigate_to(&mut self, view: View) {
        self.previous_view = Some(self.current_view.clone());
        self.current_view = view;
    }

    /// go back to previous_view
    pub fn go_back(&mut self) {
        if let Some(prev) = self.previous_view.take() {
            self.current_view = prev;
        } else {
            self.current_view = View::Dashboard;
        }
    }

    /// Apply search filter to runs
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

    /// update search query and refilter
    pub fn update_search(&mut self, query: String) {
        self.search_query = query;
        self.apply_search_filter();
        self.selected_run_index = 0;
    }

    /// move selection up
    pub fn move_up(&mut self) {
        if self.selected_run_index > 0 {
            self.selected_run_index -= 1;
        }
    }

    /// move selection down
    pub fn move_down(&mut self) {
        let max = self.visible_runs().len().saturating_sub(1);
        if self.selected_run_index < max {
            self.selected_run_index += 1;
        }
    }

    /// cycle trough metrics in detail view
    pub fn cycle_metric(&mut self) {
        if !self.current_metric_names.is_empty() {
            self.selected_metric_index =
                (self.selected_metric_index + 1) % self.current_metric_names.len();
        }
    }
}
