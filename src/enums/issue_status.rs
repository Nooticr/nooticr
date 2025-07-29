use serde::{Deserialize, Serialize};
use crate::error::{Result, OrchestratorError};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum IssueStatus {
    #[default]
    Open,
    InProgress,
    InReview,
    Closed,
}

impl Display for IssueStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

impl FromStr for IssueStatus {
    type Err = String;  // You can define a custom error type if needed

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Open" => Ok(IssueStatus::Open),
            "InProgress" => Ok(IssueStatus::InProgress),
            "InReview" => Ok(IssueStatus::InReview),
            "Closed" => Ok(IssueStatus::Closed),
            _ => Err(format!("Invalid IssueStatus: {}", s)),
        }
    }
}

// Extension to IssueStatus for state machine functionality
impl IssueStatus {
    /// Transition to the next valid state based on current state
    pub fn transition_to(&self, next: IssueStatus) -> Result<IssueStatus> {
        match (self, &next) {
            // Normal workflow transitions
            (IssueStatus::Open, IssueStatus::InProgress) => Ok(next),
            (IssueStatus::InProgress, IssueStatus::InReview) => Ok(next),
            (IssueStatus::InReview, IssueStatus::Closed) => Ok(next),
            
            // Can go back from InReview to InProgress (for rework)
            (IssueStatus::InReview, IssueStatus::InProgress) => Ok(next),
            
            // Can reopen closed issues
            (IssueStatus::Closed, IssueStatus::Open) => Ok(next),
            
            // Direct close from Open (for invalid/duplicate issues)
            (IssueStatus::Open, IssueStatus::Closed) => Ok(next),
            
            // Invalid transitions
            _ => Err(OrchestratorError::issue_transition(
                self.clone(),
                next,
            )),
        }
    }
    
    /// Get valid next states from current state
    pub fn valid_transitions(&self) -> Vec<IssueStatus> {
        match self {
            IssueStatus::Open => vec![IssueStatus::InProgress, IssueStatus::Closed],
            IssueStatus::InProgress => vec![IssueStatus::InReview],
            IssueStatus::InReview => vec![IssueStatus::Closed, IssueStatus::InProgress],
            IssueStatus::Closed => vec![IssueStatus::Open],
        }
    }
    
    /// Attempt to progress to the next state in the normal workflow
    pub fn progress(&self) -> Result<IssueStatus> {
        match self {
            IssueStatus::Open => Ok(IssueStatus::InProgress),
            IssueStatus::InProgress => Ok(IssueStatus::InReview),
            IssueStatus::InReview => Ok(IssueStatus::Closed),
            IssueStatus::Closed => Err(OrchestratorError::issue_terminal(IssueStatus::Closed)),
        }
    }
}