use super::agent::Agent;
use crate::enums::{CodeStatus, CommentType, Priority, TaskStatus};
use crate::error::{OrchestratorError, Result};
use crate::models::comment::Comment;
use crate::models::pull_request::PullRequest;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: Uuid,
    pub title: String,
    pub status: TaskStatus,
    pub code_status: CodeStatus,
    pub assigned_to: Option<String>,
    pub priority: Priority,
    pub is_overdue: bool,
    pub ci_attemps: u32,
    pub comment_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub status_history: Vec<(TaskStatus, chrono::DateTime<chrono::Utc>)>,
    pub code_status: CodeStatus,
    pub code_status_history: Vec<(CodeStatus, chrono::DateTime<chrono::Utc>)>,
    pub rapporter: Option<Agent>,
    pub assigned_to: Option<Agent>,
    pub priority: Priority,
    pub estimated_complexity: Option<u8>, // 1-10 scale
    pub estimated_duration: Option<u32>,  // in minutes
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub comments: Vec<Comment>,
    pub ci_attemps: u32,
    pub depends_on: Vec<Uuid>,
    pub pull_request: Option<PullRequest>,
}

impl Task {
    /// Create a new task
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        priority: Priority,
    ) -> Self {
        let now = Utc::now();
        let task_status = TaskStatus::default();
        let code_status = CodeStatus::default();

        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            status: task_status.clone(),
            status_history: vec![(task_status, now)],
            code_status,
            code_status_history: vec![(code_status, now)],
            rapporter: None,
            assigned_to: None,
            priority,
            estimated_complexity: None,
            estimated_duration: None,
            created_at: Some(now),
            updated_at: Some(now),
            completed_at: None,
            due_date: None,
            tags: Vec::new(),
            comments: Vec::new(),
            ci_attemps: 0,
            depends_on: Vec::new(),
            pull_request: None,
        }
    }

    /// Transition task status with validation
    pub fn transition_task_status(&mut self, next_status: TaskStatus) -> Result<()> {
        // Validate the transition
        let new_status = self.status.transition_to(next_status)?;

        // Apply business rules and constraints
        match (&self.status, &new_status) {
            // Can only start InProgress if assigned
            (TaskStatus::Pending, TaskStatus::InProgress) if self.assigned_to.is_none() => {
                return Err(OrchestratorError::validation(
                    "Task must be assigned before starting",
                ));
            }
            // Can only complete if code is merged
            (TaskStatus::Testing, TaskStatus::Completed)
                if self.code_status != CodeStatus::Merged =>
            {
                return Err(OrchestratorError::validation(
                    "Code must be merged before completing task",
                ));
            }
            _ => {}
        }

        // Update status and history
        let now = Utc::now();
        self.status = new_status.clone();
        self.status_history.push((new_status.clone(), now));
        self.updated_at = Some(now);

        // Handle completion
        if new_status == TaskStatus::Completed {
            self.completed_at = Some(now);
        }

        Ok(())
    }

    /// Transition code status with validation
    pub fn transition_code_status(&mut self, next_status: CodeStatus) -> Result<()> {
        // Validate the transition
        let new_status = self.code_status.transition_to(next_status)?;

        // Apply business rules
        match (&self.code_status, &new_status) {
            // Can only start coding if task is InProgress
            (CodeStatus::Pending, CodeStatus::Coded) if self.status == TaskStatus::Pending => {
                return Err(OrchestratorError::validation(
                    "Task must be in progress before coding",
                ));
            }
            // Track CI attempts (only if no PR exists, otherwise PR handles this)
            (CodeStatus::PullRequest, CodeStatus::CIFailed | CodeStatus::CISuccessful)
                if self.pull_request.is_none() =>
            {
                self.ci_attemps += 1;
            }
            _ => {}
        }

        // Update status and history
        let now = Utc::now();
        self.code_status = new_status;
        self.code_status_history.push((new_status, now));
        self.updated_at = Some(now);

        Ok(())
    }

    /// Handle CI result
    pub fn handle_ci_result(&mut self, success: bool) -> Result<()> {
        if self.code_status != CodeStatus::PullRequest {
            return Err(OrchestratorError::validation(
                "CI can only run on pull requests",
            ));
        }

        let next_status = if success {
            CodeStatus::CISuccessful
        } else {
            CodeStatus::CIFailed
        };

        self.transition_code_status(next_status)
    }

    /// Add a comment to the task
    pub fn add_comment(&mut self, author: impl Into<String>, content: impl Into<String>) {
        let comment = Comment::new(author, content, CommentType::Task);
        self.comments.push(comment);
        self.updated_at = Some(Utc::now());
    }

    /// Add a comment to the task with sync status
    pub fn add_comment_with_sync(
        &mut self,
        author: impl Into<String>,
        content: impl Into<String>,
        synced: bool,
    ) {
        let comment = Comment::new_with_sync(author, content, CommentType::Task, synced);
        self.comments.push(comment);
        self.updated_at = Some(Utc::now());
    }

    /// Get all unsynced comments
    pub fn get_unsynced_comments(&self) -> Vec<&Comment> {
        self.comments.iter().filter(|c| c.needs_sync()).collect()
    }

    /// Mark all comments as synced
    pub fn mark_all_comments_synced(&mut self) {
        for comment in &mut self.comments {
            comment.mark_synced();
        }
        self.updated_at = Some(Utc::now());
    }

    /// Get comment by ID
    pub fn get_comment(&self, comment_id: Uuid) -> Option<&Comment> {
        self.comments.iter().find(|c| c.id == comment_id)
    }

    /// Get mutable comment by ID
    pub fn get_comment_mut(&mut self, comment_id: Uuid) -> Option<&mut Comment> {
        self.comments.iter_mut().find(|c| c.id == comment_id)
    }

    /// Update a comment's content
    pub fn update_comment(
        &mut self,
        comment_id: Uuid,
        new_content: impl Into<String>,
    ) -> Result<()> {
        let comment = self
            .get_comment_mut(comment_id)
            .ok_or_else(|| OrchestratorError::validation("Comment not found"))?;

        comment.update_content(new_content);
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// Remove a comment
    pub fn remove_comment(&mut self, comment_id: Uuid) -> Result<Comment> {
        let position = self
            .comments
            .iter()
            .position(|c| c.id == comment_id)
            .ok_or_else(|| OrchestratorError::validation("Comment not found"))?;

        let removed_comment = self.comments.remove(position);
        self.updated_at = Some(Utc::now());
        Ok(removed_comment)
    }

    // ===== PULL REQUEST MANAGEMENT =====

    /// Create a pull request for this task
    pub fn create_pull_request(
        &mut self,
        title: impl Into<String>,
        description: impl Into<String>,
        source_branch: impl Into<String>,
        target_branch: impl Into<String>,
        author: impl Into<String>,
    ) -> Result<()> {
        if self.code_status != CodeStatus::Coded {
            return Err(OrchestratorError::validation(
                "Can only create pull request when task is coded",
            ));
        }

        if self.pull_request.is_some() {
            return Err(OrchestratorError::validation(
                "Task already has a pull request",
            ));
        }

        let pr = PullRequest::new(title, description, source_branch, target_branch, author);
        self.pull_request = Some(pr);
        self.transition_code_status(CodeStatus::PullRequest)?;

        Ok(())
    }

    /// Get the pull request if it exists
    pub fn get_pull_request(&self) -> Option<&PullRequest> {
        self.pull_request.as_ref()
    }

    /// Get mutable pull request if it exists
    pub fn get_pull_request_mut(&mut self) -> Option<&mut PullRequest> {
        self.pull_request.as_mut()
    }

    /// Add a comment to the pull request
    pub fn add_pr_comment(
        &mut self,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<()> {
        let pr = self
            .pull_request
            .as_mut()
            .ok_or_else(|| OrchestratorError::validation("Task has no pull request"))?;

        pr.add_comment(author, content);
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// Handle CI result for the pull request
    pub fn handle_pr_ci_result(&mut self, success: bool) -> Result<()> {
        let pr = self
            .pull_request
            .as_mut()
            .ok_or_else(|| OrchestratorError::validation("Task has no pull request"))?;

        pr.handle_ci_result(success)?;

        // Sync the task's code status with the PR's code status
        self.code_status = pr.code_status;
        self.ci_attemps = pr.ci_attemps;
        self.updated_at = Some(Utc::now());

        Ok(())
    }

    /// Sync task code status with pull request status
    pub fn sync_with_pull_request(&mut self) -> Result<()> {
        let pr = self
            .pull_request
            .as_ref()
            .ok_or_else(|| OrchestratorError::validation("Task has no pull request"))?;

        self.code_status = pr.code_status;
        self.ci_attemps = pr.ci_attemps;
        self.updated_at = Some(Utc::now());

        Ok(())
    }

    /// Get all pull request comments
    pub fn get_pr_comments(&self) -> Vec<&Comment> {
        if let Some(pr) = &self.pull_request {
            pr.comments.iter().collect()
        } else {
            Vec::new()
        }
    }

    /// Get comments by type (includes both task and PR comments)
    pub fn get_comments_by_type(&self, comment_type: CommentType) -> Vec<&Comment> {
        let mut comments = self
            .comments
            .iter()
            .filter(|c| c.comment_type == comment_type)
            .collect::<Vec<_>>();

        if comment_type == CommentType::PullRequest {
            if let Some(pr) = &self.pull_request {
                comments.extend(pr.comments.iter());
            }
        }

        comments
    }

    /// Get all comments (task + PR comments)
    pub fn get_all_comments(&self) -> Vec<&Comment> {
        let mut comments: Vec<&Comment> = self.comments.iter().collect();

        if let Some(pr) = &self.pull_request {
            comments.extend(pr.comments.iter());
        }

        comments
    }

    /// Assign task to an agent
    pub fn assign_to(&mut self, agent: Agent) -> Result<()> {
        if self.status.is_terminal() {
            return Err(OrchestratorError::validation(
                "Cannot assign completed or cancelled tasks",
            ));
        }

        self.assigned_to = Some(agent);
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// Assign task to an agent by name (convenience method)
    pub fn assign_to_by_name(&mut self, agent_name: impl Into<String>) -> Result<()> {
        if self.status.is_terminal() {
            return Err(OrchestratorError::validation(
                "Cannot assign completed or cancelled tasks",
            ));
        }

        // Create a minimal agent with just the name for assignment
        let agent = Agent::new(
            &agent_name.into(),
            std::path::PathBuf::from("/tmp/placeholder.json"),
            "Assigned agent",
        );

        self.assigned_to = Some(agent);
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// Check if task is overdue
    pub fn is_overdue(&self) -> bool {
        match self.due_date {
            Some(due) => Utc::now() > due && !self.status.is_terminal(),
            None => false,
        }
    }

    /// Get task duration in minutes
    pub fn get_duration_minutes(&self) -> Option<i64> {
        match (self.created_at, self.completed_at) {
            (Some(start), Some(end)) => Some((end - start).num_minutes()),
            _ => None,
        }
    }

    /// Get time in current status
    pub fn time_in_current_status(&self) -> chrono::Duration {
        if let Some((_, timestamp)) = self.status_history.last() {
            Utc::now() - *timestamp
        } else {
            chrono::Duration::zero()
        }
    }

    /// Get a summary of the task
    pub fn summary(&self) -> TaskSummary {
        TaskSummary {
            id: self.id,
            title: self.title.clone(),
            status: self.status.clone(),
            code_status: self.code_status,
            assigned_to: self.assigned_to.as_ref().map(|agent| agent.name.clone()),
            priority: self.priority.clone(),
            is_overdue: self.is_overdue(),
            ci_attemps: self.ci_attemps,
            comment_count: self.comments.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_comment_management() {
        let mut task = Task::new("Test Task", "Description", Priority::Medium);

        // Add regular comment
        task.add_comment("user1", "Regular comment");
        assert_eq!(task.comments.len(), 1);
        assert_eq!(task.comments[0].comment_type, CommentType::Task);
        assert!(task.comments[0].needs_sync());

        // Add synced comment
        task.add_comment_with_sync("user2", "Synced comment", true);
        assert_eq!(task.comments.len(), 2);
        assert!(!task.comments[1].needs_sync());

        // Test unsynced comments
        let unsynced = task.get_unsynced_comments();
        assert_eq!(unsynced.len(), 1);

        // Mark all as synced
        task.mark_all_comments_synced();
        assert_eq!(task.get_unsynced_comments().len(), 0);
    }

    #[test]
    fn test_task_pull_request_management() {
        let mut task = Task::new("Test Task", "Description", Priority::Medium);

        // Should fail to add PR comment when no PR exists
        assert!(task.add_pr_comment("user1", "PR comment").is_err());

        // Assign task to an agent first
        use crate::models::agent::Agent;
        let agent = Agent::new(
            "Test Agent",
            std::path::PathBuf::from("/tmp/test.json"),
            "Test agent",
        );
        task.assigned_to = Some(agent);

        // Transition task to InProgress first, then to coded
        task.transition_task_status(TaskStatus::InProgress).unwrap();
        task.transition_code_status(CodeStatus::Coded).unwrap();

        // Create a pull request
        assert!(
            task.create_pull_request(
                "Fix: Test Task",
                "This fixes the test task",
                "feature/test-task",
                "main",
                "developer"
            )
            .is_ok()
        );

        // Should now have a pull request and be in PR status
        assert!(task.pull_request.is_some());
        assert_eq!(task.code_status, CodeStatus::PullRequest);

        // Now should be able to add PR comment
        assert!(task.add_pr_comment("user1", "PR comment").is_ok());
        let pr_comments = task.get_pr_comments();
        assert_eq!(pr_comments.len(), 1);
        assert_eq!(pr_comments[0].comment_type, CommentType::PullRequest);

        // Test CI handling
        assert!(task.handle_pr_ci_result(false).is_ok());
        assert_eq!(task.code_status, CodeStatus::CIFailed);
        assert_eq!(task.ci_attemps, 1);

        // Reset PR to PullRequest status for next CI test (go through Coded first)
        if let Some(pr) = task.get_pull_request_mut() {
            pr.transition_code_status(CodeStatus::Coded).unwrap();
            pr.transition_code_status(CodeStatus::PullRequest).unwrap();
        }
        task.sync_with_pull_request().unwrap();

        // Test successful CI
        assert!(task.handle_pr_ci_result(true).is_ok());
        assert_eq!(task.code_status, CodeStatus::CISuccessful);
        assert_eq!(task.ci_attemps, 2);
    }

    #[test]
    fn test_task_comment_operations() {
        let mut task = Task::new("Test Task", "Description", Priority::Medium);

        task.add_comment("user1", "Original comment");
        let comment_id = task.comments[0].id;

        // Test comment retrieval
        assert!(task.get_comment(comment_id).is_some());
        assert!(task.get_comment_mut(comment_id).is_some());

        // Test comment update
        assert!(task.update_comment(comment_id, "Updated comment").is_ok());
        assert_eq!(
            task.get_comment(comment_id).unwrap().content,
            "Updated comment"
        );

        // Test comment removal
        let removed = task.remove_comment(comment_id).unwrap();
        assert_eq!(removed.content, "Updated comment");
        assert_eq!(task.comments.len(), 0);

        // Test operations on non-existent comment
        let fake_id = Uuid::new_v4();
        assert!(task.update_comment(fake_id, "test").is_err());
        assert!(task.remove_comment(fake_id).is_err());
    }
}
