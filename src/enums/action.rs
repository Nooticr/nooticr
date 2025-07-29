use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;
use tokio::{fs, io::AsyncWriteExt};
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Write { path: String, content: String },
    Read { path: String },
    Delete { path: String },
    Update { path: String, content: String },
    Replace { path: String, old_content: String, new_content: String },
    Move { old_path: String, new_path: String },
    Copy { old_path: String, new_path: String },
    RunCommand { command: String, env: Option<Vec<(String, String)>> },
}

impl Action {
    pub async fn execute(&self) -> Result<(), std::io::Error> {
        match self {
            Action::Write { path, content } => {
                let file_path = PathBuf::from(path);

                // Create parent directories if they don't exist
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).await?;
                }

                let mut file = fs::File::create(&file_path).await?;
                file.write_all(content.as_bytes()).await?;
                file.flush().await?;
                debug!("Wrote to {}: {}", path, content);
            }

            Action::Read { path } => {
                let file_path = PathBuf::from(path);
                let content = fs::read_to_string(&file_path).await?;
                debug!("Read from {}: {}", path, content);
            }

            Action::Delete { path } => {
                let file_path = PathBuf::from(path);
                fs::remove_file(&file_path).await?;
                debug!("Deleted {}", path);
            }

            Action::Update { path, content } => {
                let file_path = PathBuf::from(path);
                let mut file = fs::File::create(&file_path).await?;
                file.write_all(content.as_bytes()).await?;
                debug!("Updated {}", path);
            }

            Action::Replace { path, old_content, new_content } => {
                let file_path = PathBuf::from(path);
                let content = fs::read_to_string(&file_path).await?;
                let updated_content = content.replace(old_content, new_content);
                fs::write(&file_path, updated_content).await?;
                debug!("Replaced content in {}", path);
            }

            Action::Move { old_path, new_path } => {
                let old_file_path = PathBuf::from(old_path);
                let new_file_path = PathBuf::from(new_path);
                fs::rename(&old_file_path, &new_file_path).await?;
                debug!("Moved {} to {}", old_path, new_path);
            }

            Action::Copy { old_path, new_path } => {
                let old_file_path = PathBuf::from(old_path);
                let new_file_path = PathBuf::from(new_path);
                let content = fs::read(&old_file_path).await?;
                fs::write(&new_file_path, content).await?;
                debug!("Copied {} to {}", old_path, new_path);
            }

            Action::RunCommand { command, env } => {
                let environment = env.clone().unwrap_or_default();
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .envs(environment)
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .output()
                    .await?;
                debug!("Ran command `{}` with status {}", command, output.status);
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_write_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        let action = Action::Write {
            path: file_path.to_string_lossy().to_string(),
            content: "Hello, World!".to_string(),
        };

        action.execute().await.expect("Write action should succeed");

        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).await.expect("Should read file content");
        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_read_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        // First create a file with content
        fs::write(&file_path, "Test content").await.expect("Should write test file");

        let action = Action::Read {
            path: file_path.to_string_lossy().to_string(),
        };

        // Read action doesn't return content, it just logs it
        action.execute().await.expect("Read action should succeed");

        // Verify the file still exists and has the expected content
        let content = fs::read_to_string(&file_path).await.expect("Should read file content");
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_delete_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        // First create a file
        fs::write(&file_path, "Content to delete").await.expect("Should write test file");
        assert!(file_path.exists());

        let action = Action::Delete {
            path: file_path.to_string_lossy().to_string(),
        };

        action.execute().await.expect("Delete action should succeed");
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_update_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        let action = Action::Update {
            path: file_path.to_string_lossy().to_string(),
            content: "Updated content".to_string(),
        };

        action.execute().await.expect("Update action should succeed");

        let content = fs::read_to_string(&file_path).await.expect("Should read file content");
        assert_eq!(content, "Updated content");
    }

    #[tokio::test]
    async fn test_replace_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        // First create a file with initial content
        fs::write(&file_path, "Hello, World!").await.expect("Should write initial content");

        let action = Action::Replace {
            path: file_path.to_string_lossy().to_string(),
            old_content: "Hello".to_string(),
            new_content: "Goodbye".to_string(),
        };

        action.execute().await.expect("Replace action should succeed");

        let content = fs::read_to_string(&file_path).await.expect("Should read file content");
        assert_eq!(content, "Goodbye, World!");
    }

    #[tokio::test]
    async fn test_move_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let old_path = temp_dir.path().join("test.txt");
        let new_path = temp_dir.path().join("new_test.txt");

        // First create a file
        fs::write(&old_path, "Content to move").await.expect("Should write test file");
        assert!(old_path.exists());

        let action = Action::Move {
            old_path: old_path.to_string_lossy().to_string(),
            new_path: new_path.to_string_lossy().to_string(),
        };

        action.execute().await.expect("Move action should succeed");

        assert!(!old_path.exists());
        assert!(new_path.exists());

        let content = fs::read_to_string(&new_path).await.expect("Should read moved file content");
        assert_eq!(content, "Content to move");
    }

    #[tokio::test]
    async fn test_copy_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");

        // First create a source file
        fs::write(&source_path, "Content to copy").await.expect("Should write source file");

        let action = Action::Copy {
            old_path: source_path.to_string_lossy().to_string(),
            new_path: dest_path.to_string_lossy().to_string(),
        };

        action.execute().await.expect("Copy action should succeed");

        assert!(source_path.exists());
        assert!(dest_path.exists());

        let source_content = fs::read_to_string(&source_path).await.expect("Should read source content");
        let dest_content = fs::read_to_string(&dest_path).await.expect("Should read dest content");
        assert_eq!(source_content, "Content to copy");
        assert_eq!(dest_content, "Content to copy");
    }

    #[tokio::test]
    async fn test_run_command_action() {
        let action = Action::RunCommand {
            command: "echo 'Hello, World!'".to_string(),
            env: None,
        };

        action.execute().await.expect("Command should execute successfully");
    }

    #[tokio::test]
    async fn test_run_command_with_env_action() {
        let env_vars = vec![("TEST_VAR".to_string(), "test_value".to_string())];

        let action = Action::RunCommand {
            command: "echo $TEST_VAR".to_string(),
            env: Some(env_vars),
        };

        action.execute().await.expect("Command with env should execute successfully");
    }

    #[tokio::test]
    async fn test_replace_action_no_match() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        // Create a file with content that won't match the replacement
        fs::write(&file_path, "Original content").await.expect("Should write initial content");

        let action = Action::Replace {
            path: file_path.to_string_lossy().to_string(),
            old_content: "NonExistent".to_string(),
            new_content: "Replacement".to_string(),
        };

        action.execute().await.expect("Replace action should succeed even with no matches");

        let content = fs::read_to_string(&file_path).await.expect("Should read file content");
        assert_eq!(content, "Original content"); // Should remain unchanged
    }

    #[tokio::test]
    async fn test_write_action_creates_directories() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let nested_path = temp_dir.path().join("nested").join("directory").join("test.txt");

        let action = Action::Write {
            path: nested_path.to_string_lossy().to_string(),
            content: "Nested file content".to_string(),
        };

        action.execute().await.expect("Write action should succeed");

        assert!(nested_path.exists());
        let content = fs::read_to_string(&nested_path).await.expect("Should read file content");
        assert_eq!(content, "Nested file content");
    }
}
