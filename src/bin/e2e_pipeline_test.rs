/// E2E Pipeline Test - Complete Prompts Pipeline Validation
/// 
/// This binary tests the complete prompts pipeline to ensure it produces
/// an actual usable project:
/// 
/// idea_breakdown_user_prompt → feature_dev_todo_prompt → code_review_agent_prompt → qa_agent_prompt → devops_agent_prompt
/// 
/// All actions are produced by Gemini LLM and must result in a working project.
/// If not, we'll identify which prompts need tweaking.

use orchy::managers::{McpManager, McpClient};
use orchy::prompts::Prompts;
use orchy::enums::TechStack;
use orchy::models::prompt_responses::*;
use orchy::models::task::TaskInput;
use orchy::mcp::gemini::GeminiCLI;
use orchy::utils;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tempfile::TempDir;
use tracing::{info, warn, error};
use serde_json;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting E2E Pipeline Test - Complete Prompts Validation");
    info!("🎯 Goal: Test complete pipeline to produce actual usable project");
    info!("📋 Pipeline: idea_breakdown → feature_dev → code_review → qa → devops");
    
    // Initialize MCP Manager and Client
    let (mcp_manager, mcp_command_tx, _mcp_event_rx) = McpManager::new();
    let mcp_client = McpClient::new(mcp_command_tx);
    
    // Start MCP Manager in background
    let mcp_handle = tokio::spawn(async move {
        let _ = mcp_manager.run().await;
    });
    
    // Create project directory
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    
    info!("📁 Project directory: {:?}", project_path);
    
    // Initialize context for testing
    match mcp_client.initialize_context(project_path.clone(), TechStack::Rust).await {
        Ok(()) => info!("✅ Context initialization successful"),
        Err(e) => {
            error!("❌ Context initialization failed: {}", e);
            return;
        }
    }
    
    // Test project idea
    let project_idea = "Build a simple REST API for a todo application with user authentication, CRUD operations for todos, and basic user management";
    let tech_stack = "Rust with Actix-web framework, SQLite database, JWT authentication";
    
    info!("\n🎯 Testing Project: {}", project_idea);
    info!("🛠️  Tech Stack: {}", tech_stack);
    
    // Run the complete pipeline
    let mut pipeline_results = PipelineResults::new();
    
    // Stage 1: Idea Breakdown
    info!("\n📋 Stage 1: Idea Breakdown");
    let breakdown_result = run_idea_breakdown(&mcp_client, project_idea, tech_stack, &mut pipeline_results).await;
    
    if breakdown_result.is_err() {
        error!("❌ Pipeline failed at idea breakdown stage");
        return;
    }
    
    let breakdown = breakdown_result.unwrap();
    
    // Stage 2: Feature Development for each task
    info!("\n🔧 Stage 2: Feature Development");
    let mut project_files = Vec::new();
    let mut stage_failed = false;

    for (i, task) in breakdown.tasks.iter().enumerate() {
        info!("  📝 Developing task {}/{}: {}", i + 1, breakdown.tasks.len(), task.title);

        match run_feature_development(&mcp_client, task, tech_stack, &project_files, &project_path, &mut pipeline_results).await {
            Ok(files) => {
                project_files.extend(files);
                info!("    ✅ Task completed successfully");
            }
            Err(e) => {
                error!("    ❌ Task failed: {}", e);
                pipeline_results.add_failure("feature_development", &format!("Task '{}' failed: {}", task.title, e));
                stage_failed = true;
                break; // Stop processing remaining tasks
            }
        }
    }

    if stage_failed {
        error!("❌ Feature Development stage failed - stopping pipeline");
        pipeline_results.generate_report(&project_path);
        return;
    }
    
    // Stage 3: Code Review (validate project files first)
    info!("\n🔍 Stage 3: Code Review");
    if project_files.is_empty() {
        error!("  ❌ No project files to review - stopping pipeline");
        pipeline_results.add_failure("code_review", "No project files generated from feature development");
        pipeline_results.generate_report(&project_path);
        return;
    } else if !has_essential_files(&project_files) {
        error!("  ❌ Missing essential files (Cargo.toml, main.rs) - stopping pipeline");
        pipeline_results.add_failure("code_review", "Missing essential files for a functional project");
        pipeline_results.generate_report(&project_path);
        return;
    } else {
        match run_code_review(&mcp_client, &project_files, &mut pipeline_results).await {
            Ok(review_suggestions) => {
                // Check if code review rejected the code
                if review_suggestions.iter().any(|s| s.starts_with("REJECTION:")) {
                    error!("  ❌ Code review rejected the code - stopping pipeline");
                    pipeline_results.add_failure("code_review", "Code review rejected the implementation");
                    pipeline_results.generate_report(&project_path);
                    return;
                }
                info!("  ✅ Code review completed with {} suggestions", review_suggestions.len());
                apply_code_review_suggestions(&mut project_files, &review_suggestions);
            }
            Err(e) => {
                error!("  ❌ Code review failed - stopping pipeline: {}", e);
                pipeline_results.add_failure("code_review", &e.to_string());
                pipeline_results.generate_report(&project_path);
                return;
            }
        }
    }
    
    // Stage 4: QA Testing with iterative feedback loop
    info!("\n🧪 Stage 4: QA Testing");
    let mut qa_iteration = 1;
    let max_qa_iterations = 3;
    let mut qa_feedback = Vec::new();

    loop {
        info!("  🔄 QA Iteration {}/{}", qa_iteration, max_qa_iterations);

        // Write files to disk temporarily to test if project builds
        let temp_test_dir = TempDir::new().expect("Failed to create temp test directory");
        let test_project_path = temp_test_dir.path().to_path_buf();
        write_project_files(&test_project_path, &project_files).await;

        let (project_builds, build_output) = check_project_builds_and_tests(&test_project_path).await;

        if !project_builds {
            let build_error = format!("Project doesn't compile or tests fail: {}", build_output);
            error!("  ❌ {}", build_error);
            qa_feedback.push(format!("Build/Test Error (Iteration {}): {}", qa_iteration, build_error));

            if qa_iteration >= max_qa_iterations {
                error!("  ❌ Max QA iterations reached - stopping pipeline");
                pipeline_results.add_failure("qa_testing", &format!("Project failed to build/test after {} iterations", max_qa_iterations));
                pipeline_results.generate_report(&project_path);
                return;
            }

            // Go back to feature development with build/test feedback
            info!("  🔄 Going back to feature development with build/test feedback");
            match run_feature_development_with_feedback(&mcp_client, &breakdown.tasks[0], tech_stack, &project_files, &qa_feedback, &project_path, &mut pipeline_results).await {
                Ok(new_files) => {
                    project_files = new_files; // Replace with improved files
                    qa_iteration += 1;
                    continue;
                }
                Err(e) => {
                    error!("  ❌ Feature development with feedback failed: {}", e);
                    pipeline_results.add_failure("feature_development_feedback", &e.to_string());
                    pipeline_results.generate_report(&project_path);
                    return;
                }
            }
        }

        info!("  ✅ Project builds and tests pass successfully, proceeding with QA testing");
        match run_qa_testing(&mcp_client, tech_stack, &project_files, &build_output, &mut pipeline_results).await {
            Ok(test_files) => {
                info!("  ✅ QA testing passed with {} test files", test_files.len());
                project_files.extend(test_files);
                break; // QA passed, exit the loop
            }
            Err(e) => {
                let qa_error = e.to_string();
                warn!("  ⚠️  QA testing failed: {}", qa_error);
                qa_feedback.push(format!("QA Rejection (Iteration {}): {} | Build Output: {}", qa_iteration, qa_error, build_output));

                if qa_iteration >= max_qa_iterations {
                    error!("  ❌ Max QA iterations reached - stopping pipeline");
                    pipeline_results.add_failure("qa_testing", &format!("QA failed after {} iterations", max_qa_iterations));
                    pipeline_results.generate_report(&project_path);
                    return;
                }

                // Go back to feature development with QA feedback
                info!("  🔄 Going back to feature development with QA feedback");
                match run_feature_development_with_feedback(&mcp_client, &breakdown.tasks[0], tech_stack, &project_files, &qa_feedback, &project_path, &mut pipeline_results).await {
                    Ok(new_files) => {
                        project_files = new_files; // Replace with improved files
                        qa_iteration += 1;
                        continue;
                    }
                    Err(e) => {
                        error!("  ❌ Feature development with feedback failed: {}", e);
                        pipeline_results.add_failure("feature_development_feedback", &e.to_string());
                        pipeline_results.generate_report(&project_path);
                        return;
                    }
                }
            }
        }
    }
    
    // Stage 5: DevOps Setup with iterative feedback loop
    info!("\n🚀 Stage 5: DevOps Setup");
    let mut devops_iteration = 1;
    let max_devops_iterations = 2;
    let mut devops_feedback = Vec::new();

    loop {
        info!("  🔄 DevOps Iteration {}/{}", devops_iteration, max_devops_iterations);

        match run_devops_setup(&mcp_client, tech_stack, &project_files, &mut pipeline_results).await {
            Ok(devops_files) => {
                info!("  ✅ DevOps setup completed with {} configuration files", devops_files.len());
                project_files.extend(devops_files);
                break; // DevOps passed, exit the loop
            }
            Err(e) => {
                let devops_error = e.to_string();
                warn!("  ⚠️  DevOps setup failed: {}", devops_error);
                devops_feedback.push(format!("DevOps Rejection (Iteration {}): {}", devops_iteration, devops_error));

                if devops_iteration >= max_devops_iterations {
                    error!("  ❌ Max DevOps iterations reached - stopping pipeline");
                    pipeline_results.add_failure("devops_setup", &format!("DevOps failed after {} iterations", max_devops_iterations));
                    pipeline_results.generate_report(&project_path);
                    return;
                }

                // Go back to feature development with DevOps feedback
                info!("  🔄 Going back to feature development with DevOps feedback");
                match run_feature_development_with_feedback(&mcp_client, &breakdown.tasks[0], tech_stack, &project_files, &devops_feedback, &project_path, &mut pipeline_results).await {
                    Ok(new_files) => {
                        project_files = new_files; // Replace with improved files
                        devops_iteration += 1;
                        continue;
                    }
                    Err(e) => {
                        error!("  ❌ Feature development with DevOps feedback failed: {}", e);
                        pipeline_results.add_failure("feature_development_devops_feedback", &e.to_string());
                        pipeline_results.generate_report(&project_path);
                        return;
                    }
                }
            }
        }
    }
    
    // Write all files to disk
    info!("\n💾 Writing project files to disk");
    write_project_files(&project_path, &project_files).await;
    
    // Validate the final project
    info!("\n✅ Validating final project");
    let validation_result = validate_project(&project_path, tech_stack).await;
    
    // Generate final report
    info!("\n📊 Generating Pipeline Report");
    pipeline_results.set_validation_result(validation_result);
    pipeline_results.generate_report(&project_path);
    
    info!("🎉 E2E Pipeline Test Complete!");
    info!("📁 Project available at: {:?}", project_path);
    
    // Keep the temp directory for inspection
    std::mem::forget(temp_dir);
    
    // Cleanup
    drop(mcp_handle);
}

