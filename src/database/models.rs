use crate::models::project::{Project, ProjectStatus};
use crate::models::task::Task;
use crate::models::agent::Agent;
use crate::models::issue::Issue;
use crate::models::comment::Comment;
use crate::models::pull_request::PullRequest;
use crate::models::code_review::CodeReview;
use crate::enums::*;
use crate::error::{OrchestratorError, Result};
use chrono::{DateTime, Utc};
use rusqlite::Row;
use serde_json;
use uuid::Uuid;

/// Database representation of a Project
#[derive(Debug, Clone)]
pub struct DbProject {
    pub id: String,
    pub idea: String,
    pub name: String,
    pub repository_url: Option<String>,
    pub project_path: String,
    pub status: String,
    pub tech_stack: String,
    pub created_at: String,
    pub updated_at: String,
}

impl DbProject {
    /// Convert from domain Project to database representation
    pub fn from_project(project: &Project) -> Self {
        Self {
            id: project.id.to_string(),
            idea: project.idea.clone(),
            name: project.name.clone(),
            repository_url: project.repository_url.clone(),
            project_path: project.project_path.clone(),
            status: serde_json::to_string(&project.status).unwrap_or_default(),
            tech_stack: serde_json::to_string(&project.tech_stack).unwrap_or_default(),
            created_at: project.created_at.to_rfc3339(),
            updated_at: project.updated_at.to_rfc3339(),
        }
    }

    /// Convert from database row to DbProject
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            idea: row.get("idea")?,
            name: row.get("name")?,
            repository_url: row.get("repository_url")?,
            project_path: row.get("project_path")?,
            status: row.get("status")?,
            tech_stack: row.get("tech_stack")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }

    /// Convert to domain Project (without collections)
    pub fn to_project(&self) -> Result<Project> {
        let id = Uuid::parse_str(&self.id)
            .map_err(|e| OrchestratorError::validation(format!("Invalid project ID: {}", e)))?;
        
        let status: ProjectStatus = serde_json::from_str(&self.status)
            .map_err(|e| OrchestratorError::json_parsing("project status", e))?;
        
        let tech_stack: TechStack = serde_json::from_str(&self.tech_stack)
            .map_err(|e| OrchestratorError::json_parsing("tech stack", e))?;
        
        let created_at = DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(|e| OrchestratorError::validation(format!("Invalid created_at date: {}", e)))?
            .with_timezone(&Utc);
        
        let updated_at = DateTime::parse_from_rfc3339(&self.updated_at)
            .map_err(|e| OrchestratorError::validation(format!("Invalid updated_at date: {}", e)))?
            .with_timezone(&Utc);

        Ok(Project {
            id,
            idea: self.idea.clone(),
            name: self.name.clone(),
            repository_url: self.repository_url.clone(),
            project_path: self.project_path.clone(),
            status,
            tech_stack,
            tasks: Vec::new(), // Will be loaded separately
            issues: Vec::new(), // Will be loaded separately
            agents: Vec::new(), // Will be loaded separately
            tasks_history: Vec::new(), // Will be loaded separately
            created_at,
            updated_at,
            dependencies_urls: None, // Will be loaded separately
        })
    }
}

/// Database representation of a Task
#[derive(Debug, Clone)]
pub struct DbTask {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub estimated_complexity: Option<u8>,
    pub estimated_duration: Option<u32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub completed_at: Option<String>,
    pub due_date: Option<String>,
    pub rapporter_id: Option<String>,
    pub assigned_to_id: Option<String>,
    pub pull_request_id: Option<String>,
}

impl DbTask {
    /// Convert from domain Task to database representation
    pub fn from_task(task: &Task, project_id: &Uuid) -> Self {
        Self {
            id: task.id.to_string(),
            project_id: project_id.to_string(),
            title: task.title.clone(),
            description: task.description.clone(),
            status: serde_json::to_string(&task.status).unwrap_or_default(),
            priority: serde_json::to_string(&task.priority).unwrap_or_default(),
            estimated_complexity: task.estimated_complexity,
            estimated_duration: task.estimated_duration,
            created_at: task.created_at.map(|dt| dt.to_rfc3339()),
            updated_at: task.updated_at.map(|dt| dt.to_rfc3339()),
            completed_at: task.completed_at.map(|dt| dt.to_rfc3339()),
            due_date: task.due_date.map(|dt| dt.to_rfc3339()),
            rapporter_id: task.rapporter.as_ref().map(|a| a.id.to_string()),
            assigned_to_id: task.assigned_to.as_ref().map(|a| a.id.to_string()),
            pull_request_id: task.pull_request.as_ref().map(|pr| pr.id.to_string()),
        }
    }

