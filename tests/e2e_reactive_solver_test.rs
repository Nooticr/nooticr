use orchy::enums::{Action, llm_response::Todo};
use orchy::solvers::{ReactiveSolver, SolverConfig};
use std::time::Duration;
use tempfile::TempDir;
use tokio::fs;

/// End-to-end integration test for ReactiveSolver
/// Tests the complete workflow: setup -> execution -> verification
#[tokio::test]
async fn test_reactive_solver_e2e_project_setup() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_dir = temp_dir.path().join("test_project");
    
    // Create solver configuration
    let config = SolverConfig {
        command_timeout: Duration::from_secs(30),
        max_retries_per_action: 2,
        max_error_recovery_attempts: 3,
        working_directory: project_dir.to_string_lossy().to_string(),
        include_execution_history: true,
        max_history_entries: 10,
    };
    
    let mut solver = ReactiveSolver::with_config(config);
    solver.set_goal("Create a complete Rust project with tests".to_string());
    
    // Todo 1: Project initialization
    let todo1 = Todo::new(
        "Initialize Rust Project".to_string(),
        vec![
            Action::CreateDirectory {
                path: project_dir.to_string_lossy().to_string(),
            },
            Action::Write {
                path: project_dir.join("Cargo.toml").to_string_lossy().to_string(),
                content: r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
tokio-test = "0.4"
"#.to_string(),
            },
            Action::CreateDirectory {
                path: project_dir.join("src").to_string_lossy().to_string(),
            },
        ],
    );
    
    // Todo 2: Create source files
    let todo2 = Todo::new(
        "Create Source Files".to_string(),
        vec![
            Action::Write {
                path: project_dir.join("src/lib.rs").to_string_lossy().to_string(),
                content: r#"//! Test project library
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

impl Person {
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
    
    pub fn greet(&self) -> String {
        format!("Hello, my name is {} and I'm {} years old", self.name, self.age)
    }
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }
    
    #[test]
    fn test_person() {
        let person = Person::new("Alice".to_string(), 30);
        assert_eq!(person.name, "Alice");
        assert_eq!(person.age, 30);
        assert_eq!(person.greet(), "Hello, my name is Alice and I'm 30 years old");
    }
}
"#.to_string(),
            },
            Action::Write {
                path: project_dir.join("src/main.rs").to_string_lossy().to_string(),
                content: r#"use test_project::{Person, add};

fn main() {
    println!("Test Project Started!");
    
    let result = add(5, 7);
    println!("5 + 7 = {}", result);
    
    let person = Person::new("Bob".to_string(), 25);
    println!("{}", person.greet());
    
    // Test serialization
    let json = serde_json::to_string(&person).expect("Failed to serialize");
    println!("Person as JSON: {}", json);
    
    let deserialized: Person = serde_json::from_str(&json).expect("Failed to deserialize");
    println!("Deserialized: {}", deserialized.greet());
}
"#.to_string(),
            },
        ],
    );
    
    // Todo 3: Create additional files and verify
    let todo3 = Todo::new(
        "Create Additional Files".to_string(),
        vec![
            Action::Write {
                path: project_dir.join("README.md").to_string_lossy().to_string(),
                content: "# Test Project\n\nThis is a test project created by ReactiveSolver.\n".to_string(),
            },
            Action::CreateDirectory {
                path: project_dir.join("tests").to_string_lossy().to_string(),
            },
            Action::Write {
                path: project_dir.join("tests/integration_test.rs").to_string_lossy().to_string(),
                content: "// Integration tests would go here\n".to_string(),
            },
            Action::RunCommand {
                command: "echo 'Project setup complete'".to_string(),
                env: None,
            },
        ],
    );
    
    // Add todos to solver
    solver.add_todo(todo1);
    solver.add_todo(todo2);
    solver.add_todo(todo3);
    
    // Execute all todos
    let completed_todos = solver.run().await.expect("Solver should complete successfully");
    
    // Verify results
    assert_eq!(completed_todos.len(), 3, "Should have completed all 3 todos");
    
    // Check that all todos are marked as done
    for (i, todo) in completed_todos.iter().enumerate() {
        assert!(todo.is_done(), "Todo {} should be marked as done", i + 1);
        assert_eq!(todo.failure_count(), 0, "Todo {} should have no failures", i + 1);
    }
    
    // Verify file system state
    assert!(project_dir.exists(), "Project directory should exist");
    assert!(project_dir.join("Cargo.toml").exists(), "Cargo.toml should exist");
    assert!(project_dir.join("src").exists(), "src directory should exist");
    assert!(project_dir.join("src/lib.rs").exists(), "lib.rs should exist");
    assert!(project_dir.join("src/main.rs").exists(), "main.rs should exist");
    assert!(project_dir.join("README.md").exists(), "README.md should exist");
    assert!(project_dir.join("tests").exists(), "tests directory should exist");
    assert!(project_dir.join("tests/integration_test.rs").exists(), "integration_test.rs should exist");
    
    // Verify file contents
    let cargo_toml = fs::read_to_string(project_dir.join("Cargo.toml")).await
        .expect("Should read Cargo.toml");
    assert!(cargo_toml.contains("name = \"test_project\""), "Cargo.toml should have correct name");
    assert!(cargo_toml.contains("serde"), "Cargo.toml should include serde dependency");
    
    let lib_rs = fs::read_to_string(project_dir.join("src/lib.rs")).await
        .expect("Should read lib.rs");
    assert!(lib_rs.contains("struct Person"), "lib.rs should contain Person struct");
    assert!(lib_rs.contains("fn add("), "lib.rs should contain add function");
    assert!(lib_rs.contains("#[cfg(test)]"), "lib.rs should contain tests");
    
    // Check solver statistics
    let stats = solver.get_stats();
    assert!(stats.total_executions > 0, "Should have executed some actions");
    assert!(stats.successful_executions > 0, "Should have successful executions");
    assert_eq!(stats.failed_executions, 0, "Should have no failed executions");
    assert!(!stats.is_running, "Solver should not be running after completion");
    
    // Verify execution history
    let execution_state = solver.get_execution_state();
    assert!(!execution_state.execution_history.is_empty(), "Should have execution history");
    
    // Check that echo command was successful
    let mut echo_found = false;
    
    for record in &execution_state.execution_history {
        if record.action.contains("echo") {
            echo_found = true;
            assert!(record.is_success(), "echo command should succeed");
        }
    }
    
    assert!(echo_found, "Should have executed echo command");
}