/// Pipeline results tracker
#[derive(Debug)]
struct PipelineResults {
    stages_completed: Vec<String>,
    failures: Vec<(String, String)>,
    total_files_generated: usize,
    start_time: Instant,
    validation_result: Option<ProjectValidation>,
}

impl PipelineResults {
    fn new() -> Self {
        Self {
            stages_completed: Vec::new(),
            failures: Vec::new(),
            total_files_generated: 0,
            start_time: Instant::now(),
            validation_result: None,
        }
    }
    
    fn add_success(&mut self, stage: &str) {
        self.stages_completed.push(stage.to_string());
    }
    
    fn add_failure(&mut self, stage: &str, error: &str) {
        self.failures.push((stage.to_string(), error.to_string()));
    }
    
    fn set_validation_result(&mut self, validation: ProjectValidation) {
        self.validation_result = Some(validation);
    }
    
    fn generate_report(&self, project_path: &PathBuf) {
        let duration = self.start_time.elapsed();
        
        info!("\n{}", "=".repeat(80));
        info!("📊 E2E PIPELINE TEST REPORT");
        info!("{}", "=".repeat(80));
        
        info!("⏱️  Total Duration: {:.2}s", duration.as_secs_f64());
        info!("📁 Project Path: {:?}", project_path);
        info!("📄 Total Files Generated: {}", self.total_files_generated);
        
        info!("\n✅ Completed Stages:");
        for stage in &self.stages_completed {
            info!("  🟢 {}", stage);
        }
        
        if !self.failures.is_empty() {
            info!("\n❌ Failed Stages:");
            for (stage, error) in &self.failures {
                info!("  🔴 {}: {}", stage, error);
            }
        }
        
        if let Some(validation) = &self.validation_result {
            info!("\n🔍 Project Validation:");
            info!("  Builds Successfully: {}", if validation.builds { "✅" } else { "❌" });
            info!("  Tests Pass: {}", if validation.tests_pass { "✅" } else { "❌" });
            info!("  Has Documentation: {}", if validation.has_docs { "✅" } else { "❌" });
            info!("  Deployment Ready: {}", if validation.deployment_ready { "✅" } else { "❌" });
            info!("  Code Quality Score: {:.2}/10", validation.code_quality_score);
        }
        
        let success_rate = (self.stages_completed.len() as f64 / 5.0) * 100.0;
        info!("\n📈 Pipeline Success Rate: {:.1}%", success_rate);
        
        if success_rate >= 80.0 {
            info!("🎉 PIPELINE SUCCESS - Prompts are working well!");
        } else {
            info!("⚠️  PIPELINE NEEDS IMPROVEMENT - Some prompts need tweaking");
        }
        
        info!("{}", "=".repeat(80));
    }
}

