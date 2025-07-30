use crate::models::agent::Agent;
use crate::models::task::Task;
use crate::models::issue::Issue;
use crate::enums::{Priority, TechStack};
use crate::utils::cli::*;
use crate::managers::{McpManager, McpClient, ProjectManager};
use clap::ArgMatches;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use uuid::Uuid;
use tracing::debug;

/// Parse tech stack from string
fn parse_tech_stack(tech_stack_str: &str) -> Result<TechStack, Box<dyn std::error::Error>> {
    match tech_stack_str.to_lowercase().as_str() {
        "rust" => Ok(TechStack::Rust),
        "vue" => Ok(TechStack::Vue),
        "react" => Ok(TechStack::React),
        "fullstack-rust-vue" | "rust-vue" => Ok(TechStack::FullstackRustVue),
        "fullstack-rust-react" | "rust-react" => Ok(TechStack::FullstackRustReact),
        _ => Err(format!("Invalid tech stack: {}. Valid options: rust, vue, react, fullstack-rust-vue, fullstack-rust-react", tech_stack_str).into()),
    }
}

/// Collect existing files in the project directory for context
async fn collect_existing_files(project_path: &PathBuf) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    debug!("📂 Collecting existing files from: {:?}", project_path);
    let mut files = Vec::new();
    
    // Define file extensions we want to include for context
    let relevant_extensions = vec![
        "rs", "js", "ts", "jsx", "tsx", "vue", "py", "go", "java", "cpp", "c", "h",
        "json", "toml", "yaml", "yml", "md", "txt", "html", "css", "scss", "less",
        "php", "rb", "swift", "kt", "cs", "scala", "sh", "dockerfile", "xml"
    ];
    
    fn collect_files_recursive(
        dir: &PathBuf, 
        files: &mut Vec<(String, String)>, 
        relevant_extensions: &[&str],
        base_path: &PathBuf
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Skip hidden files and directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') {
                        continue;
                    }
                }
                
                // Skip common directories that usually don't contain source code
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if matches!(name, "node_modules" | "target" | "build" | "dist" | "__pycache__" | ".git") {
                            continue;
                        }
                    }
                    collect_files_recursive(&path, files, relevant_extensions, base_path)?;
                } else if path.is_file() {
                    // Check if file has a relevant extension
                    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                        if relevant_extensions.contains(&extension) {
                            // Get relative path from project root
                            let relative_path = path.strip_prefix(base_path)
                                .unwrap_or(&path)
                                .to_string_lossy()
                                .to_string();
                            
                            // Read file content (limit size to avoid overwhelming the AI)
                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    let truncated_content = if content.len() > 2000 {
                                        format!("{}...\n[Content truncated - {} total characters]", 
                                               &content[..2000], content.len())
                                    } else {
                                        content
                                    };
                                    files.push((relative_path, truncated_content));
                                },
                                Err(e) => {
                                    debug!("⚠️  Could not read file {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    collect_files_recursive(project_path, &mut files, &relevant_extensions, project_path)?;
    
    debug!("📋 Collected {} files for context:", files.len());
    for (path, content) in &files {
        debug!("   📄 {} ({} characters)", path, content.len());
    }
    
    Ok(files)
}

/// Execute all tasks in dependency order via ProjectManager (now handled internally)
async fn execute_all_tasks_via_project_manager(
    project_manager: &ProjectManager,
    mcp_client: &McpClient,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("🎯 CLI: Delegating task execution to Project Manager...");
    
    project_manager.execute_all_tasks_in_order(mcp_client.clone()).await
        .map_err(|e| format!("Project Manager task execution failed: {}", e).into())
}

/// Handle the create project command - now delegates to ProjectManager
pub async fn handle_create_project(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let name = matches.get_one::<String>("name").unwrap().clone();
    let idea = matches.get_one::<String>("idea").unwrap().clone();
    let path = matches.get_one::<String>("path").unwrap().clone();
    let repository_url = matches.get_one::<String>("repository-url").map(|s| s.clone());
    let dependencies_urls = matches.get_one::<String>("dependencies-urls")
        .map(|deps| deps.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>());
    let tech_stack_str = matches.get_one::<String>("tech-stack");

    debug!("🎯 CLI: Handling create project command");
    debug!("📝 CLI: Name: {}, Idea: {}, Path: {}", name, idea, path);

    // Parse tech stack
    let tech_stack = if let Some(ts) = tech_stack_str {
        parse_tech_stack(ts)?
    } else {
        TechStack::default()
    };

    debug!("🔧 CLI: Tech Stack: {:?}", tech_stack);

    // Initialize MCP Manager
    debug!("🤖 CLI: Starting MCP Manager...");
    let (mcp_manager, mcp_command_tx, _mcp_event_rx) = McpManager::new();
    let mcp_client = McpClient::new(mcp_command_tx.clone());

    // Start MCP Manager in background
    let mcp_handle = tokio::spawn(async move {
        if let Err(e) = mcp_manager.run().await {
            debug!("MCP Manager error: {}", e);
        }
    });

    // Delegate project creation to ProjectManager
    debug!("🏗️  CLI: Delegating project creation to ProjectManager...");
    let project_manager = match ProjectManager::create_project(
        name.clone(),
        idea.clone(),
        path.clone(),
        tech_stack.clone(),
        repository_url,
        dependencies_urls,
        mcp_client.clone(),
    ).await {
        Ok(manager) => {
            debug!("✅ CLI: Project created successfully via ProjectManager");
            manager
        }
        Err(e) => {
            debug!("❌ CLI: Project creation failed: {}", e);
            
            // Cleanup MCP manager
            let _ = mcp_client.shutdown().await;
            let _ = mcp_handle.await;
            
            return Err(format!("Project creation failed: {}", e).into());
        }
    };

    // Execute all tasks via ProjectManager
    debug!("🎯 CLI: Starting task execution via ProjectManager...");
    match execute_all_tasks_via_project_manager(&project_manager, &mcp_client).await {
        Ok(_) => {
            debug!("🎉 CLI: All tasks completed successfully!");
            
            // Final verification of project structure
            debug!("🔍 CLI: Final project verification:");
            let project_path = PathBuf::from(&path);
            match std::fs::read_dir(&project_path) {
                Ok(entries) => {
                    debug!("📋 CLI: Final project structure:");
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            debug!("   📁 {}/", entry.file_name().to_string_lossy());
                        } else {
                            debug!("   📄 {}", entry.file_name().to_string_lossy());
                        }
                    }
                }
                Err(e) => {
                    debug!("❌ CLI: Failed to list final project directory: {}", e);
                }
            }
        }
        Err(e) => {
            debug!("❌ CLI: Task execution failed: {}", e);
            debug!("⚠️  CLI: Some tasks may have been completed, but the process was interrupted");
        }
    }

    debug!("✅ CLI: Project '{}' setup complete!", name);
    debug!("📁 CLI: Project location: {}", path);

    // Shutdown managers
    project_manager.shutdown().await;
    let _ = mcp_client.shutdown().await;
    let _ = mcp_handle.await;

    debug!("🎉 CLI: Project setup complete! You can now:");
    debug!("   • View tasks: orchy list-tasks");
    debug!("   • Add agents: orchy add-agent");
    debug!("   • Start development: cd {} && orchy start", path);

    Ok(())
}