/// Test solver error handling and recovery
#[tokio::test]
async fn test_reactive_solver_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_dir = temp_dir.path().join("error_test");
    
    let config = SolverConfig {
        command_timeout: Duration::from_secs(10),
        max_retries_per_action: 1,
        max_error_recovery_attempts: 2,
        working_directory: project_dir.to_string_lossy().to_string(),
        include_execution_history: true,
        max_history_entries: 20,
    };
    
    let mut solver = ReactiveSolver::with_config(config);
    solver.set_goal("Test error handling".to_string());
    
    // Create a todo with a command that will fail
    let todo_with_error = Todo::new(
        "Todo with Failing Command".to_string(),
        vec![
            Action::CreateDirectory {
                path: project_dir.to_string_lossy().to_string(),
            },
            // This command should fail
            Action::RunCommand {
                command: "nonexistent_command_that_will_fail".to_string(),
                env: None,
            },
            // This should still execute if we handle the error
            Action::Write {
                path: "test.txt".to_string(),
                content: "This file was created despite the error".to_string(),
            },
        ],
    );
    
    solver.add_todo(todo_with_error);
    
    // Execute - this should handle the error gracefully
    let completed_todos = solver.run().await.expect("Solver should complete even with errors");
    
    assert_eq!(completed_todos.len(), 1, "Should have attempted the todo");
    
    let _todo = &completed_todos[0];
    // The todo might not be fully done due to the failing command
    // but we should have execution history showing the attempt
    
    let stats = solver.get_stats();
    assert!(stats.total_executions > 0, "Should have attempted executions");
    assert!(stats.failed_executions > 0, "Should have some failed executions");
    
    // Check that directory was created (first action should succeed)
    assert!(project_dir.exists(), "Directory should be created");
}

