use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: i64,
    pub run_id: i64,
    pub name: String,
    pub epoch: Option<i64>,
    pub step: Option<i64>,
    pub value: f64,
    pub recorded_at: NaiveDateTime,
}

impl Metric {
    /// Gett the x-axis value for charting
    pub fn x_value(&self) -> f64 {
        self.step
            .or(self.epoch)
            .map(|v| v as f64)
            .unwrap_or(self.id as f64)
    }
}
