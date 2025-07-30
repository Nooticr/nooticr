use crate::database::{Database, helpers::DatabaseHelpers, models::*};
use crate::models::project::Project;
use crate::error::Result;
use rusqlite::params;

/// Simplified repository using the new helper functions
pub struct SimplifiedProjectRepository {
    db: Database,
}

impl SimplifiedProjectRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Save a project using the simplified helper pattern
    pub fn save_project(&self, project: &Project) -> Result<()> {
        DatabaseHelpers::with_transaction(&self.db.get_connection(), |tx| {
            // Save main project data
            self.save_project_data(tx, project)?;
            
            // Save related entities using batch operations
            DatabaseHelpers::save_collection(
                tx,
                &project.agents,
                |conn, agent| self.save_agent_data(conn, agent, &project.id),
                "agents"
            )?;
            
            DatabaseHelpers::save_collection(
                tx,
                &project.tasks,
                |conn, task| self.save_task_data(conn, task, &project.id),
                "tasks"
            )?;
            
            DatabaseHelpers::save_collection(
                tx,
                &project.issues,
                |conn, issue| self.save_issue_data(conn, issue, &project.id),
                "issues"
            )?;
            
            Ok(())
        })
    }

    /// Load a project using simplified helper pattern
    pub fn load_project(&self, project_path: &str) -> Result<Project> {
        DatabaseHelpers::with_connection(&self.db.get_connection(), |conn| {
            // Load main project data
            let db_project = DatabaseHelpers::query_single_row(
                conn,
                "SELECT id, idea, name, repository_url, project_path, status, tech_stack, created_at, updated_at 
                 FROM projects WHERE project_path = ?1",
                &[&project_path],
                |row| DbProject::from_row(row),
                "project"
            )?;
            
            let mut project = db_project.to_project()?;
            
            // Load related entities
            project.agents = self.load_agents(conn, &project.id)?;
            project.tasks = self.load_tasks(conn, &project.id)?;
            project.issues = self.load_issues(conn, &project.id)?;
            
            Ok(project)
        })
    }

    /// Check if project exists using helper
    pub fn project_exists(&self, project_path: &str) -> Result<bool> {
        DatabaseHelpers::with_connection(&self.db.get_connection(), |conn| {
            DatabaseHelpers::exists(conn, "projects", "project_path", project_path, "project")
        })
    }

    /// Delete project using helper
    pub fn delete_project(&self, project_path: &str) -> Result<()> {
        DatabaseHelpers::with_connection(&self.db.get_connection(), |conn| {
            DatabaseHelpers::insert_or_replace(
                conn,
                "DELETE FROM projects WHERE project_path = ?1",
                &[&project_path],
                "delete project"
            )
        })
    }

    // Private helper methods using the new patterns

    fn save_project_data(&self, conn: &rusqlite::Transaction, project: &Project) -> Result<()> {
        let db_project = DbProject::from_project(project);
        
        DatabaseHelpers::insert_or_replace(
            conn,
            "INSERT OR REPLACE INTO projects 
             (id, idea, name, repository_url, project_path, status, tech_stack, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            &[
                &db_project.id,
                &db_project.idea,
                &db_project.name,
                &db_project.repository_url,
                &db_project.project_path,
                &db_project.status,
                &db_project.tech_stack,
                &db_project.created_at,
                &db_project.updated_at
            ],
            "save project"
        )
    }

    fn save_agent_data(&self, conn: &rusqlite::Transaction, agent: &crate::models::agent::Agent, project_id: &uuid::Uuid) -> Result<()> {
        let db_agent = DbAgent::from_agent(agent, project_id);
        
        DatabaseHelpers::insert_or_replace(
            conn,
            "INSERT OR REPLACE INTO agents 
             (id, project_id, name, file_path, description, agent_type, status, created_at, updated_at,
              last_active_at, error_count, total_tasks_completed, recovery_attempts, last_error_recovery_at,
              autonomous_recovery_enabled, max_recovery_attempts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            &[
                &db_agent.id,
                &db_agent.project_id,
                &db_agent.name,
                &db_agent.file_path,
                &db_agent.description,
                &db_agent.agent_type,
                &db_agent.status,
                &db_agent.created_at,
                &db_agent.updated_at,
                &db_agent.last_active_at,
                &db_agent.error_count.to_string(),
                &db_agent.total_tasks_completed.to_string(),
                &db_agent.recovery_attempts.to_string(),
                &db_agent.last_error_recovery_at,
                &db_agent.autonomous_recovery_enabled.to_string(),
                &db_agent.max_recovery_attempts.to_string()
            ],
            "save agent"
        )?;

        // Save agent status history using batch operation
        DatabaseHelpers::save_collection(
            conn,
            &agent.status_history,
            |conn, status_change| {
                DatabaseHelpers::insert_or_replace(
                    conn,
                    "INSERT OR REPLACE INTO agent_status_history 
                     (agent_id, from_status, to_status, reason, timestamp)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    &[
                        &agent.id.to_string(),
                        &status_change.from.as_ref().map(|s| DatabaseHelpers::serialize_enum(s, "agent status").unwrap_or_default()).unwrap_or_default(),
                        &DatabaseHelpers::serialize_enum(&status_change.to, "agent status").unwrap_or_default(),
                        &status_change.reason.as_deref().unwrap_or(""),
                        &status_change.timestamp.to_rfc3339()
                    ],
                    "save agent status history"
                )
            },
            "agent status history"
        )?;

        Ok(())
    }

    fn save_task_data(&self, conn: &rusqlite::Transaction, task: &crate::models::task::Task, project_id: &uuid::Uuid) -> Result<()> {
        let db_task = DbTask::from_task(task, project_id);
        
        DatabaseHelpers::insert_or_replace(
            conn,
            "INSERT OR REPLACE INTO tasks 
             (id, project_id, title, description, status, priority, estimated_complexity, estimated_duration,
              created_at, updated_at, completed_at, due_date, rapporter_id, assigned_to_id, pull_request_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            &[
                &db_task.id,
                &db_task.project_id,
                &db_task.title,
                &db_task.description,
                &db_task.status,
                &db_task.priority,
                &db_task.estimated_complexity.map(|c| c.to_string()).as_deref().unwrap_or(""),
                &db_task.estimated_duration.map(|d| d.to_string()).as_deref().unwrap_or(""),
                &db_task.created_at.as_deref().unwrap_or(""),
                &db_task.updated_at.as_deref().unwrap_or(""),
                &db_task.completed_at.as_deref().unwrap_or(""),
                &db_task.due_date.as_deref().unwrap_or(""),
                &db_task.rapporter_id.as_deref().unwrap_or(""),
                &db_task.assigned_to_id.as_deref().unwrap_or(""),
                &db_task.pull_request_id.as_deref().unwrap_or("")
            ],
            "save task"
        )
    }

    fn save_issue_data(&self, conn: &rusqlite::Transaction, issue: &crate::models::issue::Issue, project_id: &uuid::Uuid) -> Result<()> {
        let db_issue = DbIssue::from_issue(issue, project_id);
        
        DatabaseHelpers::insert_or_replace(
            conn,
            "INSERT OR REPLACE INTO issues 
             (id, project_id, task_id, github_issue_number, title, body, assignee, branch_name,
              issue_type, status, created_at, updated_at, closed_at, reopened_count, remotly_synced)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            &[
                &db_issue.id,
                &db_issue.project_id,
                &db_issue.task_id,
                &db_issue.github_issue_number.map(|n| n.to_string()).as_deref().unwrap_or(""),
                &db_issue.title,
                &db_issue.body,
                &db_issue.assignee.as_deref().unwrap_or(""),
                &db_issue.branch_name.as_deref().unwrap_or(""),
                &db_issue.issue_type.as_deref().unwrap_or(""),
                &db_issue.status,
                &db_issue.created_at,
                &db_issue.updated_at,
                &db_issue.closed_at.as_deref().unwrap_or(""),
                &db_issue.reopened_count.to_string(),
                &db_issue.remotly_synced.to_string()
            ],
            "save issue"
        )
    }

    fn load_agents(&self, conn: &rusqlite::Connection, project_id: &uuid::Uuid) -> Result<Vec<crate::models::agent::Agent>> {
        // Simplified placeholder - would use the helper functions for actual implementation
        Ok(Vec::new())
    }

    fn load_tasks(&self, conn: &rusqlite::Connection, project_id: &uuid::Uuid) -> Result<Vec<crate::models::task::Task>> {
        // Simplified placeholder - would use the helper functions for actual implementation
        Ok(Vec::new())
    }

    fn load_issues(&self, conn: &rusqlite::Connection, project_id: &uuid::Uuid) -> Result<Vec<crate::models::issue::Issue>> {
        // Simplified placeholder - would use the helper functions for actual implementation
        Ok(Vec::new())
    }
}
