use crate::error::{OrchestratorError, Result};
use serde::{Deserialize, Serialize};

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
            (
                TaskStatus::InProgress | TaskStatus::UnderReview | TaskStatus::Testing,
                TaskStatus::Blocked | TaskStatus::Failed,
            ) => Ok(next),

            // From Blocked or Failed, can go back to InProgress
            (TaskStatus::Blocked | TaskStatus::Failed, TaskStatus::InProgress) => Ok(next),

            // Can be cancelled from any non-terminal state
            (
                TaskStatus::Pending
                | TaskStatus::InProgress
                | TaskStatus::UnderReview
                | TaskStatus::Testing
                | TaskStatus::Blocked
                | TaskStatus::Failed,
                TaskStatus::Cancelled,
            ) => Ok(next),

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
            TaskStatus::Blocked | TaskStatus::Failed => {
                vec![TaskStatus::InProgress, TaskStatus::Cancelled]
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_default() {
        assert_eq!(TaskStatus::default(), TaskStatus::Pending);
    }

    #[test]
    fn test_task_status_normal_flow() {
        let mut status = TaskStatus::Pending;

        // Pending -> InProgress
        status = status
            .transition_to(TaskStatus::InProgress)
            .expect("Should transition to InProgress");
        assert_eq!(status, TaskStatus::InProgress);

        // InProgress -> UnderReview
        status = status
            .transition_to(TaskStatus::UnderReview)
            .expect("Should transition to UnderReview");
        assert_eq!(status, TaskStatus::UnderReview);

        // UnderReview -> Testing
        status = status
            .transition_to(TaskStatus::Testing)
            .expect("Should transition to Testing");
        assert_eq!(status, TaskStatus::Testing);

        // Testing -> Completed
        status = status
            .transition_to(TaskStatus::Completed)
            .expect("Should transition to Completed");
        assert_eq!(status, TaskStatus::Completed);
    }

    #[test]
    fn test_task_status_progress_method() {
        assert_eq!(
            TaskStatus::Pending.progress().unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!(
            TaskStatus::InProgress.progress().unwrap(),
            TaskStatus::UnderReview
        );
        assert_eq!(
            TaskStatus::UnderReview.progress().unwrap(),
            TaskStatus::Testing
        );
        assert_eq!(
            TaskStatus::Testing.progress().unwrap(),
            TaskStatus::Completed
        );
        assert_eq!(
            TaskStatus::Blocked.progress().unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!(
            TaskStatus::Failed.progress().unwrap(),
            TaskStatus::InProgress
        );

        // Terminal states should error
        assert!(TaskStatus::Completed.progress().is_err());
        assert!(TaskStatus::Cancelled.progress().is_err());
    }

    #[test]
    fn test_task_status_blocking_and_failure() {
        // Can be blocked from InProgress, UnderReview, Testing
        assert!(
            TaskStatus::InProgress
                .transition_to(TaskStatus::Blocked)
                .is_ok()
        );
        assert!(
            TaskStatus::UnderReview
                .transition_to(TaskStatus::Blocked)
                .is_ok()
        );
        assert!(
            TaskStatus::Testing
                .transition_to(TaskStatus::Blocked)
                .is_ok()
        );

        // Can be failed from InProgress, UnderReview, Testing
        assert!(
            TaskStatus::InProgress
                .transition_to(TaskStatus::Failed)
                .is_ok()
        );
        assert!(
            TaskStatus::UnderReview
                .transition_to(TaskStatus::Failed)
                .is_ok()
        );
        assert!(
            TaskStatus::Testing
                .transition_to(TaskStatus::Failed)
                .is_ok()
        );

        // Can recover from Blocked/Failed to InProgress
        assert!(
            TaskStatus::Blocked
                .transition_to(TaskStatus::InProgress)
                .is_ok()
        );
        assert!(
            TaskStatus::Failed
                .transition_to(TaskStatus::InProgress)
                .is_ok()
        );
    }

    #[test]
    fn test_task_status_cancellation() {
        // Can be cancelled from any non-terminal state
        assert!(
            TaskStatus::Pending
                .transition_to(TaskStatus::Cancelled)
                .is_ok()
        );
        assert!(
            TaskStatus::InProgress
                .transition_to(TaskStatus::Cancelled)
                .is_ok()
        );
        assert!(
            TaskStatus::UnderReview
                .transition_to(TaskStatus::Cancelled)
                .is_ok()
        );
        assert!(
            TaskStatus::Testing
                .transition_to(TaskStatus::Cancelled)
                .is_ok()
        );
        assert!(
            TaskStatus::Blocked
                .transition_to(TaskStatus::Cancelled)
                .is_ok()
        );
        assert!(
            TaskStatus::Failed
                .transition_to(TaskStatus::Cancelled)
                .is_ok()
        );

        // Cannot be cancelled from terminal states
        assert!(
            TaskStatus::Completed
                .transition_to(TaskStatus::Cancelled)
                .is_err()
        );
        assert!(
            TaskStatus::Cancelled
                .transition_to(TaskStatus::Cancelled)
                .is_err()
        );
    }

    #[test]
    fn test_task_status_invalid_transitions() {
        // Cannot skip states in normal flow
        assert!(
            TaskStatus::Pending
                .transition_to(TaskStatus::UnderReview)
                .is_err()
        );
        assert!(
            TaskStatus::Pending
                .transition_to(TaskStatus::Testing)
                .is_err()
        );
        assert!(
            TaskStatus::Pending
                .transition_to(TaskStatus::Completed)
                .is_err()
        );

        // Cannot go backwards in normal flow
        assert!(
            TaskStatus::UnderReview
                .transition_to(TaskStatus::InProgress)
                .is_err()
        );
        assert!(
            TaskStatus::Testing
                .transition_to(TaskStatus::UnderReview)
                .is_err()
        );
        assert!(
            TaskStatus::Completed
                .transition_to(TaskStatus::Testing)
                .is_err()
        );

        // Cannot transition from terminal states
        assert!(
            TaskStatus::Completed
                .transition_to(TaskStatus::InProgress)
                .is_err()
        );
        assert!(
            TaskStatus::Cancelled
                .transition_to(TaskStatus::InProgress)
                .is_err()
        );
    }

    #[test]
    fn test_task_status_is_terminal() {
        assert!(!TaskStatus::Pending.is_terminal());
        assert!(!TaskStatus::InProgress.is_terminal());
        assert!(!TaskStatus::UnderReview.is_terminal());
        assert!(!TaskStatus::Testing.is_terminal());
        assert!(!TaskStatus::Blocked.is_terminal());
        assert!(!TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_task_status_valid_transitions() {
        let pending_transitions = TaskStatus::Pending.valid_transitions();
        assert!(pending_transitions.contains(&TaskStatus::InProgress));
        assert!(pending_transitions.contains(&TaskStatus::Cancelled));
        assert_eq!(pending_transitions.len(), 2);

        let in_progress_transitions = TaskStatus::InProgress.valid_transitions();
        assert!(in_progress_transitions.contains(&TaskStatus::UnderReview));
        assert!(in_progress_transitions.contains(&TaskStatus::Blocked));
        assert!(in_progress_transitions.contains(&TaskStatus::Failed));
        assert!(in_progress_transitions.contains(&TaskStatus::Cancelled));
        assert_eq!(in_progress_transitions.len(), 4);

        let completed_transitions = TaskStatus::Completed.valid_transitions();
        assert!(completed_transitions.is_empty());

        let cancelled_transitions = TaskStatus::Cancelled.valid_transitions();
        assert!(cancelled_transitions.is_empty());
    }
}
