use crate::enums::{IssueStatus, IssueType};
use crate::models::task::Task;
use chrono::{DateTime, Utc, TimeDelta};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::comment::Comment;
use crate::models::issue_status_change::StatusChange;
use crate::error::{Result, OrchestratorError};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: Uuid,
    pub github_issue_number: Option<u64>,
    pub task_id: Uuid,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub assignee: Option<String>,
    pub branch_name: Option<String>,
    pub issue_type: Option<IssueType>,
    pub status: IssueStatus,
    pub status_history: Vec<StatusChange>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub reopened_count: u32,
    pub comments: Vec<Comment>,
}


impl Issue {
    /// Create a new issue from a task
    pub fn from_task(task: &Task) -> Self {
        let now = Utc::now();
        let initial_status = IssueStatus::default();
        
        Self {
            id: Uuid::new_v4(),
            github_issue_number: None,
            task_id: task.id,
            title: task.title.clone(),
            body: task.description.clone(),
            labels: vec![
                format!("priority:{:?}", task.priority).to_lowercase()
            ],
            assignee: task.assigned_to.as_ref().map(|agent| agent.name.clone()),
            branch_name: None,
            issue_type: None,
            status: initial_status.clone(),
            status_history: vec![StatusChange {
                from: None,
                to: initial_status,
                changed_by: task.rapporter.as_ref().map(|agent| agent.name.clone()).unwrap_or_else(|| "system".to_string()),
                reason: Some("Issue created from task".to_string()),
                timestamp: now,
            }],
            created_at: now,
            updated_at: now,
            closed_at: None,
            reopened_count: 0,
            comments: Vec::new(),
        }
    }
    
    /// Generate a branch name from the issue title
    pub fn generate_branch_name(&self) -> String {
        let sanitized_title = self
            .title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
            
        // Include issue number if available
        match self.github_issue_number {
            Some(num) => format!("feature/{}-{}", num, sanitized_title),
            None => format!("feature/{}", sanitized_title),
        }
    }
    
    /// Transition the issue to a new status with validation
    pub fn transition_to(
        &mut self, 
        new_status: IssueStatus, 
        changed_by: impl Into<String>, 
        reason: Option<String>
    ) -> Result<()> {
        // Validate the transition using the state machine
        let next_status = self.status.transition_to(new_status)?;
        let now = Utc::now();
        
        // Apply business rules
        match (&self.status, &next_status) {
            // Can only start work if assigned
            (IssueStatus::Open, IssueStatus::InProgress) if self.assignee.is_none() => {
                return Err(OrchestratorError::validation(
                    "Issue must be assigned before starting work"
                ));
            }
            // Generate branch name when starting work
            (IssueStatus::Open, IssueStatus::InProgress) => {
                if self.branch_name.is_none() {
                    self.branch_name = Some(self.generate_branch_name());
                }
            }
            _ => {}
        }
        
        // Track reopening
        if self.status == IssueStatus::Closed && next_status == IssueStatus::Open {
            self.reopened_count += 1;
            self.closed_at = None;
        }
        
        // Track closing
        if next_status == IssueStatus::Closed {
            self.closed_at = Some(now);
        }
        
        // Record the change
        self.status_history.push(StatusChange {
            from: Some(self.status.clone()),
            to: next_status.clone(),
            changed_by: changed_by.into(),
            reason,
            timestamp: now,
        });
        
        self.status = next_status;
        self.updated_at = now;
        
        Ok(())
    }
    
    /// Progress the issue to the next state in the workflow
    pub fn progress(&mut self, changed_by: impl Into<String>, reason: Option<String>) -> Result<()> {
        let next_status = self.status.progress()?;
        self.transition_to(next_status, changed_by, reason)
    }
    
    /// Assign the issue to someone
    pub fn assign_to(&mut self, assignee: impl Into<String>) -> Result<()> {
        if self.status == IssueStatus::Closed {
            return Err(OrchestratorError::validation(
                "Cannot assign closed issues"
            ));
        }
        
        self.assignee = Some(assignee.into());
        self.updated_at = Utc::now();
        Ok(())
    }
    
    /// Unassign the issue
    pub fn unassign(&mut self) {
        self.assignee = None;
        self.updated_at = Utc::now();
    }
    