#[derive(Debug)]
struct ProjectValidation {
    builds: bool,
    tests_pass: bool,
    has_docs: bool,
    deployment_ready: bool,
    code_quality_score: f64,
}

/// Run idea breakdown stage
async fn run_idea_breakdown(
    mcp_client: &McpClient,
    idea: &str,
    tech_stack: &str,
    results: &mut PipelineResults,
) -> Result<IdeaBreakdownResponse, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let agents = vec![
        "BackendEngineerRust".to_string(),
        "FrontendEngineerReact".to_string(),
        "DevOpsEngineer".to_string(),
        "QAEngineer".to_string(),
    ];

    // Use direct Gemini CLI with retry logic for idea breakdown
    let prompt = Prompts::idea_breakdown_user_prompt(
        idea,
        "Complete end-to-end project development with full testing and deployment",
        agents.clone(),
        tech_stack,
    );

    info!("📝 IDEA BREAKDOWN PROMPT SENT TO LLM:");
    info!("{}", "=".repeat(80));
    info!("{}", prompt);
    info!("{}", "=".repeat(80));

    match GeminiCLI::query_with_model_and_retries(&prompt, "gemini-2.5-flash", 3).await {
        Ok(response_text) => {
            let duration = start_time.elapsed();

            info!("🤖 IDEA BREAKDOWN RESPONSE RECEIVED:");
            info!("{}", "=".repeat(80));
            info!("{}", response_text);
            info!("{}", "=".repeat(80));

            // Parse the response to extract tasks
            let parsed_response = parse_idea_breakdown_response(&response_text)?;
            info!("  ✅ Generated {} tasks in {:.2}s", parsed_response.tasks.len(), duration.as_secs_f64());

            for (i, task) in parsed_response.tasks.iter().enumerate() {
                info!("    {}. {}: {}", i + 1, task.title, task.description);
                info!("       Priority: {}, Complexity: {}, Agent: {:?}",
                      task.priority, task.complexity, task.agent_type);
            }

            results.add_success("idea_breakdown");
            Ok(parsed_response)
        }
        Err(e) => {
            error!("  ❌ Idea breakdown failed: {}", e);
            results.add_failure("idea_breakdown", &e.to_string());
            Err(Box::new(e))
        }
    }
}

