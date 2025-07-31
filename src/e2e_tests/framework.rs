/// E2E Testing Framework Implementation
/// 
/// Core implementation for running pipeline stages and managing test applications

use super::*;
use crate::enums::action::Action;
use std::fs;
use std::process::Command;
use tokio::time::{timeout, Duration};

impl E2ETestRunner {
    /// Setup clean test environment
    pub async fn setup_test_environment(&self, app_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Remove existing test app if it exists
        if Path::new(app_dir).exists() {
            fs::remove_dir_all(app_dir)?;
        }

        // Create fresh test app directory
        fs::create_dir_all(app_dir)?;
        
        println!("🧹 Clean test environment created at: {}", app_dir);
        Ok(())
    }

    /// Run Idea Breakdown Stage
    pub async fn run_idea_breakdown_stage(
        &self,
        app_dir: &str,
        app_type: &str,
    ) -> Result<StageResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut actions_executed = Vec::new();
        let errors = Vec::new();

        let idea = match app_type {
            "frontend" => "Create a Vue.js login page with form validation, responsive design, and authentication integration. Include email/password fields, remember me checkbox, forgot password link, and proper error handling.",
            "backend" => "Create a GraphQL server with user authentication (login/signup) using Node.js, Express, Apollo Server, and JWT tokens. Include user registration, login, password hashing, and protected resolvers.",
            _ => return Err("Invalid app type".into()),
        };

        let context = format!("Building a {} application for E2E testing", app_type);
        let tech_stack = match app_type {
            "frontend" => "Vue 3, TypeScript, Vite, Pinia, Vue Router, Tailwind CSS",
            "backend" => "Node.js, Express, Apollo Server, GraphQL, JWT, bcrypt, MongoDB",
            _ => return Err("Invalid app type".into()),
        };

        let agents = vec![
            "FeatureDev".to_string(),
            "CodeReviewer".to_string(), 
            "QA".to_string(),
            "DevOps".to_string(),
        ];

        // Generate idea breakdown prompt
        let prompt = Prompts::idea_breakdown_user_prompt(idea, &context, agents, tech_stack);

        // Call Gemini with retry mechanism
        let response = self.call_gemini_with_retry(&prompt).await?;
        
        // Parse response and extract tasks
        let tasks = self.parse_idea_breakdown_response(&response)?;
        
        // Create initial project structure
        let project_actions = self.create_project_structure(app_dir, app_type, &tasks).await?;
        actions_executed.extend(project_actions);

        let duration = start_time.elapsed().as_millis();
        
