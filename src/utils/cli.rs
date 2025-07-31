use crate::models::project::Project;
use crate::database::{Database, repository::ProjectRepository};
use std::fs;
use std::path::PathBuf;

/// Discover all projects from database
pub async fn discover_projects() -> Result<Vec<(String, Project)>, Box<dyn std::error::Error>> {
    // Use default database location
    let db_path = get_default_database_path();

    // If database doesn't exist, return empty list
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    discover_projects_from_database(db_path).await
}

/// Get the default database path
pub fn get_default_database_path() -> PathBuf {
    // Use a standard location for the database
    if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".orchy").join("orchy.db")
    } else {
        PathBuf::from("orchy.db")
    }
}

/// Save project to database (default method)
pub async fn save_project(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_default_database_path();

    // Ensure the database directory exists
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    save_project_to_database(project, db_path).await
}

/// Save project to database
pub async fn save_project_to_database(project: &Project, db_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::new(db_path)?;
    let repository = ProjectRepository::new(database);
    repository.save_project(project)?;
    Ok(())
}

/// Load project from database
pub async fn load_project_from_database(project_path: &str, db_path: PathBuf) -> Result<Project, Box<dyn std::error::Error>> {
    let database = Database::new(db_path)?;
    let repository = ProjectRepository::new(database);
    let project = repository.load_project(project_path)?;
    Ok(project)
}

/// Discover all projects from database
pub async fn discover_projects_from_database(db_path: PathBuf) -> Result<Vec<(String, Project)>, Box<dyn std::error::Error>> {
    let database = Database::new(db_path)?;
    let repository = ProjectRepository::new(database);
    let projects = repository.list_projects()?;
    Ok(projects)
}

/// Check if a project exists in database
pub async fn project_exists_in_database(project_path: &str, db_path: PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
    let database = Database::new(db_path)?;
    let repository = ProjectRepository::new(database);
    let exists = repository.project_exists(project_path)?;
    Ok(exists)
}

/// Format task status for display
pub fn format_task_status(status: &crate::enums::TaskStatus) -> &'static str {
    match status {
        crate::enums::TaskStatus::Pending => "⏳ Pending",
        crate::enums::TaskStatus::InProgress => "🔄 In Progress",
        crate::enums::TaskStatus::UnderReview => "👀 Under Review",
        crate::enums::TaskStatus::Testing => "🧪 Testing",
        crate::enums::TaskStatus::Completed => "✅ Completed",
        crate::enums::TaskStatus::Blocked => "🚫 Blocked",
        crate::enums::TaskStatus::Failed => "❌ Failed",
        crate::enums::TaskStatus::Cancelled => "🚮 Cancelled",
    }
}

/// Format agent status for display
pub fn format_agent_status(status: &crate::enums::AgentStatus) -> &'static str {
    match status {
        crate::enums::AgentStatus::Idle => "⚪ Idle",
        crate::enums::AgentStatus::Working => "🔄 Working",
        crate::enums::AgentStatus::Active => "🟢 Active",
        crate::enums::AgentStatus::Busy => "🟡 Busy",
        crate::enums::AgentStatus::Error => "❌ Error",
        crate::enums::AgentStatus::Maintenance => "🔧 Maintenance",
    }
}

/// Format issue status for display
pub fn format_issue_status(status: &crate::enums::IssueStatus) -> &'static str {
    match status {
        crate::enums::IssueStatus::Open => "🔓 Open",
        crate::enums::IssueStatus::InProgress => "🔄 In Progress",
        crate::enums::IssueStatus::InReview => "👀 In Review",
        crate::enums::IssueStatus::Closed => "🔒 Closed",
    }
}

/// List tasks for a specific project
pub fn list_tasks_for_project(project: &Project) {
    println!("\n📋 Tasks for project '{}':", project.name);
    println!("{}", "=".repeat(50));

    if project.tasks.is_empty() {
        println!("No tasks found in this project.");
        return;
    }

    for (i, task) in project.tasks.iter().enumerate() {
        println!("{}. {} [{}]", i + 1, task.title, format_task_status(&task.status));
        println!("   Description: {}", task.description);
        println!("   Priority: {:?}", task.priority);
        
        if let Some(agent) = &task.assigned_to {
            println!("   Assigned to: {}", agent.name);
        } else {
            println!("   Assigned to: Unassigned");
        }
        
        if !task.depends_on.is_empty() {
            println!("   Dependencies: {} task(s)", task.depends_on.len());
        }
        
        if let Some(created_at) = task.created_at {
            println!("   Created: {}", created_at.format("%Y-%m-%d %H:%M:%S"));
        }
        println!();
    }
}

/// List agents for a specific project
pub fn list_agents_for_project(project: &Project) {
    println!("\n🤖 Agents for project '{}':", project.name);
    println!("{}", "=".repeat(50));

    if project.agents.is_empty() {
        println!("No agents found in this project.");
        return;
    }

    for (i, agent) in project.agents.iter().enumerate() {
        println!("{}. {} [{}]", i + 1, agent.name, format_agent_status(&agent.status));
        println!("   Description: {}", agent.description);
        println!("   File Path: {}", agent.file_path.display());
        println!("   Created: {}", agent.created_at.format("%Y-%m-%d %H:%M:%S"));
        
        if let Some(last_active) = agent.last_active_at {
            println!("   Last Active: {}", last_active.format("%Y-%m-%d %H:%M:%S"));
        }
        println!();
    }
}

/// List issues for a specific project
pub fn list_issues_for_project(project: &Project) {
    println!("\n🐛 Issues for project '{}':", project.name);
    println!("{}", "=".repeat(50));

    if project.issues.is_empty() {
        println!("No issues found in this project.");
        return;
    }

    for (i, issue) in project.issues.iter().enumerate() {
        println!("{}. {} [{}]", i + 1, issue.title, format_issue_status(&issue.status));
        println!("   Description: {}", issue.body);
        
        if let Some(github_num) = issue.github_issue_number {
            println!("   GitHub Issue: #{}", github_num);
        }
        
        if let Some(assignee) = &issue.assignee {
            println!("   Assignee: {}", assignee);
        } else {
            println!("   Assignee: Unassigned");
        }
        
        if !issue.labels.is_empty() {
            println!("   Labels: {}", issue.labels.join(", "));
        }
        
        println!("   Comments: {}", issue.comments.len());
        println!("   Synced: {}", if issue.remotly_synced { "✅" } else { "❌" });
        println!("   Created: {}", issue.created_at.format("%Y-%m-%d %H:%M:%S"));
        
        if let Some(closed_at) = issue.closed_at {
            println!("   Closed: {}", closed_at.format("%Y-%m-%d %H:%M:%S"));
        }
        println!();
    }
}