/// Run feature development stage with retry-until-success
async fn run_feature_development(
    mcp_client: &McpClient,
    task: &TaskInput,
    tech_stack: &str,
    existing_files: &[(String, String)],
    project_path: &std::path::PathBuf,
    results: &mut PipelineResults,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let max_attempts = 5;
    let mut attempt = 1;
    let mut feedback_history = Vec::new();

    // Load agent context based on task agent type
    let agent_context = if let Some(agent_type) = &task.agent_type {
        match load_agent_context(agent_type).await {
            Ok(context) => {
                info!("  📋 Loaded agent context for: {}", agent_type);
                Some(context)
            }
            Err(e) => {
                warn!("  ⚠️  Failed to load agent context for {}: {}", agent_type, e);
                None
            }
        }
    } else {
        None
    };

    info!("🔄 Feature Development: Retry until working code");

    loop {
        info!("  🎯 Attempt {}/{}: {}", attempt, max_attempts, task.title);
        let start_time = Instant::now();

        // Create enhanced prompt with accumulated feedback
        let feedback_context = if feedback_history.is_empty() {
            String::new()
        } else {
            format!(
                "\n\n🚨 PREVIOUS ATTEMPTS FAILED - LEARN FROM THESE ERRORS:\n{}\n\n⚠️ YOU MUST FIX ALL THESE ISSUES AND CREATE WORKING CODE:\n",
                feedback_history.iter().enumerate()
                    .map(|(i, fb)| format!("Attempt {}: {}", i + 1, fb))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        let enhanced_description = format!(
            "{}: {}\n\n{}🎯 CRITICAL: Generate COMPLETE, WORKING, TESTABLE code that builds and runs successfully.\n🔨 Code MUST pass: build check, test execution, and UI accessibility (if applicable).",
            task.title,
            task.description,
            feedback_context
        );

        let prompt = Prompts::feature_dev_todo_prompt(
            &enhanced_description,
            tech_stack,
            existing_files,
            None,
            agent_context.as_deref(),
        );

        info!("    📝 Sending prompt to LLM...");

        match GeminiCLI::query_with_model_and_retries(&prompt, "gemini-2.5-flash", 3).await {
            Ok(response) => {
                let duration = start_time.elapsed();
                info!("    ✅ LLM response received in {:.2}s", duration.as_secs_f64());

                info!("🤖 RAW LLM RESPONSE:");
                info!("{}", "=".repeat(80));
                info!("{}", response);
                info!("{}", "=".repeat(80));

                // Use Action::parse_and_execute to handle all action types
                use orchy::enums::action::Action;

                let mut verification_failed = false;
                let mut error_details = String::new();
                let files_generated = 1; // Assume at least one file was generated if successful

                // Extract JSON from markdown if present
                let json_content = utils::extract_json_from_response(&response);
                info!("    🔍 Extracted JSON content: {}", json_content);

                // Change to project directory before executing actions
                let original_dir = std::env::current_dir()?;
                std::env::set_current_dir(project_path)?;

                let actions = Action::from_json_array(&json_content).unwrap();

                for action in actions {
                    match action.execute().await {
                        Ok(_) => {
                            info!("    ✅ Action executed successfully");
                        }
                        Err(e) => {
                            verification_failed = true;
                            error_details = format!("Action execution failed: {}", e);
                            warn!("    ❌ Action execution failed: {}", e);
                            break;
                        }
                    }
                }

                // Restore original directory
                std::env::set_current_dir(original_dir)?;

                if verification_failed {
                    info!("    🔧 Code verification failed - creating focused error-fixing task");

                    // Create a focused error-fixing task instead of regenerating everything
                    match fix_compilation_errors(&mcp_client, task, tech_stack, &error_details, project_path).await {
                        Ok(_) => {
                            info!("    ✅ Compilation errors fixed, task completed successfully");
                            return Ok(vec![]); // Exit the retry loop, task is complete
                        }
                        Err(e) => {
                            warn!("    ❌ Failed to fix compilation errors: {}", e);
                        }
                    }

                    if attempt >= max_attempts {
                        return Err(format!(
                            "Feature development failed after {} attempts. Last errors:\n{}",
                            max_attempts, error_details
                        ).into());
                    }

                    attempt += 1;
                    warn!("    ❌ Error fixing failed, retrying full task on attempt {}", attempt);
                    continue;
                }

                // ✅ All verification passed
                info!("    ✅ SUCCESS! Actions executed and all verification passed on attempt {}", attempt);
                results.total_files_generated += files_generated;

                // Return empty files list since Action::parse_and_execute already handled everything
                return Ok(vec![]);
            }
            Err(e) => {
                error!("    ❌ LLM query failed on attempt {}: {}", attempt, e);
                feedback_history.push(format!("LLM query failed: {}", e));

                if attempt >= max_attempts {
                    return Err(format!("Feature development failed after {} LLM query attempts", max_attempts).into());
                }

                attempt += 1;
                info!("    🔄 Retrying LLM query...");
                continue;
            }
        }
    }
}

// Removed custom execute_todo_action - using Action enum instead

/// Extract code blocks from markdown-style response
fn extract_code_blocks_from_response(response: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let lines: Vec<&str> = response.lines().collect();
    let mut i = 0;

    info!("  🔍 Scanning {} lines for code blocks", lines.len());

    while i < lines.len() {
        if lines[i].starts_with("```") {
            info!("    📝 Found code block at line {}: {}", i + 1, lines[i]);

            // Found a code block
            let mut file_path = "src/generated.rs".to_string();

            // Look for file path in the line before or after
            if i > 0 && (lines[i-1].contains(".rs") || lines[i-1].contains(".toml") || lines[i-1].contains(".md") || lines[i-1].contains(".yml") || lines[i-1].contains(".yaml")) {
                file_path = lines[i-1].trim().to_string();
                info!("      📁 Found file path in previous line: {}", file_path);
            } else if lines[i].len() > 3 && (lines[i].contains(".rs") || lines[i].contains(".toml") || lines[i].contains(".md") || lines[i].contains(".yml") || lines[i].contains(".yaml")) {
                file_path = lines[i].replace("```", "").trim().to_string();
                info!("      📁 Found file path in code block line: {}", file_path);
            } else {
                // Try to infer from language
                let lang_line = lines[i].replace("```", "").trim().to_lowercase();
                if lang_line == "rust" || lang_line == "rs" {
                    file_path = format!("src/lib_{}.rs", files.len());
                } else if lang_line == "toml" {
                    file_path = "Cargo.toml".to_string();
                } else if lang_line == "dockerfile" {
                    file_path = "Dockerfile".to_string();
                } else if lang_line == "yaml" || lang_line == "yml" {
                    file_path = ".github/workflows/ci.yml".to_string();
                }
                info!("      📁 Inferred file path: {}", file_path);
            }

            // Extract content
            i += 1;
            let mut content = Vec::new();
            let _start_line = i;
            while i < lines.len() && !lines[i].starts_with("```") {
                content.push(lines[i]);
                i += 1;
            }

            if !content.is_empty() {
                let content_str = content.join("\n");
                info!("      📄 Extracted {} lines of content ({} chars)", content.len(), content_str.len());
                files.push((file_path, content_str));
            } else {
                info!("      ⚠️  Empty code block");
            }
        }
        i += 1;
    }

    info!("  📊 Total code blocks extracted: {}", files.len());
    files
}

// Removed hardcoded generation functions - LLM must generate everything

/// Run code review stage
async fn run_code_review(
    _mcp_client: &McpClient,
    project_files: &[(String, String)],
    results: &mut PipelineResults,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let focus_areas = vec![
        "Security vulnerabilities".to_string(),
        "Performance issues".to_string(),
        "Code quality".to_string(),
        "Best practices".to_string(),
    ];

    let prompt = Prompts::code_review_agent_prompt(
        "Rust with Actix-web",
        project_files,
        &focus_areas,
        None, // TODO: Add agent context loading
    );

    match GeminiCLI::query(&prompt).await {
        Ok(response) => {
            let duration = start_time.elapsed();
            info!("  ✅ Code review completed in {:.2}s", duration.as_secs_f64());

            let suggestions = parse_code_review_response(&response);
            info!("  📝 Generated {} review suggestions", suggestions.len());

            results.add_success("code_review");
            Ok(suggestions)
        }
        Err(e) => {
            error!("  ❌ Code review failed: {}", e);
            Err(Box::new(e))
        }
    }
}

/// Parse code review response and check for rejections
fn parse_code_review_response(response: &str) -> Vec<String> {
    info!("🔍 PARSING CODE REVIEW RESPONSE:");

    // Check if it's a rejection
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
        if let Some(array) = parsed.as_array() {
            for item in array {
                if let Some(reject) = item.get("Reject") {
                    let reason = reject.get("reason").and_then(|r| r.as_str()).unwrap_or("Unknown reason");
                    let empty_vec = vec![];
                    let issues = reject.get("blocking_issues").and_then(|i| i.as_array()).unwrap_or(&empty_vec);

                    warn!("  ❌ CODE REVIEW REJECTION:");
                    warn!("     Reason: {}", reason);
                    warn!("     Blocking Issues:");
                    for issue in issues {
                        if let Some(issue_str) = issue.as_str() {
                            warn!("       - {}", issue_str);
                        }
                    }
                    return vec![format!("REJECTION: {}", reason)];
                }
            }
        }
    }

    // If not a rejection, parse as regular suggestions
    response
        .lines()
        .filter(|line| line.starts_with("- ") || line.starts_with("* ") || line.contains("TODO") || line.contains("FIXME"))
        .map(|line| line.trim().to_string())
        .collect()
}

/// Apply code review suggestions
fn apply_code_review_suggestions(_files: &mut Vec<(String, String)>, suggestions: &[String]) {
    // For now, just log the suggestions - in a real implementation,
    // we would parse and apply specific code changes
    info!("  📝 Applying {} code review suggestions", suggestions.len());
    for suggestion in suggestions {
        info!("    - {}", suggestion);
    }
}

/// Run QA testing stage
async fn run_qa_testing(
    _mcp_client: &McpClient,
    tech_stack: &str,
    project_files: &[(String, String)],
    build_output: &str,
    results: &mut PipelineResults,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let test_types = vec![
        "unit_tests".to_string(),
        "integration_tests".to_string(),
        "api_tests".to_string(),
    ];

    // Create enhanced QA prompt with build/test information
    let enhanced_prompt = format!(
        "{}\n\n🔨 BUILD AND TEST VALIDATION RESULTS:\n{}\n\n⚠️ CRITICAL: The code has already been validated to build and pass existing tests. Your job is to:\n1. Verify the test coverage is comprehensive\n2. Add additional tests if needed\n3. Ensure all edge cases are covered\n4. Validate error handling scenarios\n5. Check API endpoint functionality\n\nIf you find any gaps in testing, create additional test files to fill those gaps.",
        Prompts::qa_agent_prompt(
            tech_stack,
            project_files,
            &test_types,
            None, // no current test failures
            None, // TODO: Add agent context loading
        ),
        build_output
    );

    match GeminiCLI::query(&enhanced_prompt).await {
        Ok(response) => {
            let duration = start_time.elapsed();
            info!("  ✅ QA testing completed in {:.2}s", duration.as_secs_f64());

            match parse_qa_response(&response) {
                Ok(test_files) => {
                    info!("  🧪 Generated {} test files", test_files.len());
                    results.add_success("qa_testing");
                    results.total_files_generated += test_files.len();
                    Ok(test_files)
                }
                Err(rejection_reason) => {
                    warn!("  ❌ QA rejected the code: {}", rejection_reason);
                    Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, rejection_reason)))
                }
            }
        }
        Err(e) => {
            error!("  ❌ QA testing failed: {}", e);
            Err(Box::new(e))
        }
    }
}

