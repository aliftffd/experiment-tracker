use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
    Stopped,
}

impl fmt::Display for RunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunStatus::Running => write!(f, "running"),
            RunStatus::Completed => write!(f, "completed"),
            RunStatus::Failed => write!(f, "failed"),
            RunStatus::Stopped => write!(f, "stopped"),
        }
    }
}

impl From<&str> for RunStatus {
    fn from(s: &str) -> Self {
        match s {
            "completed" => RunStatus::Completed,
            "failed" => RunStatus::Failed,
            "stopped" => RunStatus::Stopped,
            _ => RunStatus::Running,
        }
    }
}

impl RunStatus {
    pub fn symbol(&self) -> &str {
        match self {
            RunStatus::Running => "⟳",
            RunStatus::Completed => "✓",
            RunStatus::Failed => "✗",
            RunStatus::Stopped => "■",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: i64,
    pub name: String,
    pub status: RunStatus,
    pub log_status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub notes: String,
}
