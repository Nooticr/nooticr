


use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::enums::agent_status::AgentStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusChange {
    pub from: Option<AgentStatus>,
    pub to: AgentStatus,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}