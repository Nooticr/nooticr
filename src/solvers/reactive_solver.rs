use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};

use crate::enums::{Action, action::ActionResult, llm_response::Todo};
use crate::mcp::gemini::GeminiCLI as GeminiCli;

/// Represents the current state of the execution environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    /// Current working directory
    pub working_directory: String,
    /// Last action result
    pub last_result: Option<ActionResult>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Execution history (action -> result)
    pub execution_history: Vec<ExecutionRecord>,
    /// Current step being executed
    pub current_step: usize,
    /// Total steps in current todo
    pub total_steps: usize,
}

/// Record of a single action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,
    /// Action that was executed
    pub action: String,
    /// Result of the action execution
    pub result: ActionResult,
    /// Duration of execution
    pub duration_ms: u64,
    /// Whether the execution was successful
    pub success: bool,
}

/// User input for modifying the current plan
#[derive(Debug, Clone)]
pub struct UserModification {
    /// User's instruction for modification
    pub instruction: String,
    /// Timestamp when modification was requested
    pub timestamp: DateTime<Utc>,
}

/// Configuration for the ReactiveSolver
#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Maximum execution time for each command (in seconds)
    pub command_timeout: Duration,
    /// Maximum number of retry attempts per action
    pub max_retries_per_action: u32,
    /// Maximum number of error recovery attempts
    pub max_error_recovery_attempts: u32,
    /// Working directory for command execution
    pub working_directory: String,
    /// Whether to include detailed execution history in LLM context
    pub include_execution_history: bool,
    /// Maximum history entries to include in LLM context
    pub max_history_entries: usize,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            command_timeout: Duration::from_secs(300), // 5 minutes
            max_retries_per_action: 3,
            max_error_recovery_attempts: 5,
            working_directory: std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
            include_execution_history: true,
            max_history_entries: 10,
        }
    }
}

/// Reactive solver that processes todos by observing state, generating actions, and executing them
pub struct ReactiveSolver {
    /// Current execution state
    execution_state: ExecutionState,
    /// Solver configuration
    config: SolverConfig,
    /// Current todo being processed
    current_todo: Option<Todo>,
    /// Queue of todos to process
    todo_queue: Vec<Todo>,
    /// User modifications pending application
    pending_modifications: Vec<UserModification>,
    /// Whether the solver is currently running
    is_running: bool,
    /// Current goal/objective description
    current_goal: Option<String>,
}

impl ReactiveSolver {
    /// Creates a new ReactiveSolver instance
    pub fn new() -> Self {
        let config = SolverConfig::default();
        Self::with_config(config)
    }
    
    /// Creates a new ReactiveSolver with custom configuration
    pub fn with_config(config: SolverConfig) -> Self {
        let execution_state = ExecutionState {
            working_directory: config.working_directory.clone(),
            last_result: None,
            environment: std::env::vars().collect(),
            execution_history: Vec::new(),
            current_step: 0,
            total_steps: 0,
        };
        
        Self {
            execution_state,
            config,
            current_todo: None,
            todo_queue: Vec::new(),
            pending_modifications: Vec::new(),
            is_running: false,
            current_goal: None,
        }
    }
}

impl ExecutionRecord {
    /// Creates a new execution record
    pub fn new(
        action: String,
        result: ActionResult,
        duration_ms: u64,
        success: bool,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            action,
            result,
            duration_ms,
            success,
        }
    }
    
    /// Checks if this execution was successful
    pub fn is_success(&self) -> bool {
        self.success
    }
    
    /// Checks if this execution had an error
    pub fn has_error(&self) -> bool {
        !self.success
    }
    
    /// Gets a summary of this execution for logging
    pub fn summary(&self) -> String {
        let success_indicator = if self.success { "✓" } else { "✗" };
        format!(
            "Action: {} | {} | Duration: {}ms",
            self.action, success_indicator, self.duration_ms
        )
    }
    
    /// Gets the result as LLM-readable string
    pub fn get_result_string(&self) -> String {
        self.result.to_llm_string()
    }
}

