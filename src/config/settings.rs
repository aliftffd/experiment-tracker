use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub ui: UiConfig,
    pub parser: ParserConfig,
    #[serde(default)]
    pub gpu: Option<GpuConfig>,
    #[serde(default)]
    pub docker: Option<DockerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub watch_dirs: Vec<String>,
    pub refresh_rate_ms: u64,
    pub db_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_sparklines: bool,
    pub show_gpu_bar: bool,
    pub max_chart_points: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    pub default_format: String,
    pub epoch_field: String,
    pub step_field: String,
    pub loss_field: String,
    pub accuracy_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    pub poll_interval_secs: u64,
    pub temp_warning: u32,
    pub temp_critical: u32,
    pub vram_warning: u32,
    pub vram_critical: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub default_image: String,
    pub gpu: bool,
    pub container_workdir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                watch_dirs: vec!["./experiments".into()],
                refresh_rate_ms: 1000,
                db_path: "~/.local/share/experiment-tracker/tracker.db".into(),
            },
            ui: UiConfig {
                theme: "dark".into(),
                show_sparklines: true,
                show_gpu_bar: true,
                max_chart_points: 500,
            },
            parser: ParserConfig {
                default_format: "auto".into(),
                epoch_field: "epoch".into(),
                step_field: "step".into(),
                loss_field: "loss".into(),
                accuracy_field: "accuracy".into(),
            },

            gpu: Some(GpuConfig {
                poll_interval_secs: 2,
                temp_warning: 80,
                temp_critical: 90,
                vram_warning: 80,
                vram_critical: 95,
            }),
            docker: Some(DockerConfig {
                default_image: "thesis-training:latest".into(),
                gpu: true,
                container_workdir: "/workspace/output".into(),
            }),

        }
    }
}

impl AppConfig {
    /// Load config from file, falling back to defaults
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path(),
        };

        let config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config: {}", config_path.display()))?;
            toml::from_str(&content).with_context(|| "Failed to parse config")?
        } else {
            AppConfig::default()
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate config values are within acceptable ranges
    pub fn validate(&self) -> Result<()> {
        if self.general.watch_dirs.is_empty() {
            anyhow::bail!("Config error: watch_dirs must not be empty");
        }
        if self.general.refresh_rate_ms < 50 {
            anyhow::bail!("Config error: refresh_rate_ms must be >= 50 (got {})", self.general.refresh_rate_ms);
        }
        if self.general.db_path.is_empty() {
            anyhow::bail!("Config error: db_path must not be empty");
        }
        if let Some(gpu) = &self.gpu {
            if gpu.poll_interval_secs < 1 {
                anyhow::bail!("Config error: gpu.poll_interval_secs must be >= 1 (got {})", gpu.poll_interval_secs);
            }
            if gpu.temp_warning >= gpu.temp_critical {
                anyhow::bail!("Config error: gpu.temp_warning ({}) must be < gpu.temp_critical ({})", gpu.temp_warning, gpu.temp_critical);
            }
            if gpu.vram_warning >= gpu.vram_critical {
                anyhow::bail!("Config error: gpu.vram_warning ({}) must be < gpu.vram_critical ({})", gpu.vram_warning, gpu.vram_critical);
            }
        }
        if self.ui.max_chart_points == 0 {
            anyhow::bail!("Config error: ui.max_chart_points must be > 0");
        }
        Ok(())
    }

    /// Resolve the database path (expand ~)
    pub fn resolved_db_path(&self) -> PathBuf {
        expand_tilde(&self.general.db_path)
    }

    /// Resolve watch directories (expand ~)
    pub fn resolved_watch_dirs(&self) -> Vec<PathBuf> {
        self.general
            .watch_dirs
            .iter()
            .map(|d| expand_tilde(d))
            .collect()
    }

    fn default_config_path() -> PathBuf {
        let config_dir = dirs_or_default();
        config_dir.join("experiment-tracker").join("config.toml")
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn dirs_or_default() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").unwrap_or_default();
            PathBuf::from(home).join(".config")
        })
}
