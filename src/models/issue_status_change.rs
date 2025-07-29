use serde::{Deserialize, Serialize};
use crate::enums::issue_status::IssueStatus;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChange {
    pub from: Option<IssueStatus>,
    pub to: IssueStatus,
    pub changed_by: String,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}