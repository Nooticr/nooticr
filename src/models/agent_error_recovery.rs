use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::path::PathBuf;

/// Detailed error information captured during action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionError {
    pub id: Uuid,
    pub action_type: String,
    pub action_description: String,
    pub error_message: String,
    pub error_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub working_directory: Option<PathBuf>,
    pub environment_vars: Option<std::collections::HashMap<String, String>>,
    pub timestamp: DateTime<Utc>,
    pub retry_count: u32,
}

/// Context information for error recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryContext {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub agent_type: String,
    pub task_id: Option<Uuid>,
    pub task_title: Option<String>,
    pub project_path: PathBuf,
    pub tech_stack: String,
    pub relevant_files: Vec<FileContext>,
    pub action_error: ActionError,
    pub previous_actions: Vec<String>,
    pub project_structure: Vec<String>,
}

/// File content and metadata for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    pub path: PathBuf,
    pub relative_path: String,
    pub content: String,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub is_generated: bool,
}

/// Recovery action suggested by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    pub id: Uuid,
    pub action_type: String,
    pub description: String,
    pub command: Option<String>,
    pub file_path: Option<PathBuf>,
    pub content: Option<String>,
    pub priority: u8, // 1-10, higher is more important
    pub estimated_success_rate: Option<f32>, // 0.0-1.0
}

/// Response from the error recovery LLM prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryResponse {
    pub analysis: String,
    pub root_cause: String,
    pub confidence_level: f32, // 0.0-1.0
    pub recovery_actions: Vec<RecoveryAction>,
    pub preventive_measures: Vec<String>,
    pub should_retry_original: bool,
    pub estimated_recovery_time: Option<u32>, // minutes
}

impl ActionError {
    /// Create a new action error from execution failure
    pub fn new(
        action_type: impl Into<String>,
        action_description: impl Into<String>,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            action_type: action_type.into(),
            action_description: action_description.into(),
            error_message: error_message.into(),
            error_code: None,
            stdout: None,
            stderr: None,
            working_directory: None,
            environment_vars: None,
            timestamp: Utc::now(),
            retry_count: 0,
        }
    }

    /// Create action error from command execution failure
    pub fn from_command_failure(
        command: impl Into<String>,
        exit_code: i32,
        stdout: impl Into<String>,
        stderr: impl Into<String>,
        working_dir: Option<PathBuf>,
    ) -> Self {
        let command_str = command.into();
        Self {
            id: Uuid::new_v4(),
            action_type: "CommandExecution".to_string(),
            action_description: format!("Execute command: {}", command_str),
            error_message: format!("Command '{}' failed with exit code {}", command_str, exit_code),
            error_code: Some(exit_code),
            stdout: Some(stdout.into()),
            stderr: Some(stderr.into()),
            working_directory: working_dir,
            environment_vars: Some(std::env::vars().collect()),
            timestamp: Utc::now(),
            retry_count: 0,
        }
    }

    /// Create action error from file operation failure
    pub fn from_file_operation_failure(
        operation: impl Into<String>,
        file_path: impl Into<PathBuf>,
        error: impl Into<String>,
    ) -> Self {
        let file_path_buf = file_path.into();
        let operation_str = operation.into();
        Self {
            id: Uuid::new_v4(),
            action_type: "FileOperation".to_string(),
            action_description: format!("File operation '{}' on {}", operation_str, file_path_buf.display()),
            error_message: error.into(),
            error_code: None,
            stdout: None,
            stderr: None,
            working_directory: file_path_buf.parent().map(|p| p.to_path_buf()),
            environment_vars: None,
            timestamp: Utc::now(),
            retry_count: 0,
        }
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Check if error should be retried based on retry count
    pub fn should_retry(&self, max_retries: u32) -> bool {
        self.retry_count < max_retries
    }
}