/// Handle the list tasks command with interactive project selection
pub async fn handle_list_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let projects = discover_projects().await?;
    
    if projects.is_empty() {
        debug!("No projects found. Create a project first using 'orchy create'.");
        return Ok(());
    }

    debug!("📋 Available Projects:");
    for (i, (name, _)) in projects.iter().enumerate() {
        debug!("  {}. {}", i + 1, name);
    }

    print!("\nSelect a project (1-{}): ", projects.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let selection: usize = input.trim().parse().map_err(|_| "Invalid selection")?;
    
    if selection == 0 || selection > projects.len() {
        return Err("Invalid project selection".into());
    }

    let (_project_name, project) = &projects[selection - 1];
    list_tasks_for_project(project);

    Ok(())
}

/// Handle the list agents command
pub async fn handle_list_agents() -> Result<(), Box<dyn std::error::Error>> {
    let projects = discover_projects().await?;
    
    if projects.is_empty() {
        debug!("No projects found. Create a project first using 'orchy create'.");
        return Ok(());
    }

    debug!("🤖 All Agents:");
    debug!("{}", "=".repeat(50));

    let mut all_agents: HashMap<Uuid, (&Agent, &str)> = HashMap::new();
    
    // Collect all unique agents from all projects
    for (project_name, project) in &projects {
        for agent in &project.agents {
            all_agents.insert(agent.id, (agent, project_name));
        }
    }

    if all_agents.is_empty() {
        debug!("No agents found in any project.");
        return Ok(());
    }

    for (i, (_, (agent, project_name))) in all_agents.iter().enumerate() {
        debug!("{}. {} [{}]", i + 1, agent.name, format_agent_status(&agent.status));
        debug!("   Description: {}", agent.description);
        debug!("   Project: {}", project_name);
        debug!("   File Path: {}", agent.file_path.display());
        debug!("   Created: {}", agent.created_at.format("%Y-%m-%d %H:%M:%S"));
        
        if let Some(last_active) = agent.last_active_at {
            debug!("   Last Active: {}", last_active.format("%Y-%m-%d %H:%M:%S"));
        }
        
        debug!("");
    }

    Ok(())
}

