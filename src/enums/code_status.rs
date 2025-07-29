use serde::{Deserialize, Serialize};
use crate::error::{Result, OrchestratorError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CodeStatus {
    #[default]
    Pending,
    Coded,
    PullRequest,
    Mergeable,
    Merged,
    Abandoned,
    CIFailed,
    MergeConflict,
    CISuccessful,
}

impl CodeStatus {
    /// Transition to the next valid state based on current state
    pub fn transition_to(&self, next: CodeStatus) -> Result<CodeStatus> {
        match (self, &next) {
            // Normal flow
            (CodeStatus::Pending, CodeStatus::Coded) => Ok(next),
            (CodeStatus::Coded, CodeStatus::PullRequest) => Ok(next),

            // From PullRequest, can go to CI states or MergeConflict
            (CodeStatus::PullRequest, CodeStatus::CISuccessful | CodeStatus::CIFailed | CodeStatus::MergeConflict) => Ok(next),

            // Only CISuccessful can proceed to Mergeable
            (CodeStatus::CISuccessful, CodeStatus::Mergeable) => Ok(next),

            // From Mergeable to Merged
            (CodeStatus::Mergeable, CodeStatus::Merged) => Ok(next),

            // Failed states can go back to Coded for fixes
            (CodeStatus::CIFailed | CodeStatus::MergeConflict, CodeStatus::Coded) => Ok(next),

            // Can abandon from various states
            (CodeStatus::Pending | CodeStatus::Coded | CodeStatus::PullRequest |
             CodeStatus::CIFailed | CodeStatus::MergeConflict | CodeStatus::CISuccessful |
             CodeStatus::Mergeable, CodeStatus::Abandoned) => Ok(next),

            // Invalid transitions
            _ => Err(OrchestratorError::code_transition(*self, next)),
        }
    }
    
    /// Get valid next states from current state
    pub fn valid_transitions(&self) -> Vec<CodeStatus> {
        match self {
            CodeStatus::Pending => vec![CodeStatus::Coded, CodeStatus::Abandoned],
            CodeStatus::Coded => vec![CodeStatus::PullRequest, CodeStatus::Abandoned],
            CodeStatus::PullRequest => vec![
                CodeStatus::CISuccessful,
                CodeStatus::CIFailed,
                CodeStatus::MergeConflict,
                CodeStatus::Abandoned,
            ],
            CodeStatus::CISuccessful => vec![CodeStatus::Mergeable, CodeStatus::Abandoned],
            CodeStatus::CIFailed => vec![CodeStatus::Coded, CodeStatus::Abandoned],
            CodeStatus::MergeConflict => vec![CodeStatus::Coded, CodeStatus::Abandoned],
            CodeStatus::Mergeable => vec![CodeStatus::Merged, CodeStatus::Abandoned],
            CodeStatus::Merged => vec![], // Terminal state
            CodeStatus::Abandoned => vec![], // Terminal state
        }
    }
    
    /// Check if the current state is terminal (no further transitions possible)
    pub fn is_terminal(&self) -> bool {
        matches!(self, CodeStatus::Merged | CodeStatus::Abandoned)
    }
    
    /// Check if the current state indicates a failure that needs fixing
    pub fn is_failure_state(&self) -> bool {
        matches!(self, CodeStatus::CIFailed | CodeStatus::MergeConflict)
    }
    
    /// Check if CI passed
    pub fn is_ci_passed(&self) -> bool {
        matches!(self, CodeStatus::CISuccessful)
    }
    
    /// Attempt to progress to the next state in the normal workflow
    pub fn progress(&self) -> Result<CodeStatus> {
        match self {
            CodeStatus::Pending => Ok(CodeStatus::Coded),
            CodeStatus::Coded => Ok(CodeStatus::PullRequest),
            CodeStatus::PullRequest => {
                Err(OrchestratorError::code_external_action("Pull request needs CI to run"))
            }
            CodeStatus::CISuccessful => Ok(CodeStatus::Mergeable),
            CodeStatus::Mergeable => Ok(CodeStatus::Merged),
            CodeStatus::CIFailed | CodeStatus::MergeConflict => Ok(CodeStatus::Coded),
            CodeStatus::Merged | CodeStatus::Abandoned => {
                Err(OrchestratorError::code_terminal(*self))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_status_default() {
        assert_eq!(CodeStatus::default(), CodeStatus::Pending);
    }

    #[test]
    fn test_code_status_normal_flow() {
        let mut status = CodeStatus::Pending;

        // Pending -> Coded
        status = status.transition_to(CodeStatus::Coded).expect("Should transition to Coded");
        assert_eq!(status, CodeStatus::Coded);

        // Coded -> PullRequest
        status = status.transition_to(CodeStatus::PullRequest).expect("Should transition to PullRequest");
        assert_eq!(status, CodeStatus::PullRequest);

        // PullRequest -> CISuccessful
        status = status.transition_to(CodeStatus::CISuccessful).expect("Should transition to CISuccessful");
        assert_eq!(status, CodeStatus::CISuccessful);

        // CISuccessful -> Mergeable
        status = status.transition_to(CodeStatus::Mergeable).expect("Should transition to Mergeable");
        assert_eq!(status, CodeStatus::Mergeable);

        // Mergeable -> Merged
        status = status.transition_to(CodeStatus::Merged).expect("Should transition to Merged");
        assert_eq!(status, CodeStatus::Merged);
    }

    #[test]
    fn test_code_status_progress_method() {
        assert_eq!(CodeStatus::Pending.progress().unwrap(), CodeStatus::Coded);
        assert_eq!(CodeStatus::Coded.progress().unwrap(), CodeStatus::PullRequest);
        assert_eq!(CodeStatus::CISuccessful.progress().unwrap(), CodeStatus::Mergeable);
        assert_eq!(CodeStatus::Mergeable.progress().unwrap(), CodeStatus::Merged);
        assert_eq!(CodeStatus::CIFailed.progress().unwrap(), CodeStatus::Coded);
        assert_eq!(CodeStatus::MergeConflict.progress().unwrap(), CodeStatus::Coded);

        // PullRequest needs external action (CI)
        assert!(CodeStatus::PullRequest.progress().is_err());

        // Terminal states should error
        assert!(CodeStatus::Merged.progress().is_err());
        assert!(CodeStatus::Abandoned.progress().is_err());
    }

    #[test]
    fn test_code_status_ci_workflow() {
        let mut status = CodeStatus::PullRequest;

        // PullRequest -> CIFailed
        status = status.transition_to(CodeStatus::CIFailed).expect("Should transition to CIFailed");
        assert_eq!(status, CodeStatus::CIFailed);

        // CIFailed -> Coded (for fixes)
        status = status.transition_to(CodeStatus::Coded).expect("Should transition to Coded");
        assert_eq!(status, CodeStatus::Coded);

        // Coded -> PullRequest (retry)
        status = status.transition_to(CodeStatus::PullRequest).expect("Should transition to PullRequest");
        assert_eq!(status, CodeStatus::PullRequest);

        // PullRequest -> CISuccessful
        status = status.transition_to(CodeStatus::CISuccessful).expect("Should transition to CISuccessful");
        assert_eq!(status, CodeStatus::CISuccessful);
    }

    #[test]
    fn test_code_status_merge_conflict_workflow() {
        let mut status = CodeStatus::PullRequest;

        // PullRequest -> MergeConflict
        status = status.transition_to(CodeStatus::MergeConflict).expect("Should transition to MergeConflict");
        assert_eq!(status, CodeStatus::MergeConflict);

        // MergeConflict -> Coded (for fixes)
        status = status.transition_to(CodeStatus::Coded).expect("Should transition to Coded");
        assert_eq!(status, CodeStatus::Coded);
    }

    #[test]
    fn test_code_status_abandonment() {
        // Can abandon from most states
        assert!(CodeStatus::Pending.transition_to(CodeStatus::Abandoned).is_ok());
        assert!(CodeStatus::Coded.transition_to(CodeStatus::Abandoned).is_ok());
        assert!(CodeStatus::PullRequest.transition_to(CodeStatus::Abandoned).is_ok());
        assert!(CodeStatus::CIFailed.transition_to(CodeStatus::Abandoned).is_ok());
        assert!(CodeStatus::MergeConflict.transition_to(CodeStatus::Abandoned).is_ok());
        assert!(CodeStatus::CISuccessful.transition_to(CodeStatus::Abandoned).is_ok());
        assert!(CodeStatus::Mergeable.transition_to(CodeStatus::Abandoned).is_ok());

        // Cannot abandon from terminal states
        assert!(CodeStatus::Merged.transition_to(CodeStatus::Abandoned).is_err());
        assert!(CodeStatus::Abandoned.transition_to(CodeStatus::Abandoned).is_err());
    }

    #[test]
    fn test_code_status_invalid_transitions() {
        // Cannot skip states in normal flow
        assert!(CodeStatus::Pending.transition_to(CodeStatus::PullRequest).is_err());
        assert!(CodeStatus::Pending.transition_to(CodeStatus::Mergeable).is_err());
        assert!(CodeStatus::Coded.transition_to(CodeStatus::Mergeable).is_err());

        // Cannot go to Mergeable without CI success
        assert!(CodeStatus::PullRequest.transition_to(CodeStatus::Mergeable).is_err());
        assert!(CodeStatus::CIFailed.transition_to(CodeStatus::Mergeable).is_err());
        assert!(CodeStatus::MergeConflict.transition_to(CodeStatus::Mergeable).is_err());

        // Cannot transition from terminal states
        assert!(CodeStatus::Merged.transition_to(CodeStatus::Coded).is_err());
        assert!(CodeStatus::Abandoned.transition_to(CodeStatus::Coded).is_err());

        // Cannot stay in same state
        assert!(CodeStatus::Pending.transition_to(CodeStatus::Pending).is_err());
        assert!(CodeStatus::Coded.transition_to(CodeStatus::Coded).is_err());
    }

    #[test]
    fn test_code_status_is_terminal() {
        assert!(!CodeStatus::Pending.is_terminal());
        assert!(!CodeStatus::Coded.is_terminal());
        assert!(!CodeStatus::PullRequest.is_terminal());
        assert!(!CodeStatus::CISuccessful.is_terminal());
        assert!(!CodeStatus::CIFailed.is_terminal());
        assert!(!CodeStatus::MergeConflict.is_terminal());
        assert!(!CodeStatus::Mergeable.is_terminal());
        assert!(CodeStatus::Merged.is_terminal());
        assert!(CodeStatus::Abandoned.is_terminal());
    }

    #[test]
    fn test_code_status_is_failure_state() {
        assert!(!CodeStatus::Pending.is_failure_state());
        assert!(!CodeStatus::Coded.is_failure_state());
        assert!(!CodeStatus::PullRequest.is_failure_state());
        assert!(!CodeStatus::CISuccessful.is_failure_state());
        assert!(CodeStatus::CIFailed.is_failure_state());
        assert!(CodeStatus::MergeConflict.is_failure_state());
        assert!(!CodeStatus::Mergeable.is_failure_state());
        assert!(!CodeStatus::Merged.is_failure_state());
        assert!(!CodeStatus::Abandoned.is_failure_state());
    }

    #[test]
    fn test_code_status_is_ci_passed() {
        assert!(!CodeStatus::Pending.is_ci_passed());
        assert!(!CodeStatus::Coded.is_ci_passed());
        assert!(!CodeStatus::PullRequest.is_ci_passed());
        assert!(CodeStatus::CISuccessful.is_ci_passed());
        assert!(!CodeStatus::CIFailed.is_ci_passed());
        assert!(!CodeStatus::MergeConflict.is_ci_passed());
        assert!(!CodeStatus::Mergeable.is_ci_passed());
        assert!(!CodeStatus::Merged.is_ci_passed());
        assert!(!CodeStatus::Abandoned.is_ci_passed());
    }

    #[test]
    fn test_code_status_valid_transitions() {
        let pending_transitions = CodeStatus::Pending.valid_transitions();
        assert!(pending_transitions.contains(&CodeStatus::Coded));
        assert!(pending_transitions.contains(&CodeStatus::Abandoned));
        assert_eq!(pending_transitions.len(), 2);

        let pr_transitions = CodeStatus::PullRequest.valid_transitions();
        assert!(pr_transitions.contains(&CodeStatus::CISuccessful));
        assert!(pr_transitions.contains(&CodeStatus::CIFailed));
        assert!(pr_transitions.contains(&CodeStatus::MergeConflict));
        assert!(pr_transitions.contains(&CodeStatus::Abandoned));
        assert_eq!(pr_transitions.len(), 4);

        let merged_transitions = CodeStatus::Merged.valid_transitions();
        assert!(merged_transitions.is_empty());

        let abandoned_transitions = CodeStatus::Abandoned.valid_transitions();
        assert!(abandoned_transitions.is_empty());
    }
}

