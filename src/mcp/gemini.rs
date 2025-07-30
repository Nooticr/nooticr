//! Gemini CLI integration module
//!
//! This module provides integration with the Gemini CLI for intelligent
//! software development tasks including code generation, task breakdown,
//! and documentation creation.

use crate::error::{Result, OrchestratorError};
use tokio::process::Command;

/// Gemini CLI utility functions
pub struct GeminiCLI;

impl GeminiCLI {
    /// Check if Gemini CLI is available and working
    pub async fn is_available() -> bool {
        match Command::new("gemini").arg("--version").output().await {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Get Gemini CLI version
    pub async fn version() -> Result<String> {
        let output = Command::new("gemini")
            .arg("--version")
            .output()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Failed to get Gemini version: {}", e)))?;

        if !output.status.success() {
            return Err(OrchestratorError::internal("Gemini CLI not available"));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    }

    /// Send a query with session management
    pub async fn query_with_session(
        _session_id: &str,
        prompt: &str,
        model: Option<&str>,
    ) -> Result<String> {
        let mut command = Command::new("gemini");
        
        // Add model selection if specified
        if let Some(model) = model {
            command.args(&["--model", model]);
        }

        // Use -p/--prompt for non-interactive mode as indicated by the help message
        let output = command
            .args(&["-p", prompt])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Failed to run Gemini CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(OrchestratorError::internal(format!(
                "Gemini CLI failed: stderr: {}, stdout: {}",
                stderr,
                stdout
            )));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        Ok(response.to_string())
    }

    /// Send a query with session management from a specific working directory
    pub async fn query_with_session_from_dir(
        _session_id: &str,
        prompt: &str,
        model: Option<&str>,
        working_dir: &std::path::Path,
    ) -> Result<String> {
        let mut command = Command::new("gemini");
        
        // Set the working directory so Gemini can access CLAUDE.md and GEMINI.md
        command.current_dir(working_dir);
        
        // Ensure GEMINI_API_KEY is set from environment
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            command.env("GEMINI_API_KEY", api_key);
        }
        
        // Add model selection if specified
        if let Some(model) = model {
            command.args(&["--model", model]);
        }

        // Use -p/--prompt for non-interactive mode as indicated by the help message
        let output = command
            .args(&["-p", prompt])
            .output()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Failed to run Gemini CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(OrchestratorError::internal(format!(
                "Gemini CLI failed: stderr: {}, stdout: {}",
                stderr,
                stdout
            )));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        Ok(response.to_string())
    }

    /// Continue an existing session
    pub async fn continue_session_query(
        _session_id: &str,
        prompt: &str,
        model: Option<&str>,
    ) -> Result<String> {
        let mut command = Command::new("gemini");
        
        // Add model selection if specified
        if let Some(model) = model {
            command.args(&["--model", model]);
        }

        // Use -p/--prompt for non-interactive mode as indicated by the help message
        let output = command
            .args(&["-p", prompt])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Failed to run Gemini CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(OrchestratorError::internal(format!(
                "Gemini CLI failed: stderr: {}, stdout: {}",
                stderr,
                stdout
            )));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        Ok(response.to_string())
    }

    /// Send a simple query to Gemini CLI
    pub async fn query(prompt: &str) -> Result<String> {
        let output = Command::new("gemini")
            .args(&["-p", prompt])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Failed to run Gemini CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(OrchestratorError::internal(format!(
                "Gemini CLI failed: stderr: {}, stdout: {}",
                stderr,
                stdout
            )));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        Ok(response.to_string())
    }

    /// Send a query with specific model selection
    pub async fn query_with_model(prompt: &str, model: &str) -> Result<String> {
        let mut command = Command::new("gemini");
        
        // Ensure GEMINI_API_KEY is set from environment
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            command.env("GEMINI_API_KEY", api_key);
        }
        
        let output = command
            .args(&["--model", model, "-p", prompt])
            .output()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Failed to run Gemini CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(OrchestratorError::internal(format!(
                "Gemini CLI failed: stderr: {}, stdout: {}",
                stderr,
                stdout
            )));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        Ok(response.to_string())
    }

    /// Send a query expecting JSON response
    pub async fn query_json(prompt: &str) -> Result<serde_json::Value> {
        let output = Command::new("gemini")
            .args(&["--format", "json"])
            .arg(prompt)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Failed to run Gemini CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OrchestratorError::internal(format!("Gemini CLI failed: {}", stderr)));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| OrchestratorError::json_parsing("Gemini CLI response", e))?;