/// Handle the list issues command with interactive project selection
pub async fn handle_list_issues() -> Result<(), Box<dyn std::error::Error>> {
    let projects = discover_projects().await?;
    
    if projects.is_empty() {
        debug!("No projects found. Create a project first using 'orchy create'.");
        return Ok(());
    }

    debug!("📋 Available Projects:");
    for (i, (name, _)) in projects.iter().enumerate() {
        debug!("  {}. {}", i + 1, name);
    }

    print!("\nSelect a project (1-{}): ", projects.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let selection: usize = input.trim().parse().map_err(|_| "Invalid selection")?;
    
    if selection == 0 || selection > projects.len() {
        return Err("Invalid project selection".into());
    }

    let (_project_name, project) = &projects[selection - 1];
    list_issues_for_project(project);

    Ok(())
}

/// Handle adding sample data to a project
pub async fn handle_add_sample_data(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let project_name = matches.get_one::<String>("project-name").unwrap();
    let projects = discover_projects().await?;
    
    // Find the project
    let (_, mut project) = projects.into_iter()
        .find(|(name, _)| name == project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    debug!("Adding sample data to project '{}'...", project_name);

    // Add sample agents
    let agent1 = Agent::new("Alice Developer", PathBuf::from("/tmp/alice.json"), "Senior full-stack developer");
    let agent2 = Agent::new("Bob Tester", PathBuf::from("/tmp/bob.json"), "QA engineer and test automation specialist");
    
    project.add_agent(agent1.clone())?;
    project.add_agent(agent2.clone())?;

    // Add sample tasks
    let mut task1 = Task::new("Implement user authentication", "Create login/logout functionality with JWT tokens", Priority::High);
    let task2 = Task::new("Design database schema", "Create tables for users, projects, and tasks", Priority::Critical);
    let mut task3 = Task::new("Write unit tests", "Add comprehensive test coverage for authentication", Priority::Medium);
    
    // Set some tasks as assigned
    task1.assigned_to = Some(agent1.clone());
    task3.assigned_to = Some(agent2.clone());
    
    // Add dependencies
    task3.depends_on = vec![task1.id];
    
    project.add_task(task1.clone())?;
    project.add_task(task2)?;
    project.add_task(task3.clone())?;

    // Add sample issue
    let issue = Issue::from_task(&task1);
    project.add_issue(issue)?;

    // Save the updated project
    save_project(&project).await?;

    debug!("✅ Sample data added successfully!");
    debug!("   - 2 agents added");
    debug!("   - 3 tasks added (1 with dependencies)");
    debug!("   - 1 issue added");

    Ok(())
}

/// Handle adding an agent from command line
pub async fn handle_add_agent(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let project_name = matches.get_one::<String>("project-name").unwrap();
    let name = matches.get_one::<String>("name").unwrap();
    let description = matches.get_one::<String>("description").unwrap();
    let file_path = matches.get_one::<String>("file-path").unwrap();

    let projects = discover_projects().await?;
    let (_, mut project) = projects.into_iter()
        .find(|(pname, _)| pname == project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    let agent = Agent::new(name, PathBuf::from(file_path), description);
    project.add_agent(agent.clone())?;

    // Save the updated project
    save_project(&project).await?;

    debug!("✅ Agent '{}' added to project '{}'!", name, project_name);

    Ok(())
}