    /// Convert from database row to DbTask
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            status: row.get("status")?,
            priority: row.get("priority")?,
            estimated_complexity: row.get("estimated_complexity")?,
            estimated_duration: row.get("estimated_duration")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            completed_at: row.get("completed_at")?,
            due_date: row.get("due_date")?,
            rapporter_id: row.get("rapporter_id")?,
            assigned_to_id: row.get("assigned_to_id")?,
            pull_request_id: row.get("pull_request_id")?,
        })
    }
}

/// Database representation of an Agent
#[derive(Debug, Clone)]
pub struct DbAgent {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub file_path: String,
    pub description: String,
    pub agent_type: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_active_at: Option<String>,
    pub error_count: u32,
    pub total_tasks_completed: u64,
    pub recovery_attempts: u32,
    pub last_error_recovery_at: Option<String>,
    pub autonomous_recovery_enabled: bool,
    pub max_recovery_attempts: u32,
}

impl DbAgent {
    /// Convert from domain Agent to database representation
    pub fn from_agent(agent: &Agent, project_id: &Uuid) -> Self {
        Self {
            id: agent.id.to_string(),
            project_id: project_id.to_string(),
            name: agent.name.clone(),
            file_path: agent.file_path.to_string_lossy().to_string(),
            description: agent.description.clone(),
            agent_type: serde_json::to_string(&agent.agent_type).unwrap_or_default(),
            status: serde_json::to_string(&agent.status).unwrap_or_default(),
            created_at: agent.created_at.to_rfc3339(),
            updated_at: agent.updated_at.to_rfc3339(),
            last_active_at: agent.last_active_at.map(|dt| dt.to_rfc3339()),
            error_count: agent.error_count,
            total_tasks_completed: agent.total_tasks_completed,
            recovery_attempts: agent.recovery_attempts,
            last_error_recovery_at: agent.last_error_recovery_at.map(|dt| dt.to_rfc3339()),
            autonomous_recovery_enabled: agent.autonomous_recovery_enabled,
            max_recovery_attempts: agent.max_recovery_attempts,
        }
    }

    /// Convert from database row to DbAgent
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            name: row.get("name")?,
            file_path: row.get("file_path")?,
            description: row.get("description")?,
            agent_type: row.get("agent_type")?,
            status: row.get("status")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            last_active_at: row.get("last_active_at")?,
            error_count: row.get("error_count")?,
            total_tasks_completed: row.get("total_tasks_completed")?,
            recovery_attempts: row.get("recovery_attempts")?,
            last_error_recovery_at: row.get("last_error_recovery_at")?,
            autonomous_recovery_enabled: row.get("autonomous_recovery_enabled")?,
            max_recovery_attempts: row.get("max_recovery_attempts")?,
        })
    }
}

/// Database representation of an Issue
#[derive(Debug, Clone)]
pub struct DbIssue {
    pub id: String,
    pub project_id: String,
    pub task_id: String,
    pub github_issue_number: Option<u64>,
    pub title: String,
    pub body: String,
    pub assignee: Option<String>,
    pub branch_name: Option<String>,
    pub issue_type: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub reopened_count: u32,
    pub remotly_synced: bool,
}

impl DbIssue {
    /// Convert from domain Issue to database representation
    pub fn from_issue(issue: &Issue, project_id: &Uuid) -> Self {
        Self {
            id: issue.id.to_string(),
            project_id: project_id.to_string(),
            task_id: issue.task_id.to_string(),
            github_issue_number: issue.github_issue_number,
            title: issue.title.clone(),
            body: issue.body.clone(),
            assignee: issue.assignee.clone(),
            branch_name: issue.branch_name.clone(),
            issue_type: issue.issue_type.as_ref().map(|t| serde_json::to_string(t).unwrap_or_default()),
            status: serde_json::to_string(&issue.status).unwrap_or_default(),
            created_at: issue.created_at.to_rfc3339(),
            updated_at: issue.updated_at.to_rfc3339(),
            closed_at: issue.closed_at.map(|dt| dt.to_rfc3339()),
            reopened_count: issue.reopened_count,
            remotly_synced: issue.remotly_synced,
        }
    }

