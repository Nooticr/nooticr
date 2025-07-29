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