    /// Start work on the issue (convenience method)
    pub fn start_work(&mut self, assignee: impl Into<String>) -> Result<()> {
        if self.status != IssueStatus::Open {
            return Err(OrchestratorError::issue_transition(
                self.status.clone(),
                IssueStatus::InProgress,
            ));
        }
        
        let assignee_str = assignee.into();
        self.assign_to(assignee_str.clone())?;
        self.transition_to(IssueStatus::InProgress, assignee_str, Some("Started work".to_string()))
    }
    
    /// Submit for review (convenience method)
    pub fn submit_for_review(&mut self, submitter: impl Into<String>) -> Result<()> {
        if self.status != IssueStatus::InProgress {
            return Err(OrchestratorError::issue_transition(
                self.status.clone(),
                IssueStatus::InReview,
            ));
        }
        
        self.transition_to(IssueStatus::InReview, submitter, Some("Submitted for review".to_string()))
    }
    
    /// Request changes (send back to InProgress)
    pub fn request_changes(&mut self, reviewer: impl Into<String>, reason: impl Into<String>) -> Result<()> {
        if self.status != IssueStatus::InReview {
            return Err(OrchestratorError::issue_transition(
                self.status.clone(),
                IssueStatus::InProgress,
            ));
        }
        
        self.transition_to(IssueStatus::InProgress, reviewer, Some(reason.into()))
    }
    
    /// Close the issue
    pub fn close(&mut self, closer: impl Into<String>, reason: impl Into<String>) -> Result<()> {
        self.transition_to(IssueStatus::Closed, closer, Some(reason.into()))
    }
    
    /// Reopen the issue
    pub fn reopen(&mut self, opener: impl Into<String>, reason: impl Into<String>) -> Result<()> {
        if self.status != IssueStatus::Closed {
            return Err(OrchestratorError::issue_transition(
                self.status.clone(),
                IssueStatus::Open,
            ));
        }
        
        self.transition_to(IssueStatus::Open, opener, Some(reason.into()))
    }
    
    /// Add a comment to the issue
    pub fn add_comment(&mut self, comment: Comment) {
        self.comments.push(comment);
        self.updated_at = Utc::now();
    }
    
    /// Update GitHub issue number
    pub fn set_github_issue_number(&mut self, issue_number: u64) {
        self.github_issue_number = Some(issue_number);
        self.updated_at = Utc::now();
    }
    
    /// Add a label to the issue
    pub fn add_label(&mut self, label: impl Into<String>) {
        let label = label.into();
        if !self.labels.contains(&label) {
            self.labels.push(label);
            self.updated_at = Utc::now();
        }
    }
    
    /// Remove a label from the issue
    pub fn remove_label(&mut self, label: &str) {
        self.labels.retain(|l| l != label);
        self.updated_at = Utc::now();
    }
    
    /// Set the issue type
    pub fn set_issue_type(&mut self, issue_type: IssueType) {
        self.issue_type = Some(issue_type);
        self.updated_at = Utc::now();
    }
    
    /// Get the total time the issue has been in a specific status
    pub fn time_in_status(&self, status: IssueStatus) -> TimeDelta {
        let mut total = TimeDelta::zero();
        let mut in_status = false;
        let mut start_time = None;
        
        for change in &self.status_history {
            if change.to == status {
                in_status = true;
                start_time = Some(change.timestamp);
            } else if in_status && change.from == Some(status.clone()) {
                if let Some(start) = start_time {
                    total = total + (change.timestamp - start);
                }
                in_status = false;
                start_time = None;
            }
        }
        
        // If still in the status, add time until now
        if in_status {
            if let Some(start) = start_time {
                total = total + (Utc::now() - start);
            }
        }
        
        total
    }
    
    /// Get the cycle time (time from InProgress to Closed)
    pub fn cycle_time(&self) -> Option<TimeDelta> {
        if self.status != IssueStatus::Closed {
            return None;
        }
        
        let first_in_progress = self.status_history.iter()
            .find(|change| change.to == IssueStatus::InProgress)
            .map(|change| change.timestamp)?;
            
        let closed_at = self.closed_at?;
        
        Some(closed_at - first_in_progress)
    }
    
    /// Check if the issue is active (not closed)
    pub fn is_active(&self) -> bool {
        self.status != IssueStatus::Closed
    }
    
    /// Get the age of the issue
    pub fn age(&self) -> TimeDelta {
        Utc::now() - self.created_at
    }
}