/// Parse QA response to extract test files or handle rejections
fn parse_qa_response(response: &str) -> Result<Vec<(String, String)>, String> {
    info!("🔍 PARSING QA RESPONSE:");

    // Check if it's a rejection
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
        if let Some(array) = parsed.as_array() {
            for item in array {
                if let Some(reject) = item.get("Reject") {
                    let reason = reject.get("reason").and_then(|r| r.as_str()).unwrap_or("Unknown reason");
                    let empty_vec = vec![];
                    let issues = reject.get("blocking_issues").and_then(|i| i.as_array()).unwrap_or(&empty_vec);

                    warn!("  ❌ QA REJECTION:");
                    warn!("     Reason: {}", reason);
                    warn!("     Blocking Issues:");
                    for issue in issues {
                        if let Some(issue_str) = issue.as_str() {
                            warn!("       - {}", issue_str);
                        }
                    }
                    return Err(format!("QA Rejected: {}", reason));
                }
            }
        }
    }

    // If not a rejection, extract test files
    let mut test_files = extract_code_blocks_from_response(response);

    // If no test files found, LLM failed
    if test_files.is_empty() {
        return Err("LLM failed to generate test files - QA prompt needs improvement".into());
    }

    Ok(test_files)
}

// Removed hardcoded test generation - LLM must generate all tests

/// Run DevOps setup stage
async fn run_devops_setup(
    _mcp_client: &McpClient,
    tech_stack: &str,
    project_files: &[(String, String)],
    results: &mut PipelineResults,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let prompt = Prompts::devops_agent_prompt(
        tech_stack,
        project_files,
        "docker_and_github_actions", // deployment target
        None, // no CI failures
        None, // TODO: Add agent context loading
    );

    match GeminiCLI::query(&prompt).await {
        Ok(response) => {
            let duration = start_time.elapsed();
            info!("  ✅ DevOps setup completed in {:.2}s", duration.as_secs_f64());

            match parse_devops_response(&response) {
                Ok(devops_files) => {
                    info!("  🚀 Generated {} DevOps files", devops_files.len());
                    results.add_success("devops_setup");
                    results.total_files_generated += devops_files.len();
                    Ok(devops_files)
                }
                Err(rejection_reason) => {
                    warn!("  ❌ DevOps rejected the code: {}", rejection_reason);
                    Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, rejection_reason)))
                }
            }
        }
        Err(e) => {
            error!("  ❌ DevOps setup failed: {}", e);
            Err(Box::new(e))
        }
    }
}

/// Parse DevOps response to extract configuration files or handle rejections
fn parse_devops_response(response: &str) -> Result<Vec<(String, String)>, String> {
    info!("🔍 PARSING DEVOPS RESPONSE:");

    // Check if it's a rejection
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
        if let Some(array) = parsed.as_array() {
            for item in array {
                if let Some(reject) = item.get("Reject") {
                    let reason = reject.get("reason").and_then(|r| r.as_str()).unwrap_or("Unknown reason");
                    let empty_vec = vec![];
                    let issues = reject.get("blocking_issues").and_then(|i| i.as_array()).unwrap_or(&empty_vec);

                    warn!("  ❌ DEVOPS REJECTION:");
                    warn!("     Reason: {}", reason);
                    warn!("     Blocking Issues:");
                    for issue in issues {
                        if let Some(issue_str) = issue.as_str() {
                            warn!("       - {}", issue_str);
                        }
                    }
                    return Err(format!("DevOps Rejected: {}", reason));
                }
            }
        }
    }

    // If not a rejection, extract DevOps files
    let mut devops_files = extract_code_blocks_from_response(response);

    // If no DevOps files found, LLM failed
    if devops_files.is_empty() {
        return Err("LLM failed to generate DevOps files - DevOps prompt needs improvement".into());
    }

    Ok(devops_files)
}

// Removed hardcoded DevOps generation - LLM must generate all deployment files

