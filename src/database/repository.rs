use crate::database::{Database, models::*};
use crate::models::project::Project;
use crate::models::task::Task;
use crate::models::agent::Agent;
use crate::models::issue::Issue;
use crate::models::comment::Comment;
use crate::models::pull_request::PullRequest;
use crate::enums::*;
use crate::error::{OrchestratorError, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde_json;
use std::path::PathBuf;
use uuid::Uuid;

/// Repository for database operations
pub struct ProjectRepository {
    db: Database,
}

impl ProjectRepository {
    /// Create a new repository
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Save a complete project to the database
    pub fn save_project(&self, project: &Project) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;

        // Start transaction
        let tx = conn.unchecked_transaction()
            .map_err(|e| OrchestratorError::database(format!("Failed to start transaction: {}", e)))?;

        // Save project
        self.save_project_data(&tx, project)?;

        // Save agents
        for agent in &project.agents {
            self.save_agent_data(&tx, agent, &project.id)?;
        }

        // Save tasks
        for task in &project.tasks {
            self.save_task_data(&tx, task, &project.id)?;
        }

        // Save issues
        for issue in &project.issues {
            self.save_issue_data(&tx, issue, &project.id)?;
        }

        // Save project dependencies
        if let Some(deps) = &project.dependencies_urls {
            self.save_project_dependencies(&tx, &project.id, deps)?;
        }

        // Save tasks history
        for (task, timestamp) in &project.tasks_history {
            self.save_task_history(&tx, &project.id, task, timestamp)?;
        }

        // Commit transaction
        tx.commit()
            .map_err(|e| OrchestratorError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    /// Load a complete project from the database
    pub fn load_project(&self, project_path: &str) -> Result<Project> {
        let conn = self.db.get_connection();
        let conn = conn.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;

        // Load project
        let mut project = self.load_project_data(&conn, project_path)?;

        // Load agents
        project.agents = self.load_agents(&conn, &project.id)?;

        // Load tasks
        project.tasks = self.load_tasks(&conn, &project.id)?;

        // Load issues
        project.issues = self.load_issues(&conn, &project.id)?;

        // Load project dependencies
        project.dependencies_urls = self.load_project_dependencies(&conn, &project.id)?;

        // Load tasks history
        project.tasks_history = self.load_tasks_history(&conn, &project.id)?;

        Ok(project)
    }

    /// Check if a project exists
    pub fn project_exists(&self, project_path: &str) -> Result<bool> {
        let conn = self.db.get_connection();
        let conn = conn.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;

        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM projects WHERE project_path = ?1",
            params![project_path],
            |row| row.get(0)
        ).map_err(|e| OrchestratorError::database(format!("Failed to check project existence: {}", e)))?;

        Ok(count > 0)
    }

    /// List all projects
    pub fn list_projects(&self) -> Result<Vec<(String, Project)>> {
        let conn = self.db.get_connection();
        let conn = conn.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, idea, name, repository_url, project_path, status, tech_stack, created_at, updated_at 
             FROM projects ORDER BY created_at DESC"
        ).map_err(|e| OrchestratorError::database(format!("Failed to prepare statement: {}", e)))?;

        let project_iter = stmt.query_map([], |row| {
            DbProject::from_row(row)
        }).map_err(|e| OrchestratorError::database(format!("Failed to query projects: {}", e)))?;

        let mut projects = Vec::new();
        for project_result in project_iter {
            let db_project = project_result
                .map_err(|e| OrchestratorError::database(format!("Failed to parse project row: {}", e)))?;
            
            let project = db_project.to_project()?;
            projects.push((project.project_path.clone(), project));
        }

        Ok(projects)
    }

    /// Delete a project and all its related data
    pub fn delete_project(&self, project_path: &str) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;

        conn.execute(
            "DELETE FROM projects WHERE project_path = ?1",
            params![project_path]
        ).map_err(|e| OrchestratorError::database(format!("Failed to delete project: {}", e)))?;

        Ok(())
    }

    // Private helper methods

    /// Save project data to database
    fn save_project_data(&self, conn: &Connection, project: &Project) -> Result<()> {
        let db_project = DbProject::from_project(project);

        conn.execute(
            "INSERT OR REPLACE INTO projects 
             (id, idea, name, repository_url, project_path, status, tech_stack, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                db_project.id,
                db_project.idea,
                db_project.name,
                db_project.repository_url,
                db_project.project_path,
                db_project.status,
                db_project.tech_stack,
                db_project.created_at,
                db_project.updated_at
            ]
        ).map_err(|e| OrchestratorError::database(format!("Failed to save project: {}", e)))?;

        Ok(())
    }

    /// Load project data from database
    fn load_project_data(&self, conn: &Connection, project_path: &str) -> Result<Project> {
        let db_project = conn.query_row(
            "SELECT id, idea, name, repository_url, project_path, status, tech_stack, created_at, updated_at 
             FROM projects WHERE project_path = ?1",
            params![project_path],
            |row| DbProject::from_row(row)
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                OrchestratorError::validation(format!("Project not found at path: {}", project_path))
            }
            _ => OrchestratorError::database(format!("Failed to load project: {}", e))
        })?;

        db_project.to_project()
    }

    /// Save agent data to database
    fn save_agent_data(&self, conn: &Connection, agent: &Agent, project_id: &Uuid) -> Result<()> {
        let db_agent = DbAgent::from_agent(agent, project_id);

        conn.execute(
            "INSERT OR REPLACE INTO agents 
             (id, project_id, name, file_path, description, agent_type, status, created_at, updated_at,
              last_active_at, error_count, total_tasks_completed, recovery_attempts, last_error_recovery_at,
              autonomous_recovery_enabled, max_recovery_attempts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                db_agent.id,
                db_agent.project_id,
                db_agent.name,
                db_agent.file_path,
                db_agent.description,
                db_agent.agent_type,
                db_agent.status,
                db_agent.created_at,
                db_agent.updated_at,
                db_agent.last_active_at,
                db_agent.error_count,
                db_agent.total_tasks_completed,
                db_agent.recovery_attempts,
                db_agent.last_error_recovery_at,
                db_agent.autonomous_recovery_enabled,
                db_agent.max_recovery_attempts
            ]
        ).map_err(|e| OrchestratorError::database(format!("Failed to save agent: {}", e)))?;

        // Save agent status history
        for status_change in &agent.status_history {
            conn.execute(
                "INSERT OR REPLACE INTO agent_status_history 
                 (agent_id, from_status, to_status, reason, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    agent.id.to_string(),
                    status_change.from.as_ref().map(|s| serde_json::to_string(s).unwrap_or_default()),
                    serde_json::to_string(&status_change.to).unwrap_or_default(),
                    status_change.reason,
                    status_change.timestamp.to_rfc3339()
                ]
            ).map_err(|e| OrchestratorError::database(format!("Failed to save agent status history: {}", e)))?;
        }

        // Save agent errors
        for error in &agent.recent_errors {
            conn.execute(
                "INSERT OR REPLACE INTO agent_errors
                 (agent_id, error_type, error_message, context, timestamp, resolved)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    agent.id.to_string(),
                    error.action_type.clone(),
                    error.error_message.clone(),
                    error.action_description.clone(), // Using action_description as context
                    error.timestamp.to_rfc3339(),
                    false // Assuming errors in recent_errors are not resolved
                ]
            ).map_err(|e| OrchestratorError::database(format!("Failed to save agent error: {}", e)))?;
        }

        Ok(())
    }

    /// Load agents from database
    fn load_agents(&self, conn: &Connection, project_id: &Uuid) -> Result<Vec<Agent>> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, file_path, description, agent_type, status, created_at, updated_at,
                    last_active_at, error_count, total_tasks_completed, recovery_attempts, last_error_recovery_at,
                    autonomous_recovery_enabled, max_recovery_attempts
             FROM agents WHERE project_id = ?1"
        ).map_err(|e| OrchestratorError::database(format!("Failed to prepare agents query: {}", e)))?;

        let agent_iter = stmt.query_map(params![project_id.to_string()], |row| {
            DbAgent::from_row(row)
        }).map_err(|e| OrchestratorError::database(format!("Failed to query agents: {}", e)))?;

        let mut agents = Vec::new();
        for agent_result in agent_iter {
            let db_agent = agent_result
                .map_err(|e| OrchestratorError::database(format!("Failed to parse agent row: {}", e)))?;
            
            // Convert DbAgent to Agent (this would need implementation)
            // For now, we'll create a placeholder
            // TODO: Implement full conversion with status history and errors
            agents.push(self.db_agent_to_agent(&db_agent, conn)?);
        }

        Ok(agents)
    }

    /// Convert DbAgent to Agent with all related data
    fn db_agent_to_agent(&self, db_agent: &DbAgent, _conn: &Connection) -> Result<Agent> {
        // This is a placeholder implementation
        // TODO: Implement full conversion including status history and errors
        let id = Uuid::parse_str(&db_agent.id)
            .map_err(|e| OrchestratorError::validation(format!("Invalid agent ID: {}", e)))?;

        let agent_type: AgentType = serde_json::from_str(&db_agent.agent_type)
            .map_err(|e| OrchestratorError::json_parsing("agent type", e))?;

        let status: AgentStatus = serde_json::from_str(&db_agent.status)
            .map_err(|e| OrchestratorError::json_parsing("agent status", e))?;

        let created_at = DateTime::parse_from_rfc3339(&db_agent.created_at)
            .map_err(|e| OrchestratorError::validation(format!("Invalid created_at date: {}", e)))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&db_agent.updated_at)
            .map_err(|e| OrchestratorError::validation(format!("Invalid updated_at date: {}", e)))?
            .with_timezone(&Utc);

        let last_active_at = if let Some(date_str) = &db_agent.last_active_at {
            Some(DateTime::parse_from_rfc3339(date_str)
                .map_err(|e| OrchestratorError::validation(format!("Invalid last_active_at date: {}", e)))?
                .with_timezone(&Utc))
        } else {
            None
        };

        let last_error_recovery_at = if let Some(date_str) = &db_agent.last_error_recovery_at {
            Some(DateTime::parse_from_rfc3339(date_str)
                .map_err(|e| OrchestratorError::validation(format!("Invalid last_error_recovery_at date: {}", e)))?
                .with_timezone(&Utc))
        } else {
            None
        };

        // TODO: Load status history and recent errors from database

        Ok(Agent {
            id,
            name: db_agent.name.clone(),
            file_path: PathBuf::from(&db_agent.file_path),
            description: db_agent.description.clone(),
            agent_type,
            status,
            status_history: Vec::new(), // TODO: Load from database
            created_at,
            updated_at,
            last_active_at,
            error_count: db_agent.error_count,
            total_tasks_completed: db_agent.total_tasks_completed,
            recent_errors: Vec::new(), // TODO: Load from database
            recovery_attempts: db_agent.recovery_attempts,
            last_error_recovery_at,
            autonomous_recovery_enabled: db_agent.autonomous_recovery_enabled,
            max_recovery_attempts: db_agent.max_recovery_attempts,
        })
    }

    /// Save task data to database
    fn save_task_data(&self, conn: &Connection, task: &Task, project_id: &Uuid) -> Result<()> {
        let db_task = DbTask::from_task(task, project_id);

        conn.execute(
            "INSERT OR REPLACE INTO tasks
             (id, project_id, title, description, status, priority, estimated_complexity, estimated_duration,
              created_at, updated_at, completed_at, due_date, rapporter_id, assigned_to_id, pull_request_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                db_task.id,
                db_task.project_id,
                db_task.title,
                db_task.description,
                db_task.status,
                db_task.priority,
                db_task.estimated_complexity,
                db_task.estimated_duration,
                db_task.created_at,
                db_task.updated_at,
                db_task.completed_at,
                db_task.due_date,
                db_task.rapporter_id,
                db_task.assigned_to_id,
                db_task.pull_request_id
            ]
        ).map_err(|e| OrchestratorError::database(format!("Failed to save task: {}", e)))?;

        // Save task status history
        for (status, timestamp) in &task.status_history {
            conn.execute(
                "INSERT OR REPLACE INTO task_status_history (task_id, status, timestamp)
                 VALUES (?1, ?2, ?3)",
                params![
                    task.id.to_string(),
                    serde_json::to_string(status).unwrap_or_default(),
                    timestamp.to_rfc3339()
                ]
            ).map_err(|e| OrchestratorError::database(format!("Failed to save task status history: {}", e)))?;
        }

        // Save task tags
        for tag in &task.tags {
            conn.execute(
                "INSERT OR REPLACE INTO task_tags (task_id, tag) VALUES (?1, ?2)",
                params![task.id.to_string(), tag]
            ).map_err(|e| OrchestratorError::database(format!("Failed to save task tag: {}", e)))?;
        }

        // Save task dependencies
        for dep_id in &task.depends_on {
            conn.execute(
                "INSERT OR REPLACE INTO task_dependencies (task_id, depends_on_task_id) VALUES (?1, ?2)",
                params![task.id.to_string(), dep_id.to_string()]
            ).map_err(|e| OrchestratorError::database(format!("Failed to save task dependency: {}", e)))?;
        }

        // Save task comments
        for comment in &task.comments {
            self.save_comment_data(conn, comment, &task.id.to_string(), "task")?;
        }

        // Save pull request if exists
        if let Some(pr) = &task.pull_request {
            self.save_pull_request_data(conn, pr)?;
        }

        Ok(())
    }

    /// Load tasks from database
    fn load_tasks(&self, _conn: &Connection, _project_id: &Uuid) -> Result<Vec<Task>> {
        // TODO: Implement full task loading with all related data
        // This is a placeholder that returns empty vector
        Ok(Vec::new())
    }

    /// Save issue data to database
    fn save_issue_data(&self, conn: &Connection, issue: &Issue, project_id: &Uuid) -> Result<()> {
        let db_issue = DbIssue::from_issue(issue, project_id);

        conn.execute(
            "INSERT OR REPLACE INTO issues
             (id, project_id, task_id, github_issue_number, title, body, assignee, branch_name,
              issue_type, status, created_at, updated_at, closed_at, reopened_count, remotly_synced)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                db_issue.id,
                db_issue.project_id,
                db_issue.task_id,
                db_issue.github_issue_number,
                db_issue.title,
                db_issue.body,
                db_issue.assignee,
                db_issue.branch_name,
                db_issue.issue_type,
                db_issue.status,
                db_issue.created_at,
                db_issue.updated_at,
                db_issue.closed_at,
                db_issue.reopened_count,
                db_issue.remotly_synced
            ]
        ).map_err(|e| OrchestratorError::database(format!("Failed to save issue: {}", e)))?;

        // Save issue labels
        for label in &issue.labels {
            conn.execute(
                "INSERT OR REPLACE INTO issue_labels (issue_id, label) VALUES (?1, ?2)",
                params![issue.id.to_string(), label]
            ).map_err(|e| OrchestratorError::database(format!("Failed to save issue label: {}", e)))?;
        }

        // Save issue comments
        for comment in &issue.comments {
            self.save_comment_data(conn, comment, &issue.id.to_string(), "issue")?;
        }

        Ok(())
    }

    /// Load issues from database
    fn load_issues(&self, _conn: &Connection, _project_id: &Uuid) -> Result<Vec<Issue>> {
        // TODO: Implement full issue loading with all related data
        // This is a placeholder that returns empty vector
        Ok(Vec::new())
    }

    /// Save comment data to database
    fn save_comment_data(&self, conn: &Connection, comment: &Comment, parent_id: &str, parent_type: &str) -> Result<()> {
        let db_comment = DbComment::from_comment(comment, parent_id, parent_type);

        conn.execute(
            "INSERT OR REPLACE INTO comments
             (id, content, author, comment_type, created_at, updated_at, remotly_synced,
              task_id, issue_id, pull_request_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                db_comment.id,
                db_comment.content,
                db_comment.author,
                db_comment.comment_type,
                db_comment.created_at,
                db_comment.updated_at,
                db_comment.remotly_synced,
                db_comment.task_id,
                db_comment.issue_id,
                db_comment.pull_request_id
            ]
        ).map_err(|e| OrchestratorError::database(format!("Failed to save comment: {}", e)))?;

        Ok(())
    }

    /// Save pull request data to database
    fn save_pull_request_data(&self, conn: &Connection, pr: &PullRequest) -> Result<()> {
        let db_pr = DbPullRequest::from_pull_request(pr);

        conn.execute(
            "INSERT OR REPLACE INTO pull_requests
             (id, github_pr_number, title, description, source_branch, target_branch, author,
              code_status, ci_attemps, created_at, updated_at, merged_at, closed_at, remotly_synced)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                db_pr.id,
                db_pr.github_pr_number,
                db_pr.title,
                db_pr.description,
                db_pr.source_branch,
                db_pr.target_branch,
                db_pr.author,
                db_pr.code_status,
                db_pr.ci_attemps,
                db_pr.created_at,
                db_pr.updated_at,
                db_pr.merged_at,
                db_pr.closed_at,
                db_pr.remotly_synced
            ]
        ).map_err(|e| OrchestratorError::database(format!("Failed to save pull request: {}", e)))?;

        // Save PR assignees, reviewers, labels, comments, and code reviews
        // TODO: Implement these saves

        Ok(())
    }

    /// Save project dependencies
    fn save_project_dependencies(&self, conn: &Connection, project_id: &Uuid, deps: &[String]) -> Result<()> {
        // First delete existing dependencies
        conn.execute(
            "DELETE FROM project_dependencies WHERE project_id = ?1",
            params![project_id.to_string()]
        ).map_err(|e| OrchestratorError::database(format!("Failed to delete old dependencies: {}", e)))?;

        // Insert new dependencies
        for dep in deps {
            conn.execute(
                "INSERT INTO project_dependencies (project_id, dependency_url) VALUES (?1, ?2)",
                params![project_id.to_string(), dep]
            ).map_err(|e| OrchestratorError::database(format!("Failed to save dependency: {}", e)))?;
        }

        Ok(())
    }

    /// Load project dependencies
    fn load_project_dependencies(&self, conn: &Connection, project_id: &Uuid) -> Result<Option<Vec<String>>> {
        let mut stmt = conn.prepare(
            "SELECT dependency_url FROM project_dependencies WHERE project_id = ?1"
        ).map_err(|e| OrchestratorError::database(format!("Failed to prepare dependencies query: {}", e)))?;

        let dep_iter = stmt.query_map(params![project_id.to_string()], |row| {
            Ok(row.get::<_, String>("dependency_url")?)
        }).map_err(|e| OrchestratorError::database(format!("Failed to query dependencies: {}", e)))?;

        let mut deps = Vec::new();
        for dep_result in dep_iter {
            let dep = dep_result
                .map_err(|e| OrchestratorError::database(format!("Failed to parse dependency row: {}", e)))?;
            deps.push(dep);
        }

        if deps.is_empty() {
            Ok(None)
        } else {
            Ok(Some(deps))
        }
    }

    /// Save task history
    fn save_task_history(&self, conn: &Connection, project_id: &Uuid, task: &Task, timestamp: &DateTime<Utc>) -> Result<()> {
        let task_data = serde_json::to_string(task)
            .map_err(|e| OrchestratorError::json_parsing("task history", e))?;

        conn.execute(
            "INSERT INTO tasks_history (project_id, task_id, task_data, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![
                project_id.to_string(),
                task.id.to_string(),
                task_data,
                timestamp.to_rfc3339()
            ]
        ).map_err(|e| OrchestratorError::database(format!("Failed to save task history: {}", e)))?;

        Ok(())
    }

    /// Load tasks history
    fn load_tasks_history(&self, _conn: &Connection, _project_id: &Uuid) -> Result<Vec<(Task, DateTime<Utc>)>> {
        // TODO: Implement full task history loading
        // This is a placeholder that returns empty vector
        Ok(Vec::new())
    }
}
