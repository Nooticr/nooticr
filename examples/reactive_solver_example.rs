use orchy::enums::{Action, llm_response::Todo};
use orchy::solvers::{ReactiveSolver, SolverConfig};
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Create a custom configuration
    let config = SolverConfig {
        command_timeout: Duration::from_secs(60),
        max_retries_per_action: 2,
        max_error_recovery_attempts: 3,
        working_directory: "/tmp/reactive_solver_test".to_string(),
        include_execution_history: true,
        max_history_entries: 5,
    };

    // Create the ReactiveSolver
    let mut solver = ReactiveSolver::with_config(config);

    // Set the overall goal
    solver.set_goal("Create a simple Rust project with a hello world program".to_string());

    // Create some example todos
    let todo1 = Todo::new(
        "Setup Project Structure".to_string(),
        vec![
            Action::RunCommand {
                command: "mkdir -p /tmp/reactive_solver_test".to_string(),
                env: None,
            },
            Action::RunCommand {
                command: "cd /tmp/reactive_solver_test && cargo init --name hello_world".to_string(),
                env: None,
            },
        ],
    );

    let todo2 = Todo::new(
        "Create Hello World Program".to_string(),
        vec![
            Action::Write {
                path: "src/main.rs".to_string(),
                content: r#"fn main() {
    println!("Hello, World from ReactiveSolver!");
    println!("Current directory: {:?}", std::env::current_dir().unwrap());
}
"#.to_string(),
            },
            Action::RunCommand {
                command: "cargo check".to_string(),
                env: None,
            },
            Action::RunCommand {
                command: "cargo build".to_string(),
                env: None,
            },
        ],
    );

    let todo3 = Todo::new(
        "Test the Program".to_string(),
        vec![
            Action::RunCommand {
                command: "cargo run".to_string(),
                env: None,
            },
        ],
    );

    // Add todos to the solver
    solver.add_todo(todo1);
    solver.add_todo(todo2);
    solver.add_todo(todo3);

    info!("Starting ReactiveSolver with {} todos", solver.remaining_todos());

    // Note: In a real scenario, user modifications would come from another thread
    // For this example, we'll demonstrate adding a modification before running
    // solver.add_user_modification("Add a test that verifies the output".to_string());

    // Run the solver
    match solver.run().await {
        Ok(completed_todos) => {
            info!("Successfully completed {} todos", completed_todos.len());
            
            for (i, todo) in completed_todos.iter().enumerate() {
                info!("Todo {}: {} - Done: {}, Failures: {}", 
                      i + 1, todo.title, todo.is_done(), todo.failure_count());
                
                if todo.has_failures() {
                    for (j, message) in todo.get_failure_messages().iter().enumerate() {
                        info!("  Failure {}: {}", j + 1, message);
                    }
                }
            }
            
            // Print statistics
            let stats = solver.get_stats();
            info!("Solver Statistics:");
            info!("  Total executions: {}", stats.total_executions);
            info!("  Successful: {}", stats.successful_executions);
            info!("  Failed: {}", stats.failed_executions);
            info!("  Average duration: {}ms", stats.average_duration_ms);
        }
        Err(e) => {
            eprintln!("Error running solver: {}", e);
        }
    }

    // Clean up
    let _ = std::fs::remove_dir_all("/tmp/reactive_solver_test");

    Ok(())
}

// Example of how the reactive solver would work in a more complex scenario
async fn advanced_example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut solver = ReactiveSolver::new();
    
    // Set a more complex goal
    solver.set_goal("Build a complete web service with database".to_string());

    // Create a complex todo with multiple steps
    let complex_todo = Todo::with_details(
        "Full Stack Application".to_string(),
        Some("Build a REST API with database integration".to_string()),
        vec![
            Action::RunCommand {
                command: "cargo new web_service --bin".to_string(),
                env: None,
            },
            Action::Write {
                path: "Cargo.toml".to_string(),
                content: r#"[package]
name = "web_service"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
warp = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }
"#.to_string(),
            },
            Action::Write {
                path: "src/main.rs".to_string(),
                content: r#"use warp::Filter;

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
"#.to_string(),
            },
            Action::RunCommand {
                command: "cargo check".to_string(),
                env: None,
            },
        ],
        Some("High".to_string()),
        Some("High".to_string()),
        false,
    );

    solver.add_todo(complex_todo);

    // This would run the solver and handle errors reactively
    let _results = solver.run().await?;

    Ok(())
}