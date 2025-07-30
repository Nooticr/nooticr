use super::agent_status_change::AgentStatusChange;
use super::agent_error_recovery::{ActionError, ErrorRecoveryResponse, FileContext};
use crate::enums::{AgentStatus, AgentType};
use crate::error::{OrchestratorError, Result};
use crate::managers::McpClient;
use crate::prompts::Prompts;
use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub file_path: PathBuf,
    pub description: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub status_history: Vec<AgentStatusChange>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub error_count: u32,
    pub total_tasks_completed: u64,
    // Error recovery fields
    pub recent_errors: Vec<ActionError>,
    pub recovery_attempts: u32,
    pub last_error_recovery_at: Option<DateTime<Utc>>,
    pub autonomous_recovery_enabled: bool,
    pub max_recovery_attempts: u32,
}

impl Agent {
    /// Create a new agent
    pub fn new(
        name: impl Into<String>,
        file_path: PathBuf,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        let initial_status = AgentStatus::default();

        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            file_path,
            description: description.into(),
            agent_type: AgentType::default(),
            status: initial_status,
            status_history: vec![AgentStatusChange {
                from: None,
                to: initial_status,
                reason: Some("Agent created".to_string()),
                timestamp: now,
            }],
            created_at: now,
            updated_at: now,
            last_active_at: None,
            error_count: 0,
            total_tasks_completed: 0,
            // Error recovery initialization
            recent_errors: Vec::new(),
            recovery_attempts: 0,
            last_error_recovery_at: None,
            autonomous_recovery_enabled: true,
            max_recovery_attempts: 3,
        }
    }

    /// Create a new agent with specific type
    pub fn new_with_type(
        name: impl Into<String>,
        file_path: PathBuf,
        description: impl Into<String>,
        agent_type: AgentType,
    ) -> Self {
        let now = Utc::now();
        let initial_status = AgentStatus::default();

        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            file_path,
            description: description.into(),
            agent_type,
            status: initial_status,
            status_history: vec![AgentStatusChange {
                from: None,
                to: initial_status,
                reason: Some("Agent created".to_string()),
                timestamp: now,
            }],
            created_at: now,
            updated_at: now,
            last_active_at: None,
            error_count: 0,
            total_tasks_completed: 0,
            // Error recovery initialization
            recent_errors: Vec::new(),
            recovery_attempts: 0,
            last_error_recovery_at: None,
            autonomous_recovery_enabled: true,
            max_recovery_attempts: 3,
        }
    }

    /// Set the agent type
    pub fn set_agent_type(&mut self, agent_type: AgentType) {
        self.agent_type = agent_type;
        self.updated_at = Utc::now();
    }

    /// Parse agent type from filename
    pub fn parse_agent_type_from_filename(filename: &str) -> AgentType {
        match filename {
            "backend_engineer_rust.md" => AgentType::BackendEngineerRust,
            "backend_qa_rust.md" => AgentType::BackendQARust,
            "frontend_engineer_vue.md" => AgentType::FrontendEngineerVue,
            "frontend_qa_vue.md" => AgentType::FrontendQAVue,
            "frontend_engineer_react.md" => AgentType::FrontendEngineerReact,
            "frontend_qa_react.md" => AgentType::FrontendQAReact,
            "devops.md" => AgentType::DevOps,
            "performance_engineer.md" => AgentType::PerformanceEngineer,
            "security_engineer.md" => AgentType::SecurityEngineer,
            "codereview_eng.md" => AgentType::CodeReviewEngineer,
            "release_mmanager.md" => AgentType::ReleaseManager,
            _ => AgentType::default(),
        }
    }

    /// Load agents from embedded data or directory
    pub async fn load_agents_from_directory(_agents_dir: &str) -> Result<Vec<Agent>> {
        // First try to load from embedded agent data
        if let Ok(agents) = Self::load_embedded_agents() {
            return Ok(agents);
        }

        // Fallback to loading from filesystem
        Self::load_agents_from_filesystem().await
    }

    /// Load agents from embedded data (compiled into binary)
    fn load_embedded_agents() -> Result<Vec<Agent>> {
        let embedded_agents = vec![
            ("backend_engineer_rust.md", include_str!("../../agents/backend_engineer_rust.md")),
            ("backend_qa_rust.md", include_str!("../../agents/backend_qa_rust.md")),
            ("codereview_eng.md", include_str!("../../agents/codereview_eng.md")),
            ("devops.md", include_str!("../../agents/devops.md")),
            ("frontend_engineer_react.md", include_str!("../../agents/frontend_engineer_react.md")),
            ("frontend_engineer_vue.md", include_str!("../../agents/frontend_engineer_vue.md")),
            ("frontend_qa_react.md", include_str!("../../agents/frontend_qa_react.md")),
            ("frontend_qa_vue.md", include_str!("../../agents/frontend_qa_vue.md")),
            ("performance_engineer.md", include_str!("../../agents/performance_engineer.md")),
            ("release_mmanager.md", include_str!("../../agents/release_mmanager.md")),
            ("security_engineer.md", include_str!("../../agents/security_engineer.md")),
        ];

        let mut agents = Vec::new();
        for (filename, content) in embedded_agents {
            let agent_type = Self::parse_agent_type_from_filename(filename);

            // Extract name from filename (remove .md extension)
            let name = filename.trim_end_matches(".md").replace('_', " ");
            let name = name.split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            // Parse YAML front matter to extract description
            let description = Self::extract_description_from_content(content, &name);

            let agent = Agent::new_with_type(
                name,
                std::path::PathBuf::from(format!("embedded://{}", filename)),
                description,
                agent_type
            );
            agents.push(agent);
        }

        Ok(agents)
    }

    /// Load agents from filesystem (fallback)
    async fn load_agents_from_filesystem() -> Result<Vec<Agent>> {
        use std::fs;
        use std::path::Path;

        // Try multiple possible locations for the agents directory
        let possible_paths = vec![
            Path::new("./agents").to_path_buf(),
            Path::new("../agents").to_path_buf(),
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.join("agents")))
                .unwrap_or_else(|| Path::new("agents").to_path_buf()),
            dirs::home_dir()
                .map(|h| h.join(".orchy").join("agents"))
                .unwrap_or_else(|| Path::new("agents").to_path_buf()),
            // Try relative to the source directory (for development)
            Path::new(env!("CARGO_MANIFEST_DIR")).join("agents"),
        ];

        let mut agents_path = None;
        for path in &possible_paths {
            if path.exists() && path.is_dir() {
                agents_path = Some(path.clone());
                break;
            }
        }

        let agents_path = agents_path.ok_or_else(|| {
            OrchestratorError::validation(
                "No agents directory found and embedded agents failed to load".to_string()
            )
        })?;

        let mut agents = Vec::new();
        let entries = fs::read_dir(&agents_path)
            .map_err(|e| OrchestratorError::internal(format!("Failed to read agents directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| OrchestratorError::internal(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    let agent_type = Self::parse_agent_type_from_filename(filename);

                    // Read the file content to get description
                    let content = fs::read_to_string(&path)
                        .map_err(|e| OrchestratorError::internal(format!("Failed to read agent file: {}", e)))?;

                    // Extract name from filename (remove .md extension)
                    let name = filename.trim_end_matches(".md").replace('_', " ");
                    let name = name.split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ");

                    // Parse YAML front matter to extract description
                    let description = Self::extract_description_from_content(&content, &name);

                    let agent = Agent::new_with_type(name, path, description, agent_type);
                    agents.push(agent);
                }
            }
        }

        Ok(agents)
    }

    /// Extract description from agent file content (YAML front matter)
    fn extract_description_from_content(content: &str, fallback_name: &str) -> String {
        // Check if content starts with YAML front matter
        if content.starts_with("---") {
            let lines: Vec<&str> = content.lines().collect();
            let mut in_front_matter = false;
            let mut front_matter_end = 0;

            // Find the end of front matter
            for (i, line) in lines.iter().enumerate() {
                if i == 0 && line.trim() == "---" {
                    in_front_matter = true;
                    continue;
                }
                if in_front_matter && line.trim() == "---" {
                    front_matter_end = i;
                    break;
                }
            }

            // Extract description from front matter
            if front_matter_end > 0 {
                for i in 1..front_matter_end {
                    let line = lines[i].trim();
                    if line.starts_with("description:") {
                        let description = line.trim_start_matches("description:")
                            .trim()
                            .trim_matches('\'')
                            .trim_matches('"')
                            .to_string();
                        if !description.is_empty() {
                            return description;
                        }
                    }
                }
            }
        }

        // Fallback: use the first non-empty line that's not front matter or markdown header
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with("---")
                && !trimmed.starts_with('#')
                && !trimmed.starts_with("name:")
                && !trimmed.starts_with("description:")
                && !trimmed.starts_with("tools:")
                && !trimmed.starts_with("technologies:")
            {
                return trimmed.to_string();
            }
        }

        // Final fallback: use the agent name
        fallback_name.to_string()
    }



    /// Transition the agent to a new status with validation
    pub fn transition_to(&mut self, new_status: AgentStatus, reason: Option<String>) -> Result<()> {
        // Validate the transition
        let next_status = self.status.transition_to(new_status)?;
        let now = Utc::now();

        // Apply business rules
        match (&self.status, &next_status) {
            // Track when agent becomes active
            (_, AgentStatus::Active | AgentStatus::Working) => {
                self.last_active_at = Some(now);
            }
            // Increment error count
            (_, AgentStatus::Error) => {
                self.error_count += 1;
            }
            // Reset error count after maintenance
            (AgentStatus::Maintenance, AgentStatus::Idle) => {
                self.error_count = 0;
            }
            _ => {}
        }

        // Record the change
        self.status_history.push(AgentStatusChange {
            from: Some(self.status),
            to: next_status,
            reason,
            timestamp: now,
        });

        self.status = next_status;
        self.updated_at = now;

        Ok(())
    }

    /// Start working on a task
    pub fn start_work(&mut self, task_description: impl Into<String>) -> Result<()> {
        if !self.status.is_available() {
            return Err(OrchestratorError::agent_constraint(format!(
                "Agent is not available. Current status: {}",
                self.status
            )));
        }

        self.transition_to(
            AgentStatus::Working,
            Some(format!("Started: {}", task_description.into())),
        )
    }

    /// Complete current work and become active
    pub fn complete_work(&mut self) -> Result<()> {
        if self.status != AgentStatus::Working {
            return Err(OrchestratorError::agent_constraint(
                "Agent is not currently working",
            ));
        }

        self.total_tasks_completed += 1;
        self.transition_to(AgentStatus::Active, Some("Work completed".to_string()))
    }

    /// Mark agent as busy (handling multiple tasks)
    pub fn mark_busy(&mut self, reason: impl Into<String>) -> Result<()> {
        if !matches!(self.status, AgentStatus::Working | AgentStatus::Active) {
            return Err(OrchestratorError::agent_constraint(
                "Can only become busy from Working or Active state",
            ));
        }

        self.transition_to(AgentStatus::Busy, Some(reason.into()))
    }

    /// Report an error
    pub fn report_error(&mut self, error_description: impl Into<String>) -> Result<()> {
        if self.status == AgentStatus::Maintenance {
            return Err(OrchestratorError::agent_constraint(
                "Cannot report error during maintenance",
            ));
        }

        self.transition_to(AgentStatus::Error, Some(error_description.into()))
    }

    /// Start maintenance
    pub fn start_maintenance(&mut self, reason: impl Into<String>) -> Result<()> {
        if !matches!(self.status, AgentStatus::Idle | AgentStatus::Error) {
            return Err(OrchestratorError::agent_constraint(
                "Can only start maintenance from Idle or Error state",
            ));
        }

        self.transition_to(AgentStatus::Maintenance, Some(reason.into()))
    }

    /// Complete maintenance and return to idle
    pub fn complete_maintenance(&mut self) -> Result<()> {
        if self.status != AgentStatus::Maintenance {
            return Err(OrchestratorError::agent_constraint(
                "Agent is not in maintenance",
            ));
        }

        self.transition_to(AgentStatus::Idle, Some("Maintenance completed".to_string()))
    }

    /// Go idle
    pub fn go_idle(&mut self, reason: Option<String>) -> Result<()> {
        if !matches!(
            self.status,
            AgentStatus::Active | AgentStatus::Busy | AgentStatus::Error
        ) {
            return Err(OrchestratorError::agent_constraint(
                "Cannot go idle from current state",
            ));
        }

        self.transition_to(
            AgentStatus::Idle,
            reason.or_else(|| Some("Going idle".to_string())),
        )
    }

    /// Get uptime (time since creation)
    pub fn uptime(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }

    /// Get time in current status
    pub fn time_in_current_status(&self) -> chrono::Duration {
        if let Some(last_change) = self.status_history.last() {
            Utc::now() - last_change.timestamp
        } else {
            chrono::Duration::zero()
        }
    }

    /// Get total time in a specific status
    pub fn total_time_in_status(&self, status: AgentStatus) -> chrono::Duration {
        let mut total = chrono::Duration::zero();
        let mut in_status = false;
        let mut start_time = None;

        for change in &self.status_history {
            if change.to == status {
                in_status = true;
                start_time = Some(change.timestamp);
            } else if in_status && change.from == Some(status) {
                if let Some(start) = start_time {
                    total = total + (change.timestamp - start);
                }
                in_status = false;
                start_time = None;
            }
        }

        // If still in the status, add time until now
        if in_status {
            if let Some(start) = start_time {
                total = total + (Utc::now() - start);
            }
        }

        total
    }

    /// Get agent health score (0-100)
    pub fn health_score(&self) -> u8 {
        let base_score = 100u8;
        let error_penalty = (self.error_count * 10).min(50) as u8;
        let status_penalty = match self.status {
            AgentStatus::Error => 30,
            AgentStatus::Maintenance => 20,
            AgentStatus::Busy => 5,
            _ => 0,
        };

        base_score
            .saturating_sub(error_penalty)
            .saturating_sub(status_penalty)
    }

    /// Save agent state to file
    pub async fn save_state(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(&self.file_path).await?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }

    /// Load agent state from file
    pub async fn load_state(
        file_path: PathBuf,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let contents = tokio::fs::read_to_string(&file_path).await?;
        let agent: Agent = serde_json::from_str(&contents)?;
        Ok(agent)
    }

    // ===== ERROR RECOVERY METHODS =====

    /// Record an action error for later recovery analysis
    pub fn record_action_error(&mut self, error: ActionError) {
        self.recent_errors.push(error);
        self.error_count += 1;
        self.updated_at = Utc::now();

        // Keep only the last 10 errors to prevent memory bloat
        if self.recent_errors.len() > 10 {
            self.recent_errors.drain(0..self.recent_errors.len() - 10);
        }
    }

    /// Check if agent should attempt autonomous recovery
    pub fn should_attempt_recovery(&self) -> bool {
        self.autonomous_recovery_enabled && 
        self.recovery_attempts < self.max_recovery_attempts &&
        !self.recent_errors.is_empty()
    }

    /// Collect relevant project files for error context
    async fn collect_relevant_files(
        &self, 
        project_path: &Path, 
        error: &ActionError,
        max_files: usize
    ) -> Result<Vec<FileContext>> {
        let mut relevant_files = Vec::new();
        
        // Collect files based on error context
        let search_patterns: Vec<String> = match error.action_type.as_str() {
            "CommandExecution" => vec!["package.json", "Cargo.toml", "requirements.txt", "*.config.*"].into_iter().map(String::from).collect(),
            "FileOperation" => {
                if let Some(working_dir) = &error.working_directory {
                    let parent = working_dir.parent().unwrap_or(working_dir);
                    vec![parent.to_string_lossy().to_string()]
                } else {
                    vec!["src/*", "*.json", "*.toml", "*.yaml"].into_iter().map(String::from).collect()
                }
            },
            _ => vec!["*.json", "*.toml", "*.yaml", "*.md", "src/*"].into_iter().map(String::from).collect(),
        };

        // Try to collect files matching patterns
        for pattern in search_patterns {
            if relevant_files.len() >= max_files {
                break;
            }

            // Simple pattern matching for common files
            let files_to_check = if pattern.contains('*') {
                self.glob_files(project_path, &pattern).await.unwrap_or_default()
            } else {
                vec![project_path.join(&pattern)]
            };

            for file_path in files_to_check {
                if relevant_files.len() >= max_files {
                    break;
                }

                if file_path.exists() && file_path.is_file() {
                    match FileContext::from_path(file_path.clone(), &project_path.to_path_buf()).await {
                        Ok(mut file_context) => {
                            file_context.truncate_content(2000); // Limit content size
                            relevant_files.push(file_context);
                        }
                        Err(_) => continue, // Skip files we can't read
                    }
                }
            }
        }

        Ok(relevant_files)
    }

    /// Simple glob-like file matching
    async fn glob_files(&self, base_path: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        // Handle simple patterns like "*.json", "src/*"
        if pattern.starts_with("*.{") {
            let extensions_str = &pattern[3..pattern.len()-1];
            let extensions: Vec<&str> = extensions_str.split(',').map(|s| s.trim()).collect();
            for ext in extensions {
                self.collect_files_by_extension(&base_path.to_path_buf(), ext, &mut files).await?;
            }
        } else if pattern.starts_with("*.") {
            let extension = &pattern[2..];
            self.collect_files_by_extension(&base_path.to_path_buf(), extension, &mut files).await?;
        } else if pattern.ends_with("/*") {
            let dir_name = &pattern[..pattern.len()-2];
            let dir_path = base_path.join(dir_name);
            if dir_path.exists() && dir_path.is_dir() {
                self.collect_files_in_directory(&dir_path, &mut files).await?;
            }
        } else {
            // Exact file match
            let file_path = base_path.join(pattern);
            if file_path.exists() {
                files.push(file_path);
            }
        }

        Ok(files)
    }

    /// Collect files by extension recursively
    #[async_recursion]
    async fn collect_files_by_extension(
        &self, 
        dir: &PathBuf, 
        extension: &str, 
        files: &mut Vec<PathBuf>
    ) -> Result<()> {
        if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == extension {
                            files.push(path);
                        }
                    }
                } else if path.is_dir() && files.len() < 20 {
                    // Recurse into subdirectories but limit depth
                    let _ = self.collect_files_by_extension(&path, extension, files).await;
                }
            }
        }
        Ok(())
    }

    /// Collect files in a specific directory
    async fn collect_files_in_directory(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    /// Build project structure snapshot
    async fn build_project_structure(&self, project_path: &Path) -> Vec<String> {
        let mut structure = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                if path.is_dir() {
                    structure.push(format!("{}/", name));
                    
                    // Add some files from important directories
                    if matches!(name.as_str(), "src" | "lib" | "app" | "components") {
                        if let Ok(sub_entries) = std::fs::read_dir(&path) {
                            for sub_entry in sub_entries.flatten().take(5) {
                                let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                                structure.push(format!("  {}/{}", name, sub_name));
                            }
                        }
                    }
                } else {
                    structure.push(name);
                }
            }
        }

        structure.sort();
        structure
    }

    /// Attempt autonomous error recovery using MCP
    pub async fn attempt_error_recovery(
        &mut self,
        mcp_client: &McpClient,
        project_path: PathBuf,
        tech_stack: String,
        task_title: Option<String>,
        previous_actions: Vec<String>,
    ) -> Result<ErrorRecoveryResponse> {
        if !self.should_attempt_recovery() {
            return Err(OrchestratorError::agent_constraint(
                "Agent not eligible for autonomous recovery"
            ));
        }

        let last_error = self.recent_errors.last()
            .ok_or_else(|| OrchestratorError::validation("No recent errors to recover from"))?;

        debug!("🔧 Agent {} attempting autonomous recovery for error: {}", 
               self.name, last_error.error_message);

        // Increment recovery attempts
        self.recovery_attempts += 1;
        self.last_error_recovery_at = Some(Utc::now());
        self.updated_at = Utc::now();

        // Collect context for recovery
        let relevant_files = self.collect_relevant_files(&project_path, last_error, 5).await?;
        let project_structure = self.build_project_structure(&project_path).await;

        // Convert FileContext to (String, String) for prompt
        let relevant_files_for_prompt: Vec<(String, String)> = relevant_files
            .iter()
            .map(|fc| (fc.relative_path.clone(), fc.content.clone()))
            .collect();

        // Generate error recovery prompt
        let prompt = Prompts::agent_error_recovery_prompt(
            &self.name,
            &format!("{:?}", self.agent_type),
            task_title.as_deref(),
            &project_path.to_string_lossy(),
            &tech_stack,
            &last_error.action_type,
            &last_error.action_description,
            &last_error.error_message,
            last_error.error_code,
            last_error.working_directory.as_deref().map(|p| p.to_string_lossy().into_owned()).as_deref(),
            last_error.retry_count,
            last_error.stdout.as_deref(),
            last_error.stderr.as_deref(),
            &previous_actions,
            &project_structure,
            &relevant_files_for_prompt,
        );

        debug!("🧠 Sending error recovery request to LLM for agent {}", self.name);

        // Call LLM for error recovery analysis
        match mcp_client.error_recovery_analysis(prompt, crate::managers::McpModel::Gemini).await {
            Ok(response) => {
                debug!("✅ Agent {} received recovery response with {} actions", 
                       self.name, response.recovery_actions.len());
                
                // Update agent status to indicate recovery is in progress
                let _ = self.transition_to(
                    AgentStatus::Maintenance, 
                    Some("Attempting autonomous error recovery".to_string())
                );

                Ok(response)
            }
            Err(e) => {
                debug!("❌ Agent {} error recovery analysis failed: {}", self.name, e);
                
                // Mark recovery attempt as failed
                let _ = self.report_error(format!("Error recovery analysis failed: {}", e));
                
                Err(e)
            }
        }
    }

    /// Check if agent has recent errors that need attention
    pub fn needs_error_attention(&self) -> bool {
        !self.recent_errors.is_empty() && 
        self.error_count > 0 &&
        self.recovery_attempts < self.max_recovery_attempts
    }

    /// Get the most recent action error
    pub fn get_latest_error(&self) -> Option<&ActionError> {
        self.recent_errors.last()
    }

    /// Clear recent errors (called after successful recovery)
    pub fn clear_recent_errors(&mut self) {
        self.recent_errors.clear();
        self.recovery_attempts = 0;
        self.updated_at = Utc::now();
    }

    /// Enable or disable autonomous recovery
    pub fn set_autonomous_recovery(&mut self, enabled: bool) {
        self.autonomous_recovery_enabled = enabled;
        self.updated_at = Utc::now();
    }

    /// Set maximum recovery attempts
    pub fn set_max_recovery_attempts(&mut self, max_attempts: u32) {
        self.max_recovery_attempts = max_attempts;
        self.updated_at = Utc::now();
    }

    /// Get recovery statistics
    pub fn get_recovery_stats(&self) -> (u32, u32, Option<DateTime<Utc>>) {
        (self.recovery_attempts,
            self.recent_errors.len() as u32,
            self.last_error_recovery_at,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("TestAgent", PathBuf::from("/tmp/agent.json"), "Test agent");
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.error_count, 0);
        assert_eq!(agent.total_tasks_completed, 0);
    }

    #[test]
    fn test_normal_workflow() {
        let mut agent = Agent::new("Worker", PathBuf::from("/tmp/worker.json"), "Worker agent");

        // Start work
        agent
            .start_work("Process data")
            .expect("Should be able to start work");
        assert_eq!(agent.status, AgentStatus::Working);
        assert!(agent.last_active_at.is_some());

        // Complete work
        agent
            .complete_work()
            .expect("Should be able to complete work");
        assert_eq!(agent.status, AgentStatus::Active);
        assert_eq!(agent.total_tasks_completed, 1);

        // Go idle
        agent.go_idle(None).expect("Should be able to go idle");
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn test_error_handling() {
        let mut agent = Agent::new("ErrorAgent", PathBuf::from("/tmp/error.json"), "Test error");

        agent
            .start_work("Risky task")
            .expect("Should be able to start work");
        agent
            .report_error("Task failed")
            .expect("Should be able to report error");
        assert_eq!(agent.status, AgentStatus::Error);
        assert_eq!(agent.error_count, 1);

        // Start maintenance
        agent
            .start_maintenance("Fixing errors")
            .expect("Should be able to start maintenance");
        assert_eq!(agent.status, AgentStatus::Maintenance);

        // Complete maintenance
        agent
            .complete_maintenance()
            .expect("Should be able to complete maintenance");
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.error_count, 0); // Reset after maintenance
    }

    #[test]
    fn test_busy_state() {
        let mut agent = Agent::new("BusyAgent", PathBuf::from("/tmp/busy.json"), "Busy agent");

        agent
            .start_work("Task 1")
            .expect("Should be able to start work");
        agent
            .mark_busy("Handling multiple requests")
            .expect("Should be able to mark busy");
        assert_eq!(agent.status, AgentStatus::Busy);

        // Can go back to active
        agent
            .transition_to(AgentStatus::Active, Some("Load reduced".to_string()))
            .expect("Should be able to transition to active");
        assert_eq!(agent.status, AgentStatus::Active);
    }

    #[test]
    fn test_invalid_transitions() {
        let mut agent = Agent::new("TestAgent", PathBuf::from("/tmp/test.json"), "Test");

        // Cannot go from Idle to Active directly
        assert!(agent.transition_to(AgentStatus::Active, None).is_err());

        // Cannot go from Idle to Error
        assert!(agent.transition_to(AgentStatus::Error, None).is_err());

        // Cannot work during maintenance
        agent
            .start_maintenance("Updates")
            .expect("Should be able to start maintenance");
        assert!(agent.start_work("Task").is_err());
    }

    #[test]
    fn test_health_score() {
        let mut agent = Agent::new(
            "HealthAgent",
            PathBuf::from("/tmp/health.json"),
            "Health test",
        );

        // Perfect health initially
        assert_eq!(agent.health_score(), 100);

        // Errors reduce health
        agent
            .start_work("Task")
            .expect("Should be able to start work");
        agent
            .report_error("Failed")
            .expect("Should be able to report error");
        agent.go_idle(None).expect("Should be able to go idle");
        agent
            .start_work("Task2")
            .expect("Should be able to start work again");
        agent
            .report_error("Failed again")
            .expect("Should be able to report error again");

        // 2 errors = -20, Error status = -30, Total = 50
        assert_eq!(agent.health_score(), 50);
    }
}
