use serde::{Deserialize, Serialize};
use crate::error::{Result, OrchestratorError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    UnderReview,
    Testing,
    Completed,
    Blocked,
    Failed,
    Cancelled,
}

impl TaskStatus {
    /// Transition to the next valid state based on current state
    pub fn transition_to(&self, next: TaskStatus) -> Result<TaskStatus> {
        match (self, &next) {
            // Normal flow transitions
            (TaskStatus::Pending, TaskStatus::InProgress) => Ok(next),
            (TaskStatus::InProgress, TaskStatus::UnderReview) => Ok(next),
            (TaskStatus::UnderReview, TaskStatus::Testing) => Ok(next),
            (TaskStatus::Testing, TaskStatus::Completed) => Ok(next),

            // Can be blocked or failed from InProgress, UnderReview, or Testing
            (TaskStatus::InProgress | TaskStatus::UnderReview | TaskStatus::Testing,
             TaskStatus::Blocked | TaskStatus::Failed) => Ok(next),

            // From Blocked or Failed, can go back to InProgress
            (TaskStatus::Blocked | TaskStatus::Failed, TaskStatus::InProgress) => Ok(next),

            // Can be cancelled from any non-terminal state
            (TaskStatus::Pending | TaskStatus::InProgress | TaskStatus::UnderReview |
             TaskStatus::Testing | TaskStatus::Blocked | TaskStatus::Failed,
             TaskStatus::Cancelled) => Ok(next),

            // Invalid transitions
            _ => Err(OrchestratorError::task_transition(self.clone(), next)),
        }
    }
    
    /// Get valid next states from current state
    pub fn valid_transitions(&self) -> Vec<TaskStatus> {
        match self {
            TaskStatus::Pending => vec![TaskStatus::InProgress, TaskStatus::Cancelled],
            TaskStatus::InProgress => vec![
                TaskStatus::UnderReview,
                TaskStatus::Blocked,
                TaskStatus::Failed,
                TaskStatus::Cancelled,
            ],
            TaskStatus::UnderReview => vec![
                TaskStatus::Testing,
                TaskStatus::Blocked,
                TaskStatus::Failed,
                TaskStatus::Cancelled,
            ],
            TaskStatus::Testing => vec![
                TaskStatus::Completed,
                TaskStatus::Blocked,
                TaskStatus::Failed,
                TaskStatus::Cancelled,
            ],
            TaskStatus::Completed => vec![], // Terminal state
            TaskStatus::Blocked | TaskStatus::Failed => vec![
                TaskStatus::InProgress,
                TaskStatus::Cancelled,
            ],
            TaskStatus::Cancelled => vec![], // Terminal state
        }
    }
    
    /// Check if the current state is terminal (no further transitions possible)
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Completed | TaskStatus::Cancelled)
    }
    
    /// Attempt to progress to the next state in the normal workflow
    pub fn progress(&self) -> Result<TaskStatus> {
        match self {
            TaskStatus::Pending => Ok(TaskStatus::InProgress),
            TaskStatus::InProgress => Ok(TaskStatus::UnderReview),
            TaskStatus::UnderReview => Ok(TaskStatus::Testing),
            TaskStatus::Testing => Ok(TaskStatus::Completed),
            TaskStatus::Blocked | TaskStatus::Failed => Ok(TaskStatus::InProgress),
            TaskStatus::Completed | TaskStatus::Cancelled => {
                Err(OrchestratorError::task_terminal(self.clone()))
            }
        }
    }
}




