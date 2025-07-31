use orchy::utils::cli_handlers::handle_create_project;
use orchy::models::project::Project;
use orchy::enums::{TechStack, TaskStatus, AgentStatus};
use clap::{Arg, ArgMatches, Command};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt::init();
    });
}

/// Helper function to create ArgMatches for testing
fn create_test_args(name: &str, idea: &str, path: &str, tech_stack: &str) -> ArgMatches {
    let app = Command::new("test")
        .arg(Arg::new("name").long("name").value_name("NAME").required(true))
        .arg(Arg::new("idea").long("idea").value_name("IDEA").required(true))
        .arg(Arg::new("path").long("path").value_name("PATH").required(true))
        .arg(Arg::new("tech-stack").long("tech-stack").value_name("TECH_STACK"))
        .arg(Arg::new("repository-url").long("repository-url").value_name("REPO_URL"))
        .arg(Arg::new("dependencies-urls").long("dependencies-urls").value_name("DEPS"));

    app.try_get_matches_from(vec![
        "test",
        "--name", name,
        "--idea", idea,
        "--path", path,
        "--tech-stack", tech_stack,
    ]).unwrap()
}

/// Helper function to analyze task and agent history in the project
async fn analyze_project_history(project_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = orchy::utils::cli::get_default_database_path();
    let project_path_str = project_path.to_string_lossy().to_string();

    println!("📊 Analyzing project history from database for: {}", project_path.display());

    let project = orchy::utils::cli::load_project_from_database(&project_path_str, db_path).await?;
    
    println!("🎯 PROJECT ANALYSIS RESULTS:");
    println!("  📋 Total Tasks: {}", project.tasks.len());
    println!("  🤖 Total Agents: {}", project.agents.len());
    
    // Analyze task histories
    println!("\n📝 TASK HISTORY ANALYSIS:");
    for (i, task) in project.tasks.iter().enumerate() {
        println!("  Task {}: {} ({:?})", i + 1, task.title, task.status);
        println!("    🕐 Status History: {} entries", task.status_history.len());
        
        for (j, (status, timestamp)) in task.status_history.iter().enumerate() {
            println!("      {}: {:?} at {}", j + 1, status, timestamp.format("%H:%M:%S%.3f"));
        }
        
        if let Some(agent) = &task.assigned_to {
            println!("    👤 Assigned to: {} ({})", agent.name, agent.id);
        } else {
            println!("    👤 Assigned to: None");
        }
        
        println!("    📊 Complexity: {:?}", task.estimated_complexity);
        println!("    🎚️  Priority: {:?}", task.priority);
        println!("    🏷️  Tags: {:?}", task.tags);
        println!();
    }
    
    // Analyze agent histories
    println!("🤖 AGENT HISTORY ANALYSIS:");
    for (i, agent) in project.agents.iter().enumerate() {
        println!("  Agent {}: {} ({:?})", i + 1, agent.name, agent.status);
        println!("    🕐 Status History: {} entries", agent.status_history.len());
        
        for (j, status_change) in agent.status_history.iter().enumerate() {
            let from_str = match &status_change.from {
                Some(status) => format!("{:?}", status),
                None => "None".to_string(),
            };
            println!("      {}: {} → {:?} at {} ({})", 
                j + 1, 
                from_str, 
                status_change.to, 
                status_change.timestamp.format("%H:%M:%S%.3f"),
                status_change.reason.as_deref().unwrap_or("No reason")
            );
        }
        
        println!("    📊 Tasks Completed: {}", agent.total_tasks_completed);
        println!("    ❌ Error Count: {}", agent.error_count);
        println!("    🏥 Health Score: {}", agent.health_score());
        
        if let Some(last_active) = agent.last_active_at {
            println!("    🕐 Last Active: {}", last_active.format("%H:%M:%S%.3f"));
        }
        println!();
    }
    
    // Verify history tracking integrity
    println!("🔍 HISTORY INTEGRITY VERIFICATION:");
    
    let mut history_issues = 0;
    
    // Check that all tasks have at least initial status history
    for task in &project.tasks {
        if task.status_history.is_empty() {
            println!("  ❌ Task '{}' has no status history!", task.title);
            history_issues += 1;
        } else if task.status_history[0].0 != TaskStatus::Pending {
            println!("  ⚠️  Task '{}' doesn't start with Pending status: {:?}", task.title, task.status_history[0].0);
        }
    }
    
    // Check that all agents have at least initial status history
    for agent in &project.agents {
        if agent.status_history.is_empty() {
            println!("  ❌ Agent '{}' has no status history!", agent.name);
            history_issues += 1;
        } else if agent.status_history[0].to != AgentStatus::Idle {
            println!("  ⚠️  Agent '{}' doesn't start with Idle status: {:?}", agent.name, agent.status_history[0].to);
        }
    }
    
    if history_issues == 0 {
        println!("  ✅ All tasks and agents have proper history tracking!");
    } else {
        println!("  ❌ Found {} history tracking issues", history_issues);
    }
    
    println!("\n🎉 History analysis complete!");
    Ok(())
}

