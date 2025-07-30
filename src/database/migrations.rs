use crate::error::{OrchestratorError, Result};
use rusqlite::Connection;
use super::schema::ALL_TABLES;

/// Current database schema version
pub const CURRENT_VERSION: i32 = 1;

/// Run all database migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version = get_user_version(conn)?;
    
    if current_version < CURRENT_VERSION {
        tracing::info!("Running database migrations from version {} to {}", current_version, CURRENT_VERSION);
        
        // Run migrations based on current version
        match current_version {
            0 => {
                // Initial migration - create all tables
                migrate_to_v1(conn)?;
            }
            _ => {
                return Err(OrchestratorError::database(format!(
                    "Unknown database version: {}. Expected version <= {}",
                    current_version, CURRENT_VERSION
                )));
            }
        }
        
        // Update version
        set_user_version(conn, CURRENT_VERSION)?;
        tracing::info!("Database migrations completed successfully");
    } else if current_version > CURRENT_VERSION {
        return Err(OrchestratorError::database(format!(
            "Database version {} is newer than supported version {}. Please update the application.",
            current_version, CURRENT_VERSION
        )));
    }
    
    Ok(())
}

/// Get the current user version from the database
fn get_user_version(conn: &Connection) -> Result<i32> {
    let version: i32 = conn.query_row(
        "SELECT user_version FROM pragma_user_version",
        [],
        |row| row.get(0)
    ).map_err(|e| OrchestratorError::database(format!("Failed to get database version: {}", e)))?;
    
    Ok(version)
}

/// Set the user version in the database
fn set_user_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(&format!("PRAGMA user_version = {}", version), [])
        .map_err(|e| OrchestratorError::database(format!("Failed to set database version: {}", e)))?;
    
    Ok(())
}

/// Migration to version 1 - create initial schema
fn migrate_to_v1(conn: &Connection) -> Result<()> {
    tracing::info!("Creating initial database schema (v1)");
    
    // Create all tables
    for table_sql in ALL_TABLES {
        conn.execute(table_sql, [])
            .map_err(|e| OrchestratorError::database(format!("Failed to create table: {}", e)))?;
    }
    
    // Create indexes for better performance
    create_indexes(conn)?;
    
    tracing::info!("Initial database schema created successfully");
    Ok(())
}

/// Create database indexes for better query performance
fn create_indexes(conn: &Connection) -> Result<()> {
    let indexes = [
        // Project indexes
        "CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status)",
        "CREATE INDEX IF NOT EXISTS idx_projects_tech_stack ON projects(tech_stack)",
        "CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects(created_at)",
        
        // Task indexes
        "CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_assigned_to ON tasks(assigned_to_id)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date)",
        
        // Agent indexes
        "CREATE INDEX IF NOT EXISTS idx_agents_project_id ON agents(project_id)",
        "CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status)",
        "CREATE INDEX IF NOT EXISTS idx_agents_type ON agents(agent_type)",
        "CREATE INDEX IF NOT EXISTS idx_agents_last_active ON agents(last_active_at)",
        
        // Issue indexes
        "CREATE INDEX IF NOT EXISTS idx_issues_project_id ON issues(project_id)",
        "CREATE INDEX IF NOT EXISTS idx_issues_task_id ON issues(task_id)",
        "CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status)",
        "CREATE INDEX IF NOT EXISTS idx_issues_github_number ON issues(github_issue_number)",
        "CREATE INDEX IF NOT EXISTS idx_issues_synced ON issues(remotly_synced)",
        
        // Pull request indexes
        "CREATE INDEX IF NOT EXISTS idx_pull_requests_status ON pull_requests(code_status)",
        "CREATE INDEX IF NOT EXISTS idx_pull_requests_author ON pull_requests(author)",
        "CREATE INDEX IF NOT EXISTS idx_pull_requests_github_number ON pull_requests(github_pr_number)",
        "CREATE INDEX IF NOT EXISTS idx_pull_requests_synced ON pull_requests(remotly_synced)",
        
        // Comment indexes
        "CREATE INDEX IF NOT EXISTS idx_comments_task_id ON comments(task_id)",
        "CREATE INDEX IF NOT EXISTS idx_comments_issue_id ON comments(issue_id)",
        "CREATE INDEX IF NOT EXISTS idx_comments_pull_request_id ON comments(pull_request_id)",
        "CREATE INDEX IF NOT EXISTS idx_comments_type ON comments(comment_type)",
        "CREATE INDEX IF NOT EXISTS idx_comments_author ON comments(author)",
        "CREATE INDEX IF NOT EXISTS idx_comments_synced ON comments(remotly_synced)",
        
        // History indexes
        "CREATE INDEX IF NOT EXISTS idx_task_status_history_task_id ON task_status_history(task_id)",
        "CREATE INDEX IF NOT EXISTS idx_task_status_history_timestamp ON task_status_history(timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_agent_status_history_agent_id ON agent_status_history(agent_id)",
        "CREATE INDEX IF NOT EXISTS idx_agent_status_history_timestamp ON agent_status_history(timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_agent_errors_agent_id ON agent_errors(agent_id)",
        "CREATE INDEX IF NOT EXISTS idx_agent_errors_resolved ON agent_errors(resolved)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_history_project_id ON tasks_history(project_id)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_history_timestamp ON tasks_history(timestamp)",
        
        // Code review indexes
        "CREATE INDEX IF NOT EXISTS idx_code_reviews_pull_request_id ON code_reviews(pull_request_id)",
        "CREATE INDEX IF NOT EXISTS idx_code_reviews_reviewer ON code_reviews(reviewer)",
        "CREATE INDEX IF NOT EXISTS idx_code_reviews_approved ON code_reviews(approved)",
        
        // Junction table indexes
        "CREATE INDEX IF NOT EXISTS idx_task_tags_task_id ON task_tags(task_id)",
        "CREATE INDEX IF NOT EXISTS idx_task_tags_tag ON task_tags(tag)",
        "CREATE INDEX IF NOT EXISTS idx_task_dependencies_task_id ON task_dependencies(task_id)",
        "CREATE INDEX IF NOT EXISTS idx_task_dependencies_depends_on ON task_dependencies(depends_on_task_id)",
        "CREATE INDEX IF NOT EXISTS idx_issue_labels_issue_id ON issue_labels(issue_id)",
        "CREATE INDEX IF NOT EXISTS idx_issue_labels_label ON issue_labels(label)",
    ];
    
    for index_sql in &indexes {
        conn.execute(index_sql, [])
            .map_err(|e| OrchestratorError::database(format!("Failed to create index: {}", e)))?;
    }
    
    tracing::info!("Database indexes created successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations() {
        let conn = Connection::open_in_memory().unwrap();
        
        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        
        // Run migrations
        run_migrations(&conn).unwrap();
        
        // Check version
        let version = get_user_version(&conn).unwrap();
        assert_eq!(version, CURRENT_VERSION);
        
        // Verify some tables exist
        let table_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0)
        ).unwrap();
        
        assert!(table_count > 0);
    }

    #[test]
    fn test_version_management() {
        let conn = Connection::open_in_memory().unwrap();
        
        // Initial version should be 0
        let initial_version = get_user_version(&conn).unwrap();
        assert_eq!(initial_version, 0);
        
        // Set version
        set_user_version(&conn, 5).unwrap();
        
        // Check version
        let new_version = get_user_version(&conn).unwrap();
        assert_eq!(new_version, 5);
    }
}
