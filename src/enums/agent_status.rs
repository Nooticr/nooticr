use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use crate::error::{Result, OrchestratorError};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum AgentStatus {
    #[default]
    Idle,
    Working,
    Active,
    Error,
    Maintenance,
    Busy,
}

impl Display for AgentStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

impl FromStr for AgentStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Idle" => Ok(AgentStatus::Idle),
            "Working" => Ok(AgentStatus::Working),
            "Active" => Ok(AgentStatus::Active),
            "Error" => Ok(AgentStatus::Error),
            "Maintenance" => Ok(AgentStatus::Maintenance),
            "Busy" => Ok(AgentStatus::Busy),
            _ => Err(format!("Invalid AgentStatus: {}", s)),
        }
    }
}

impl AgentStatus {
    /// Transition to the next valid state based on current state
    pub fn transition_to(&self, next: AgentStatus) -> Result<AgentStatus> {
        match (self, &next) {
            // From Idle, can go to Working or Maintenance
            (AgentStatus::Idle, AgentStatus::Working | AgentStatus::Maintenance) => Ok(next),

            // From Working, can go to Active, Busy, Error, or Idle
            (AgentStatus::Working, AgentStatus::Active | AgentStatus::Busy | AgentStatus::Error | AgentStatus::Idle) => Ok(next),

            // From Active, can go to Working, Busy, Error, or Idle
            (AgentStatus::Active, AgentStatus::Working | AgentStatus::Busy | AgentStatus::Error | AgentStatus::Idle) => Ok(next),

            // From Busy, can go to Active, Working, Error, or Idle
            (AgentStatus::Busy, AgentStatus::Active | AgentStatus::Working | AgentStatus::Error | AgentStatus::Idle) => Ok(next),

            // From Error, can go to Idle or Maintenance
            (AgentStatus::Error, AgentStatus::Idle | AgentStatus::Maintenance) => Ok(next),

            // From Maintenance, can go to Idle
            (AgentStatus::Maintenance, AgentStatus::Idle) => Ok(next),

            // Invalid transitions
            _ => Err(OrchestratorError::agent_transition(*self, next)),
        }
    }

    /// Check if the agent is available for new work
    pub fn is_available(&self) -> bool {
        matches!(self, AgentStatus::Idle | AgentStatus::Active)
    }

    /// Check if the agent is currently working
    pub fn is_working(&self) -> bool {
        matches!(self, AgentStatus::Working | AgentStatus::Busy)
    }

    /// Check if the agent is in an error state
    pub fn is_error(&self) -> bool {
        matches!(self, AgentStatus::Error)
    }

    /// Check if the agent is under maintenance
    pub fn is_maintenance(&self) -> bool {
        matches!(self, AgentStatus::Maintenance)
    }
}