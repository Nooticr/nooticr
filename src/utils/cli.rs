use crate::models::project::Project;
use std::fs;
use std::path::PathBuf;

/// Discover all projects by looking for orchy.json files
pub async fn discover_projects() -> Result<Vec<(String, Project)>, Box<dyn std::error::Error>> {
    let mut projects = Vec::new();
    
    // Look in current directory and common project directories
    let search_paths = vec![
        PathBuf::from("."),
        PathBuf::from("./projects"),
        PathBuf::from("../"),
        dirs::home_dir().map(|h| h.join("projects")).unwrap_or_else(|| PathBuf::from(".")),
    ];

    for search_path in search_paths {
        if !search_path.exists() {
            continue;
        }

        // Look for orchy.json files recursively (up to 2 levels deep)
        if let Ok(entries) = fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Check if it's a directory
                if path.is_dir() {
                    let config_path = path.join("orchy.json");
                    if config_path.exists() {
                        if let Ok(content) = fs::read_to_string(&config_path) {
                            if let Ok(project) = serde_json::from_str::<Project>(&content) {
                                projects.push((project.name.clone(), project));
                            }
                        }
                    }
                }
                
                // Also check if the entry itself is an orchy.json file
                if path.file_name().and_then(|n| n.to_str()) == Some("orchy.json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(project) = serde_json::from_str::<Project>(&content) {
                            projects.push((project.name.clone(), project));
                        }
                    }
                }
            }
        }
    }

    // Remove duplicates based on project ID
    projects.sort_by(|a, b| a.1.id.cmp(&b.1.id));
    projects.dedup_by(|a, b| a.1.id == b.1.id);

    Ok(projects)
}

/// Save project to file
pub async fn save_project(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
    let project_path = PathBuf::from(&project.project_path);
    let config_path = project_path.join("orchy.json");
    let project_json = serde_json::to_string_pretty(project)?;
    fs::write(&config_path, project_json)?;
    Ok(())
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