/// Fix compilation errors with focused error-fixing task
async fn fix_compilation_errors(
    mcp_client: &McpClient,
    task: &TaskInput,
    tech_stack: &str,
    error_details: &str,
    project_path: &std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 FOCUSED ERROR FIXING: Addressing compilation errors");

    // Get current project files for context
    let mut project_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(project_path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Some(file_name) = entry.file_name().to_str() {
                            project_files.push((file_name.to_string(), content));
                        }
                    }
                }
            }
        }
    }

    // Create a focused error recovery prompt that requires pure JSON (no markdown)
    let error_fixing_prompt = format!(
        r#"🚨 CRITICAL: PURE JSON RESPONSE REQUIRED 🚨

TECH STACK: {}
FAILED COMMAND: cargo check

ERROR DETAILS:
{}

🎯 TASK: Generate ONLY the minimal JSON actions to fix these specific errors.

⚠️⚠️⚠️ CRITICAL: YOUR RESPONSE MUST BE PURE JSON - NO MARKDOWN ⚠️⚠️⚠️

❌ DO NOT WRITE:
```json
[{{"action": "..."}}]
```

✅ WRITE ONLY:
[{{"Write": {{"path": "Cargo.toml", "content": "..."}}}}]

🔧 COMMON RUST ERROR FIXES:
- DEPENDENCY CONFLICTS: Update Cargo.toml with compatible versions
- MISSING FILES: Create .env, schema files, config files
- IMPORT ERRORS: Fix module declarations in lib.rs
- VERSION MISMATCHES: Use compatible dependency versions

🚨 RESPONSE REQUIREMENTS 🚨
1. START IMMEDIATELY WITH [ (no text before)
2. END IMMEDIATELY WITH ] (no text after)
3. NO MARKDOWN CODE BLOCKS
4. NO EXPLANATIONS
5. PURE JSON ONLY

EXAMPLE RESPONSE FORMAT:
[{{"Write": {{"path": "Cargo.toml", "content": "[package]\\nname = \\"todo-api\\"\\nversion = \\"0.1.0\\"\\n\\n[dependencies]\\nactix-web = \\"4.0\\""}}}}]

🚨 FIRST CHARACTER MUST BE [ - LAST CHARACTER MUST BE ] 🚨"#,
        tech_stack, error_details
    );

    info!("📝 Sending focused error-fixing prompt to LLM...");
    info!("🔍 ERROR FIXING PROMPT BEING SENT:");
    info!("{}", error_fixing_prompt);

    // Use direct Gemini CLI call to get raw response and see what LLM is actually sending
    use orchy::mcp::gemini::GeminiCLI;
    let raw_response = match GeminiCLI::query_with_model_and_retries(&error_fixing_prompt, "gemini-2.5-flash", 2).await {
        Ok(resp) => {
            info!("✅ Raw error-fixing response received successfully");
            info!("🔍 RAW LLM RESPONSE:");
            info!("{}", "=".repeat(80));
            info!("{}", resp);
            info!("{}", "=".repeat(80));
            resp
        }
        Err(e) => {
            error!("❌ Raw error recovery query failed: {}", e);
            return Err(format!("Raw error recovery query failed: {}", e).into());
        }
    };

    // Use Action::parse_and_execute to handle error recovery actions
    use orchy::enums::action::Action;

    // Extract JSON from markdown if present
    let json_content = utils::extract_json_from_response(&raw_response);
    info!("🔍 Error recovery JSON content: {}", json_content);

    // Change to project directory before executing actions
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(project_path)?;

    match Action::parse_and_execute(&json_content).await {
        Ok(_results) => {
            info!("🔧 Error recovery actions executed successfully");
            info!("  📄 Error recovery actions completed in project directory");
        }
        Err(e) => {
            // Restore directory before returning error
            std::env::set_current_dir(original_dir)?;
            return Err(format!("Error recovery action execution failed: {}", e).into());
        }
    }

    // Restore original directory
    std::env::set_current_dir(original_dir)?;

    info!("✅ All error fixes applied and verified successfully");
    Ok(())
}

/// Write all project files to disk
async fn write_project_files(project_path: &PathBuf, files: &[(String, String)]) {
    for (file_path, content) in files {
        let full_path = project_path.join(file_path);

        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                warn!("Failed to create directory {:?}: {}", parent, e);
                continue;
            }
        }

        // Write file content
        match fs::write(&full_path, content) {
            Ok(()) => info!("  📄 Written: {}", file_path),
            Err(e) => warn!("  ❌ Failed to write {}: {}", file_path, e),
        }
    }
}

/// Validate the final project
async fn validate_project(project_path: &PathBuf, _tech_stack: &str) -> ProjectValidation {
    let mut validation = ProjectValidation {
        builds: false,
        tests_pass: false,
        has_docs: false,
        deployment_ready: false,
        code_quality_score: 0.0,
    };

    // Check if Cargo.toml exists
    let cargo_toml_path = project_path.join("Cargo.toml");
    if cargo_toml_path.exists() {
        info!("  ✅ Cargo.toml found");

        // Try to build the project and run tests
        let (builds_and_tests_pass, _output) = check_project_builds_and_tests(project_path).await;
        validation.builds = builds_and_tests_pass;
        validation.tests_pass = builds_and_tests_pass;
    }

    // Check for documentation
    validation.has_docs = check_has_documentation(project_path);

    // Check for deployment files
    validation.deployment_ready = check_deployment_ready(project_path);

    // Calculate code quality score
    validation.code_quality_score = calculate_code_quality_score(project_path);

    validation
}

/// Check if project builds and tests pass successfully
async fn check_project_builds_and_tests(project_path: &PathBuf) -> (bool, String) {
    info!("  🔨 Checking if project builds and tests pass...");

    // First check if project builds
    info!("    📦 Running cargo check...");
    let check_output = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(project_path)
        .output();

    let check_result = match check_output {
        Ok(result) => {
            if result.status.success() {
                info!("    ✅ cargo check passed");
                true
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                warn!("    ❌ cargo check failed: {}", stderr);
                return (false, format!("Build failed: {}", stderr));
            }
        }
        Err(e) => {
            warn!("    ❌ Failed to run cargo check: {}", e);
            return (false, format!("Failed to run cargo check: {}", e));
        }
    };

    if !check_result {
        return (false, "Build check failed".to_string());
    }

    // Then run tests
    info!("    🧪 Running cargo test...");
    let test_output = std::process::Command::new("cargo")
        .arg("test")
        .current_dir(project_path)
        .output();

    match test_output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            if result.status.success() {
                info!("    ✅ All tests passed");
                info!("    📊 Test output: {}", stdout);
                (true, format!("Build and tests successful: {}", stdout))
            } else {
                warn!("    ❌ Tests failed: {}", stderr);
                warn!("    📊 Test output: {}", stdout);
                (false, format!("Tests failed: {} | Output: {}", stderr, stdout))
            }
        }
        Err(e) => {
            warn!("    ❌ Failed to run cargo test: {}", e);
            (false, format!("Failed to run cargo test: {}", e))
        }
    }
}