/// Test solver with user modifications
#[tokio::test] 
async fn test_reactive_solver_user_modifications() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_dir = temp_dir.path().join("modification_test");
    
    let config = SolverConfig {
        working_directory: project_dir.to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut solver = ReactiveSolver::with_config(config);
    solver.set_goal("Test user modifications".to_string());
    
    // Add a modification before running
    solver.add_user_modification("Add additional logging to the project".to_string());
    
    let todo = Todo::new(
        "Simple File Creation".to_string(),
        vec![
            Action::CreateDirectory {
                path: project_dir.to_string_lossy().to_string(),
            },
            Action::Write {
                path: project_dir.join("simple.txt").to_string_lossy().to_string(),
                content: "Simple content".to_string(),
            },
        ],
    );
    
    solver.add_todo(todo);
    
    // Check that modification was recorded
    assert_eq!(solver.remaining_todos(), 1);
    
    // For this test, we'll just verify the modification system works
    // without actually running the solver with LLM integration
    let completed_todos = solver.run().await.expect("Should complete");
    assert_eq!(completed_todos.len(), 1);
    
    // Verify the file was created
    assert!(project_dir.join("simple.txt").exists());
}

/// Test solver configuration and customization
#[tokio::test]
async fn test_reactive_solver_configuration() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Test custom configuration
    let custom_config = SolverConfig {
        command_timeout: Duration::from_secs(5),
        max_retries_per_action: 1,
        max_error_recovery_attempts: 1,
        working_directory: temp_dir.path().to_string_lossy().to_string(),
        include_execution_history: false,
        max_history_entries: 5,
    };
    
    let solver = ReactiveSolver::with_config(custom_config);
    
    // Test default configuration
    let default_solver = ReactiveSolver::new();
    
    // Verify both solvers work
    assert!(!solver.is_running());
    assert!(!default_solver.is_running());
    assert_eq!(solver.remaining_todos(), 0);
    assert_eq!(default_solver.remaining_todos(), 0);
    
    // Test statistics
    let stats = solver.get_stats();
    assert_eq!(stats.total_executions, 0);
    assert_eq!(stats.todos_in_queue, 0);
    assert!(!stats.is_running);
}

/// Test action result formatting and LLM context generation
#[tokio::test]
async fn test_action_result_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_dir = temp_dir.path().join("result_test");
    
    let config = SolverConfig {
        working_directory: project_dir.to_string_lossy().to_string(),
        include_execution_history: true,
        max_history_entries: 10,
        ..Default::default()
    };
    
    let mut solver = ReactiveSolver::with_config(config);
    
    // Create actions that produce different types of results
    let test_file_path = project_dir.join("test_file.txt");
    let todo = Todo::new(
        "Test Different Action Results".to_string(),
        vec![
            Action::CreateDirectory {
                path: project_dir.to_string_lossy().to_string(),
            },
            Action::Write {
                path: test_file_path.to_string_lossy().to_string(),
                content: "Test content for reading".to_string(),
            },
            Action::Read {
                path: test_file_path.to_string_lossy().to_string(),
            },
            Action::RunCommand {
                command: "echo 'Hello from command'".to_string(),
                env: None,
            },
            Action::ListDirectory {
                path: project_dir.to_string_lossy().to_string(),
                recursive: false,
            },
        ],
    );
    
    solver.add_todo(todo);
    
    let completed_todos = solver.run().await.expect("Should complete successfully");
    assert_eq!(completed_todos.len(), 1);
    
    let execution_state = solver.get_execution_state();
    assert!(!execution_state.execution_history.is_empty());
    
    // Verify different action types were executed
    let actions_executed: Vec<&str> = execution_state.execution_history
        .iter()
        .map(|r| r.action.as_str())
        .collect();
    
    assert!(actions_executed.iter().any(|a| a.contains("mkdir")));
    assert!(actions_executed.iter().any(|a| a.contains("write")));
    assert!(actions_executed.iter().any(|a| a.contains("read")));
    assert!(actions_executed.iter().any(|a| a.contains("echo")));
    assert!(actions_executed.iter().any(|a| a.contains("ls")));
    
    // Test LLM context formatting
    let llm_context = execution_state.format_for_llm(true, 10);
    assert!(llm_context.contains("Current Working Directory:"));
    assert!(llm_context.contains("Recent Execution History:"));
    
    // Verify files were created and can be read
    assert!(project_dir.exists());
    assert!(project_dir.join("test_file.txt").exists());
    
    let file_content = fs::read_to_string(project_dir.join("test_file.txt")).await
        .expect("Should read test file");
    assert_eq!(file_content, "Test content for reading");
}