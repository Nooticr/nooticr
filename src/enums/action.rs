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
                let mut file = fs::File::create(&file_path).await?;
                file.write_all(content.as_bytes()).await?;
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