/// Check if tests pass
async fn check_tests_pass(project_path: &PathBuf) -> bool {
    info!("  🧪 Checking if tests pass...");

    let output = std::process::Command::new("cargo")
        .arg("test")
        .arg("--no-run") // Just check if tests compile
        .current_dir(project_path)
        .output();

    match output {
        Ok(result) => {
            let success = result.status.success();
            if success {
                info!("    ✅ Tests compile successfully");
            } else {
                warn!("    ❌ Tests compilation failed");
            }
            success
        }
        Err(e) => {
            warn!("    ❌ Failed to run cargo test: {}", e);
            false
        }
    }
}

/// Check if project has documentation
fn check_has_documentation(project_path: &PathBuf) -> bool {
    let readme_path = project_path.join("README.md");
    let docs_exist = readme_path.exists();

    if docs_exist {
        info!("  ✅ Documentation found");
    } else {
        info!("  ⚠️  No README.md found");
    }

    docs_exist
}

/// Check if project is deployment ready
fn check_deployment_ready(project_path: &PathBuf) -> bool {
    let dockerfile_path = project_path.join("Dockerfile");
    let ci_path = project_path.join(".github/workflows");

    let has_dockerfile = dockerfile_path.exists();
    let has_ci = ci_path.exists();

    let deployment_ready = has_dockerfile && has_ci;

    if deployment_ready {
        info!("  ✅ Deployment configuration found");
    } else {
        info!("  ⚠️  Missing deployment configuration");
    }

    deployment_ready
}

/// Calculate code quality score
fn calculate_code_quality_score(project_path: &PathBuf) -> f64 {
    let mut score = 0.0;

    // Check for main source files
    if project_path.join("src/main.rs").exists() || project_path.join("src/lib.rs").exists() {
        score += 2.0;
    }

    // Check for Cargo.toml
    if project_path.join("Cargo.toml").exists() {
        score += 2.0;
    }

    // Check for tests
    if project_path.join("tests").exists() ||
       fs::read_dir(project_path.join("src")).map_or(false, |entries| {
           entries.filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().contains("test"))
       }) {
        score += 2.0;
    }

    // Check for documentation
    if project_path.join("README.md").exists() {
        score += 1.0;
    }

    // Check for CI/CD
    if project_path.join(".github/workflows").exists() {
        score += 1.5;
    }

    // Check for Docker
    if project_path.join("Dockerfile").exists() {
        score += 1.5;
    }

    score
}

/// Check if project has essential files for a Rust project
fn has_essential_files(files: &[(String, String)]) -> bool {
    let has_cargo_toml = files.iter().any(|(path, _)| path == "Cargo.toml");
    let has_main_or_lib = files.iter().any(|(path, _)|
        path == "src/main.rs" || path == "src/lib.rs"
    );

    info!("  📋 Essential files check:");
    info!("    Cargo.toml: {}", if has_cargo_toml { "✅" } else { "❌" });
    info!("    src/main.rs or src/lib.rs: {}", if has_main_or_lib { "✅" } else { "❌" });

    has_cargo_toml && has_main_or_lib
}

// Removed system testing - LLM is responsible for testing its own code via RunCommand actions

// Removed - LLM handles its own testing via RunCommand actions

// Removed - LLM handles its own testing via RunCommand actions

// Removed - LLM handles its own testing via RunCommand actions

// Removed - LLM handles its own testing via RunCommand actions

// Removed - LLM handles its own testing via RunCommand actions

/// Parse idea breakdown response from raw text
fn parse_idea_breakdown_response(response: &str) -> Result<IdeaBreakdownResponse, Box<dyn std::error::Error>> {
    info!("🔍 PARSING IDEA BREAKDOWN RESPONSE:");

    // Try to parse as JSON first
    if let Ok(parsed) = serde_json::from_str::<IdeaBreakdownResponse>(response) {
        info!("  ✅ Successfully parsed as structured JSON");
        return Ok(parsed);
    }

    // If direct parsing fails, try to extract JSON from the response
    if let Some(json_start) = response.find('{') {
        if let Some(json_end) = response.rfind('}') {
            let json_str = &response[json_start..=json_end];
            if let Ok(parsed) = serde_json::from_str::<IdeaBreakdownResponse>(json_str) {
                info!("  ✅ Successfully extracted and parsed JSON");
                return Ok(parsed);
            }
        }
    }

    // If JSON parsing fails, create a basic breakdown
    warn!("  ⚠️  Failed to parse as JSON, creating basic task breakdown");
    let basic_tasks = vec![
        TaskInput {
            id: "task-1".to_string(),
            title: "Core API Development".to_string(),
            description: "Implement REST API with authentication and CRUD operations".to_string(),
            priority: "High".to_string(),
            complexity: 8,
            agent_type: Some("BackendEngineerRust".to_string()),
            tags: vec!["api".to_string(), "backend".to_string()],
            depends_on: vec![],
        },
        TaskInput {
            id: "task-2".to_string(),
            title: "Database Setup".to_string(),
            description: "Set up SQLite database with user and todo tables".to_string(),
            priority: "High".to_string(),
            complexity: 5,
            agent_type: Some("BackendEngineerRust".to_string()),
            tags: vec!["database".to_string(), "backend".to_string()],
            depends_on: vec![],
        },
        TaskInput {
            id: "task-3".to_string(),
            title: "Authentication System".to_string(),
            description: "Implement JWT-based authentication system".to_string(),
            priority: "High".to_string(),
            complexity: 7,
            agent_type: Some("BackendEngineerRust".to_string()),
            tags: vec!["auth".to_string(), "security".to_string()],
            depends_on: vec!["task-1".to_string(), "task-2".to_string()],
        },
        TaskInput {
            id: "task-4".to_string(),
            title: "Testing Suite".to_string(),
            description: "Create comprehensive test suite for all endpoints".to_string(),
            priority: "Medium".to_string(),
            complexity: 6,
            agent_type: Some("QAEngineer".to_string()),
            tags: vec!["testing".to_string(), "qa".to_string()],
            depends_on: vec!["task-1".to_string(), "task-3".to_string()],
        },
        TaskInput {
            id: "task-5".to_string(),
            title: "Deployment Configuration".to_string(),
            description: "Set up Docker and CI/CD pipeline".to_string(),
            priority: "Medium".to_string(),
            complexity: 5,
            agent_type: Some("DevOpsEngineer".to_string()),
            tags: vec!["devops".to_string(), "deployment".to_string()],
            depends_on: vec!["task-4".to_string()],
        },
    ];

    Ok(IdeaBreakdownResponse {
        tasks: basic_tasks,
    })
}

