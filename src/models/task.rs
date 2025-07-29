use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::comment::Comment;
use crate::enums::{TaskStatus, CodeStatus, Priority};
use crate::error::{Result, OrchestratorError};



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
    pub rapporter: Option<String>,
    pub assigned_to: Option<String>,
    pub priority: Priority,
    pub estimated_complexity: Option<u8>, // 1-10 scale
    pub estimated_duration: Option<u32>, // in minutes
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub comments: Vec<Comment>,
    pub ci_attemps: u32,
    pub depends_on: Vec<Uuid>,
}

impl Task {
    /// Create a new task
    pub fn new(title: impl Into<String>, description: impl Into<String>, priority: Priority) -> Self {
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
                    "Task must be assigned before starting"
                ));
            }
            // Can only complete if code is merged
            (TaskStatus::Testing, TaskStatus::Completed) if self.code_status != CodeStatus::Merged => {
                return Err(OrchestratorError::validation(
                    "Code must be merged before completing task"
                ));
            }
            _ => {}
        }
        
        // Update status and history
        let now = Utc::now();
        self.status = new_status.clone();
        self.status_history.push((new_status, now));
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
                    "Task must be in progress before coding"
                ));
            }
            // Track CI attempts
            (CodeStatus::PullRequest, CodeStatus::CIFailed | CodeStatus::CISuccessful) => {
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
                "CI can only run on pull requests"
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
        let comment = Comment {
            id: Uuid::new_v4(),
            author: author.into(),
            content: content.into(),
            created_at: Utc::now(),
        };
        self.comments.push(comment);
        self.updated_at = Some(Utc::now());
    }
    
    /// Assign task to a user
    pub fn assign_to(&mut self, user: impl Into<String>) -> Result<()> {
        if self.status.is_terminal() {
            return Err(OrchestratorError::validation(
                "Cannot assign completed or cancelled tasks"
            ));
        }
        
        self.assigned_to = Some(user.into());
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
            assigned_to: self.assigned_to.clone(),
            priority: self.priority,
            is_overdue: self.is_overdue(),
            ci_attemps: self.ci_attemps,
            comment_count: self.comments.len(),
        }
    }
}