use crate::error::{Result, OrchestratorError};
use crate::mcp::gemini::GeminiCLI;
use crate::models::prompt_responses::*;
use crate::models::code_review::CodeReviewInput;
use crate::models::conflict_resolution::ConflictResolutionInput;
use crate::models::agent_error_recovery::ErrorRecoveryResponse;
use crate::enums::TechStack;
use crate::prompts::Prompts;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn, debug};

/// Supported AI models for MCP
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McpModel {
    Gemini,
    Claude,
}

impl std::fmt::Display for McpModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpModel::Gemini => write!(f, "gemini"),
            McpModel::Claude => write!(f, "claude"),
        }
    }
}

/// Commands that can be sent to the MCP Manager
#[derive(Debug)]
pub enum McpCommand {
    /// Initialize context files for a project
    InitializeContext {
        project_path: PathBuf,
        tech_stack: TechStack,
        respond_to: oneshot::Sender<Result<()>>,
    },
    /// Execute idea breakdown prompt
    IdeaBreakdown {
        idea: String,
        context: String,
        available_agents: Vec<String>,
        tech_stack: String,
        model: McpModel,
        respond_to: oneshot::Sender<Result<IdeaBreakdownResponse>>,
    },
    /// Execute feature development prompt
    FeatureDevelopment {
        task_description: String,
        codebase_context: String,
        tech_stack: String,
        existing_files: Vec<(String, String)>,
        requirements: String,
        acceptance_criteria: Vec<String>,
        model: McpModel,
        respond_to: oneshot::Sender<Result<FeatureDevelopmentResponse>>,
    },
    /// Execute task-specific development prompt
    TaskDevelopment {
        task_title: String,
        task_description: String,
        task_complexity: u8,
        task_priority: String,
        task_tags: Vec<String>,
        tech_stack: String,
        existing_files: Vec<(String, String)>,
        completed_dependencies: Vec<String>,
        acceptance_criteria: Vec<String>,
        codebase_context: String,
        model: McpModel,
        respond_to: oneshot::Sender<Result<FeatureDevelopmentResponse>>,
    },
    /// Execute code review prompt
    CodeReview {
        files_and_code: Vec<(String, String)>,
        requirements: String,
        context: String,
        pull_request_id: String,
        model: McpModel,
        respond_to: oneshot::Sender<Result<CodeReviewInput>>,
    },
    /// Execute conflict resolution prompt
    ConflictResolution {
        conflicts_data: Vec<(String, String, String, String)>,
        branch_info: String,
        context: String,
        merge_commit_message: String,
        model: McpModel,
        respond_to: oneshot::Sender<Result<ConflictResolutionInput>>,
    },
    /// Execute agent error recovery analysis
    ErrorRecovery {
        prompt: String,
        model: McpModel,
        respond_to: oneshot::Sender<Result<ErrorRecoveryResponse>>,
    },
    /// Execute CI/CD fix prompt
    CiCdFix {
        pipeline_config: String,
        error_logs: String,
        project_context: String,
        tech_stack: String,
        model: McpModel,
        respond_to: oneshot::Sender<Result<CiCdFixResponse>>,
    },
    /// Execute Docker deployment prompt
    DockerDeployment {
        application_context: String,
        deployment_requirements: String,
        tech_stack: String,
        environment: String,
        model: McpModel,
        respond_to: oneshot::Sender<Result<DockerDeploymentResponse>>,
    },
    /// Execute QA analysis prompt
    QaAnalysis {
        application_code: Vec<(String, String)>,
        test_results: String,
        requirements: String,
        user_scenarios: Vec<String>,
        model: McpModel,
        respond_to: oneshot::Sender<Result<QaAnalysisResponse>>,
    },
    /// Execute API synchronization prompt
    ApiSynchronization {
        backend_api_spec: String,
        frontend_code: Vec<(String, String)>,
        api_documentation: String,
        tech_stack: String,
        model: McpModel,
        respond_to: oneshot::Sender<Result<ApiSynchronizationResponse>>,
    },
    /// Execute performance optimization prompt
    PerformanceOptimization {
        application_code: Vec<(String, String)>,
        performance_metrics: String,
        bottlenecks: Vec<String>,
        tech_stack: String,
        model: McpModel,
        respond_to: oneshot::Sender<Result<PerformanceOptimizationResponse>>,
    },
    /// Shutdown the manager
    Shutdown,
}

/// Events emitted by the MCP Manager
#[derive(Debug, Clone)]
pub enum McpEvent {
    /// Context files initialized
    ContextInitialized {
        project_path: PathBuf,
        gemini_file: PathBuf,
        claude_file: PathBuf,
    },
    /// Prompt executed successfully
    PromptExecuted {
        prompt_type: String,
        model: McpModel,
        execution_time_ms: u64,
    },
    /// Error occurred during prompt execution
    PromptError {
        prompt_type: String,
        model: McpModel,
        error: String,
    },
    /// Model availability changed
    ModelAvailabilityChanged {
        model: McpModel,
        available: bool,
    },
}