    /// Convert from database row to DbIssue
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            task_id: row.get("task_id")?,
            github_issue_number: row.get("github_issue_number")?,
            title: row.get("title")?,
            body: row.get("body")?,
            assignee: row.get("assignee")?,
            branch_name: row.get("branch_name")?,
            issue_type: row.get("issue_type")?,
            status: row.get("status")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            closed_at: row.get("closed_at")?,
            reopened_count: row.get("reopened_count")?,
            remotly_synced: row.get("remotly_synced")?,
        })
    }
}

/// Database representation of a Comment
#[derive(Debug, Clone)]
pub struct DbComment {
    pub id: String,
    pub content: String,
    pub author: String,
    pub comment_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub remotly_synced: bool,
    pub task_id: Option<String>,
    pub issue_id: Option<String>,
    pub pull_request_id: Option<String>,
}

impl DbComment {
    /// Convert from domain Comment to database representation
    pub fn from_comment(comment: &Comment, parent_id: &str, parent_type: &str) -> Self {
        let (task_id, issue_id, pull_request_id) = match parent_type {
            "task" => (Some(parent_id.to_string()), None, None),
            "issue" => (None, Some(parent_id.to_string()), None),
            "pull_request" => (None, None, Some(parent_id.to_string())),
            _ => (None, None, None),
        };

        Self {
            id: comment.id.to_string(),
            content: comment.content.clone(),
            author: comment.author.clone(),
            comment_type: serde_json::to_string(&comment.comment_type).unwrap_or_default(),
            created_at: comment.created_at.to_rfc3339(),
            updated_at: comment.updated_at.to_rfc3339(),
            remotly_synced: comment.remotly_synced,
            task_id,
            issue_id,
            pull_request_id,
        }
    }

    /// Convert from database row to DbComment
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            content: row.get("content")?,
            author: row.get("author")?,
            comment_type: row.get("comment_type")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            remotly_synced: row.get("remotly_synced")?,
            task_id: row.get("task_id")?,
            issue_id: row.get("issue_id")?,
            pull_request_id: row.get("pull_request_id")?,
        })
    }
}

/// Database representation of a Pull Request
#[derive(Debug, Clone)]
pub struct DbPullRequest {
    pub id: String,
    pub github_pr_number: Option<u64>,
    pub title: String,
    pub description: String,
    pub source_branch: String,
    pub target_branch: String,
    pub author: String,
    pub code_status: String,
    pub ci_attemps: u32,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
    pub remotly_synced: bool,
}

impl DbPullRequest {
    /// Convert from domain PullRequest to database representation
    pub fn from_pull_request(pr: &PullRequest) -> Self {
        Self {
            id: pr.id.to_string(),
            github_pr_number: pr.github_pr_number,
            title: pr.title.clone(),
            description: pr.description.clone(),
            source_branch: pr.source_branch.clone(),
            target_branch: pr.target_branch.clone(),
            author: pr.author.clone(),
            code_status: serde_json::to_string(&pr.code_status).unwrap_or_default(),
            ci_attemps: pr.ci_attemps,
            created_at: pr.created_at.to_rfc3339(),
            updated_at: pr.updated_at.to_rfc3339(),
            merged_at: pr.merged_at.map(|dt| dt.to_rfc3339()),
            closed_at: pr.closed_at.map(|dt| dt.to_rfc3339()),
            remotly_synced: pr.remotly_synced,
        }
    }

    /// Convert from database row to DbPullRequest
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            github_pr_number: row.get("github_pr_number")?,
            title: row.get("title")?,
            description: row.get("description")?,
            source_branch: row.get("source_branch")?,
            target_branch: row.get("target_branch")?,
            author: row.get("author")?,
            code_status: row.get("code_status")?,
            ci_attemps: row.get("ci_attemps")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            merged_at: row.get("merged_at")?,
            closed_at: row.get("closed_at")?,
            remotly_synced: row.get("remotly_synced")?,
        })
    }
}

/// Database representation of a Code Review
#[derive(Debug, Clone)]
pub struct DbCodeReview {
    pub id: String,
    pub pull_request_id: String,
    pub reviewer: String,
    pub approved: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl DbCodeReview {
    /// Convert from domain CodeReview to database representation
    pub fn from_code_review(review: &CodeReview) -> Self {
        Self {
            id: review.id.to_string(),
            pull_request_id: review.pull_request_id.clone(),
            reviewer: review.reviewer.clone(),
            approved: review.approved,
            created_at: review.created_at.to_rfc3339(),
            updated_at: review.updated_at.to_rfc3339(),
        }
    }

    /// Convert from database row to DbCodeReview
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            pull_request_id: row.get("pull_request_id")?,
            reviewer: row.get("reviewer")?,
            approved: row.get("approved")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}