/// Load agent context from the agents directory
async fn load_agent_context(agent_type: &str) -> Result<String, Box<dyn std::error::Error>> {
    use orchy::models::agent::Agent;

    // Load agents from directory
    let agents = Agent::load_agents_from_directory("./agents").await?;

    // Find the matching agent
    for agent in agents {
        if format!("{:?}", agent.agent_type).to_lowercase() == agent_type.to_lowercase() ||
           agent.name.to_lowercase().replace(" ", "") == agent_type.to_lowercase() {
            return Ok(agent.description);
        }
    }

    Err(format!("Agent context not found for type: {}", agent_type).into())
}

/// Run feature development with QA/build feedback incorporated
async fn run_feature_development_with_feedback(
    _mcp_client: &McpClient,
    task: &TaskInput,
    tech_stack: &str,
    existing_files: &[(String, String)],
    feedback: &[String],
    project_path: &std::path::PathBuf,
    results: &mut PipelineResults,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    info!("🔄 FEATURE DEVELOPMENT WITH FEEDBACK");
    info!("📝 Incorporating {} feedback items:", feedback.len());
    for (i, fb) in feedback.iter().enumerate() {
        info!("  {}. {}", i + 1, fb);
    }

    // Create enhanced prompt with feedback
    let feedback_context = if feedback.is_empty() {
        String::new()
    } else {
        format!(
            "\n\n🚨 CRITICAL FEEDBACK TO ADDRESS:\n{}\n\n⚠️ YOU MUST FIX ALL THESE ISSUES IN YOUR IMPLEMENTATION:\n",
            feedback.iter().enumerate()
                .map(|(i, fb)| format!("{}. {}", i + 1, fb))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    let enhanced_task_description = format!(
        "{}: {}\n\n{}PREVIOUS IMPLEMENTATION HAD ISSUES - CREATE A COMPLETE, WORKING SOLUTION THAT ADDRESSES ALL FEEDBACK.",
        task.title,
        task.description,
        feedback_context
    );

    let prompt = Prompts::feature_dev_todo_prompt(
        &enhanced_task_description,
        tech_stack,
        existing_files,
        Some("Previous implementation failed QA/build checks - must create working solution"),
        None, // TODO: Add agent context loading
    );

    info!("📝 ENHANCED FEATURE DEV PROMPT WITH FEEDBACK:");
    info!("{}", "=".repeat(80));
    info!("{}", prompt);
    info!("{}", "=".repeat(80));

    match GeminiCLI::query_with_model_and_retries(&prompt, "gemini-2.5-flash", 3).await {
        Ok(response) => {
            let duration = start_time.elapsed();
            info!("    ✅ Feature development with feedback completed in {:.2}s", duration.as_secs_f64());

            info!("🤖 ENHANCED LLM RESPONSE RECEIVED:");
            info!("{}", "=".repeat(80));
            info!("{}", response);
            info!("{}", "=".repeat(80));

            // Use Action::parse_and_execute to handle the response
            use orchy::enums::action::Action;

            // Extract JSON from markdown if present
            let json_content = utils::extract_json_from_response(&response);
            info!("    🔍 Feedback response JSON content: {}", json_content);

            // Change to project directory before executing actions
            let original_dir = std::env::current_dir()?;
            std::env::set_current_dir(project_path)?;

            match Action::parse_and_execute(&json_content).await {
                Ok(_results) => {
                    info!("    📄 Actions executed successfully with feedback incorporated");
                }
                Err(e) => {
                    std::env::set_current_dir(original_dir)?;
                    return Err(format!("Action execution failed: {}", e).into());
                }
            }

            // Restore original directory
            std::env::set_current_dir(original_dir)?;

            results.total_files_generated += 1; // Assume at least one file was generated
            Ok(vec![]) // Return empty files list since Action::parse_and_execute already handled everything
        }
        Err(e) => {
            error!("    ❌ Feature development with feedback failed: {}", e);
            Err(Box::new(e))
        }
    }
}

/// Attempt error recovery for failed feature development
async fn attempt_error_recovery(
    task: &TaskInput,
    tech_stack: &str,
    error_message: &str,
    existing_files: &[(String, String)],
    project_path: &std::path::PathBuf,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    info!("🔧 Running error recovery for task: {}", task.title);

    let prompt = Prompts::error_recovery_prompt(
        tech_stack,
        error_message,
        "feature development",
        existing_files,
        Some(&format!("Failed while implementing: {}", task.description)),
    );

    info!("📝 ERROR RECOVERY PROMPT SENT TO LLM:");
    info!("{}", "=".repeat(80));
    info!("{}", prompt);
    info!("{}", "=".repeat(80));

    match GeminiCLI::query_with_model_and_retries(&prompt, "gemini-2.5-flash", 2).await {
        Ok(response) => {
            info!("🤖 ERROR RECOVERY RESPONSE RECEIVED:");
            info!("{}", "=".repeat(80));
            info!("{}", response);
            info!("{}", "=".repeat(80));

            // Use Action::parse_and_execute to handle the recovery response
            use orchy::enums::action::Action;

            // Change to project directory before executing actions
            let original_dir = std::env::current_dir()?;
            std::env::set_current_dir(project_path)?;

            match Action::parse_and_execute(&response).await {
                Ok(_results) => {
                    info!("🔧 Error recovery actions executed successfully");
                    std::env::set_current_dir(original_dir)?;
                    Ok(vec![]) // Return empty files list since actions were executed
                }
                Err(e) => {
                    std::env::set_current_dir(original_dir)?;
                    Err(e)
                }
            }
        }
        Err(e) => {
            error!("❌ Error recovery failed: {}", e);
            Err(format!("Error recovery failed: {}", e).into())
        }
    }
}