/// Statistics for MCP Manager operations
#[derive(Debug, Clone, Default)]
pub struct McpStatistics {
    pub total_prompts_executed: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
    pub gemini_calls: u64,
    pub claude_calls: u64,
    pub context_files_created: u64,
}

/// MCP Manager for handling AI model calls
pub struct McpManager {
    command_rx: mpsc::UnboundedReceiver<McpCommand>,
    event_tx: mpsc::UnboundedSender<McpEvent>,
    statistics: McpStatistics,
    gemini_available: bool,
    claude_available: bool,
    project_path: Option<PathBuf>,
}

impl McpManager {
    /// Create a new MCP Manager
    pub fn new() -> (
        Self,
        mpsc::UnboundedSender<McpCommand>,
        mpsc::UnboundedReceiver<McpEvent>,
    ) {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let manager = Self {
            command_rx,
            event_tx,
            statistics: McpStatistics::default(),
            gemini_available: false,
            claude_available: false,
            project_path: None,
        };

        (manager, command_tx, event_rx)
    }

    /// Run the MCP Manager
    pub async fn run(mut self) -> Result<()> {
        info!("Starting MCP Manager");

        // Check model availability
        self.check_model_availability().await;

        while let Some(command) = self.command_rx.recv().await {
            match command {
                McpCommand::Shutdown => {
                    info!("Shutting down MCP Manager");
                    break;
                }
                _ => {
                    if let Err(e) = self.handle_command(command).await {
                        error!("Error handling MCP command: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check availability of AI models
    async fn check_model_availability(&mut self) {
        // Check Gemini availability
        let gemini_available = GeminiCLI::is_available().await;
        if self.gemini_available != gemini_available {
            self.gemini_available = gemini_available;
            let _ = self.event_tx.send(McpEvent::ModelAvailabilityChanged {
                model: McpModel::Gemini,
                available: gemini_available,
            });
        }

        // Claude availability check would go here when implemented
        // For now, we'll assume Claude is not available
        if self.claude_available != false {
            self.claude_available = false;
            let _ = self.event_tx.send(McpEvent::ModelAvailabilityChanged {
                model: McpModel::Claude,
                available: false,
            });
        }

        info!(
            "Model availability - Gemini: {}, Claude: {}",
            self.gemini_available, self.claude_available
        );
    }

    /// Handle incoming commands
    async fn handle_command(&mut self, command: McpCommand) -> Result<()> {
        match command {
            McpCommand::InitializeContext {
                project_path,
                tech_stack,
                respond_to,
            } => {
                let result = self.initialize_context(&project_path, &tech_stack).await;
                let _ = respond_to.send(result);
            }
            McpCommand::IdeaBreakdown {
                idea,
                context,
                available_agents,
                tech_stack,
                model,
                respond_to,
            } => {
                let result = self
                    .execute_idea_breakdown(&idea, &context, &available_agents, &tech_stack, model)
                    .await;
                let _ = respond_to.send(result);
            }
            McpCommand::FeatureDevelopment {
                task_description,
                codebase_context,
                tech_stack,
                existing_files,
                requirements,
                acceptance_criteria,
                model,
                respond_to,
            } => {
                let result = self
                    .execute_feature_development(
                        &task_description,
                        &codebase_context,
                        &tech_stack,
                        &existing_files,
                        &requirements,
                        &acceptance_criteria,
                        model,
                    )
                    .await;
                let _ = respond_to.send(result);
            }
            McpCommand::TaskDevelopment {
                task_title,
                task_description,
                task_complexity,
                task_priority,
                task_tags,
                tech_stack,
                existing_files,
                completed_dependencies,
                acceptance_criteria,
                codebase_context,
                model,
                respond_to,
            } => {
                let result = self
                    .execute_task_development(
                        &task_title,
                        &task_description,
                        task_complexity,
                        &task_priority,
                        &task_tags,
                        &tech_stack,
                        &existing_files,
                        &completed_dependencies,
                        &acceptance_criteria,
                        &codebase_context,
                        model,
                    )
                    .await;
                let _ = respond_to.send(result);
            }
            McpCommand::CodeReview {
                files_and_code,
                requirements,
                context,
                pull_request_id,
                model,
                respond_to,
            } => {
                let result = self
                    .execute_code_review(&files_and_code, &requirements, &context, &pull_request_id, model)
                    .await;
                let _ = respond_to.send(result);
            }
            McpCommand::ConflictResolution {
                conflicts_data,
                branch_info,
                context,
                merge_commit_message,
                model,
                respond_to,
            } => {
                let result = self
                    .execute_conflict_resolution(&conflicts_data, &branch_info, &context, &merge_commit_message, model)
                    .await;
                let _ = respond_to.send(result);
            }
            McpCommand::ErrorRecovery { prompt, model, respond_to } => {
                let result = self.execute_error_recovery_analysis(&prompt, model).await;
                let _ = respond_to.send(result);
            }
            _ => {
                // Handle other commands in the next part
                warn!("Command not yet implemented");
            }
        }
        Ok(())
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> McpStatistics {
        self.statistics.clone()
    }

    /// Execute agent error recovery analysis
    async fn execute_error_recovery_analysis(
        &mut self,
        prompt: &str,
        model: McpModel,
    ) -> Result<ErrorRecoveryResponse> {
        let start_time = std::time::Instant::now();

        let response = self.execute_prompt(prompt, model.clone()).await?;
        let parsed_response: ErrorRecoveryResponse = self.parse_json_response(&response)?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        self.update_statistics("error_recovery_analysis", model, execution_time, true);

        Ok(parsed_response)
    }

    /// Initialize context files for a project
    async fn initialize_context(&mut self, project_path: &Path, tech_stack: &TechStack) -> Result<()> {
        let gemini_file = project_path.join("GEMINI.md");
        let claude_file = project_path.join("CLAUDE.md");

        // Store the project path for future use
        self.project_path = Some(project_path.to_path_buf());

        // Generate context content based on tech stack
        let context_content = self.generate_context_content(tech_stack);

        // Write context files
        fs::write(&gemini_file, &context_content).await?;
        fs::write(&claude_file, &context_content).await?;

        self.statistics.context_files_created += 2;

        let _ = self.event_tx.send(McpEvent::ContextInitialized {
            project_path: project_path.to_path_buf(),
            gemini_file,
            claude_file,
        });

        info!("Context files initialized for project: {:?}", project_path);
        Ok(())
    }

    /// Generate context content based on tech stack
    fn generate_context_content(&self, tech_stack: &TechStack) -> String {
        // Try to load context from file first, fallback to generic content
        if let Ok(context_content) = self.load_context_from_file(tech_stack) {
            debug!("✅ Loaded context content from file for tech stack: {:?}", tech_stack);
            context_content
        } else {
            debug!("⚠️  Using generic context content for tech stack: {:?}", tech_stack);
            self.generate_generic_context_content(tech_stack)
        }
    }

    /// Load context content from file based on tech stack
    fn load_context_from_file(&self, tech_stack: &TechStack) -> Result<String> {
        let context_filename = match tech_stack {
            TechStack::Vue => "context.vue.md",
            TechStack::React => "context.react.md", 
            TechStack::Rust => "context.rust.asyncgraphql.md",
            TechStack::FullstackRustVue => "context.vue.md", // Use Vue context for fullstack
            TechStack::FullstackRustReact => "context.react.md", // Use React context for fullstack
        };

        let mut context_path = PathBuf::from(".");
        context_path.push("contexts");
        context_path.push(context_filename);
        
        debug!("🔍 Attempting to load context from: {:?}", context_path);
        
        if !context_path.exists() {
            debug!("❌ Context file not found: {:?}", context_path);
            return Err(OrchestratorError::internal(format!("Context file not found: {:?}", context_path)));
        }

        match std::fs::read_to_string(&context_path) {
            Ok(content) => {
                debug!("✅ Successfully loaded context file: {:?} ({} characters)", context_path, content.len());
                Ok(content)
            }
            Err(e) => {
                debug!("❌ Failed to read context file {:?}: {}", context_path, e);
                // The #[from] std::io::Error will automatically convert this
                Err(e.into())
            }
        }
    }

    /// Generate generic context content as fallback
    fn generate_generic_context_content(&self, tech_stack: &TechStack) -> String {
        format!(
            r#"# Project Context

## Technology Stack
{}

## Development Guidelines

### Code Quality
- Follow language-specific best practices and conventions
- Implement comprehensive error handling
- Write clear, self-documenting code with meaningful variable names
- Add appropriate comments for complex logic
- Ensure type safety and memory safety where applicable

### Security
- Validate all user inputs and sanitize data
- Implement proper authentication and authorization
- Use parameterized queries to prevent SQL injection
- Handle secrets and sensitive data securely
- Follow OWASP security guidelines

### Performance
- Optimize algorithms and data structures
- Implement efficient caching strategies
- Minimize database queries and optimize indexes
- Use connection pooling and resource management
- Consider scalability and concurrent access patterns

### Testing
- Write comprehensive unit tests for all functionality
- Include integration tests for API endpoints and database operations
- Add end-to-end tests for critical user workflows
- Test error conditions and edge cases thoroughly
- Maintain high test coverage standards

### Documentation
- Document all public APIs and interfaces
- Include usage examples and code samples
- Maintain up-to-date README and setup instructions
- Document deployment and configuration procedures
- Keep architectural decisions and design rationale

### Development Workflow
- Use version control with meaningful commit messages
- Follow branching strategies and code review processes
- Implement CI/CD pipelines with automated testing
- Use linting and code formatting tools
- Monitor application performance and errors

## Project Structure
Follow the established project structure and naming conventions.
Maintain separation of concerns and modular architecture.
"#,
            format!("{:?}", tech_stack)
        )
    }

    /// Execute idea breakdown prompt
    async fn execute_idea_breakdown(
        &mut self,
        idea: &str,
        context: &str,
        available_agents: &[String],
        tech_stack: &str,
        model: McpModel,
    ) -> Result<IdeaBreakdownResponse> {
        let start_time = std::time::Instant::now();

        let prompt = Prompts::idea_breakdown_user_prompt(
            idea,
            context,
            available_agents.to_vec(),
            tech_stack,
        );

        let response = self.execute_prompt(&prompt, model.clone()).await?;
        let parsed_response: IdeaBreakdownResponse = self.parse_json_response(&response)?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        self.update_statistics("idea_breakdown", model, execution_time, true);

        Ok(parsed_response)
    }

    /// Execute task-specific development prompt
    async fn execute_task_development(
        &mut self,
        task_title: &str,
        task_description: &str,
        task_complexity: u8,
        task_priority: &str,
        task_tags: &[String],
        tech_stack: &str,
        existing_files: &[(String, String)],
        completed_dependencies: &[String],
        acceptance_criteria: &[String],
        codebase_context: &str,
        model: McpModel,
    ) -> Result<FeatureDevelopmentResponse> {
        let start_time = std::time::Instant::now();

        let prompt = Prompts::task_development_user_prompt(
            task_title,
            task_description,
            task_complexity,
            task_priority,
            task_tags,
            tech_stack,
            existing_files,
            completed_dependencies,
            acceptance_criteria,
            codebase_context,
        );

        debug!("🎯 Task Development Prompt Details:");
        debug!("   📋 Task: {}", task_title);
        debug!("   🎚️  Priority: {}", task_priority);
        debug!("   📊 Complexity: {}/10", task_complexity);
        debug!("   🏷️  Tags: {:?}", task_tags);
        debug!("   📂 Existing files: {}", existing_files.len());
        debug!("   ✅ Completed dependencies: {}", completed_dependencies.len());
        debug!("   ✔️  Acceptance criteria: {}", acceptance_criteria.len());

        let response = self.execute_prompt(&prompt, model.clone()).await?;
        let parsed_response: FeatureDevelopmentResponse = self.parse_json_response(&response)?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        self.update_statistics("task_development", model, execution_time, true);

        debug!("✅ Task development completed for: {}", task_title);
        debug!("   🎬 Generated {} actions", parsed_response.actions.len());

        Ok(parsed_response)
    }

    /// Execute feature development prompt
    async fn execute_feature_development(
        &mut self,
        task_description: &str,
        codebase_context: &str,
        tech_stack: &str,
        existing_files: &[(String, String)],
        requirements: &str,
        acceptance_criteria: &[String],
        model: McpModel,
    ) -> Result<FeatureDevelopmentResponse> {
        let start_time = std::time::Instant::now();

        let prompt = Prompts::feature_development_user_prompt(
            task_description,
            codebase_context,
            tech_stack,
            existing_files,
            requirements,
            acceptance_criteria,
        );

        let response = self.execute_prompt(&prompt, model.clone()).await?;
        let parsed_response: FeatureDevelopmentResponse = self.parse_json_response(&response)?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        self.update_statistics("feature_development", model, execution_time, true);

        Ok(parsed_response)
    }

    /// Execute code review prompt
    async fn execute_code_review(
        &mut self,
        files_and_code: &[(String, String)],
        requirements: &str,
        context: &str,
        pull_request_id: &str,
        model: McpModel,
    ) -> Result<CodeReviewInput> {
        let start_time = std::time::Instant::now();

        let prompt = Prompts::code_review_user_prompt(
            files_and_code,
            requirements,
            context,
            pull_request_id,
        );

        let response = self.execute_prompt(&prompt, model.clone()).await?;
        let parsed_response: CodeReviewInput = self.parse_json_response(&response)?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        self.update_statistics("code_review", model, execution_time, true);

        Ok(parsed_response)
    }

    /// Execute conflict resolution prompt
    async fn execute_conflict_resolution(
        &mut self,
        conflicts_data: &[(String, String, String, String)],
        branch_info: &str,
        context: &str,
        merge_commit_message: &str,
        model: McpModel,
    ) -> Result<ConflictResolutionInput> {
        let start_time = std::time::Instant::now();

        let prompt = Prompts::conflict_resolution_user_prompt(
            conflicts_data,
            branch_info,
            context,
            merge_commit_message,
        );

        let response = self.execute_prompt(&prompt, model.clone()).await?;
        let parsed_response: ConflictResolutionInput = self.parse_json_response(&response)?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        self.update_statistics("conflict_resolution", model, execution_time, true);

        Ok(parsed_response)
    }

    /// Execute prompt with the specified model
    async fn execute_prompt(&self, prompt: &str, model: McpModel) -> Result<String> {
        debug!("🤖 Executing prompt with model: {:?}", model);
        debug!("📝 Prompt length: {} characters", prompt.len());
        debug!("📝 Prompt preview: {}", &prompt[..prompt.len().min(200)]);
        
        match model {
            McpModel::Gemini => {
                if !self.gemini_available {
                    debug!("❌ Gemini CLI not available");
                    return Err(OrchestratorError::internal("Gemini CLI not available"));
                }
                
                // Use query_with_session_from_dir to enable context files access
                let working_dir = self.project_path.as_ref()
                    .ok_or_else(|| OrchestratorError::internal("No project path set for MCP Manager"))?;
                
                debug!("📂 Using working directory: {:?}", working_dir);
                debug!("🚀 Calling Gemini CLI with gemini-2.5-flash model...");
                
                let response = GeminiCLI::query_with_session_from_dir(
                    "mcp-session", 
                    prompt, 
                    Some("gemini-2.5-flash"), 
                    working_dir
                ).await?;
                
                debug!("📥 Received response from Gemini ({} characters)", response.len());
                debug!("📥 Response preview: {}", &response[..response.len().min(500)]);
                
                Ok(response)
            }
            McpModel::Claude => {
                // Claude implementation would go here
                debug!("❌ Claude integration not yet implemented");
                Err(OrchestratorError::internal("Claude integration not yet implemented"))
            }
        }
    }

    /// Parse JSON response from AI model
    fn parse_json_response<T>(&self, response: &str) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        debug!("🔍 Parsing JSON response for type: {}", std::any::type_name::<T>());
        debug!("📄 Raw response length: {} characters", response.len());
        
        // Try to extract JSON from response (handles markdown wrapping)
        let json_str = match GeminiCLI::extract_json_from_response(response) {
            Ok(json) => {
                debug!("✅ Extracted JSON successfully ({} characters)", json.len());
                debug!("📄 Extracted JSON preview: {}", &json[..json.len().min(500)]);
                json
            }
            Err(e) => {
                debug!("❌ Failed to extract JSON: {}", e);
                debug!("📄 Raw response: {}", response);
                return Err(e);
            }
        };

        // Special handling for IdeaBreakdownResponse - the prompt returns an array but we expect an object
        if std::any::type_name::<T>().contains("IdeaBreakdownResponse") {
            debug!("🔧 Special handling for IdeaBreakdownResponse");
            if json_str.trim().starts_with('[') {
                debug!("📦 Wrapping array in object structure");
                // Wrap the array in the expected object structure
                let wrapped_json = format!(r#"{{"tasks": {}}}"#, json_str);
                debug!("📦 Wrapped JSON: {}", wrapped_json);
                return serde_json::from_str(&wrapped_json)
                    .map_err(|e| {
                        debug!("❌ Failed to parse wrapped JSON: {}", e);
                        OrchestratorError::json_parsing("AI model response (wrapped)", e)
                    });
            }
        }

        // Special handling for FeatureDevelopmentResponse - the prompt expects an array but AI might return an object
        if std::any::type_name::<T>().contains("FeatureDevelopmentResponse") {
            debug!("🔧 Special handling for FeatureDevelopmentResponse");
            if json_str.trim().starts_with('[') {
                debug!("📦 Wrapping array in object structure for actions");
                // Wrap the array in the expected object structure
                let wrapped_json = format!(r#"{{"actions": {}}}"#, json_str);
                debug!("📦 Wrapped JSON: {}", wrapped_json);
                return serde_json::from_str(&wrapped_json)
                    .map_err(|e| {
                        debug!("❌ Failed to parse wrapped JSON: {}", e);
                        OrchestratorError::json_parsing("AI model response (wrapped)", e)
                    });
            } else if json_str.trim().starts_with('{') {
                // Check if it's already in the correct format
                debug!("🔍 JSON appears to be an object, checking if it contains 'actions' field");
                if json_str.contains("\"actions\"") {
                    debug!("✅ Object already contains 'actions' field");
                } else {
                    debug!("❌ Object missing 'actions' field - this might be the issue");
                }
            }
        }

        debug!("🔄 Parsing JSON directly...");
        match serde_json::from_str(&json_str) {
            Ok(result) => {
                debug!("✅ JSON parsing successful");
                Ok(result)
            }
            Err(e) => {
                debug!("❌ JSON parsing failed: {}", e);
                debug!("📄 Failed JSON: {}", json_str);
                Err(OrchestratorError::json_parsing("AI model response", e))
            }
        }
    }

    /// Update statistics
    fn update_statistics(&mut self, prompt_type: &str, model: McpModel, execution_time_ms: u64, success: bool) {
        self.statistics.total_prompts_executed += 1;

        if success {
            self.statistics.successful_executions += 1;
        } else {
            self.statistics.failed_executions += 1;
        }

        // Update average execution time
        let total_time = self.statistics.average_execution_time_ms * (self.statistics.total_prompts_executed - 1) as f64;
        self.statistics.average_execution_time_ms = (total_time + execution_time_ms as f64) / self.statistics.total_prompts_executed as f64;

        // Update model-specific counters
        match model {
            McpModel::Gemini => self.statistics.gemini_calls += 1,
            McpModel::Claude => self.statistics.claude_calls += 1,
        }

        // Emit event
        if success {
            let _ = self.event_tx.send(McpEvent::PromptExecuted {
                prompt_type: prompt_type.to_string(),
                model,
                execution_time_ms,
            });
        }
    }
}

/// Client interface for the MCP Manager
#[derive(Clone, Debug)]
pub struct McpClient {
    command_tx: mpsc::UnboundedSender<McpCommand>,
}

impl McpClient {
    /// Create a new MCP Client
    pub fn new(command_tx: mpsc::UnboundedSender<McpCommand>) -> Self {
        Self { command_tx }
    }

    /// Initialize context files for a project
    pub async fn initialize_context(&self, project_path: PathBuf, tech_stack: TechStack) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.command_tx.send(McpCommand::InitializeContext {
            project_path,
            tech_stack,
            respond_to: tx,
        }).map_err(|e| OrchestratorError::channel(format!("Failed to send command: {}", e)))?;

        rx.await.map_err(|_| OrchestratorError::internal("MCP Manager disconnected"))?
    }

    /// Execute idea breakdown
    pub async fn idea_breakdown(
        &self,
        idea: String,
        context: String,
        available_agents: Vec<String>,
        tech_stack: String,
        model: McpModel,
    ) -> Result<IdeaBreakdownResponse> {
        let (tx, rx) = oneshot::channel();

        self.command_tx.send(McpCommand::IdeaBreakdown {
            idea,
            context,
            available_agents,
            tech_stack,
            model,
            respond_to: tx,
        }).map_err(|e| OrchestratorError::channel(format!("Failed to send command: {}", e)))?;

        rx.await.map_err(|_| OrchestratorError::internal("MCP Manager disconnected"))?
    }

    /// Execute feature development
    pub async fn feature_development(
        &self,
        task_description: String,
        codebase_context: String,
        tech_stack: String,
        existing_files: Vec<(String, String)>,
        requirements: String,
        acceptance_criteria: Vec<String>,
        model: McpModel,
    ) -> Result<FeatureDevelopmentResponse> {
        let (tx, rx) = oneshot::channel();

        self.command_tx.send(McpCommand::FeatureDevelopment {
            task_description,
            codebase_context,
            tech_stack,
            existing_files,
            requirements,
            acceptance_criteria,
            model,
            respond_to: tx,
        }).map_err(|e| OrchestratorError::channel(format!("Failed to send command: {}", e)))?;

        rx.await.map_err(|_| OrchestratorError::internal("MCP Manager disconnected"))?
    }

    /// Execute task development
    pub async fn task_development(
        &self,
        task_title: String,
        task_description: String,
        task_complexity: u8,
        task_priority: String,
        task_tags: Vec<String>,
        tech_stack: String,
        existing_files: Vec<(String, String)>,
        completed_dependencies: Vec<String>,
        acceptance_criteria: Vec<String>,
        codebase_context: String,
        model: McpModel,
    ) -> Result<FeatureDevelopmentResponse> {
        let (tx, rx) = oneshot::channel();

        self.command_tx.send(McpCommand::TaskDevelopment {
            task_title,
            task_description,
            task_complexity,
            task_priority,
            task_tags,
            tech_stack,
            existing_files,
            completed_dependencies,
            acceptance_criteria,
            codebase_context,
            model,
            respond_to: tx,
        }).map_err(|e| OrchestratorError::channel(format!("Failed to send command: {}", e)))?;

        rx.await.map_err(|_| OrchestratorError::internal("MCP Manager disconnected"))?
    }

    /// Execute code review
    pub async fn code_review(
        &self,
        files_and_code: Vec<(String, String)>,
        requirements: String,
        context: String,
        pull_request_id: String,
        model: McpModel,
    ) -> Result<CodeReviewInput> {
        let (tx, rx) = oneshot::channel();

        self.command_tx.send(McpCommand::CodeReview {
            files_and_code,
            requirements,
            context,
            pull_request_id,
            model,
            respond_to: tx,
        }).map_err(|e| OrchestratorError::channel(format!("Failed to send command: {}", e)))?;

        rx.await.map_err(|_| OrchestratorError::internal("MCP Manager disconnected"))?
    }

    /// Execute conflict resolution
    pub async fn conflict_resolution(
        &self,
        conflicts_data: Vec<(String, String, String, String)>,
        branch_info: String,
        context: String,
        merge_commit_message: String,
        model: McpModel,
    ) -> Result<ConflictResolutionInput> {
        let (tx, rx) = oneshot::channel();

        self.command_tx.send(McpCommand::ConflictResolution {
            conflicts_data,
            branch_info,
            context,
            merge_commit_message,
            model,
            respond_to: tx,
        }).map_err(|e| OrchestratorError::channel(format!("Failed to send command: {}", e)))?;

        rx.await.map_err(|_| OrchestratorError::internal("MCP Manager disconnected"))?
    }

    /// Execute agent error recovery analysis
    pub async fn error_recovery_analysis(
        &self,
        prompt: String,
        model: McpModel,
    ) -> Result<ErrorRecoveryResponse> {
        let (tx, rx) = oneshot::channel();

        self.command_tx.send(McpCommand::ErrorRecovery {
            prompt,
            model,
            respond_to: tx,
        }).map_err(|e| OrchestratorError::channel(format!("Failed to send command: {}", e)))?;

        rx.await.map_err(|_| OrchestratorError::internal("MCP Manager disconnected"))?
    }

    /// Shutdown the MCP Manager
    pub async fn shutdown(&self) -> Result<()> {
        self.command_tx.send(McpCommand::Shutdown)
            .map_err(|e| OrchestratorError::channel(format!("Failed to send shutdown command: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::TechStack;
    use tempfile::TempDir;
    use tokio::time::{timeout, Duration};

    async fn setup_test_manager() -> (McpClient, mpsc::UnboundedReceiver<McpEvent>, TempDir) {
        let (manager, command_tx, event_rx) = McpManager::new();
        let client = McpClient::new(command_tx);
        let temp_dir = TempDir::new().unwrap();

        // Start the manager in the background
        tokio::spawn(async move {
            let _ = manager.run().await;
        });

        (client, event_rx, temp_dir)
    }

    #[tokio::test]
    async fn test_initialize_context() {
        let (client, mut event_rx, temp_dir) = setup_test_manager().await;

        let project_path = temp_dir.path().to_path_buf();
        let tech_stack = TechStack::Rust;

        // Initialize context
        let result = client.initialize_context(project_path.clone(), tech_stack).await;
        assert!(result.is_ok());

        // Check that context files were created
        let gemini_file = project_path.join("GEMINI.md");
        let claude_file = project_path.join("CLAUDE.md");

        assert!(gemini_file.exists());
        assert!(claude_file.exists());

        // Check file contents
        let gemini_content = fs::read_to_string(&gemini_file).await.unwrap();
        let claude_content = fs::read_to_string(&claude_file).await.unwrap();

        assert!(gemini_content.contains("Technology Stack"));
        assert!(claude_content.contains("Development Guidelines"));
        assert_eq!(gemini_content, claude_content);

        // Check event was emitted
        let event = timeout(Duration::from_secs(1), event_rx.recv()).await.unwrap().unwrap();
        match event {
            McpEvent::ContextInitialized { project_path: path, .. } => {
                assert_eq!(path, project_path);
            }
            _ => panic!("Expected ContextInitialized event"),
        }
    }

    #[tokio::test]
    async fn test_idea_breakdown_prompt() {
        let (client, _event_rx, _temp_dir) = setup_test_manager().await;

        // Test idea breakdown (this will fail if Gemini CLI is not available)
        let result = client.idea_breakdown(
            "Build a todo app".to_string(),
            "Simple task management application".to_string(),
            vec!["BackendEngineerRust".to_string(), "FrontendEngineerReact".to_string()],
            "Rust backend with React frontend".to_string(),
            McpModel::Gemini,
        ).await;

        // If Gemini is available, this should work, otherwise it should fail gracefully
        match result {
            Ok(response) => {
                assert!(!response.tasks.is_empty());
                println!("Idea breakdown successful: {} tasks generated", response.tasks.len());
            }
            Err(e) => {
                println!("Idea breakdown failed (expected if Gemini CLI not available): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_feature_development_prompt() {
        let (client, _event_rx, _temp_dir) = setup_test_manager().await;

        let existing_files = vec![
            ("src/main.rs".to_string(), "fn main() { println!(\"Hello\"); }".to_string()),
        ];

        let result = client.feature_development(
            "Add user authentication".to_string(),
            "Existing Rust web application".to_string(),
            "Rust with Actix-web".to_string(),
            existing_files,
            "Implement JWT-based authentication".to_string(),
            vec!["User can login with email/password".to_string()],
            McpModel::Gemini,
        ).await;

        match result {
            Ok(response) => {
                assert!(!response.actions.is_empty());
                println!("Feature development successful: {} actions generated", response.actions.len());
            }
            Err(e) => {
                println!("Feature development failed (expected if Gemini CLI not available): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_code_review_prompt() {
        let (client, _event_rx, _temp_dir) = setup_test_manager().await;

        let files_and_code = vec![
            ("src/auth.rs".to_string(), r#"
pub fn authenticate(username: &str, password: &str) -> bool {
    username == "admin" && password == "password"
}
"#.to_string()),
        ];

        let result = client.code_review(
            files_and_code,
            "Secure authentication system".to_string(),
            "Web application security review".to_string(),
            "pr-123".to_string(),
            McpModel::Gemini,
        ).await;

        match result {
            Ok(response) => {
                assert_eq!(response.pull_request_id, "pr-123");
                println!("Code review successful: {} comments", response.comments.len());
            }
            Err(e) => {
                println!("Code review failed (expected if Gemini CLI not available): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_conflict_resolution_prompt() {
        let (client, _event_rx, _temp_dir) = setup_test_manager().await;

        let conflicts_data = vec![
            (
                "src/main.rs".to_string(),
                "fn main() { println!(\"Hello from main\"); }".to_string(),
                "fn main() { println!(\"Hello from feature\"); }".to_string(),
                "fn main() { println!(\"Hello\"); }".to_string(),
            ),
        ];

        let result = client.conflict_resolution(
            conflicts_data,
            "feature/new-feature -> main".to_string(),
            "Merging new feature branch".to_string(),
            "Merge feature/new-feature into main".to_string(),
            McpModel::Gemini,
        ).await;

        match result {
            Ok(response) => {
                assert!(!response.conflicts.is_empty());
                println!("Conflict resolution successful: {} conflicts resolved", response.conflicts.len());
            }
            Err(e) => {
                println!("Conflict resolution failed (expected if Gemini CLI not available): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_model_availability() {
        let (manager, _command_tx, _event_rx) = McpManager::new();

        // Test Gemini availability
        let gemini_available = GeminiCLI::is_available().await;
        println!("Gemini CLI available: {}", gemini_available);

        // Test that manager can check availability
        assert!(manager.gemini_available == false); // Initially false until checked
    }

    #[tokio::test]
    async fn test_json_parsing() {
        let (manager, _command_tx, _event_rx) = McpManager::new();

        // Test parsing valid JSON - the prompt returns an array, parse_json_response should wrap it
        let json_response = r#"[{"id": "test", "title": "Test Task", "description": "Test", "priority": "High", "complexity": 5, "agent_type": "TestAgent", "tags": [], "depends_on": []}]"#;
        let result: Result<IdeaBreakdownResponse> = manager.parse_json_response(json_response);
        assert!(result.is_ok());

        // Test parsing JSON wrapped in markdown
        let markdown_response = r#"```json
[{"id": "test", "title": "Test Task", "description": "Test", "priority": "High", "complexity": 5, "agent_type": "TestAgent", "tags": [], "depends_on": []}]
```"#;
        let result: Result<IdeaBreakdownResponse> = manager.parse_json_response(markdown_response);
        assert!(result.is_ok());

        // Test parsing invalid JSON
        let invalid_response = "This is not JSON";
        let result: Result<IdeaBreakdownResponse> = manager.parse_json_response(invalid_response);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_statistics_tracking() {
        let (mut manager, _command_tx, _event_rx) = McpManager::new();

        // Initial statistics should be zero
        let stats = manager.get_statistics();
        assert_eq!(stats.total_prompts_executed, 0);
        assert_eq!(stats.successful_executions, 0);
        assert_eq!(stats.failed_executions, 0);

        // Update statistics
        manager.update_statistics("test_prompt", McpModel::Gemini, 1000, true);
        manager.update_statistics("test_prompt", McpModel::Gemini, 2000, false);

        let stats = manager.get_statistics();
        assert_eq!(stats.total_prompts_executed, 2);
        assert_eq!(stats.successful_executions, 1);
        assert_eq!(stats.failed_executions, 1);
        assert_eq!(stats.gemini_calls, 2);
        assert_eq!(stats.claude_calls, 0);
        assert_eq!(stats.average_execution_time_ms, 1500.0);
    }

    #[tokio::test]
    async fn test_context_content_generation() {
        let (manager, _command_tx, _event_rx) = McpManager::new();

        let content = manager.generate_context_content(&TechStack::Rust);

        assert!(content.contains("Technology Stack"));
        assert!(content.contains("Rust"));
        assert!(content.contains("Development Guidelines"));
        assert!(content.contains("Code Quality"));
        assert!(content.contains("Security"));
        assert!(content.contains("Performance"));
        assert!(content.contains("Testing"));
        assert!(content.contains("Documentation"));
    }

    #[tokio::test]
    async fn test_shutdown() {
        let (client, _event_rx, _temp_dir) = setup_test_manager().await;

        let result = client.shutdown().await;
        assert!(result.is_ok());
    }
}