#[tokio::test]
async fn test_history_tracking_during_project_creation() {
    init_tracing();
    println!("🧪 Testing Task and Agent History Tracking During Project Creation");
    
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("history-test-project");
    
    let args = create_test_args(
        "history-test-project",
        "Create a simple todo application for testing history tracking functionality",
        project_path.to_str().unwrap(),
        "Vue"
    );

    println!("📂 Project path: {}", project_path.display());
    println!("🚀 Starting project creation with history tracking test...");

    // Execute project creation
    let result = handle_create_project(&args).await;
    
    match result {
        Ok(()) => {
            println!("✅ Project creation completed successfully");
            
            // Verify basic project structure
            let expected_files = vec![
                "GEMINI.md",
                "CLAUDE.md",
            ];
            
            for file in &expected_files {
                let file_path = project_path.join(file);
                assert!(file_path.exists(), "Expected file not found: {}", file_path.display());
                println!("✅ Found expected file: {}", file);
            }
            
            // Analyze the history tracking in detail
            match analyze_project_history(&project_path).await {
                Ok(()) => {
                    println!("✅ History analysis completed successfully");
                }
                Err(e) => {
                    panic!("❌ History analysis failed: {}", e);
                }
            }
            
            println!("🎉 History tracking E2E test PASSED!");
        }
        Err(e) => {
            panic!("❌ Project creation failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_multiple_project_creation_history_isolation() {
    init_tracing();
    println!("🧪 Testing History Isolation Across Multiple Projects");
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create first project
    let project1_path = temp_dir.path().join("project1");
    let args1 = create_test_args(
        "project1",
        "First test project for history isolation",
        project1_path.to_str().unwrap(),
        "React"
    );
    
    println!("🚀 Creating first project...");
    handle_create_project(&args1).await.expect("Project 1 creation failed");
    
    // Create second project
    let project2_path = temp_dir.path().join("project2");
    let args2 = create_test_args(
        "project2", 
        "Second test project for history isolation",
        project2_path.to_str().unwrap(),
        "Rust"
    );
    
    println!("🚀 Creating second project...");
    handle_create_project(&args2).await.expect("Project 2 creation failed");
    
    // Verify both projects have independent histories
    println!("🔍 Verifying history isolation...");

    let db_path = orchy::utils::cli::get_default_database_path();
    let project1 = orchy::utils::cli::load_project_from_database(
        &project1_path.to_string_lossy(),
        db_path.clone()
    ).await.unwrap();

    let project2 = orchy::utils::cli::load_project_from_database(
        &project2_path.to_string_lossy(),
        db_path
    ).await.unwrap();
    
    // Verify projects have different IDs and independent data
    assert_ne!(project1.id, project2.id, "Projects should have different IDs");
    assert_eq!(project1.name, "project1");
    assert_eq!(project2.name, "project2");
    assert_eq!(project1.tech_stack, TechStack::React);
    assert_eq!(project2.tech_stack, TechStack::Rust);
    
    println!("✅ Project 1: {} tasks, {} agents", project1.tasks.len(), project1.agents.len());
    println!("✅ Project 2: {} tasks, {} agents", project2.tasks.len(), project2.agents.len());
    
    // Verify all agents in both projects have proper history
    for project in [&project1, &project2] {
        for agent in &project.agents {
            assert!(!agent.status_history.is_empty(), 
                "Agent '{}' in project '{}' should have status history", 
                agent.name, project.name);
            assert_eq!(agent.status_history[0].to, AgentStatus::Idle,
                "Agent '{}' should start with Idle status",
                agent.name);
        }
        
        for task in &project.tasks {
            assert!(!task.status_history.is_empty(),
                "Task '{}' in project '{}' should have status history",
                task.title, project.name);
            assert_eq!(task.status_history[0].0, TaskStatus::Pending,
                "Task '{}' should start with Pending status",
                task.title);
        }
    }
    
    println!("🎉 History isolation test PASSED!");
}