        Ok(json)
    }

    /// Send a query with specific temperature setting
    pub async fn query_with_temperature(prompt: &str, temperature: f32) -> Result<String> {
        let temp_str = temperature.to_string();
        let output = Command::new("gemini")
            .args(&["--format", "json", "--temperature", &temp_str])
            .arg(prompt)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Failed to run Gemini CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(OrchestratorError::internal(format!(
                "Gemini CLI failed: stderr: {}, stdout: {}",
                stderr,
                stdout
            )));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        Ok(response.to_string())
    }

    /// List available Gemini models
    pub async fn list_models() -> Result<Vec<String>> {
        let output = Command::new("gemini")
            .args(&["--list-models"])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Failed to run Gemini CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OrchestratorError::internal(format!("Gemini CLI failed: {}", stderr)));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        let models: Vec<String> = response
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(models)
    }

    /// Extract JSON from Gemini response (handles markdown wrapping)
    pub fn extract_json_from_response(response: &str) -> Result<String> {
        let response = response.trim();

        // If response starts with ```json, extract the JSON
        if response.starts_with("```json") {
            let start = response
                .find('[')
                .or_else(|| response.find('{'))
                .ok_or_else(|| OrchestratorError::internal("No JSON found in response"))?;
            let end = if response[start..].starts_with('[') {
                response
                    .rfind(']')
                    .ok_or_else(|| OrchestratorError::internal("No closing bracket found"))?
            } else {
                response
                    .rfind('}')
                    .ok_or_else(|| OrchestratorError::internal("No closing brace found"))?
            };
            Ok(response[start..=end].to_string())
        }
        // If response starts with [ or {, assume it's pure JSON
        else if response.starts_with('[') || response.starts_with('{') {
            Ok(response.to_string())
        }
        // Try to find JSON in the response
        else {
            let start = response
                .find('[')
                .or_else(|| response.find('{'))
                .ok_or_else(|| OrchestratorError::internal("No JSON found in response"))?;
            let end = if response[start..].starts_with('[') {
                response
                    .rfind(']')
                    .ok_or_else(|| OrchestratorError::internal("No closing bracket found"))?
            } else {
                response
                    .rfind('}')
                    .ok_or_else(|| OrchestratorError::internal("No closing brace found"))?
            };
            Ok(response[start..=end].to_string())
        }
    }

    /// Test Gemini CLI with a simple query
    pub async fn test_connection() -> Result<()> {
        let response = Self::query("Respond with exactly: 'Gemini CLI is working'").await?;

        if response.trim().contains("Gemini CLI is working") {
            Ok(())
        } else {
            Err(OrchestratorError::internal(format!(
                "Unexpected response from Gemini CLI: {}",
                response
            )))
        }
    }

    /// Get model capabilities and information
    pub async fn get_model_info(model: &str) -> Result<serde_json::Value> {
        let output = Command::new("gemini")
            .args(&["--model-info", model, "--format", "json"])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Failed to run Gemini CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OrchestratorError::internal(format!("Gemini CLI failed: {}", stderr)));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| OrchestratorError::json_parsing("Gemini model info response", e))?;

        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gemini_availability() {
        // This test requires Gemini CLI to be installed
        let available = GeminiCLI::is_available().await;
        println!("Gemini CLI available: {}", available);
    }

    #[tokio::test]
    async fn test_gemini_version() {
        if GeminiCLI::is_available().await {
            let version = GeminiCLI::version().await;
            assert!(version.is_ok());
            println!("Gemini CLI version: {:?}", version);
        }
    }

    #[tokio::test]
    async fn test_gemini_query() {
        if GeminiCLI::is_available().await {
            let response = GeminiCLI::query("What is 2+2?").await;
            assert!(response.is_ok());
            println!("Gemini response: {:?}", response);
        }
    }

    #[tokio::test]
    async fn test_gemini_models() {
        if GeminiCLI::is_available().await {
            let models = GeminiCLI::list_models().await;
            assert!(models.is_ok());
            println!("Available Gemini models: {:?}", models);
        }
    }

    #[tokio::test]
    async fn test_gemini_with_model() {
        if GeminiCLI::is_available().await {
            let response = GeminiCLI::query_with_model("What is the capital of France?", "gemini-pro").await;
            assert!(response.is_ok());
            println!("Gemini with model response: {:?}", response);
        }
    }

    #[tokio::test]
    async fn test_gemini_with_temperature() {
        if GeminiCLI::is_available().await {
            let response = GeminiCLI::query_with_temperature("Write a creative story opening", 0.8).await;
            assert!(response.is_ok());
            println!("Gemini with temperature response: {:?}", response);
        }
    }

    #[test]
    fn test_json_extraction() {
        let markdown_response = r#"```json
[
  {
    "title": "Test Task",
    "description": "A test task"
  }
]
```"#;

        let extracted = GeminiCLI::extract_json_from_response(markdown_response);
        assert!(extracted.is_ok());

        if let Ok(json_str) = extracted {
            assert!(json_str.starts_with('['));
            assert!(json_str.ends_with(']'));
        }
    }

    #[test]
    fn test_pure_json_extraction() {
        let json_response = r#"{"status": "success", "data": [1, 2, 3]}"#;

        let extracted = GeminiCLI::extract_json_from_response(json_response);
        assert!(extracted.is_ok());

        if let Ok(json_str) = extracted {
            assert_eq!(json_str, json_response);
        }
    }
}