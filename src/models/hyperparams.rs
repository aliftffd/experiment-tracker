use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperParam {
    pub id: i64,
    pub run_id: i64,
    pub key: String,
    pub value: String,
}

/// Represents a diff between 2 runs' hyperparameters
#[derive(Debug, Clone)]
pub struct HyperParamDiff {
    pub key: String,
    pub left: Option<String>,
    pub right: Option<String>,
}

impl HyperParamDiff {
    pub fn is_different(&self) -> bool {
        self.left != self.right
    }
}
