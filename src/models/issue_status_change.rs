use crate::enums::issue_status::IssueStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChange {
    pub from: Option<IssueStatus>,
    pub to: IssueStatus,
    pub changed_by: String,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}