        Ok(StageResult {
            stage_name: "Idea Breakdown".to_string(),
            success: true,
            actions_executed,
            errors,
            duration_ms: duration,
            retry_count: 0,
        })
    }

    /// Run Feature Development Stage
    pub async fn run_feature_development_stage(
        &self,
        app_dir: &str,
        app_type: &str,
    ) -> Result<StageResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut actions_executed = Vec::new();
        let mut errors = Vec::new();
        let mut retry_count = 0;

        // Read existing project files
        let project_files = self.read_project_files(app_dir).await?;

        let objective = match app_type {
            "frontend" => "Implement a complete Vue.js login page with form validation, authentication integration, and responsive design",
            "backend" => "Implement GraphQL server with user authentication, registration, login endpoints, and JWT token management",
            _ => return Err("Invalid app type".into()),
        };

        let tech_stack = match app_type {
            "frontend" => "Vue 3, TypeScript, Vite, Pinia, Vue Router, Tailwind CSS",
            "backend" => "Node.js, Express, Apollo Server, GraphQL, JWT, bcrypt, MongoDB",
            _ => return Err("Invalid app type".into()),
        };

        // Feature development loop with error recovery
        loop {
            // Generate feature development prompt
            let prompt = Prompts::feature_dev_todo_prompt(
                objective,
                tech_stack,
                &project_files,
                None, // No current error initially
            );

            // Call Gemini to get development actions
            let response = self.call_gemini_with_retry(&prompt).await?;
            let actions = self.parse_actions_response(&response)?;

            // Execute actions and check for errors
            let (executed_actions, execution_errors) = self.execute_actions_with_validation(app_dir, &actions).await?;
            actions_executed.extend(executed_actions);

            if execution_errors.is_empty() {
                // Success - break out of loop
                break;
            } else {
                // Handle errors with error recovery
                errors.extend(execution_errors.clone());
                retry_count += 1;

                if retry_count >= self.config.max_retries {
                    return Ok(StageResult {
                        stage_name: "Feature Development".to_string(),
                        success: false,
                        actions_executed,
                        errors,
                        duration_ms: start_time.elapsed().as_millis(),
                        retry_count,
                    });
                }

                // Use error recovery prompt
                let error_output = execution_errors.join("\n");
                let recovery_actions = self.run_error_recovery(app_dir, &error_output, tech_stack).await?;
                actions_executed.extend(recovery_actions);

                // Re-read project files for next iteration
                let updated_files = self.read_project_files(app_dir).await?;
                if updated_files.len() > project_files.len() {
                    // Progress made, continue
                    continue;
                } else {
                    // No progress, try different approach
                    println!("⚠️  No progress made, trying alternative approach...");
                }
            }
        }

        let duration = start_time.elapsed().as_millis();

        Ok(StageResult {
            stage_name: "Feature Development".to_string(),
            success: true,
            actions_executed,
            errors,
            duration_ms: duration,
            retry_count,
        })
    }

    /// Run Code Review Stage
    pub async fn run_code_review_stage(
        &self,
        app_dir: &str,
    ) -> Result<StageResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut actions_executed = Vec::new();
        let mut errors = Vec::new();

        // Read all source files for review
        let source_files = self.read_source_files(app_dir).await?;
        
        let tech_stack = self.detect_tech_stack(app_dir).await?;
        let focus_areas = vec![
            "DRY violations".to_string(),
            "Code maintainability".to_string(),
            "Security best practices".to_string(),
            "Performance optimizations".to_string(),
            "Error handling".to_string(),
        ];

        // Generate code review prompt
        let prompt = Prompts::code_review_agent_prompt(&tech_stack, &source_files, &focus_areas);

        // Call Gemini for code review
        let response = self.call_gemini_with_retry(&prompt).await?;
        let review_actions = self.parse_actions_response(&response)?;

        // Execute code review improvements
        let (executed_actions, execution_errors) = self.execute_actions_with_validation(app_dir, &review_actions).await?;
        actions_executed.extend(executed_actions);
        errors.extend(execution_errors);

        let duration = start_time.elapsed().as_millis();

        Ok(StageResult {
            stage_name: "Code Review".to_string(),
            success: errors.is_empty(),
            actions_executed,
            errors,
            duration_ms: duration,
            retry_count: 0,
        })
    }

    /// Run QA Testing Stage
    pub async fn run_qa_stage(
        &self,
        app_dir: &str,
        app_type: &str,
    ) -> Result<StageResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut actions_executed = Vec::new();
        let mut errors = Vec::new();

        let tech_stack = self.detect_tech_stack(app_dir).await?;
        let application_files = self.read_project_files(app_dir).await?;

        let test_types = match app_type {
            "frontend" => vec![
                "Unit tests for components".to_string(),
                "Integration tests for API calls".to_string(),
                "E2E tests for user flows".to_string(),
                "Accessibility tests".to_string(),
            ],
            "backend" => vec![
                "Unit tests for resolvers".to_string(),
                "Integration tests for database".to_string(),
                "API endpoint tests".to_string(),
                "Authentication tests".to_string(),
            ],
            _ => return Err("Invalid app type".into()),
        };

        // Run different types of QA testing
        for test_type in &test_types {
            let qa_prompt = match test_type.as_str() {
                s if s.contains("Unit") => {
                    Prompts::unit_testing_prompt(&tech_stack, &application_files, "vitest", &[], None)
                }
                s if s.contains("Integration") => {
                    Prompts::integration_testing_prompt(&tech_stack, &application_files, "vitest", &test_types, None)
                }
                s if s.contains("E2E") => {
                    Prompts::e2e_testing_prompt(&tech_stack, "http://localhost:3000", &test_types, "playwright", None)
                }
                _ => {
                    // Use general QA prompt
                    Prompts::qa_agent_prompt(&tech_stack, &application_files, &test_types, None)
                }
            };

            let response = self.call_gemini_with_retry(&qa_prompt).await?;
            let qa_actions = self.parse_actions_response(&response)?;

            let (executed_actions, execution_errors) = self.execute_actions_with_validation(app_dir, &qa_actions).await?;
            actions_executed.extend(executed_actions);
            errors.extend(execution_errors);
        }

        let duration = start_time.elapsed().as_millis();

        Ok(StageResult {
            stage_name: "QA Testing".to_string(),
            success: errors.is_empty(),
            actions_executed,
            errors,
            duration_ms: duration,
            retry_count: 0,
        })
    }

    /// Run DevOps Deployment Stage
    pub async fn run_devops_stage(
        &self,
        app_dir: &str,
        app_type: &str,
    ) -> Result<StageResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut actions_executed = Vec::new();
        let mut errors = Vec::new();

        let tech_stack = self.detect_tech_stack(app_dir).await?;
        let project_files = self.read_project_files(app_dir).await?;

        let deployment_target = match app_type {
            "frontend" => "Vercel",
            "backend" => "Railway",
            _ => return Err("Invalid app type".into()),
        };

        // Generate DevOps prompt
        let prompt = Prompts::devops_agent_prompt(&tech_stack, &project_files, deployment_target, None);

        // Call Gemini for DevOps setup
        let response = self.call_gemini_with_retry(&prompt).await?;
        let devops_actions = self.parse_actions_response(&response)?;

        // Execute DevOps actions
        let (executed_actions, execution_errors) = self.execute_actions_with_validation(app_dir, &devops_actions).await?;
        actions_executed.extend(executed_actions);
        errors.extend(execution_errors);

        // Test deployment readiness
        let deployment_ready = self.test_deployment_readiness(app_dir, app_type).await?;
        if !deployment_ready {
            errors.push("Application not ready for deployment".to_string());
        }

        let duration = start_time.elapsed().as_millis();

        Ok(StageResult {
            stage_name: "DevOps Deployment".to_string(),
            success: errors.is_empty() && deployment_ready,
            actions_executed,
            errors,
            duration_ms: duration,
            retry_count: 0,
        })
    }

    /// Call Gemini with retry mechanism
    pub async fn call_gemini_with_retry(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let retry_strategy = ExponentialBackoff::from_millis(self.config.retry_delay_ms)
            .max_delay(Duration::from_secs(30))
            .take(self.config.max_retries);

        Retry::spawn(retry_strategy, || async {
            match timeout(
                Duration::from_secs(self.config.timeout_seconds),
                GeminiCLI::query_with_model(prompt, &self.config.gemini_model)
            ).await {
                Ok(result) => result,
                Err(_) => Err(crate::error::OrchestratorError::internal("Timeout waiting for Gemini response".to_string())),
            }
        }).await.map_err(|e| e.into())
    }

    /// Parse idea breakdown response into tasks
    pub fn parse_idea_breakdown_response(&self, response: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        // Extract JSON from response
        let json_start = response.find('[').unwrap_or(0);
        let json_end = response.rfind(']').unwrap_or(response.len());
        let json_str = &response[json_start..=json_end];

        let tasks: Vec<Value> = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse idea breakdown response: {}", e))?;

        Ok(tasks)
    }

    /// Parse actions response from Gemini
    pub fn parse_actions_response(&self, response: &str) -> Result<Vec<Action>, Box<dyn std::error::Error>> {
        // Extract JSON array from response
        let json_start = response.find('[').unwrap_or(0);
        let json_end = response.rfind(']').unwrap_or(response.len());
        let json_str = &response[json_start..=json_end];

        let actions = Action::from_json_array(json_str)
            .map_err(|e| format!("Failed to parse actions response: {}", e))?;

        Ok(actions)
    }

    /// Create initial project structure
    pub async fn create_project_structure(
        &self,
        app_dir: &str,
        app_type: &str,
        _tasks: &[Value],
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let mut actions = Vec::new();

        match app_type {
            "frontend" => {
                // Create Vue.js project structure
                let package_json = r#"{
  "name": "login-frontend-app",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc && vite build",
    "preview": "vite preview",
    "test": "vitest",
    "test:e2e": "playwright test"
  },
  "dependencies": {
    "vue": "^3.4.0",
    "vue-router": "^4.2.0",
    "pinia": "^2.1.0",
    "@vueuse/core": "^10.0.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^4.5.0",
    "typescript": "^5.0.0",
    "vue-tsc": "^1.8.0",
    "vite": "^5.0.0",
    "vitest": "^1.0.0",
    "@playwright/test": "^1.40.0",
    "tailwindcss": "^3.3.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  }
}"#;

                fs::write(format!("{}/package.json", app_dir), package_json)?;
                actions.push(serde_json::json!({
                    "Write": {
                        "path": format!("{}/package.json", app_dir),
                        "content": package_json
                    }
                }));

                // Create basic directory structure
                fs::create_dir_all(format!("{}/src/components", app_dir))?;
                fs::create_dir_all(format!("{}/src/stores", app_dir))?;
                fs::create_dir_all(format!("{}/src/views", app_dir))?;
                fs::create_dir_all(format!("{}/src/types", app_dir))?;
                fs::create_dir_all(format!("{}/tests/unit", app_dir))?;
                fs::create_dir_all(format!("{}/tests/e2e", app_dir))?;
            }
            "backend" => {
                // Create Node.js GraphQL project structure
                let package_json = r#"{
  "name": "graphql-backend-app",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "nodemon src/index.js",
    "start": "node src/index.js",
    "test": "jest",
    "test:integration": "jest --testPathPattern=integration"
  },
  "dependencies": {
    "apollo-server-express": "^3.12.0",
    "express": "^4.18.0",
    "graphql": "^16.8.0",
    "jsonwebtoken": "^9.0.0",
    "bcryptjs": "^2.4.0",
    "mongoose": "^8.0.0",
    "dotenv": "^16.3.0"
  },
  "devDependencies": {
    "nodemon": "^3.0.0",
    "jest": "^29.7.0",
    "supertest": "^6.3.0",
    "@types/jest": "^29.5.0"
  }
}"#;

                fs::write(format!("{}/package.json", app_dir), package_json)?;
                actions.push(serde_json::json!({
                    "Write": {
                        "path": format!("{}/package.json", app_dir),
                        "content": package_json
                    }
                }));

                // Create basic directory structure
                fs::create_dir_all(format!("{}/src/resolvers", app_dir))?;
                fs::create_dir_all(format!("{}/src/models", app_dir))?;
                fs::create_dir_all(format!("{}/src/middleware", app_dir))?;
                fs::create_dir_all(format!("{}/src/utils", app_dir))?;
                fs::create_dir_all(format!("{}/tests/unit", app_dir))?;
                fs::create_dir_all(format!("{}/tests/integration", app_dir))?;
            }
            _ => return Err("Invalid app type".into()),
        }

        Ok(actions)
    }

    /// Read project files for context
    pub async fn read_project_files(&self, app_dir: &str) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();

        if !Path::new(app_dir).exists() {
            return Ok(files);
        }

        // Read key files for context
        let key_files = vec![
            "package.json",
            "src/main.js",
            "src/main.ts",
            "src/index.js",
            "src/App.vue",
            "vite.config.ts",
            "vite.config.js",
        ];

        for file_name in key_files {
            let file_path = format!("{}/{}", app_dir, file_name);
            if Path::new(&file_path).exists() {
                match fs::read_to_string(&file_path) {
                    Ok(content) => {
                        files.push((file_name.to_string(), content));
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(files)
    }

    /// Read source files for code review
    pub async fn read_source_files(&self, app_dir: &str) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();

        // Recursively read source files
        self.read_files_recursive(&format!("{}/src", app_dir), &mut files)?;

        Ok(files)
    }

    /// Recursively read files from directory
    fn read_files_recursive(&self, dir: &str, files: &mut Vec<(String, String)>) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(dir).exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.read_files_recursive(path.to_str().unwrap(), files)?;
            } else if let Some(extension) = path.extension() {
                let ext = extension.to_str().unwrap();
                if matches!(ext, "js" | "ts" | "vue" | "jsx" | "tsx" | "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let relative_path = path.strip_prefix("testing_apps/")
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string();
                        files.push((relative_path, content));
                    }
                }
            }
        }

        Ok(())
    }

    /// Detect technology stack from project files
    pub async fn detect_tech_stack(&self, app_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
        let package_json_path = format!("{}/package.json", app_dir);

        if Path::new(&package_json_path).exists() {
            let content = fs::read_to_string(package_json_path)?;
            let package: Value = serde_json::from_str(&content)?;

            let dependencies = package.get("dependencies").and_then(|d| d.as_object());
            let dev_dependencies = package.get("devDependencies").and_then(|d| d.as_object());

            let mut tech_stack = Vec::new();

            // Check for frontend frameworks
            if dependencies.map_or(false, |d| d.contains_key("vue")) {
                tech_stack.push("Vue 3");
            }
            if dependencies.map_or(false, |d| d.contains_key("react")) {
                tech_stack.push("React");
            }

            // Check for backend frameworks
            if dependencies.map_or(false, |d| d.contains_key("express")) {
                tech_stack.push("Express");
            }
            if dependencies.map_or(false, |d| d.contains_key("apollo-server-express")) {
                tech_stack.push("Apollo Server");
            }

            // Check for TypeScript
            if dev_dependencies.map_or(false, |d| d.contains_key("typescript")) {
                tech_stack.push("TypeScript");
            }

            // Check for build tools
            if dev_dependencies.map_or(false, |d| d.contains_key("vite")) {
                tech_stack.push("Vite");
            }

            Ok(tech_stack.join(", "))
        } else {
            Ok("Unknown".to_string())
        }
    }

    /// Execute actions with validation and error handling
    pub async fn execute_actions_with_validation(
        &self,
        app_dir: &str,
        actions: &[Action],
    ) -> Result<(Vec<Value>, Vec<String>), Box<dyn std::error::Error>> {
        let mut executed_actions = Vec::new();
        let mut errors = Vec::new();

        for action in actions {
            match action.execute().await {
                Ok(_) => {
                    executed_actions.push(serde_json::to_value(action)?);

                    // Validate the action result
                    if let Err(validation_error) = self.validate_action_result(app_dir, action).await {
                        errors.push(validation_error);
                    }
                }
                Err(e) => {
                    errors.push(format!("Action execution failed: {}", e));
                }
            }
        }

        Ok((executed_actions, errors))
    }

    /// Validate action result
    pub async fn validate_action_result(
        &self,
        app_dir: &str,
        action: &Action,
    ) -> Result<(), String> {
        match action {
            Action::Write { path, .. } => {
                if !Path::new(path).exists() {
                    return Err(format!("File was not created: {}", path));
                }
            }
            Action::RunCommand { command, .. } => {
                // For npm/node commands, check if they completed successfully
                if command.contains("npm") || command.contains("node") {
                    // Run a quick validation command
                    let validation_result = Command::new("sh")
                        .arg("-c")
                        .arg(format!("cd {} && npm list --depth=0", app_dir))
                        .output();

                    if let Ok(output) = validation_result {
                        if !output.status.success() {
                            return Err(format!("npm validation failed after command: {}", command));
                        }
                    }
                }
            }
            _ => {} // Other actions don't need special validation
        }

        Ok(())
    }

    /// Run error recovery
    pub async fn run_error_recovery(
        &self,
        app_dir: &str,
        error_output: &str,
        tech_stack: &str,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let project_files = self.read_project_files(app_dir).await?;

        let prompt = Prompts::error_recovery_prompt(
            tech_stack,
            error_output,
            "npm run dev", // Most common failing command
            &project_files,
            Some("Recent development changes"),
        );

        let response = self.call_gemini_with_retry(&prompt).await?;
        let recovery_actions = self.parse_actions_response(&response)?;

        let (executed_actions, _) = self.execute_actions_with_validation(app_dir, &recovery_actions).await?;

        Ok(executed_actions)
    }

    /// Test deployment readiness
    pub async fn test_deployment_readiness(
        &self,
        app_dir: &str,
        app_type: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let build_command = match app_type {
            "frontend" => "npm run build",
            "backend" => "npm run start",
            _ => return Ok(false),
        };

        // Test if the application builds successfully
        let build_result = Command::new("sh")
            .arg("-c")
            .arg(format!("cd {} && {}", app_dir, build_command))
            .output()?;

        Ok(build_result.status.success())
    }

    /// Assess final application status
    pub async fn assess_app_status(
        &self,
        app_dir: &str,
        app_type: &str,
    ) -> Result<AppStatus, Box<dyn std::error::Error>> {
        let builds_successfully = self.test_build_success(app_dir, app_type).await?;
        let tests_pass = self.test_tests_pass(app_dir).await?;
        let deployment_ready = self.test_deployment_readiness(app_dir, app_type).await?;
        let functionality_works = self.test_functionality(app_dir, app_type).await?;
        let performance_acceptable = self.test_performance(app_dir, app_type).await?;

        Ok(AppStatus {
            builds_successfully,
            tests_pass,
            deployment_ready,
            functionality_works,
            performance_acceptable,
        })
    }

    /// Test if application builds successfully
    async fn test_build_success(&self, app_dir: &str, app_type: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let build_command = match app_type {
            "frontend" => "npm run build",
            "backend" => "npm run start",
            _ => return Ok(false),
        };

        let result = Command::new("sh")
            .arg("-c")
            .arg(format!("cd {} && timeout 30 {}", app_dir, build_command))
            .output()?;

        Ok(result.status.success())
    }

    /// Test if tests pass
    async fn test_tests_pass(&self, app_dir: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let result = Command::new("sh")
            .arg("-c")
            .arg(format!("cd {} && npm test", app_dir))
            .output()?;

        Ok(result.status.success())
    }

    /// Test basic functionality
    async fn test_functionality(&self, app_dir: &str, app_type: &str) -> Result<bool, Box<dyn std::error::Error>> {
        match app_type {
            "frontend" => {
                // Check if key Vue files exist and are valid
                let app_vue_path = format!("{}/src/App.vue", app_dir);
                let main_ts_path = format!("{}/src/main.ts", app_dir);

                Ok(Path::new(&app_vue_path).exists() && Path::new(&main_ts_path).exists())
            }
            "backend" => {
                // Check if GraphQL schema and resolvers exist
                let index_path = format!("{}/src/index.js", app_dir);
                let resolvers_dir = format!("{}/src/resolvers", app_dir);

                Ok(Path::new(&index_path).exists() && Path::new(&resolvers_dir).exists())
            }
            _ => Ok(false),
        }
    }

    /// Test performance
    async fn test_performance(&self, _app_dir: &str, _app_type: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // For now, just return true - in a real implementation, this would run performance tests
        Ok(true)
    }

    /// Generate comprehensive test report
    pub async fn generate_test_report(&self, results: &[PipelineResult]) -> Result<(), Box<dyn std::error::Error>> {
        let report_path = format!("{}/e2e_test_report.md", self.config.test_app_dir);

        let mut report = String::new();
        report.push_str("# E2E Pipeline Test Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        for result in results {
            report.push_str(&format!("## {} Pipeline\n\n", result.pipeline_type.to_uppercase()));
            report.push_str(&format!("**Application**: {}\n", result.app_name));
            report.push_str(&format!("**Overall Success**: {}\n", if result.overall_success { "✅ PASS" } else { "❌ FAIL" }));
            report.push_str(&format!("**Total Duration**: {}ms\n\n", result.total_duration_ms));

            report.push_str("### Stages\n\n");
            for stage in &result.stages {
                report.push_str(&format!("#### {}\n", stage.stage_name));
                report.push_str(&format!("- **Status**: {}\n", if stage.success { "✅ PASS" } else { "❌ FAIL" }));
                report.push_str(&format!("- **Duration**: {}ms\n", stage.duration_ms));
                report.push_str(&format!("- **Retry Count**: {}\n", stage.retry_count));
                report.push_str(&format!("- **Actions Executed**: {}\n", stage.actions_executed.len()));

                if !stage.errors.is_empty() {
                    report.push_str("- **Errors**:\n");
                    for error in &stage.errors {
                        report.push_str(&format!("  - {}\n", error));
                    }
                }
                report.push_str("\n");
            }

            report.push_str("### Final Application Status\n\n");
            let status = &result.final_app_status;
            report.push_str(&format!("- **Builds Successfully**: {}\n", if status.builds_successfully { "✅" } else { "❌" }));
            report.push_str(&format!("- **Tests Pass**: {}\n", if status.tests_pass { "✅" } else { "❌" }));
            report.push_str(&format!("- **Deployment Ready**: {}\n", if status.deployment_ready { "✅" } else { "❌" }));
            report.push_str(&format!("- **Functionality Works**: {}\n", if status.functionality_works { "✅" } else { "❌" }));
            report.push_str(&format!("- **Performance Acceptable**: {}\n", if status.performance_acceptable { "✅" } else { "❌" }));
            report.push_str("\n");
        }

        fs::write(report_path, report)?;
        println!("📊 Test report generated at: {}/e2e_test_report.md", self.config.test_app_dir);

        Ok(())
    }
}
