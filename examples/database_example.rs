use orchy::models::project::Project;
use orchy::models::task::Task;
use orchy::models::agent::Agent;
use orchy::enums::{Priority, TechStack, AgentType};
use orchy::database::{Database, repository::ProjectRepository};
use orchy::utils::cli::{save_project_to_database, load_project_from_database, discover_projects_from_database};
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Orchy Database Example ===\n");

    // Create a temporary directory for the database
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("orchy.db");
    
    println!("📁 Using database at: {}", db_path.display());

    // Create a sample project
    let mut project = Project::new_with_tech_stack(
        "Database Test Project",
        "A project to test the SQLite database implementation",
        temp_dir.path().join("test_project").to_string_lossy(),
        TechStack::Rust,
    );

    // Add some tasks
    let task1 = Task::new(
        "Setup Database",
        "Implement SQLite database for project storage",
        Priority::High,
    );
    
    let task2 = Task::new(
        "Create Repository Layer",
        "Build repository pattern for data access",
        Priority::Medium,
    );

    project.add_task(task1);
    project.add_task(task2);

    // Add an agent
    let agent = Agent::new_with_type(
        "Database Engineer",
        PathBuf::from("agents/database_engineer.md"),
        "Specialized in database design and implementation",
        AgentType::BackendEngineerRust,
    );

    project.add_agent(agent);

    println!("📊 Created project with:");
    println!("  - {} tasks", project.tasks.len());
    println!("  - {} agents", project.agents.len());
    println!("  - Tech stack: {:?}", project.tech_stack);

    // Save project to database using CLI utility
    println!("\n💾 Saving project to database...");
    save_project_to_database(&project, db_path.clone()).await?;
    println!("✅ Project saved successfully!");

    // Load project from database
    println!("\n📂 Loading project from database...");
    let loaded_project = load_project_from_database(&project.project_path, db_path.clone()).await?;
    println!("✅ Project loaded successfully!");

    println!("📊 Loaded project details:");
    println!("  - Name: {}", loaded_project.name);
    println!("  - Idea: {}", loaded_project.idea);
    println!("  - Tasks: {}", loaded_project.tasks.len());
    println!("  - Agents: {}", loaded_project.agents.len());
    println!("  - Tech stack: {:?}", loaded_project.tech_stack);

    // Discover all projects in database
    println!("\n🔍 Discovering all projects in database...");
    let all_projects = discover_projects_from_database(db_path.clone()).await?;
    println!("✅ Found {} project(s):", all_projects.len());
    
    for (path, proj) in &all_projects {
        println!("  - {} at {}", proj.name, path);
    }

    // Direct database operations example
    println!("\n🔧 Direct database operations example...");
    let database = Database::new(db_path.clone())?;
    let repository = ProjectRepository::new(database);

    // Check if project exists
    let exists = repository.project_exists(&project.project_path)?;
    println!("  - Project exists: {}", exists);

    // Get database health
    let db = Database::new(&db_path)?;
    db.health_check()?;
    println!("  - Database health: OK");

    let version = db.get_version()?;
    println!("  - Database version: {}", version);

    println!("\n🎉 Database example completed successfully!");
    println!("📝 The database file was created at: {}", db_path.display());
    println!("🗑️  Temporary files will be cleaned up automatically.");

    Ok(())
}
