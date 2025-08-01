use orchy::enums::{Action, LLMResponse};
use orchy::mcp::gemini::GeminiCLI;
use orchy::prompts::Prompts;
use orchy::solvers::{ReactiveSolver, SolverConfig};
use std::time::Duration;
use tempfile::TempDir;

/// Real end-to-end integration test with actual Gemini API calls
/// 
/// This test follows the complete workflow:
/// 1. Send idea_breakdown_user_prompt to Gemini to get tasks
/// 2. Send first task to feature_dev_todo_prompt to get todos
/// 3. Feed todos into ReactiveSolver for execution
/// 
/// Note: This test requires GEMINI_API_KEY environment variable to be set
#[tokio::test]
async fn test_real_gemini_integration_todo_manager() {
    // Skip test if no API key is available
    if std::env::var("GEMINI_API_KEY").is_err() {
        println!("Skipping real Gemini integration test - GEMINI_API_KEY not set");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_dir = temp_dir.path().join("todo_manager_project");
    
    // Step 1: Get task breakdown from idea
    println!("🚀 Step 1: Getting task breakdown from Gemini for 'Make a simple todo list manager'");
    
    let idea = "Make a simple todo list manager";
    let tech_stack = "rust"; // Using Rust for the todo manager
    let additional_context = "Create a command-line todo list manager that can add, list, complete, and delete tasks. Keep it simple with file-based storage.";
    
    let idea_breakdown_response = GeminiCLI::query_structured_from_dir(
        "idea_breakdown_session",
        &Prompts::idea_breakdown_user_prompt(idea, additional_context, vec![], tech_stack),
Some("gemini-2.5-flash"),
        temp_dir.path(),
    ).await.expect("Should get idea breakdown from Gemini");
    
    // Extract tasks from the response
    let tasks = match idea_breakdown_response {
        LLMResponse::IdeaBreakdown { tasks } => {
            println!("✅ Got {} tasks from idea breakdown", tasks.len());
            for (i, task) in tasks.iter().enumerate() {
                println!("  {}. {} (Priority: {}, Complexity: {})", 
                    i + 1, task.title, task.priority, task.complexity);
            }
            tasks
        }
        other => panic!("Expected IdeaBreakdown response, got: {:?}", other),
    };
    
    assert!(!tasks.is_empty(), "Should have received at least one task");
    
    // Step 2: Get feature development todos for the first task
    println!("\n🔧 Step 2: Getting feature development todos for first task: '{}'", tasks[0].title);
    
    let first_task = &tasks[0];
    let feature_dev_response = GeminiCLI::query_structured_from_dir(
        "feature_dev_session",
        &Prompts::feature_dev_todo_prompt(
            &first_task.title,
            tech_stack,
            &[], // existing_files - empty for now
            None, // current_error
            Some(additional_context), // agent_context
        ),
Some("gemini-2.5-flash"),
        temp_dir.path(),
    ).await.expect("Should get feature development todos from Gemini");
    
    // Extract todos from the response
    let todos = match feature_dev_response {
        LLMResponse::FeatureDevelopment { todos } => {
            println!("✅ Got {} todos from feature development", todos.len());
            for (i, todo) in todos.iter().enumerate() {
                println!("  {}. {} ({} actions)", 
                    i + 1, todo.title, todo.actions.len());
                for (j, action) in todo.actions.iter().enumerate() {
                    match action {
                        Action::Write { path, .. } => println!("    {}. Write file: {}", j + 1, path),
                        Action::RunCommand { command, .. } => println!("    {}. Run: {}", j + 1, command),
                        Action::CreateDirectory { path } => println!("    {}. Create dir: {}", j + 1, path),
                        other => println!("    {}. {:?}", j + 1, other),
                    }
                }
            }
            todos
        }
        other => panic!("Expected FeatureDevelopment response, got: {:?}", other),
    };
    
    assert!(!todos.is_empty(), "Should have received at least one todo");
    assert!(!todos[0].actions.is_empty(), "First todo should have at least one action");
    
    // Step 3: Use ReactiveSolver to execute the todos
    println!("\n⚡ Step 3: Executing todos with ReactiveSolver");
    
    // Create solver configuration
    let config = SolverConfig {
        command_timeout: Duration::from_secs(60),
        max_retries_per_action: 2,
        max_error_recovery_attempts: 3,
        working_directory: project_dir.to_string_lossy().to_string(),
        include_execution_history: true,
        max_history_entries: 10,
    };
    
    let mut solver = ReactiveSolver::with_config(config);
    solver.set_goal(format!("Implement: {}", first_task.title));
    
    // Add all todos to the solver
    for todo in todos {
        solver.add_todo(todo);
    }
    
    println!("🏃 Running solver with {} todos...", solver.remaining_todos());
    
    // Execute all todos
    let completed_todos = solver.run().await.expect("Solver should complete successfully");
    
    // Step 4: Verify results
    println!("\n✅ Step 4: Verifying results");
    
    assert!(!completed_todos.is_empty(), "Should have completed at least one todo");
    
    // Check that todos were processed
    for (i, todo) in completed_todos.iter().enumerate() {
        println!("  Todo {}: {} - Done: {}, Failures: {}", 
            i + 1, todo.title, todo.is_done(), todo.failure_count());
        
        if todo.has_failures() {
            println!("    Failures:");
            for (j, failure_msg) in todo.get_failure_messages().iter().enumerate() {
                println!("      {}. {}", j + 1, failure_msg);
            }
        }
    }
    
    // Verify that the project directory was created
    assert!(project_dir.exists(), "Project directory should have been created");
    
    // Check for common files that might have been created
    let common_files = ["Cargo.toml", "src/main.rs", "src/lib.rs", "README.md"];
    let mut files_found = Vec::new();
    
    for file in &common_files {
        let file_path = project_dir.join(file);
        if file_path.exists() {
            files_found.push(file);
            println!("  ✓ Found file: {}", file);
        }
    }
    
    if !files_found.is_empty() {
        println!("Created {} files: {:?}", files_found.len(), files_found);
    }
    
    // Check solver statistics
    let stats = solver.get_stats();
    println!("\n📊 Solver Statistics:");
    println!("  Total executions: {}", stats.total_executions);
    println!("  Successful: {}", stats.successful_executions);
    println!("  Failed: {}", stats.failed_executions);
    println!("  Average duration: {}ms", stats.average_duration_ms);
    
    assert!(stats.total_executions > 0, "Should have executed some actions");
    
    // Verify execution history exists
    let execution_state = solver.get_execution_state();
    assert!(!execution_state.execution_history.is_empty(), "Should have execution history");
    
    println!("\n🎉 Real Gemini integration test completed successfully!");
    println!("   - Generated {} tasks from idea breakdown", tasks.len());
    println!("   - Generated {} todos from feature development", completed_todos.len());
    println!("   - Executed {} total actions", stats.total_executions);
    println!("   - Created project in: {}", project_dir.display());
}

/// Lighter integration test that just verifies the API connection
#[tokio::test]
async fn test_gemini_api_connection() {
    // Skip test if no API key is available
    if std::env::var("GEMINI_API_KEY").is_err() {
        println!("Skipping Gemini API connection test - GEMINI_API_KEY not set");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Simple test to verify Gemini API is working
    let response = GeminiCLI::query_with_session_from_dir(
        "connection_test",
        "Please respond with a simple text message saying 'API connected successfully'",
        Some("gemini-2.5-flash"),
        temp_dir.path(),
    ).await;
    
    match response {
        Ok(text_response) => {
            println!("✅ Gemini API connection successful: {}", text_response);
            // Just verify we got some response - don't be too strict about format
            assert!(!text_response.is_empty(), "Response should not be empty");
        }
        Err(e) => {
            println!("❌ Gemini API connection failed: {}", e);
            panic!("Gemini API connection test failed: {}", e);
        }
    }
}

/// Test the prompts independently
#[tokio::test] 
async fn test_individual_prompts() {
    // Skip test if no API key is available
    if std::env::var("GEMINI_API_KEY").is_err() {
        println!("Skipping individual prompts test - GEMINI_API_KEY not set");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    println!("🧪 Testing idea_breakdown_user_prompt");
    let idea_prompt = Prompts::idea_breakdown_user_prompt(
        "Make a simple calculator", 
        "Command line calculator with basic operations", 
        vec![], 
        "rust"
    );
    
    println!("Generated prompt (first 200 chars): {}", 
        idea_prompt.chars().take(200).collect::<String>());
    
    let response = GeminiCLI::query_structured_from_dir(
        "prompt_test",
        &idea_prompt,
Some("gemini-2.5-flash"),
        temp_dir.path(),
    ).await.expect("Should get response from idea breakdown prompt");
    
    println!("✅ Idea breakdown prompt response: {:?}", response);
    
    // Test feature development prompt
    println!("\n🧪 Testing feature_dev_todo_prompt");
    let feature_prompt = Prompts::feature_dev_todo_prompt(
        "Basic Calculator Operations",
        "rust",
        &[], // existing_files
        None, // current_error
        Some("Command line interface"), // agent_context
    );
    
    println!("Generated prompt (first 200 chars): {}", 
        feature_prompt.chars().take(200).collect::<String>());
    
    let response2 = GeminiCLI::query_structured_from_dir(
        "feature_test", 
        &feature_prompt,
Some("gemini-2.5-flash"),
        temp_dir.path(),
    ).await.expect("Should get response from feature development prompt");
    
    println!("✅ Feature development prompt response: {:?}", response2);
}