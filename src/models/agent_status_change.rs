


use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::enums::agent_status::AgentStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusChange {
    pub from: Option<AgentStatus>,
    pub to: AgentStatus,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}