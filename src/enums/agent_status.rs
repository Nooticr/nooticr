use crate::error::{OrchestratorError, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

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
            (
                AgentStatus::Working,
                AgentStatus::Active | AgentStatus::Busy | AgentStatus::Error | AgentStatus::Idle,
            ) => Ok(next),

            // From Active, can go to Working, Busy, Error, or Idle
            (
                AgentStatus::Active,
                AgentStatus::Working | AgentStatus::Busy | AgentStatus::Error | AgentStatus::Idle,
            ) => Ok(next),

            // From Busy, can go to Active, Working, Error, or Idle
            (
                AgentStatus::Busy,
                AgentStatus::Active | AgentStatus::Working | AgentStatus::Error | AgentStatus::Idle,
            ) => Ok(next),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_default() {
        assert_eq!(AgentStatus::default(), AgentStatus::Idle);
    }

    #[test]
    fn test_agent_status_display() {
        assert_eq!(AgentStatus::Idle.to_string(), "Idle");
        assert_eq!(AgentStatus::Working.to_string(), "Working");
        assert_eq!(AgentStatus::Active.to_string(), "Active");
        assert_eq!(AgentStatus::Error.to_string(), "Error");
        assert_eq!(AgentStatus::Maintenance.to_string(), "Maintenance");
        assert_eq!(AgentStatus::Busy.to_string(), "Busy");
    }

    #[test]
    fn test_agent_status_from_str() {
        assert_eq!("Idle".parse::<AgentStatus>().unwrap(), AgentStatus::Idle);
        assert_eq!(
            "Working".parse::<AgentStatus>().unwrap(),
            AgentStatus::Working
        );
        assert_eq!(
            "Active".parse::<AgentStatus>().unwrap(),
            AgentStatus::Active
        );
        assert_eq!("Error".parse::<AgentStatus>().unwrap(), AgentStatus::Error);
        assert_eq!(
            "Maintenance".parse::<AgentStatus>().unwrap(),
            AgentStatus::Maintenance
        );
        assert_eq!("Busy".parse::<AgentStatus>().unwrap(), AgentStatus::Busy);

        assert!("Invalid".parse::<AgentStatus>().is_err());
    }

    #[test]
    fn test_agent_status_normal_workflow() {
        let mut status = AgentStatus::Idle;

        // Idle -> Working
        status = status
            .transition_to(AgentStatus::Working)
            .expect("Should transition to Working");
        assert_eq!(status, AgentStatus::Working);

        // Working -> Active
        status = status
            .transition_to(AgentStatus::Active)
            .expect("Should transition to Active");
        assert_eq!(status, AgentStatus::Active);

        // Active -> Idle
        status = status
            .transition_to(AgentStatus::Idle)
            .expect("Should transition to Idle");
        assert_eq!(status, AgentStatus::Idle);
    }

    #[test]
    fn test_agent_status_busy_workflow() {
        let mut status = AgentStatus::Active;

        // Active -> Busy
        status = status
            .transition_to(AgentStatus::Busy)
            .expect("Should transition to Busy");
        assert_eq!(status, AgentStatus::Busy);

        // Busy -> Working
        status = status
            .transition_to(AgentStatus::Working)
            .expect("Should transition to Working");
        assert_eq!(status, AgentStatus::Working);

        // Working -> Busy
        status = status
            .transition_to(AgentStatus::Busy)
            .expect("Should transition to Busy");
        assert_eq!(status, AgentStatus::Busy);

        // Busy -> Active
        status = status
            .transition_to(AgentStatus::Active)
            .expect("Should transition to Active");
        assert_eq!(status, AgentStatus::Active);
    }

    #[test]
    fn test_agent_status_error_handling() {
        // Can go to Error from Working, Active, or Busy
        assert!(
            AgentStatus::Working
                .transition_to(AgentStatus::Error)
                .is_ok()
        );
        assert!(
            AgentStatus::Active
                .transition_to(AgentStatus::Error)
                .is_ok()
        );
        assert!(AgentStatus::Busy.transition_to(AgentStatus::Error).is_ok());

        // From Error, can only go to Idle or Maintenance
        assert!(AgentStatus::Error.transition_to(AgentStatus::Idle).is_ok());
        assert!(
            AgentStatus::Error
                .transition_to(AgentStatus::Maintenance)
                .is_ok()
        );

        // Cannot go directly from Error to Working/Active/Busy
        assert!(
            AgentStatus::Error
                .transition_to(AgentStatus::Working)
                .is_err()
        );
        assert!(
            AgentStatus::Error
                .transition_to(AgentStatus::Active)
                .is_err()
        );
        assert!(AgentStatus::Error.transition_to(AgentStatus::Busy).is_err());
    }

    #[test]
    fn test_agent_status_maintenance() {
        // Can go to Maintenance from Idle or Error
        assert!(
            AgentStatus::Idle
                .transition_to(AgentStatus::Maintenance)
                .is_ok()
        );
        assert!(
            AgentStatus::Error
                .transition_to(AgentStatus::Maintenance)
                .is_ok()
        );

        // Cannot go to Maintenance from other states
        assert!(
            AgentStatus::Working
                .transition_to(AgentStatus::Maintenance)
                .is_err()
        );
        assert!(
            AgentStatus::Active
                .transition_to(AgentStatus::Maintenance)
                .is_err()
        );
        assert!(
            AgentStatus::Busy
                .transition_to(AgentStatus::Maintenance)
                .is_err()
        );

        // From Maintenance, can only go to Idle
        assert!(
            AgentStatus::Maintenance
                .transition_to(AgentStatus::Idle)
                .is_ok()
        );
        assert!(
            AgentStatus::Maintenance
                .transition_to(AgentStatus::Working)
                .is_err()
        );
        assert!(
            AgentStatus::Maintenance
                .transition_to(AgentStatus::Active)
                .is_err()
        );
        assert!(
            AgentStatus::Maintenance
                .transition_to(AgentStatus::Error)
                .is_err()
        );
        assert!(
            AgentStatus::Maintenance
                .transition_to(AgentStatus::Busy)
                .is_err()
        );
    }

    #[test]
    fn test_agent_status_invalid_transitions() {
        // Cannot go directly from Idle to Active/Busy/Error
        assert!(
            AgentStatus::Idle
                .transition_to(AgentStatus::Active)
                .is_err()
        );
        assert!(AgentStatus::Idle.transition_to(AgentStatus::Busy).is_err());
        assert!(AgentStatus::Idle.transition_to(AgentStatus::Error).is_err());

        // Cannot stay in same state
        assert!(AgentStatus::Idle.transition_to(AgentStatus::Idle).is_err());
        assert!(
            AgentStatus::Working
                .transition_to(AgentStatus::Working)
                .is_err()
        );
        assert!(
            AgentStatus::Active
                .transition_to(AgentStatus::Active)
                .is_err()
        );
        assert!(
            AgentStatus::Error
                .transition_to(AgentStatus::Error)
                .is_err()
        );
        assert!(
            AgentStatus::Maintenance
                .transition_to(AgentStatus::Maintenance)
                .is_err()
        );
        assert!(AgentStatus::Busy.transition_to(AgentStatus::Busy).is_err());
    }

    #[test]
    fn test_agent_status_predicates() {
        // is_available
        assert!(AgentStatus::Idle.is_available());
        assert!(AgentStatus::Active.is_available());
        assert!(!AgentStatus::Working.is_available());
        assert!(!AgentStatus::Busy.is_available());
        assert!(!AgentStatus::Error.is_available());
        assert!(!AgentStatus::Maintenance.is_available());

        // is_working
        assert!(AgentStatus::Working.is_working());
        assert!(AgentStatus::Busy.is_working());
        assert!(!AgentStatus::Idle.is_working());
        assert!(!AgentStatus::Active.is_working());
        assert!(!AgentStatus::Error.is_working());
        assert!(!AgentStatus::Maintenance.is_working());

        // is_error
        assert!(AgentStatus::Error.is_error());
        assert!(!AgentStatus::Idle.is_error());
        assert!(!AgentStatus::Working.is_error());
        assert!(!AgentStatus::Active.is_error());
        assert!(!AgentStatus::Maintenance.is_error());
        assert!(!AgentStatus::Busy.is_error());

        // is_maintenance
        assert!(AgentStatus::Maintenance.is_maintenance());
        assert!(!AgentStatus::Idle.is_maintenance());
        assert!(!AgentStatus::Working.is_maintenance());
        assert!(!AgentStatus::Active.is_maintenance());
        assert!(!AgentStatus::Error.is_maintenance());
        assert!(!AgentStatus::Busy.is_maintenance());
    }
}