impl ExecutionState {
    /// Updates the state with new execution results
    pub fn update_with_execution(&mut self, record: ExecutionRecord) {
        self.last_result = Some(record.result.clone());
        
        // Keep history bounded
        if self.execution_history.len() >= 50 {
            self.execution_history.remove(0);
        }
        
        self.execution_history.push(record);
    }
    
    /// Checks if the last action had an error
    pub fn has_error(&self) -> bool {
        self.last_result.as_ref().map_or(false, |result| {
            match result {
                ActionResult::CommandOutput { exit_code, .. } => *exit_code != 0,
                _ => false, // Other action types don't typically "fail" in the same way
            }
        })
    }
    
    /// Gets the last error message if available
    pub fn get_last_error(&self) -> Option<String> {
        self.last_result.as_ref().and_then(|result| {
            match result {
                ActionResult::CommandOutput { stderr, exit_code, .. } => {
                    if *exit_code != 0 && !stderr.is_empty() {
                        Some(stderr.clone())
                    } else if *exit_code != 0 {
                        Some(format!("Command failed with exit code: {}", exit_code))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
    }
    
    /// Gets a summary of recent execution history
    pub fn get_recent_history(&self, max_entries: usize) -> Vec<&ExecutionRecord> {
        let start = if self.execution_history.len() > max_entries {
            self.execution_history.len() - max_entries
        } else {
            0
        };
        
        self.execution_history[start..].iter().collect()
    }
    
    /// Formats the current state for LLM context
    pub fn format_for_llm(&self, include_history: bool, max_history: usize) -> String {
        let mut context = format!(
            "Current Working Directory: {}\n",
            self.working_directory
        );
        
        if let Some(result) = &self.last_result {
            let result_str = result.to_llm_string();
            if !result_str.is_empty() {
                context.push_str(&format!("Last Action Result:\n{}\n\n", result_str));
            }
        }
        
        if let Some(error) = self.get_last_error() {
            context.push_str(&format!("Last Error:\n{}\n\n", error));
        }
        
        if include_history && !self.execution_history.is_empty() {
            context.push_str("Recent Execution History:\n");
            for record in self.get_recent_history(max_history) {
                context.push_str(&format!("- {}\n", record.summary()));
                let result_str = record.get_result_string();
                if !result_str.is_empty() {
                    // Indent the result for readability
                    let indented = result_str.lines()
                        .map(|line| format!("  {}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    context.push_str(&format!("{}\n", indented));
                }
            }
            context.push('\n');
        }
        
        context
    }
}

impl UserModification {
    /// Creates a new user modification
    pub fn new(instruction: String) -> Self {
        Self {
            instruction,
            timestamp: Utc::now(),
        }
    }
}

/// Core implementation of ReactiveSolver
impl ReactiveSolver {
    /// Adds a todo to the processing queue
    pub fn add_todo(&mut self, todo: Todo) {
        info!("Adding todo to queue: {}", todo.title);
        self.todo_queue.push(todo);
    }
    
    /// Adds multiple todos to the processing queue
    pub fn add_todos(&mut self, todos: Vec<Todo>) {
        for todo in todos {
            self.add_todo(todo);
        }
    }
    
    /// Sets the current goal/objective
    pub fn set_goal(&mut self, goal: String) {
        info!("Setting goal: {}", goal);
        self.current_goal = Some(goal);
    }
    
    /// Adds a user modification to be applied
    pub fn add_user_modification(&mut self, instruction: String) {
        info!("User modification requested: {}", instruction);
        let modification = UserModification::new(instruction);
        self.pending_modifications.push(modification);
    }
    
    /// Gets the current execution state
    pub fn get_execution_state(&self) -> &ExecutionState {
        &self.execution_state
    }
    
    /// Checks if the solver is currently running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
    
    /// Gets the current todo being processed
    pub fn current_todo(&self) -> Option<&Todo> {
        self.current_todo.as_ref()
    }
    
    /// Gets the number of remaining todos in queue
    pub fn remaining_todos(&self) -> usize {
        self.todo_queue.len()
    }
    
    
    /// Generates the next action using LLM based on current state
    async fn generate_next_action(
        &self,
        error_context: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let state_context = self.execution_state.format_for_llm(
            self.config.include_execution_history,
            self.config.max_history_entries,
        );
        
        let todo_context = if let Some(todo) = &self.current_todo {
            format!(
                "Current Todo: {}\nActions to complete:\n{}",
                todo.title,
                todo.actions.iter()
                    .enumerate()
                    .map(|(i, action)| format!("{}. {:?}", i + 1, action))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            "No current todo".to_string()
        };
        
        let goal_context = self.current_goal
            .as_ref()
            .map(|g| format!("Goal: {}", g))
            .unwrap_or_else(|| "No specific goal set".to_string());
        
        let prompt = if let Some(error) = error_context {
            format!(
                "You are a reactive problem solver. An error occurred during execution.\n\n{}\n\n{}\n\n{}\n\nERROR CONTEXT:\n{}\n\nPlease generate a shell command to fix this error. Respond with ONLY the command, no explanations.",
                goal_context, todo_context, state_context, error
            )
        } else {
            format!(
                "You are a reactive problem solver. Generate the next command to progress toward the goal.\n\n{}\n\n{}\n\n{}\n\nPlease generate the next shell command to progress. Respond with ONLY the command, no explanations.",
                goal_context, todo_context, state_context
            )
        };
        
        debug!("Sending prompt to LLM: {}", prompt);
        
        // Use GeminiCLI to generate response
        let response = GeminiCli::query(&prompt).await.map_err(|e| {
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;
        
        // Extract command from response
        let command = response.trim().to_string();
        info!("LLM generated command: {}", command);
        
        Ok(command)
    }
    
    /// Applies pending user modifications to the current plan
    async fn apply_user_modifications(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.pending_modifications.is_empty() {
            return Ok(());
        }
        
        info!("Applying {} user modifications", self.pending_modifications.len());
        
        let current_state = self.execution_state.format_for_llm(
            self.config.include_execution_history,
            self.config.max_history_entries,
        );
        
        let todo_context = if let Some(todo) = &self.current_todo {
            format!("Current Todo: {}\nCurrent Actions: {:?}", todo.title, todo.actions)
        } else {
            "No current todo".to_string()
        };
        
        for modification in &self.pending_modifications {
            let prompt = format!(
                "You are a reactive problem solver. The user wants to modify the current plan.\n\nCURRENT STATE:\n{}\n\nCURRENT PLAN:\n{}\n\nUSER MODIFICATION REQUEST:\n{}\n\nPlease provide a modified set of actions in JSON format as an array of Action objects. Consider the user's request and current state.",
                current_state, todo_context, modification.instruction
            );
            
            debug!("Sending modification prompt to LLM: {}", prompt);
            
            let response = GeminiCli::query(&prompt).await.map_err(|e| {
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;
            
            // Try to parse the response as new actions
            match serde_json::from_str::<Vec<Action>>(&response) {
                Ok(new_actions) => {
                    if let Some(ref mut todo) = self.current_todo {
                        info!("Updating todo actions based on user modification");
                        todo.actions = new_actions;
                        todo.reset_failures(); // Reset since plan changed
                    }
                }
                Err(e) => {
                    warn!("Failed to parse LLM response as actions: {}. Response: {}", e, response);
                }
            }
        }
        
        // Clear applied modifications
        self.pending_modifications.clear();
        Ok(())
    }
    
    /// Processes a single action from the current todo
    async fn process_action(&mut self, action: &Action) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        info!("Processing action: {:?}", action);
        let start_time = std::time::Instant::now();
        
        // Set environment variables for RunCommand actions
        if let Action::RunCommand { env, .. } = action {
            if let Some(env_vars) = env {
                for (key, value) in env_vars {
                    self.execution_state.environment.insert(key.clone(), value.clone());
                }
            }
        }
        
        // Execute the action using Action::execute()
        match action.execute().await {
            Ok(result) => {
                let duration = start_time.elapsed();
                let success = match &result {
                    ActionResult::CommandOutput { exit_code, .. } => *exit_code == 0,
                    _ => true, // Non-command actions are considered successful if they don't error
                };
                
                let action_description = match action {
                    Action::Write { path, .. } => format!("write {}", path),
                    Action::Read { path } => format!("read {}", path),
                    Action::Delete { path } => format!("delete {}", path),
                    Action::Update { path, .. } => format!("update {}", path),
                    Action::Replace { path, .. } => format!("replace in {}", path),
                    Action::Move { old_path, new_path } => format!("move {} to {}", old_path, new_path),
                    Action::Copy { old_path, new_path } => format!("copy {} to {}", old_path, new_path),
                    Action::RunCommand { command, .. } => command.clone(),
                    Action::Grep { pattern, path, .. } => format!("grep '{}' in {}", pattern, path),
                    Action::CreateDirectory { path } => format!("mkdir {}", path),
                    Action::RemoveDirectory { path, .. } => format!("rmdir {}", path),
                    Action::ListDirectory { path, .. } => format!("ls {}", path),
                    Action::CreateSymlink { target, link_path } => format!("ln -s {} {}", target, link_path),
                    Action::SetPermissions { path, permissions } => format!("chmod {} {}", permissions, path),
                    Action::Append { path, .. } => format!("append to {}", path),
                    Action::Backup { path, .. } => format!("backup {}", path),
                    Action::Download { url, destination } => format!("download {} to {}", url, destination),
                    Action::Extract { archive_path, destination } => format!("extract {} to {}", archive_path, destination),
                    Action::Archive { archive_path, .. } => format!("create archive {}", archive_path),
                    Action::Watch { path, .. } => format!("watch {}", path),
                };
                
                let record = ExecutionRecord::new(
                    action_description,
                    result.clone(),
                    duration.as_millis() as u64,
                    success,
                );
                
                self.execution_state.update_with_execution(record);
                
                if success {
                    info!("Action completed successfully");
                } else {
                    warn!("Action completed with errors");
                }
                
                Ok(success)
            }
            Err(e) => {
                let duration = start_time.elapsed();
                let action_description = format!("{:?}", action);
                
                let record = ExecutionRecord::new(
                    action_description,
                    ActionResult::CommandOutput {
                        stdout: String::new(),
                        stderr: e.to_string(),
                        exit_code: -1,
                    },
                    duration.as_millis() as u64,
                    false,
                );
                
                self.execution_state.update_with_execution(record);
                error!("Action failed: {}", e);
                
                Err(Box::new(e))
            }
        }
    }
    
    /// Main reactive solving loop - processes a single todo until completion
    pub async fn solve_todo(&mut self, mut todo: Todo) -> Result<Todo, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting to solve todo: {}", todo.title);
        
        self.current_todo = Some(todo.clone());
        self.execution_state.current_step = 0;
        self.execution_state.total_steps = todo.actions.len();
        
        let mut error_recovery_attempts = 0;
        let mut current_action_index = 0;
        
        while current_action_index < todo.actions.len() && !todo.done {
            // Apply any pending user modifications
            if !self.pending_modifications.is_empty() {
                self.apply_user_modifications().await?;
                // Refresh todo from current_todo since it may have been modified
                if let Some(ref updated_todo) = self.current_todo {
                    todo = updated_todo.clone();
                }
            }
            
            self.execution_state.current_step = current_action_index + 1;
            
            let action = &todo.actions[current_action_index];
            info!("Processing action {}/{}: {:?}", 
                  current_action_index + 1, todo.actions.len(), action);
            
            // Process the action
            match self.process_action(action).await {
                Ok(success) => {
                    if success {
                        info!("Action completed successfully");
                        current_action_index += 1;
                        error_recovery_attempts = 0; // Reset on success
                    } else {
                        // Action failed, try error recovery
                        if error_recovery_attempts < self.config.max_error_recovery_attempts {
                            info!("Action failed, attempting error recovery (attempt {})", 
                                  error_recovery_attempts + 1);
                            
                            let error_context = self.execution_state.get_last_error()
                                .unwrap_or_else(|| "Unknown error".to_string());
                            
                            match self.generate_next_action(Some(&error_context)).await {
                                Ok(fix_command) => {
                                    info!("Generated fix command: {}", fix_command);
                                    
                                    // Create a RunCommand action from the generated fix command
                                    let fix_action = Action::RunCommand {
                                        command: fix_command.clone(),
                                        env: None,
                                    };
                                    
                                    match self.process_action(&fix_action).await {
                                        Ok(success) => {
                                            if success {
                                                info!("Fix command succeeded, retrying original action");
                                                error_recovery_attempts += 1;
                                                // Don't increment current_action_index, retry the same action
                                            } else {
                                                error_recovery_attempts += 1;
                                                let error_msg = self.execution_state.get_last_error()
                                                    .unwrap_or_else(|| "Fix command failed".to_string());
                                                todo.record_failure_with_details(
                                                    format!("Fix command failed: {}", fix_command),
                                                    Some(error_msg),
                                                    Some(current_action_index),
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            error_recovery_attempts += 1;
                                            todo.record_failure_with_details(
                                                format!("Fix command execution error: {}", e),
                                                None,
                                                Some(current_action_index),
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    error_recovery_attempts += 1;
                                    todo.record_failure_with_details(
                                        format!("Failed to generate fix: {}", e),
                                        None,
                                        Some(current_action_index),
                                    );
                                }
                            }
                        } else {
                            error!("Max error recovery attempts reached for action: {:?}", action);
                            let error_msg = self.execution_state.get_last_error();
                            todo.record_failure_with_details(
                                "Max error recovery attempts exceeded".to_string(),
                                error_msg,
                                Some(current_action_index),
                            );
                            break; // Give up on this todo
                        }
                    }
                }
                Err(e) => {
                    error!("Error processing action: {}", e);
                    todo.record_failure_with_details(
                        format!("Action processing error: {}", e),
                        None,
                        Some(current_action_index),
                    );
                    
                    if error_recovery_attempts < self.config.max_error_recovery_attempts {
                        error_recovery_attempts += 1;
                    } else {
                        break; // Give up
                    }
                }
            }
        }
        
        // Mark todo as done if all actions completed successfully
        if current_action_index >= todo.actions.len() {
            todo.mark_done();
            info!("Todo completed successfully: {}", todo.title);
        } else {
            warn!("Todo failed to complete: {}", todo.title);
        }
        
        self.current_todo = None;
        Ok(todo)
    }
    
    /// Runs the reactive solver on all queued todos
    pub async fn run(&mut self) -> Result<Vec<Todo>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting ReactiveSolver with {} todos in queue", self.todo_queue.len());
        
        if self.is_running {
            return Err("Solver is already running".into());
        }
        
        self.is_running = true;
        let mut completed_todos = Vec::new();
        
        while let Some(todo) = self.todo_queue.pop() {
            info!("Processing todo: {} (remaining: {})", todo.title, self.todo_queue.len());
            
            match self.solve_todo(todo).await {
                Ok(completed_todo) => {
                    completed_todos.push(completed_todo);
                }
                Err(e) => {
                    error!("Failed to solve todo: {}", e);
                    // Continue with next todo instead of stopping
                }
            }
        }
        
        self.is_running = false;
        info!("ReactiveSolver completed. Processed {} todos", completed_todos.len());
        
        Ok(completed_todos)
    }
    
    /// Stops the reactive solver (can be called from another thread)
    pub fn stop(&mut self) {
        info!("Stopping ReactiveSolver");
        self.is_running = false;
        self.todo_queue.clear();
        self.current_todo = None;
    }
    
    /// Gets statistics about the solver's performance
    pub fn get_stats(&self) -> SolverStats {
        let total_executions = self.execution_state.execution_history.len();
        let successful_executions = self.execution_state.execution_history
            .iter()
            .filter(|r| r.is_success())
            .count();
        
        let total_duration: u64 = self.execution_state.execution_history
            .iter()
            .map(|r| r.duration_ms)
            .sum();
        
        SolverStats {
            total_executions,
            successful_executions,
            failed_executions: total_executions - successful_executions,
            total_duration_ms: total_duration,
            average_duration_ms: if total_executions > 0 {
                total_duration / total_executions as u64
            } else {
                0
            },
            todos_in_queue: self.todo_queue.len(),
            is_running: self.is_running,
        }
    }
}

/// Statistics about solver performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverStats {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub total_duration_ms: u64,
    pub average_duration_ms: u64,
    pub todos_in_queue: usize,
    pub is_running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Action;
    
    // GeminiCLI is a static struct, no need for creation function
    
    #[test]
    fn test_solver_creation() {
        let solver = ReactiveSolver::new();
        
        assert!(!solver.is_running());
        assert_eq!(solver.remaining_todos(), 0);
        assert!(solver.current_todo().is_none());
    }
    
    #[test]
    fn test_todo_queue_management() {
        let mut solver = ReactiveSolver::new();
        
        let todo1 = Todo::new("Test Todo 1".to_string(), vec![
            Action::Write {
                path: "test1.txt".to_string(),
                content: "Hello World".to_string(),
            }
        ]);
        
        let todo2 = Todo::new("Test Todo 2".to_string(), vec![
            Action::RunCommand {
                command: "echo test".to_string(),
                env: None,
            }
        ]);
        
        solver.add_todo(todo1);
        solver.add_todo(todo2);
        
        assert_eq!(solver.remaining_todos(), 2);
        
        solver.set_goal("Test goal".to_string());
        assert!(solver.current_goal.is_some());
    }
    
    #[test]
    fn test_user_modifications() {
        let mut solver = ReactiveSolver::new();
        
        solver.add_user_modification("Change the approach".to_string());
        assert_eq!(solver.pending_modifications.len(), 1);
        
        let modification = &solver.pending_modifications[0];
        assert_eq!(modification.instruction, "Change the approach");
    }
    
    #[test]
    fn test_execution_state_formatting() {
        let mut state = ExecutionState {
            working_directory: "/tmp".to_string(),
            last_result: Some(ActionResult::CommandOutput {
                stdout: "Success".to_string(),
                stderr: "Warning: deprecated".to_string(),
                exit_code: 0,
            }),
            environment: HashMap::new(),
            execution_history: Vec::new(),
            current_step: 1,
            total_steps: 3,
        };
        
        let record = ExecutionRecord::new(
            "echo test".to_string(),
            ActionResult::CommandOutput {
                stdout: "test".to_string(),
                stderr: "".to_string(),
                exit_code: 0,
            },
            100,
            true,
        );
        
        state.update_with_execution(record);
        
        let formatted = state.format_for_llm(true, 5);
        assert!(formatted.contains("Current Working Directory: /tmp"));
        assert!(formatted.contains("Last Action Result:"));
        assert!(formatted.contains("Recent Execution History:"));
    }
    
    #[test]
    fn test_execution_record() {
        let record = ExecutionRecord::new(
            "ls -la".to_string(),
            ActionResult::CommandOutput {
                stdout: "file1.txt\nfile2.txt".to_string(),
                stderr: "".to_string(),
                exit_code: 0,
            },
            150,
            true,
        );
        
        assert!(record.is_success());
        assert!(!record.has_error());
        assert!(record.summary().contains("ls -la"));
        assert!(record.summary().contains("✓"));
        assert!(record.summary().contains("Duration: 150ms"));
    }
    
    #[test]
    fn test_solver_config() {
        let config = SolverConfig::default();
        
        assert_eq!(config.command_timeout, Duration::from_secs(300));
        assert_eq!(config.max_retries_per_action, 3);
        assert_eq!(config.max_error_recovery_attempts, 5);
        assert!(config.include_execution_history);
        assert_eq!(config.max_history_entries, 10);
    }
    
    #[test]
    fn test_solver_stats() {
        let solver = ReactiveSolver::new();
        
        let stats = solver.get_stats();
        
        assert_eq!(stats.total_executions, 0);
        assert_eq!(stats.successful_executions, 0);
        assert_eq!(stats.failed_executions, 0);
        assert_eq!(stats.todos_in_queue, 0);
        assert!(!stats.is_running);
    }
    
    #[test]
    fn test_error_detection() {
        let mut state = ExecutionState {
            working_directory: "/tmp".to_string(),
            last_result: Some(ActionResult::CommandOutput {
                stdout: "".to_string(),
                stderr: "Error occurred".to_string(),
                exit_code: 1,
            }),
            environment: HashMap::new(),
            execution_history: Vec::new(),
            current_step: 0,
            total_steps: 0,
        };
        
        assert!(state.has_error());
        
        state.last_result = Some(ActionResult::CommandOutput {
            stdout: "Success".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
        });
        assert!(!state.has_error());
    }
}