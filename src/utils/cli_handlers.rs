use crate::models::project::Project;
use crate::models::agent::Agent;
use crate::models::task::Task;
use crate::models::issue::Issue;
use crate::enums::{Priority, TechStack};
use crate::utils::cli::*;
use crate::managers::{McpManager, McpClient, McpModel, ProjectManager, ProjectCommand};
use clap::ArgMatches;
use std::collections::HashMap;
use std::fs;
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

/// Handle the create project command with full MCP and Project Manager integration
pub async fn handle_create_project(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let name = matches.get_one::<String>("name").unwrap();
    let idea = matches.get_one::<String>("idea").unwrap();
    let path = matches.get_one::<String>("path").unwrap();
    let repository_url = matches.get_one::<String>("repository-url");
    let dependencies_urls = matches.get_one::<String>("dependencies-urls");
    let tech_stack_str = matches.get_one::<String>("tech-stack");

    debug!("🚀 Creating project: {}", name);
    debug!("💡 Idea: {}", idea);
    debug!("📁 Path: {}", path);

    // Parse tech stack
    let tech_stack = if let Some(ts) = tech_stack_str {
        parse_tech_stack(ts)?
    } else {
        TechStack::default()
    };

    debug!("🔧 Tech Stack: {:?}", tech_stack);

    // Create project directory if it doesn't exist
    let project_path = PathBuf::from(path);
    if !project_path.exists() {
        fs::create_dir_all(&project_path)?;
        debug!("📂 Created project directory: {}", path);
    }

    // Initialize MCP Manager
    debug!("🤖 Starting MCP Manager...");
    let (mcp_manager, mcp_command_tx, _mcp_event_rx) = McpManager::new();
    let mcp_client = McpClient::new(mcp_command_tx.clone());

    // Start MCP Manager in background
    let mcp_handle = tokio::spawn(async move {
        if let Err(e) = mcp_manager.run().await {
            debug!("MCP Manager error: {}", e);
        }
    });

    // Initialize context files
    debug!("📝 Initializing context files...");
    if let Err(e) = mcp_client.initialize_context(project_path.clone(), tech_stack.clone()).await {
        debug!("Warning: Failed to initialize context files: {}", e);
    } else {
        debug!("✅ Context files created (GEMINI.md, CLAUDE.md)");
    }

    // Create the project with tech stack
    let mut project = Project::new_with_tech_stack(name, idea, path, tech_stack.clone());

    // Load agents from the agents directory
    if let Err(e) = project.load_agents_from_directory("agents").await {
        debug!("Warning: Failed to load agents from directory: {}", e);
    } else {
        debug!("🤖 Loaded {} agents from agents directory", project.agents.len());
    }

    // Set optional repository URL
    if let Some(repo_url) = repository_url {
        project.set_repository_url(repo_url);
        debug!("🔗 Repository URL: {}", repo_url);
    }

    // Set optional dependency URLs
    if let Some(deps) = dependencies_urls {
        let urls: Vec<String> = deps.split(',').map(|s| s.trim().to_string()).collect();
        for url in &urls {
            if let Err(e) = project.add_dependency_url(url) {
                debug!("Warning: Failed to add dependency URL '{}': {}", url, e);
            }
        }
        debug!("📦 Dependencies: {:?}", urls);
    }

    // Prepare agent types for idea breakdown
    let available_agents: Vec<String> = project.agents.iter()
        .map(|agent| agent.name.clone())
        .collect();

    // If no agents loaded, use default agent types based on tech stack
    let agent_types = if available_agents.is_empty() {
        match tech_stack {
            TechStack::Rust => vec!["BackendEngineerRust".to_string()],
            TechStack::Vue => vec!["FrontendEngineerVue".to_string()],
            TechStack::React => vec!["FrontendEngineerReact".to_string()],
            TechStack::FullstackRustVue => vec![
                "BackendEngineerRust".to_string(),
                "FrontendEngineerVue".to_string(),
                "DevOpsEngineer".to_string(),
            ],
            TechStack::FullstackRustReact => vec![
                "BackendEngineerRust".to_string(),
                "FrontendEngineerReact".to_string(),
                "DevOpsEngineer".to_string(),
            ],
        }
    } else {
        available_agents
    };

    // Execute idea breakdown using MCP Manager
    debug!("🧠 Breaking down idea into tasks using AI...");
    let context = format!("Project: {}\nTech Stack: {:?}\nPath: {}", name, tech_stack, path);

    match mcp_client.idea_breakdown(
        idea.clone(),
        context,
        agent_types,
        format!("{:?}", tech_stack),
        McpModel::Gemini,
    ).await {
        Ok(breakdown_response) => {
            debug!("✅ Generated {} tasks from idea breakdown", breakdown_response.tasks.len());

            // Convert TaskInput to Task and add to project
            for task_input in breakdown_response.tasks {
                let task = Task::from_input(task_input.clone(), None);
                if let Err(e) = project.add_task(task) {
                    debug!("Warning: Failed to add task '{}': {}", task_input.title, e);
                } else {
                    debug!("  ✓ Added task: {}", task_input.title);
                }
            }
        }
        Err(e) => {
            debug!("❌ CRITICAL ERROR: Failed to generate tasks from AI: {}", e);
            debug!("🔧 This application requires a working Gemini API connection.");
            debug!("📋 Please ensure:");
            debug!("   • GEMINI_API_KEY environment variable is set correctly");
            debug!("   • Gemini CLI is installed and configured");
            debug!("   • Your API key has proper permissions");
            debug!("   • You have internet connectivity");

            // Shutdown managers before exiting
            let _ = mcp_client.shutdown().await;
            let _ = mcp_handle.await;

            return Err(format!("MCP integration failed: {}", e).into());
        }
    }

    // Initialize Project Manager with MCP integration
    debug!("📋 Starting Project Manager...");
    let project_manager = ProjectManager::new(project.clone());
    let project_command_tx = project_manager.get_command_sender();
    let _project_event_rx = project_manager.subscribe_to_events();

    // Save project configuration
    let config_path = project_path.join("orchy.json");
    let project_json = serde_json::to_string_pretty(&project)?;
    fs::write(&config_path, project_json)?;

    debug!("✅ Project '{}' created successfully!", name);
    debug!("📁 Project saved to: {}", config_path.display());
    debug!("🆔 Project ID: {}", project.id);
    debug!("📊 Tasks created: {}", project.tasks.len());

    // Shutdown managers
    let _ = mcp_client.shutdown().await;
    let _ = project_command_tx.send(ProjectCommand::Shutdown);

    // Wait for MCP manager to shutdown
    let _ = mcp_handle.await;

    debug!("🎉 Project setup complete! You can now:");
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
