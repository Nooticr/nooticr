//! Gemini CLI integration module
//!
//! This module provides integration with the Gemini CLI for intelligent
//! software development tasks including code generation, task breakdown,
//! and documentation creation.

use crate::error::{Result, OrchestratorError};
use crate::enums::LLMResponse;
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
        
        // Add model selection - use gemini-2.5-flash as default if not specified
        let model_to_use = model.unwrap_or("gemini-2.5-flash");
        command.args(&["--model", model_to_use]);

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
        } else {
            tracing::warn!("⚠️  GEMINI_API_KEY not found in environment");
        }
        
        tracing::debug!("🤖 Calling Gemini CLI from directory: {:?}", working_dir);
        tracing::debug!("📝 Prompt length: {} characters", prompt.len());
        
        // Add model selection - use gemini-2.5-flash as default if not specified
        let model_to_use = model.unwrap_or("gemini-2.5-flash");
        command.args(&["--model", model_to_use]);

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

    /// Send a simple query to Gemini CLI using the preferred model (gemini-2.5-flash)
    pub async fn query(prompt: &str) -> Result<String> {
        // Use the preferred model explicitly
        Self::query_with_model(prompt, "gemini-2.5-flash").await
    }

    /// Send a query with specific model selection and retry logic
    pub async fn query_with_model(prompt: &str, model: &str) -> Result<String> {
        Self::query_with_model_and_retries(prompt, model, 3).await
    }

    /// Send a query with specific model selection and configurable retry logic
    pub async fn query_with_model_and_retries(prompt: &str, model: &str, max_retries: u32) -> Result<String> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            tracing::debug!("🤖 Gemini API attempt {}/{}", attempt, max_retries);

            let mut command = Command::new("gemini");

            // Ensure GEMINI_API_KEY is set from environment
            if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
                command.env("GEMINI_API_KEY", api_key);
            }

            let output = command
                .args(&["gen", "-m", model, "-p", prompt])
                .output()
                .await
                .map_err(|e| OrchestratorError::internal(format!("Failed to run Gemini CLI: {}", e)))?;

            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                tracing::debug!("✅ Gemini API success on attempt {}", attempt);
                return Ok(response.to_string());
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error_msg = format!("Gemini CLI failed: stderr: {}, stdout: {}", stderr, stdout);

            // Check if this is a retryable error
            let is_retryable = stderr.contains("INTERNAL") ||
                              stderr.contains("500") ||
                              stderr.contains("timeout") ||
                              stderr.contains("rate limit") ||
                              stderr.contains("temporarily unavailable");

            if !is_retryable || attempt == max_retries {
                tracing::error!("❌ Gemini API failed on attempt {}: {}", attempt, error_msg);
                return Err(OrchestratorError::internal(error_msg));
            }

            tracing::warn!("⚠️  Gemini API retryable error on attempt {}: {}", attempt, error_msg);
            last_error = Some(error_msg);

            // Exponential backoff: 1s, 2s, 4s, etc.
            let delay_seconds = 2_u64.pow(attempt - 1);
            tracing::info!("⏳ Retrying in {}s...", delay_seconds);
            tokio::time::sleep(tokio::time::Duration::from_secs(delay_seconds)).await;
        }

        Err(OrchestratorError::internal(
            last_error.unwrap_or_else(|| "All retry attempts failed".to_string())
        ))
    }

    /// Send a query expecting JSON response using the preferred model (gemini-2.5-flash)
    pub async fn query_json(prompt: &str) -> Result<serde_json::Value> {
        let output = Command::new("gemini")
            .args(&["--model", "gemini-2.5-flash", "--format", "json"])
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
    /// Note: The current Gemini CLI doesn't support temperature control, so this falls back to standard query
    pub async fn query_with_temperature(prompt: &str, _temperature: f32) -> Result<String> {
        // The Gemini CLI doesn't support temperature control, so we use the standard query method
        tracing::warn!("Temperature control not supported by Gemini CLI, using default settings");
        Self::query(prompt).await
    }

    /// List available Gemini models
    /// Note: The Gemini CLI doesn't provide a models list command, so we return known models
    pub async fn list_models() -> Result<Vec<String>> {
        // Return a list of known Gemini models since the CLI doesn't support listing them
        // gemini-2.5-flash is the preferred model for this application
        let known_models = vec![
            "gemini-2.5-flash".to_string(),
            "gemini-2.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.0-pro".to_string(),
        ];

        Ok(known_models)
    }

    /// Extract JSON from Gemini response (handles markdown wrapping)
    pub fn extract_json_from_response(response: &str) -> Result<String> {
        let response = response.trim();

        // If response starts with ```json, extract the JSON between code blocks
        if response.starts_with("```json") {
            let lines: Vec<&str> = response.lines().collect();
            let mut json_lines = Vec::new();
            let mut in_json_block = false;
            
            for line in lines {
                if line.trim().starts_with("```json") {
                    in_json_block = true;
                    continue;
                }
                if line.trim() == "```" && in_json_block {
                    break;
                }
                if in_json_block {
                    json_lines.push(line);
                }
            }
            
            let json_str = json_lines.join("\n");
            if json_str.trim().is_empty() {
                return Err(OrchestratorError::internal("No JSON found in markdown block"));
            }
            Ok(json_str)
        }
        // If response starts with [ or {, assume it's pure JSON
        else if response.starts_with('[') || response.starts_with('{') {
            Ok(response.to_string())
        }
        // Try to find JSON in the response using balanced bracket/brace matching
        else {
            let start_bracket = response.find('[');
            let start_brace = response.find('{');
            
            let (start_pos, is_array) = match (start_bracket, start_brace) {
                (Some(bracket), Some(brace)) => {
                    if bracket < brace { (bracket, true) } else { (brace, false) }
                },
                (Some(bracket), None) => (bracket, true),
                (None, Some(brace)) => (brace, false),
                (None, None) => return Err(OrchestratorError::internal("No JSON found in response")),
            };
            
            // Find the matching closing bracket/brace
            let chars: Vec<char> = response.chars().collect();
            let mut depth = 0;
            let (open_char, close_char) = if is_array { ('[', ']') } else { ('{', '}') };
            
            for i in start_pos..chars.len() {
                if chars[i] == open_char {
                    depth += 1;
                } else if chars[i] == close_char {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(response[start_pos..=i].to_string());
                    }
                }
            }
            
            Err(OrchestratorError::internal("No matching closing bracket/brace found"))
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

    /// Send a query and parse response into structured LLMResponse enum
    pub async fn query_structured(prompt: &str) -> Result<LLMResponse> {
        let response = Self::query(prompt).await?;
        Self::parse_response(&response)
    }

    /// Send a query with specific model and parse response into structured LLMResponse enum
    pub async fn query_structured_with_model(prompt: &str, model: &str) -> Result<LLMResponse> {
        let response = Self::query_with_model(prompt, model).await?;
        Self::parse_response(&response)
    }

    /// Send a query from directory and parse response into structured LLMResponse enum
    pub async fn query_structured_from_dir(
        session_id: &str,
        prompt: &str,
        model: Option<&str>,
        working_dir: &std::path::Path,
    ) -> Result<LLMResponse> {
        let response = Self::query_with_session_from_dir(session_id, prompt, model, working_dir).await?;
        Self::parse_response(&response)
    }

    /// Parse a raw LLM response string into a structured LLMResponse enum
    pub fn parse_response(response: &str) -> Result<LLMResponse> {
        // First try to extract JSON if it's wrapped in markdown
        let json_str = Self::extract_json_from_response(response)?;
        
        // Try to parse into structured response types
        match LLMResponse::from_raw_json(&json_str) {
            Ok(parsed_response) => Ok(parsed_response),
            Err(e) => {
                tracing::warn!("Failed to parse as structured response: {}", e);
                // Fallback to raw text response
                Ok(LLMResponse::Text {
                    content: response.to_string(),
                })
            }
        }
    }

    /// Get response type from raw response without full parsing
    pub fn detect_response_type(response: &str) -> &'static str {
        // Try to extract JSON and detect patterns
        if let Ok(json_str) = Self::extract_json_from_response(response) {
            // Check for common patterns in the JSON
            // Action array detection - look for array with action objects
            if json_str.starts_with('[') && (
                json_str.contains("\"Write\"") || 
                json_str.contains("\"Read\"") || 
                json_str.contains("\"Delete\"") || 
                json_str.contains("\"Update\"") ||
                json_str.contains("\"actions\"")
            ) {
                return "action_array";
            }
            if json_str.contains("\"Reject\"") && json_str.contains("\"reason\"") {
                return "agent_rejection";
            }
            if json_str.contains("\"tasks\"") && json_str.contains("\"TaskInput\"") {
                return "idea_breakdown";
            }
            if json_str.contains("\"issue_analysis\"") {
                return "ci_cd_fix";
            }
            if json_str.contains("\"deployment_strategy\"") {
                return "docker_deployment";
            }
            if json_str.contains("\"overall_quality_score\"") {
                return "qa_analysis";
            }
            if json_str.contains("\"synchronization_analysis\"") {
                return "api_synchronization";
            }
            if json_str.contains("\"performance_analysis\"") {
                return "performance_optimization";
            }
            return "structured_json";
        }
        "text"
    }

    /// Extract actions from response if present
    pub async fn extract_actions_from_response(response: &str) -> Result<Vec<crate::enums::Action>> {
        let llm_response = Self::parse_response(response)?;
        Ok(llm_response.extract_actions())
    }

    /// Check if response is a rejection
    pub fn is_rejection_response(response: &str) -> bool {
        if let Ok(llm_response) = Self::parse_response(response) {
            llm_response.is_rejection()
        } else {
            // Fallback: check for rejection patterns in raw text
            response.contains("Reject") || response.contains("rejection") || response.contains("blocking_issues")
        }
    }

    /// Get rejection details from response
    pub fn extract_rejection_details(response: &str) -> Option<(String, Vec<String>)> {
        if let Ok(llm_response) = Self::parse_response(response) {
            if let Some(reason) = llm_response.get_rejection_reason() {
                let blocking_issues = llm_response.get_blocking_issues()
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                return Some((reason.to_string(), blocking_issues));
            }
        }
        None
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
            // Test the list_models function
            let models = GeminiCLI::list_models().await;
            assert!(models.is_ok(), "Failed to get models: {:?}", models.err());

            let models_list = models.unwrap();
            assert!(!models_list.is_empty(), "Models list should not be empty");
            assert!(models_list.contains(&"gemini-2.5-flash".to_string()), "Should include preferred model");
            assert!(models_list[0] == "gemini-2.5-flash", "gemini-2.5-flash should be first in list");

            println!("Available Gemini models: {:?}", models_list);
        }
    }

    #[tokio::test]
    async fn test_gemini_with_model() {
        if GeminiCLI::is_available().await {
            let response = GeminiCLI::query_with_model("What is the capital of France?", "gemini-2.5-flash").await;
            match &response {
                Ok(resp) => println!("Gemini with model response: {}", resp),
                Err(e) => println!("Error in gemini query: {:?}", e),
            }
            assert!(response.is_ok(), "Gemini query failed: {:?}", response.err());
        }
    }

    #[tokio::test]
    async fn test_gemini_with_temperature() {
        if GeminiCLI::is_available().await {
            let response = GeminiCLI::query_with_temperature("Write a creative story opening", 0.8).await;
            match &response {
                Ok(resp) => println!("Gemini with temperature response: {}", resp),
                Err(e) => println!("Error in gemini temperature query: {:?}", e),
            }
            assert!(response.is_ok(), "Gemini temperature query failed: {:?}", response.err());
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

    #[test]
    fn test_response_type_detection() {
        // Test action array detection
        let action_response = r#"```json
[
  {
    "Write": {
      "path": "test.txt",
      "content": "test"
    }
  }
]
```"#;
        assert_eq!(GeminiCLI::detect_response_type(action_response), "action_array");

        // Test rejection detection
        let rejection_response = r#"```json
{
  "Reject": {
    "reason": "Code doesn't compile",
    "blocking_issues": ["Syntax errors"]
  }
}
```"#;
        assert_eq!(GeminiCLI::detect_response_type(rejection_response), "agent_rejection");

        // Test text response
        let text_response = "This is just plain text without JSON";
        assert_eq!(GeminiCLI::detect_response_type(text_response), "text");
    }

    #[test]
    fn test_structured_response_parsing() {
        // Test parsing action array
        let action_response = r#"```json
[
  {
    "Write": {
      "path": "test.txt",
      "content": "Hello World"
    }
  }
]
```"#;
        
        let parsed = GeminiCLI::parse_response(action_response);
        assert!(parsed.is_ok());
        
        if let Ok(llm_response) = parsed {
            assert!(llm_response.has_actions());
            assert_eq!(llm_response.response_type(), "feature_development");
            let actions = llm_response.extract_actions();
            assert_eq!(actions.len(), 1);
        }
    }

    #[test]
    fn test_rejection_detection() {
        let rejection_response = r#"Some text with Reject keyword and blocking_issues"#;
        assert!(GeminiCLI::is_rejection_response(rejection_response));

        let normal_response = r#"[{"Write": {"path": "test.txt", "content": "test"}}]"#;
        assert!(!GeminiCLI::is_rejection_response(normal_response));
    }
}