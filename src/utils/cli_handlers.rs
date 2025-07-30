use crate::models::project::Project;
use crate::models::agent::Agent;
use crate::models::task::Task;
use crate::models::issue::Issue;
use crate::enums::{Priority, TechStack};
use crate::utils::cli::*;
use crate::utils::dependency_resolver::DependencyResolver;
use crate::managers::{McpManager, McpClient, McpModel, ProjectManager, ProjectCommand};
use clap::ArgMatches;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use uuid::Uuid;
use tracing::debug;
use tokio::sync::mpsc;

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

/// Execute tasks in dependency order using Project Manager
async fn execute_tasks_in_dependency_order(
    tasks: Vec<Task>,
    project_path: &PathBuf,
    _tech_stack: &TechStack,
    mcp_client: &McpClient,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("🎯 Starting task-by-task development process via Project Manager");
    debug!("📊 Total tasks to execute: {}", tasks.len());
    
    // Load the existing project to get full context including agents
    debug!("📂 Loading existing project from orchy.json...");
    let config_path = project_path.join("orchy.json");
    let mut project = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(project_json) => {
                match serde_json::from_str::<Project>(&project_json) {
                    Ok(p) => {
                        debug!("✅ Successfully loaded project with {} tasks and {} agents", 
                               p.tasks.len(), p.agents.len());
                        p
                    }
                    Err(e) => {
                        debug!("❌ Failed to parse orchy.json: {}", e);
                        return Err(format!("Failed to parse project configuration: {}", e).into());
                    }
                }
            }
            Err(e) => {
                debug!("❌ Failed to read orchy.json: {}", e);
                return Err(format!("Failed to read project configuration: {}", e).into());
            }
        }
    } else {
        debug!("❌ orchy.json not found in project directory!");
        return Err("Project configuration file (orchy.json) not found".into());
    };
    
    // Update the project's tasks with the current task list (in case there were changes)
    project.tasks = tasks;
    
    // Create Project Manager for proper task and agent management
    debug!("🏗️  Creating Project Manager for task execution...");
    let project_manager = ProjectManager::new(project.clone());
    let command_tx = project_manager.get_command_sender();
    let _event_rx = project_manager.subscribe_to_events();
    
    // Project Manager automatically starts its background task in new()
    
    // Validate dependencies first
    debug!("🔍 Validating task dependencies...");
    DependencyResolver::validate_dependencies(&project.tasks)?;
    debug!("✅ All task dependencies are valid");
    
    // Sort tasks by dependencies
    debug!("📋 Sorting tasks by dependency order...");
    let sorted_tasks = DependencyResolver::sort_tasks_by_dependencies(project.tasks.clone())?;
    debug!("✅ Tasks sorted successfully");
    
    // Track completed tasks
    let mut completed_tasks: HashSet<Uuid> = HashSet::new();
    let mut task_name_mapping: HashMap<Uuid, String> = HashMap::new();
    
    // Build task name mapping for dependency tracking
    for task in &sorted_tasks {
        task_name_mapping.insert(task.id, task.title.clone());
    }
    
    debug!("🚀 Beginning task execution in dependency order via Project Manager...");
    
    // Execute tasks one by one using Project Manager's ExecuteTaskWithMcp command
    for (task_index, task) in sorted_tasks.iter().enumerate() {
        debug!("");
        debug!("{}", "=".repeat(80));
        debug!("🎯 EXECUTING TASK {}/{}: {}", task_index + 1, sorted_tasks.len(), task.title);
        debug!("{}", "=".repeat(80));
        debug!("📋 Task Details:");
        debug!("   🆔 ID: {}", task.id);
        debug!("   📝 Description: {}", task.description);
        debug!("   🎚️  Priority: {:?}", task.priority);
        debug!("   📊 Complexity: {:?}/10", task.estimated_complexity);
        debug!("   🏷️  Tags: {:?}", task.tags);
        debug!("   📦 Dependencies: {}", task.depends_on.len());
        
        // Log dependency information
        if !task.depends_on.is_empty() {
            debug!("   ⬅️  Depends on:");
            for (dep_index, dep_id) in task.depends_on.iter().enumerate() {
                if let Some(dep_name) = task_name_mapping.get(dep_id) {
                    let status = if completed_tasks.contains(dep_id) { "✅ COMPLETED" } else { "❌ PENDING" };
                    debug!("      {}. {} ({})", dep_index + 1, dep_name, status);
                } else {
                    debug!("      {}. Unknown task: {} (❌ NOT FOUND)", dep_index + 1, dep_id);
                }
            }
        } else {
            debug!("   🟢 No dependencies - can execute immediately");
        }
        
        // Verify all dependencies are completed
        debug!("🔍 Verifying dependencies are satisfied...");
        if !DependencyResolver::are_dependencies_satisfied(task, &completed_tasks) {
            let error_msg = format!("Task '{}' has unsatisfied dependencies", task.title);
            debug!("❌ {}", error_msg);
            return Err(error_msg.into());
        }
        debug!("✅ All dependencies satisfied, proceeding with task execution");
        
        // Execute task using Project Manager's ExecuteTaskWithMcp command
        debug!("🤖 Executing task via Project Manager...");
        let (response_tx, mut response_rx) = mpsc::unbounded_channel();
        
        if let Err(e) = command_tx.send(ProjectCommand::ExecuteTaskWithMcp {
            task_id: task.id,
            mcp_client: mcp_client.clone(),
            respond_to: response_tx,
        }) {
            debug!("❌ Failed to send ExecuteTaskWithMcp command: {}", e);
            return Err(format!("Failed to send task execution command: {}", e).into());
        }
        
        // Wait for task execution result
        match response_rx.recv().await {
            Some(Ok(_)) => {
                debug!("✅ Task '{}' executed successfully via Project Manager!", task.title);
                completed_tasks.insert(task.id);
                debug!("✅ Task '{}' marked as COMPLETED ({}/{} tasks done)", 
                       task.title, completed_tasks.len(), sorted_tasks.len());
            }
            Some(Err(e)) => {
                debug!("❌ Task '{}' execution failed via Project Manager: {}", task.title, e);
                return Err(format!("Task '{}' execution failed: {}", task.title, e).into());
            }
            None => {
                debug!("❌ Project Manager disconnected during task execution");
                return Err("Project Manager disconnected during task execution".into());
            }
        }
        
        debug!("🎉 Task '{}' completed successfully!", task.title);
        debug!("📊 Progress: {}/{} tasks completed", completed_tasks.len(), sorted_tasks.len());
    }
    
    debug!("");
    debug!("🎉 ALL TASKS COMPLETED SUCCESSFULLY!");
    debug!("📊 Final statistics:");
    debug!("   ✅ Total tasks executed: {}", sorted_tasks.len());
    debug!("   ✅ All dependencies satisfied");
    debug!("   ✅ All actions executed successfully");
    
    // Get final project state from Project Manager and save it
    debug!("📊 Getting final project state from Project Manager...");
    let (project_tx, mut project_rx) = mpsc::unbounded_channel();
    
    if let Err(e) = command_tx.send(ProjectCommand::GetProject {
        respond_to: project_tx,
    }) {
        debug!("⚠️  Failed to get final project state: {}", e);
    } else {
        if let Some(final_project) = project_rx.recv().await {
            debug!("💾 Saving final project state with updated task and agent histories...");
            match save_project(&final_project).await {
                Ok(_) => debug!("✅ Final project state saved successfully"),
                Err(e) => debug!("⚠️  Failed to save final project state: {}", e),
            }
        }
    }
    
    // Shutdown Project Manager
    debug!("🔧 Shutting down Project Manager...");
    let _ = command_tx.send(ProjectCommand::Shutdown);
    
    debug!("🏁 Task-by-task development process complete!");
    
    Ok(())
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

            // Collect acceptance criteria first before consuming tasks (for legacy compatibility)
            let _acceptance_criteria = breakdown_response.tasks.iter()
                .map(|task| task.title.clone())
                .collect::<Vec<_>>();

            // Convert TaskInput to Task and add to project
            debug!("🔄 Converting {} TaskInputs to Tasks and adding to project", breakdown_response.tasks.len());
            for (index, task_input) in breakdown_response.tasks.iter().enumerate() {
                debug!("📝 Processing task {}/{}: '{}'", index + 1, breakdown_response.tasks.len(), task_input.title);
                debug!("   - Description: {}", task_input.description);
                debug!("   - Priority: {:?}", task_input.priority);
                debug!("   - Agent type: {:?}", task_input.agent_type);
                
                let task = Task::from_input(task_input.clone(), None);
                debug!("   - Generated Task ID: {}", task.id);
                
                if let Err(e) = project.add_task(task.clone()) {
                    debug!("❌ Failed to add task '{}': {}", task_input.title, e);
                } else {
                    debug!("✅ Successfully added task '{}' to project", task_input.title);
                }
            }
            
            debug!("📊 Project now has {} tasks total", project.tasks.len());

            // Execute tasks in dependency order using the new task-by-task approach
            debug!("🎯 Starting task-by-task development process...");
            debug!("📋 Converting {} tasks for dependency-ordered execution", project.tasks.len());
            
            // Convert project tasks to owned tasks for processing
            let tasks_for_execution: Vec<Task> = project.tasks.clone();
            
            match execute_tasks_in_dependency_order(
                tasks_for_execution,
                &project_path,
                &tech_stack,
                &mcp_client,
            ).await {
                Ok(_) => {
                    debug!("🎉 All tasks completed successfully!");
                    debug!("📁 Project should now be fully developed in: {:?}", project_path);
                    
                    // Final verification of project structure
                    debug!("🔍 Final project verification:");
                    match std::fs::read_dir(&project_path) {
                        Ok(entries) => {
                            debug!("📋 Final project structure:");
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
                            debug!("❌ Failed to list final project directory: {}", e);
                        }
                    }
                }
                Err(e) => {
                    debug!("❌ Task-by-task development failed: {}", e);
                    debug!("⚠️  Some tasks may have been completed, but the process was interrupted");
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
    debug!("💾 Saving project configuration to: {}", config_path.display());
    
    let project_json = serde_json::to_string_pretty(&project)?;
    debug!("📄 Project JSON size: {} bytes", project_json.len());
    
    fs::write(&config_path, &project_json)?;
    debug!("✅ Project configuration saved successfully");
    
    // Verify file was saved
    if config_path.exists() {
        let file_size = std::fs::metadata(&config_path)?.len();
        debug!("🔍 Verification: orchy.json exists ({} bytes)", file_size);
    } else {
        debug!("⚠️  Verification: orchy.json does not exist after save!");
    }

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
