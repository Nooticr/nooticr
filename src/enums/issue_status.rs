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

    /// Check if the current state is terminal (no further transitions possible in normal flow)
    pub fn is_terminal(&self) -> bool {
        matches!(self, IssueStatus::Closed)
    }

    /// Check if the issue is active (being worked on)
    pub fn is_active(&self) -> bool {
        matches!(self, IssueStatus::InProgress | IssueStatus::InReview)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_status_default() {
        assert_eq!(IssueStatus::default(), IssueStatus::Open);
    }

    #[test]
    fn test_issue_status_display() {
        assert_eq!(IssueStatus::Open.to_string(), "Open");
        assert_eq!(IssueStatus::InProgress.to_string(), "InProgress");
        assert_eq!(IssueStatus::InReview.to_string(), "InReview");
        assert_eq!(IssueStatus::Closed.to_string(), "Closed");
    }

    #[test]
    fn test_issue_status_from_str() {
        assert_eq!("Open".parse::<IssueStatus>().unwrap(), IssueStatus::Open);
        assert_eq!("InProgress".parse::<IssueStatus>().unwrap(), IssueStatus::InProgress);
        assert_eq!("InReview".parse::<IssueStatus>().unwrap(), IssueStatus::InReview);
        assert_eq!("Closed".parse::<IssueStatus>().unwrap(), IssueStatus::Closed);

        assert!("Invalid".parse::<IssueStatus>().is_err());
    }

    #[test]
    fn test_issue_status_normal_workflow() {
        let mut status = IssueStatus::Open;

        // Open -> InProgress
        status = status.transition_to(IssueStatus::InProgress).expect("Should transition to InProgress");
        assert_eq!(status, IssueStatus::InProgress);

        // InProgress -> InReview
        status = status.transition_to(IssueStatus::InReview).expect("Should transition to InReview");
        assert_eq!(status, IssueStatus::InReview);

        // InReview -> Closed
        status = status.transition_to(IssueStatus::Closed).expect("Should transition to Closed");
        assert_eq!(status, IssueStatus::Closed);
    }

    #[test]
    fn test_issue_status_progress_method() {
        assert_eq!(IssueStatus::Open.progress().unwrap(), IssueStatus::InProgress);
        assert_eq!(IssueStatus::InProgress.progress().unwrap(), IssueStatus::InReview);
        assert_eq!(IssueStatus::InReview.progress().unwrap(), IssueStatus::Closed);

        // Closed is terminal
        assert!(IssueStatus::Closed.progress().is_err());
    }

    #[test]
    fn test_issue_status_rework_flow() {
        let mut status = IssueStatus::InReview;

        // InReview -> InProgress (for rework)
        status = status.transition_to(IssueStatus::InProgress).expect("Should transition back to InProgress");
        assert_eq!(status, IssueStatus::InProgress);

        // InProgress -> InReview (after rework)
        status = status.transition_to(IssueStatus::InReview).expect("Should transition to InReview");
        assert_eq!(status, IssueStatus::InReview);
    }

    #[test]
    fn test_issue_status_direct_close() {
        // Can close directly from Open (for invalid/duplicate issues)
        let status = IssueStatus::Open.transition_to(IssueStatus::Closed).expect("Should close directly");
        assert_eq!(status, IssueStatus::Closed);
    }

    #[test]
    fn test_issue_status_reopen() {
        // Can reopen closed issues
        let status = IssueStatus::Closed.transition_to(IssueStatus::Open).expect("Should reopen");
        assert_eq!(status, IssueStatus::Open);
    }

    #[test]
    fn test_issue_status_invalid_transitions() {
        // Cannot skip states in normal flow
        assert!(IssueStatus::Open.transition_to(IssueStatus::InReview).is_err());
        assert!(IssueStatus::InProgress.transition_to(IssueStatus::Closed).is_err());

        // Cannot go backwards except for specific cases
        assert!(IssueStatus::InProgress.transition_to(IssueStatus::Open).is_err());
        assert!(IssueStatus::Closed.transition_to(IssueStatus::InProgress).is_err());
        assert!(IssueStatus::Closed.transition_to(IssueStatus::InReview).is_err());

        // Cannot stay in same state
        assert!(IssueStatus::Open.transition_to(IssueStatus::Open).is_err());
        assert!(IssueStatus::InProgress.transition_to(IssueStatus::InProgress).is_err());
        assert!(IssueStatus::InReview.transition_to(IssueStatus::InReview).is_err());
        assert!(IssueStatus::Closed.transition_to(IssueStatus::Closed).is_err());
    }

    #[test]
    fn test_issue_status_is_terminal() {
        assert!(!IssueStatus::Open.is_terminal());
        assert!(!IssueStatus::InProgress.is_terminal());
        assert!(!IssueStatus::InReview.is_terminal());
        assert!(IssueStatus::Closed.is_terminal());
    }

    #[test]
    fn test_issue_status_is_active() {
        assert!(!IssueStatus::Open.is_active());
        assert!(IssueStatus::InProgress.is_active());
        assert!(IssueStatus::InReview.is_active());
        assert!(!IssueStatus::Closed.is_active());
    }

    #[test]
    fn test_issue_status_valid_transitions() {
        let open_transitions = IssueStatus::Open.valid_transitions();
        assert!(open_transitions.contains(&IssueStatus::InProgress));
        assert!(open_transitions.contains(&IssueStatus::Closed));
        assert_eq!(open_transitions.len(), 2);

        let in_progress_transitions = IssueStatus::InProgress.valid_transitions();
        assert!(in_progress_transitions.contains(&IssueStatus::InReview));
        assert_eq!(in_progress_transitions.len(), 1);

        let in_review_transitions = IssueStatus::InReview.valid_transitions();
        assert!(in_review_transitions.contains(&IssueStatus::Closed));
        assert!(in_review_transitions.contains(&IssueStatus::InProgress));
        assert_eq!(in_review_transitions.len(), 2);

        let closed_transitions = IssueStatus::Closed.valid_transitions();
        assert!(closed_transitions.contains(&IssueStatus::Open));
        assert_eq!(closed_transitions.len(), 1);
    }
}