impl FileContext {
    /// Create file context from path
    pub async fn from_path(path: PathBuf, project_root: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = tokio::fs::metadata(&path).await?;
        let content = tokio::fs::read_to_string(&path).await?;
        
        let relative_path = path.strip_prefix(project_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        Ok(Self {
            path: path.clone(),
            relative_path,
            content,
            size: metadata.len(),
            modified: DateTime::from(metadata.modified()?),
            is_generated: Self::is_generated_file(&path),
        })
    }

    /// Check if file is likely generated (based on common patterns)
    fn is_generated_file(path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        
        // Common generated file patterns
        path_str.contains("node_modules") ||
        path_str.contains("target") ||
        path_str.contains("build") ||
        path_str.contains("dist") ||
        path_str.contains(".git") ||
        path_str.ends_with(".lock") ||
        path_str.ends_with(".map") ||
        path_str.contains("__pycache__") ||
        path_str.contains(".cache")
    }

    /// Truncate content if too large for LLM context
    pub fn truncate_content(&mut self, max_size: usize) {
        if self.content.len() > max_size {
            let truncated = &self.content[..max_size];
            self.content = format!("{}...\n\n[Content truncated - original size: {} characters]", 
                                 truncated, self.size);
        }
    }
}

impl RecoveryAction {
    /// Create a command-based recovery action
    pub fn command(
        description: impl Into<String>,
        command: impl Into<String>,
        priority: u8,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            action_type: "Command".to_string(),
            description: description.into(),
            command: Some(command.into()),
            file_path: None,
            content: None,
            priority,
            estimated_success_rate: None,
        }
    }

    /// Create a file modification recovery action
    pub fn file_modification(
        description: impl Into<String>,
        file_path: PathBuf,
        content: impl Into<String>,
        priority: u8,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            action_type: "FileModification".to_string(),
            description: description.into(),
            command: None,
            file_path: Some(file_path),
            content: Some(content.into()),
            priority,
            estimated_success_rate: None,
        }
    }

    /// Sort recovery actions by priority (highest first)
    pub fn sort_by_priority(actions: &mut Vec<RecoveryAction>) {
        actions.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_action_error_creation() {
        let error = ActionError::new(
            "TestAction",
            "Test action description",
            "Test error message"
        );

        assert_eq!(error.action_type, "TestAction");
        assert_eq!(error.action_description, "Test action description");
        assert_eq!(error.error_message, "Test error message");
        assert_eq!(error.retry_count, 0);
    }

    #[test]
    fn test_command_failure_error() {
        let error = ActionError::from_command_failure(
            "npm install",
            1,
            "Installing packages...",
            "Error: package not found",
            Some(PathBuf::from("/project/frontend"))
        );

        assert_eq!(error.action_type, "CommandExecution");
        assert!(error.error_message.contains("npm install"));
        assert_eq!(error.error_code, Some(1));
        assert_eq!(error.stdout, Some("Installing packages...".to_string()));
        assert_eq!(error.stderr, Some("Error: package not found".to_string()));
    }

    #[test]
    fn test_retry_logic() {
        let mut error = ActionError::new("Test", "Test", "Test");
        
        assert!(error.should_retry(3));
        
        error.increment_retry();
        assert_eq!(error.retry_count, 1);
        assert!(error.should_retry(3));
        
        error.increment_retry();
        error.increment_retry();
        assert_eq!(error.retry_count, 3);
        assert!(!error.should_retry(3));
    }

    #[test]
    fn test_recovery_action_creation() {
        let action = RecoveryAction::command(
            "Fix npm dependencies",
            "rm -rf node_modules && npm install",
            8
        );

        assert_eq!(action.action_type, "Command");
        assert_eq!(action.description, "Fix npm dependencies");
        assert_eq!(action.command, Some("rm -rf node_modules && npm install".to_string()));
        assert_eq!(action.priority, 8);
    }

    #[test]
    fn test_file_context_generation_detection() {
        assert!(FileContext::is_generated_file(&PathBuf::from("node_modules/package/index.js")));
        assert!(FileContext::is_generated_file(&PathBuf::from("target/debug/app")));
        assert!(FileContext::is_generated_file(&PathBuf::from("package-lock.json")));
        assert!(!FileContext::is_generated_file(&PathBuf::from("src/main.rs")));
        assert!(!FileContext::is_generated_file(&PathBuf::from("package.json")));
    }

    #[test]
    fn test_recovery_action_priority_sorting() {
        let mut actions = vec![
            RecoveryAction::command("Low priority", "echo low", 3),
            RecoveryAction::command("High priority", "echo high", 9),
            RecoveryAction::command("Medium priority", "echo medium", 5),
        ];

        RecoveryAction::sort_by_priority(&mut actions);

        assert_eq!(actions[0].priority, 9);
        assert_eq!(actions[1].priority, 5);
        assert_eq!(actions[2].priority, 3);
    }
}