use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use tokio::{fs, io::AsyncWriteExt};
use tracing::debug;
use std::os::unix::fs::PermissionsExt;
use regex::Regex;
use std::pin::Pin;
use std::future::Future;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Write {
        path: String,
        content: String,
    },
    Read {
        path: String,
    },
    Delete {
        path: String,
    },
    Update {
        path: String,
        content: String,
    },
    Replace {
        path: String,
        old_content: String,
        new_content: String,
    },
    Move {
        old_path: String,
        new_path: String,
    },
    Copy {
        old_path: String,
        new_path: String,
    },
    RunCommand {
        command: String,
        env: Option<Vec<(String, String)>>,
    },
    /// Search for text patterns in files using grep-like functionality
    Grep {
        pattern: String,
        path: String,
        recursive: bool,
        case_sensitive: bool,
    },
    /// Create a directory (and parent directories if needed)
    CreateDirectory {
        path: String,
    },
    /// Remove a directory (and all contents if recursive)
    RemoveDirectory {
        path: String,
        recursive: bool,
    },
    /// List directory contents
    ListDirectory {
        path: String,
        recursive: bool,
    },
    /// Create a symbolic link
    CreateSymlink {
        target: String,
        link_path: String,
    },
    /// Set file permissions (Unix-style)
    SetPermissions {
        path: String,
        permissions: String, // e.g., "755", "644"
    },
    /// Append content to a file (instead of overwriting)
    Append {
        path: String,
        content: String,
    },
    /// Create a backup of a file
    Backup {
        path: String,
        backup_suffix: Option<String>, // e.g., ".bak", ".backup"
    },
    /// Download a file from a URL
    Download {
        url: String,
        destination: String,
    },
    /// Extract/decompress an archive (zip, tar, etc.)
    Extract {
        archive_path: String,
        destination: String,
    },
    /// Create an archive from files/directories
    Archive {
        source_paths: Vec<String>,
        archive_path: String,
        format: String, // "zip", "tar", "tar.gz"
    },
    /// Watch a file or directory for changes
    Watch {
        path: String,
        duration_seconds: u64,
    },
}

impl Action {
    /// Deserialize a JSON string containing an array of actions
    ///
    /// This is used to parse the output from `conflict_resolution_user_prompt`
    /// and `feature_development_user_prompt`.
    ///
    /// # Example
    /// ```rust
    /// use orchy::enums::Action;
    ///
    /// let json = r#"[
    ///     {
    ///         "Write": {
    ///             "path": "src/main.rs",
    ///             "content": "fn main() { println!(\"Hello, world!\"); }"
    ///         }
    ///     },
    ///     {
    ///         "Read": {
    ///             "path": "src/lib.rs"
    ///         }
    ///     }
    /// ]"#;
    ///
    /// let actions = Action::from_json_array(json).unwrap();
    /// assert_eq!(actions.len(), 2);
    /// ```
    pub fn from_json_array(json_str: &str) -> Result<Vec<Action>, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Parse and execute actions from a JSON string in one step
    ///
    /// This is a convenience function for processing prompt outputs.
    /// It combines `from_json_array` and `execute_batch` into a single call.
    ///
    /// # Example
    /// ```rust,no_run
    /// use orchy::enums::Action;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let json_output = r#"[
    ///     {
    ///         "Write": {
    ///             "path": "hello.txt",
    ///             "content": "Hello, World!"
    ///         }
    ///     }
    /// ]"#;
    ///
    /// Action::parse_and_execute(json_output).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn parse_and_execute(json_str: &str) -> Result<(), Box<dyn std::error::Error>> {
        let actions = Self::from_json_array(json_str)?;
        Self::execute_batch(&actions).await?;
        Ok(())
    }

    /// Execute a batch of actions sequentially
    ///
    /// All actions are executed in the order they appear in the slice.
    /// If any action fails, execution stops and the error is returned.
    ///
    /// # Example
    /// ```rust,no_run
    /// use orchy::enums::Action;
    ///
    /// # async fn example() -> Result<(), std::io::Error> {
    /// let actions = vec![
    ///     Action::Write {
    ///         path: "file1.txt".to_string(),
    ///         content: "Content 1".to_string(),
    ///     },
    ///     Action::Write {
    ///         path: "file2.txt".to_string(),
    ///         content: "Content 2".to_string(),
    ///     },
    /// ];
    ///
    /// Action::execute_batch(&actions).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_batch(actions: &[Action]) -> Result<(), std::io::Error> {
        for action in actions {
            action.execute().await?;
        }
        Ok(())
    }

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

            Action::Replace {
                path,
                old_content,
                new_content,
            } => {
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

            Action::Grep { pattern, path, recursive, case_sensitive } => {
                let regex = if *case_sensitive {
                    Regex::new(pattern).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?
                } else {
                    Regex::new(&format!("(?i){}", pattern)).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?
                };

                if *recursive {
                    Self::grep_recursive(&regex, path).await?;
                } else {
                    Self::grep_file(&regex, path).await?;
                }
            }

            Action::CreateDirectory { path } => {
                let dir_path = PathBuf::from(path);
                fs::create_dir_all(&dir_path).await?;
                debug!("Created directory {}", path);
            }

            Action::RemoveDirectory { path, recursive } => {
                let dir_path = PathBuf::from(path);
                if *recursive {
                    fs::remove_dir_all(&dir_path).await?;
                } else {
                    fs::remove_dir(&dir_path).await?;
                }
                debug!("Removed directory {}", path);
            }

            Action::ListDirectory { path, recursive } => {
                if *recursive {
                    Self::list_directory_recursive(path).await?;
                } else {
                    Self::list_directory(path).await?;
                }
            }

            Action::CreateSymlink { target, link_path } => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs;
                    fs::symlink(target, link_path)?;
                    debug!("Created symlink {} -> {}", link_path, target);
                }
                #[cfg(not(unix))]
                {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "Symlinks not supported on this platform"
                    ));
                }
            }

            Action::SetPermissions { path, permissions } => {
                #[cfg(unix)]
                {
                    let file_path = PathBuf::from(path);
                    let mode = u32::from_str_radix(permissions, 8)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
                    let metadata = fs::metadata(&file_path).await?;
                    let mut perms = metadata.permissions();
                    perms.set_mode(mode);
                    fs::set_permissions(&file_path, perms).await?;
                    debug!("Set permissions {} on {}", permissions, path);
                }
                #[cfg(not(unix))]
                {
                    debug!("Permission setting not supported on this platform");
                }
            }

            Action::Append { path, content } => {
                let file_path = PathBuf::from(path);
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path)
                    .await?;
                file.write_all(content.as_bytes()).await?;
                file.flush().await?;
                debug!("Appended to {}", path);
            }

            Action::Backup { path, backup_suffix } => {
                let file_path = PathBuf::from(path);
                let suffix = backup_suffix.as_deref().unwrap_or(".bak");
                let backup_path = format!("{}{}", path, suffix);
                let content = fs::read(&file_path).await?;
                fs::write(&backup_path, content).await?;
                debug!("Created backup {} -> {}", path, backup_path);
            }

            Action::Download { url, destination } => {
                // For now, use curl command - in a real implementation you might use reqwest
                let output = Command::new("curl")
                    .arg("-L")
                    .arg("-o")
                    .arg(destination)
                    .arg(url)
                    .output()
                    .await?;

                if !output.status.success() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Download failed: {}", String::from_utf8_lossy(&output.stderr))
                    ));
                }
                debug!("Downloaded {} to {}", url, destination);
            }

            Action::Extract { archive_path, destination } => {
                let archive_path_buf = PathBuf::from(archive_path);
                let extension = archive_path_buf.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                let command = match extension {
                    "zip" => format!("unzip -q '{}' -d '{}'", archive_path, destination),
                    "tar" => format!("tar -xf '{}' -C '{}'", archive_path, destination),
                    "gz" if archive_path.ends_with(".tar.gz") => {
                        format!("tar -xzf '{}' -C '{}'", archive_path, destination)
                    },
                    _ => return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Unsupported archive format: {}", extension)
                    )),
                };

                // Create destination directory if it doesn't exist
                fs::create_dir_all(destination).await?;

                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .output()
                    .await?;

                if !output.status.success() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Extract failed: {}", String::from_utf8_lossy(&output.stderr))
                    ));
                }
                debug!("Extracted {} to {}", archive_path, destination);
            }

            Action::Archive { source_paths, archive_path, format } => {
                let sources = source_paths.join(" ");
                let command = match format.as_str() {
                    "zip" => format!("zip -r '{}' {}", archive_path, sources),
                    "tar" => format!("tar -cf '{}' {}", archive_path, sources),
                    "tar.gz" => format!("tar -czf '{}' {}", archive_path, sources),
                    _ => return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Unsupported archive format: {}", format)
                    )),
                };

                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .output()
                    .await?;

                if !output.status.success() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Archive creation failed: {}", String::from_utf8_lossy(&output.stderr))
                    ));
                }
                debug!("Created archive {} from {:?}", archive_path, source_paths);
            }

            Action::Watch { path, duration_seconds } => {
                debug!("Watching {} for {} seconds", path, duration_seconds);
                // Simple implementation - in practice you'd use a proper file watcher
                let start_time = std::time::Instant::now();
                let duration = std::time::Duration::from_secs(*duration_seconds);

                let file_path = PathBuf::from(path);
                let initial_metadata = fs::metadata(&file_path).await.ok();

                while start_time.elapsed() < duration {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                    if let Ok(current_metadata) = fs::metadata(&file_path).await {
                        if let Some(ref initial) = initial_metadata {
                            if current_metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                                != initial.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH) {
                                debug!("File {} was modified", path);
                            }
                        }
                    }
                }
                debug!("Finished watching {}", path);
            }
        }
        Ok(())
    }

    // Helper methods for new actions
    async fn grep_file(regex: &Regex, path: &str) -> Result<(), std::io::Error> {
        let content = fs::read_to_string(path).await?;
        for (line_num, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                debug!("{}:{}: {}", path, line_num + 1, line);
            }
        }
        Ok(())
    }

    fn grep_recursive<'a>(regex: &'a Regex, path: &'a str) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>> {
        Box::pin(async move {
            let path_buf = PathBuf::from(path);
            if path_buf.is_file() {
                Self::grep_file(regex, path).await?;
            } else if path_buf.is_dir() {
                let mut entries = fs::read_dir(&path_buf).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        if let Some(path_str) = entry_path.to_str() {
                            Self::grep_file(regex, path_str).await?;
                        }
                    } else if entry_path.is_dir() {
                        if let Some(path_str) = entry_path.to_str() {
                            Self::grep_recursive(regex, path_str).await?;
                        }
                    }
                }
            }
            Ok(())
        })
    }

    async fn list_directory(path: &str) -> Result<(), std::io::Error> {
        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            let file_name = entry_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("?");

            let metadata = entry.metadata().await?;
            let file_type = if metadata.is_dir() { "d" } else { "f" };
            let size = metadata.len();

            debug!("{} {} {} bytes", file_type, file_name, size);
        }
        Ok(())
    }

    async fn list_directory_recursive(path: &str) -> Result<(), std::io::Error> {
        Self::list_directory_recursive_impl(path, 0).await
    }

    fn list_directory_recursive_impl<'a>(path: &'a str, depth: usize) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>> {
        Box::pin(async move {
            let indent = "  ".repeat(depth);
            let mut entries = fs::read_dir(path).await?;

            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let file_name = entry_path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("?");

                let metadata = entry.metadata().await?;
                if metadata.is_dir() {
                    debug!("{}d {}/", indent, file_name);
                    if let Some(path_str) = entry_path.to_str() {
                        Self::list_directory_recursive_impl(path_str, depth + 1).await?;
                    }
                } else {
                    debug!("{}f {} ({} bytes)", indent, file_name, metadata.len());
                }
            }
            Ok(())
        })
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
        let content = fs::read_to_string(&file_path)
            .await
            .expect("Should read file content");
        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_read_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        // First create a file with content
        fs::write(&file_path, "Test content")
            .await
            .expect("Should write test file");

        let action = Action::Read {
            path: file_path.to_string_lossy().to_string(),
        };

        // Read action doesn't return content, it just logs it
        action.execute().await.expect("Read action should succeed");

        // Verify the file still exists and has the expected content
        let content = fs::read_to_string(&file_path)
            .await
            .expect("Should read file content");
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_delete_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        // First create a file
        fs::write(&file_path, "Content to delete")
            .await
            .expect("Should write test file");
        assert!(file_path.exists());

        let action = Action::Delete {
            path: file_path.to_string_lossy().to_string(),
        };

        action
            .execute()
            .await
            .expect("Delete action should succeed");
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

        action
            .execute()
            .await
            .expect("Update action should succeed");

        let content = fs::read_to_string(&file_path)
            .await
            .expect("Should read file content");
        assert_eq!(content, "Updated content");
    }

    #[tokio::test]
    async fn test_replace_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        // First create a file with initial content
        fs::write(&file_path, "Hello, World!")
            .await
            .expect("Should write initial content");

        let action = Action::Replace {
            path: file_path.to_string_lossy().to_string(),
            old_content: "Hello".to_string(),
            new_content: "Goodbye".to_string(),
        };

        action
            .execute()
            .await
            .expect("Replace action should succeed");

        let content = fs::read_to_string(&file_path)
            .await
            .expect("Should read file content");
        assert_eq!(content, "Goodbye, World!");
    }

    #[tokio::test]
    async fn test_move_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let old_path = temp_dir.path().join("test.txt");
        let new_path = temp_dir.path().join("new_test.txt");

        // First create a file
        fs::write(&old_path, "Content to move")
            .await
            .expect("Should write test file");
        assert!(old_path.exists());

        let action = Action::Move {
            old_path: old_path.to_string_lossy().to_string(),
            new_path: new_path.to_string_lossy().to_string(),
        };

        action.execute().await.expect("Move action should succeed");

        assert!(!old_path.exists());
        assert!(new_path.exists());

        let content = fs::read_to_string(&new_path)
            .await
            .expect("Should read moved file content");
        assert_eq!(content, "Content to move");
    }

    #[tokio::test]
    async fn test_copy_action() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");

        // First create a source file
        fs::write(&source_path, "Content to copy")
            .await
            .expect("Should write source file");

        let action = Action::Copy {
            old_path: source_path.to_string_lossy().to_string(),
            new_path: dest_path.to_string_lossy().to_string(),
        };

        action.execute().await.expect("Copy action should succeed");

        assert!(source_path.exists());
        assert!(dest_path.exists());

        let source_content = fs::read_to_string(&source_path)
            .await
            .expect("Should read source content");
        let dest_content = fs::read_to_string(&dest_path)
            .await
            .expect("Should read dest content");
        assert_eq!(source_content, "Content to copy");
        assert_eq!(dest_content, "Content to copy");
    }

    #[tokio::test]
    async fn test_run_command_action() {
        let action = Action::RunCommand {
            command: "echo 'Hello, World!'".to_string(),
            env: None,
        };

        action
            .execute()
            .await
            .expect("Command should execute successfully");
    }

    #[tokio::test]
    async fn test_run_command_with_env_action() {
        let env_vars = vec![("TEST_VAR".to_string(), "test_value".to_string())];

        let action = Action::RunCommand {
            command: "echo $TEST_VAR".to_string(),
            env: Some(env_vars),
        };

        action
            .execute()
            .await
            .expect("Command with env should execute successfully");
    }

    #[tokio::test]
    async fn test_replace_action_no_match() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        // Create a file with content that won't match the replacement
        fs::write(&file_path, "Original content")
            .await
            .expect("Should write initial content");

        let action = Action::Replace {
            path: file_path.to_string_lossy().to_string(),
            old_content: "NonExistent".to_string(),
            new_content: "Replacement".to_string(),
        };

        action
            .execute()
            .await
            .expect("Replace action should succeed even with no matches");

        let content = fs::read_to_string(&file_path)
            .await
            .expect("Should read file content");
        assert_eq!(content, "Original content"); // Should remain unchanged
    }

    #[tokio::test]
    async fn test_write_action_creates_directories() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("directory")
            .join("test.txt");

        let action = Action::Write {
            path: nested_path.to_string_lossy().to_string(),
            content: "Nested file content".to_string(),
        };

        action.execute().await.expect("Write action should succeed");

        assert!(nested_path.exists());
        let content = fs::read_to_string(&nested_path)
            .await
            .expect("Should read file content");
        assert_eq!(content, "Nested file content");
    }

    #[tokio::test]
    async fn test_from_json_array() {
        let json_str = r#"[
            {
                "Write": {
                    "path": "src/main.rs",
                    "content": "fn main() { println!(\"Hello, world!\"); }"
                }
            },
            {
                "Read": {
                    "path": "src/lib.rs"
                }
            },
            {
                "Delete": {
                    "path": "target/debug/tempfile.txt"
                }
            },
            {
                "Update": {
                    "path": "src/config.rs",
                    "content": "pub const VERSION: &str = \"1.0.1\";"
                }
            },
            {
                "Replace": {
                    "path": "README.md",
                    "old_content": "Project Alpha",
                    "new_content": "Project Beta"
                }
            },
            {
                "Move": {
                    "old_path": "docs/old_intro.md",
                    "new_path": "docs/introduction.md"
                }
            },
            {
                "Copy": {
                    "old_path": "src/main.rs",
                    "new_path": "src/main_backup.rs"
                }
            },
            {
                "RunCommand": {
                    "command": "cargo build",
                    "env": [
                        ["RUST_LOG", "debug"],
                        ["RUST_BACKTRACE", "1"]
                    ]
                }
            }
        ]"#;

        let actions = Action::from_json_array(json_str).expect("Should parse JSON successfully");

        assert_eq!(actions.len(), 8);

        // Test each action type
        match &actions[0] {
            Action::Write { path, content } => {
                assert_eq!(path, "src/main.rs");
                assert_eq!(content, "fn main() { println!(\"Hello, world!\"); }");
            }
            _ => panic!("Expected Write action"),
        }

        match &actions[1] {
            Action::Read { path } => {
                assert_eq!(path, "src/lib.rs");
            }
            _ => panic!("Expected Read action"),
        }

        match &actions[2] {
            Action::Delete { path } => {
                assert_eq!(path, "target/debug/tempfile.txt");
            }
            _ => panic!("Expected Delete action"),
        }

        match &actions[3] {
            Action::Update { path, content } => {
                assert_eq!(path, "src/config.rs");
                assert_eq!(content, "pub const VERSION: &str = \"1.0.1\";");
            }
            _ => panic!("Expected Update action"),
        }

        match &actions[4] {
            Action::Replace { path, old_content, new_content } => {
                assert_eq!(path, "README.md");
                assert_eq!(old_content, "Project Alpha");
                assert_eq!(new_content, "Project Beta");
            }
            _ => panic!("Expected Replace action"),
        }

        match &actions[5] {
            Action::Move { old_path, new_path } => {
                assert_eq!(old_path, "docs/old_intro.md");
                assert_eq!(new_path, "docs/introduction.md");
            }
            _ => panic!("Expected Move action"),
        }

        match &actions[6] {
            Action::Copy { old_path, new_path } => {
                assert_eq!(old_path, "src/main.rs");
                assert_eq!(new_path, "src/main_backup.rs");
            }
            _ => panic!("Expected Copy action"),
        }

        match &actions[7] {
            Action::RunCommand { command, env } => {
                assert_eq!(command, "cargo build");
                assert!(env.is_some());
                let env_vars = env.as_ref().unwrap();
                assert_eq!(env_vars.len(), 2);
                assert_eq!(env_vars[0], ("RUST_LOG".to_string(), "debug".to_string()));
                assert_eq!(env_vars[1], ("RUST_BACKTRACE".to_string(), "1".to_string()));
            }
            _ => panic!("Expected RunCommand action"),
        }
    }

    #[tokio::test]
    async fn test_from_json_array_invalid_json() {
        let invalid_json = r#"[{"InvalidAction": {}}]"#;
        let result = Action::from_json_array(invalid_json);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_from_json_array_empty() {
        let empty_json = "[]";
        let actions = Action::from_json_array(empty_json).expect("Should parse empty array");
        assert_eq!(actions.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_batch() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");

        let actions = vec![
            Action::Write {
                path: file1_path.to_string_lossy().to_string(),
                content: "Content 1".to_string(),
            },
            Action::Write {
                path: file2_path.to_string_lossy().to_string(),
                content: "Content 2".to_string(),
            },
        ];

        Action::execute_batch(&actions)
            .await
            .expect("Batch execution should succeed");

        // Verify both files were created
        assert!(file1_path.exists());
        assert!(file2_path.exists());

        let content1 = fs::read_to_string(&file1_path)
            .await
            .expect("Should read file1 content");
        let content2 = fs::read_to_string(&file2_path)
            .await
            .expect("Should read file2 content");

        assert_eq!(content1, "Content 1");
        assert_eq!(content2, "Content 2");
    }

    #[tokio::test]
    async fn test_parse_and_execute() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        let json_str = format!(
            r#"[
                {{
                    "Write": {{
                        "path": "{}",
                        "content": "Hello from JSON!"
                    }}
                }}
            ]"#,
            file_path.to_string_lossy()
        );

        Action::parse_and_execute(&json_str)
            .await
            .expect("Parse and execute should succeed");

        // Verify the file was created
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path)
            .await
            .expect("Should read file content");
        assert_eq!(content, "Hello from JSON!");
    }

    #[tokio::test]
    async fn test_parse_and_execute_invalid_json() {
        let invalid_json = r#"[{"InvalidAction": {}}]"#;
        let result = Action::parse_and_execute(invalid_json).await;
        assert!(result.is_err());
    }
}
