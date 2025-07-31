/// End-to-End Testing Framework for Prompt Calibration
/// 
/// This module provides comprehensive E2E testing for the entire development pipeline:
/// Frontend: Idea Breakdown → Feature Dev → Code Review → QA → DevOps
/// Backend: Idea Breakdown → Feature Dev → Code Review → QA → DevOps
/// 
/// Tests verify that prompts work well for major edge cases and can recover from errors.

pub mod framework;

use crate::mcp::gemini::GeminiCLI;
use crate::prompts::Prompts;
use std::path::Path;
use tokio_retry::{strategy::ExponentialBackoff, Retry};
use serde_json::Value;

/// E2E Test Configuration
#[derive(Debug, Clone)]
pub struct E2ETestConfig {
    pub test_app_dir: String,
    pub max_retries: usize,
    pub retry_delay_ms: u64,
    pub timeout_seconds: u64,
    pub gemini_model: String,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            test_app_dir: "testing_apps".to_string(),
            max_retries: 3,
            retry_delay_ms: 1000,
            timeout_seconds: 300,
            gemini_model: "gemini-2.5-flash".to_string(),
        }
    }
}

/// Pipeline Stage Results
#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage_name: String,
    pub success: bool,
    pub actions_executed: Vec<Value>,
    pub errors: Vec<String>,
    pub duration_ms: u128,
    pub retry_count: usize,
}

/// Complete Pipeline Result
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub pipeline_type: String, // "frontend" or "backend"
    pub app_name: String,
    pub stages: Vec<StageResult>,
    pub overall_success: bool,
    pub total_duration_ms: u128,
    pub final_app_status: AppStatus,
}

/// Application Status after pipeline completion
#[derive(Debug, Clone)]
pub struct AppStatus {
    pub builds_successfully: bool,
    pub tests_pass: bool,
    pub deployment_ready: bool,
    pub functionality_works: bool,
    pub performance_acceptable: bool,
}

/// E2E Test Runner
pub struct E2ETestRunner {
    config: E2ETestConfig,
}

impl E2ETestRunner {
    pub fn new(config: E2ETestConfig) -> Self {
        Self { config }
    }

    /// Run complete E2E test suite
    pub async fn run_complete_test_suite(&self) -> Result<Vec<PipelineResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        // Test Frontend Pipeline
        println!("🖥️  Starting Frontend Pipeline E2E Tests...");
        let frontend_result = self.test_frontend_pipeline().await?;
        results.push(frontend_result);

        // Test Backend Pipeline  
        println!("🔧 Starting Backend Pipeline E2E Tests...");
        let backend_result = self.test_backend_pipeline().await?;
        results.push(backend_result);

        // Generate comprehensive report
        self.generate_test_report(&results).await?;

        Ok(results)
    }

    /// Test Frontend Development Pipeline
    pub async fn test_frontend_pipeline(&self) -> Result<PipelineResult, Box<dyn std::error::Error>> {
        let app_name = "login-frontend-app";
        let app_dir = format!("{}/{}", self.config.test_app_dir, app_name);

        // Ensure clean test environment
        self.setup_test_environment(&app_dir).await?;

        let start_time = std::time::Instant::now();
        let mut stages = Vec::new();

        // Stage 1: Idea Breakdown
        let idea_breakdown_result = self.run_stage_with_retry(
            "Idea Breakdown",
            || self.run_idea_breakdown_stage(&app_dir, "frontend"),
        ).await?;
        stages.push(idea_breakdown_result);

        // Stage 2: Feature Development
        let feature_dev_result = self.run_stage_with_retry(
            "Feature Development", 
            || self.run_feature_development_stage(&app_dir, "frontend"),
        ).await?;
        stages.push(feature_dev_result);

        // Stage 3: Code Review
        let code_review_result = self.run_stage_with_retry(
            "Code Review",
            || self.run_code_review_stage(&app_dir),
        ).await?;
        stages.push(code_review_result);

        // Stage 4: QA Testing
        let qa_result = self.run_stage_with_retry(
            "QA Testing",
            || self.run_qa_stage(&app_dir, "frontend"),
        ).await?;
        stages.push(qa_result);

        // Stage 5: DevOps Deployment
        let devops_result = self.run_stage_with_retry(
            "DevOps Deployment",
            || self.run_devops_stage(&app_dir, "frontend"),
        ).await?;
        stages.push(devops_result);

        let total_duration = start_time.elapsed().as_millis();
        let overall_success = stages.iter().all(|s| s.success);
        let final_app_status = self.assess_app_status(&app_dir, "frontend").await?;

        Ok(PipelineResult {
            pipeline_type: "frontend".to_string(),
            app_name: app_name.to_string(),
            stages,
            overall_success,
            total_duration_ms: total_duration,
            final_app_status,
        })
    }

    /// Test Backend Development Pipeline
    pub async fn test_backend_pipeline(&self) -> Result<PipelineResult, Box<dyn std::error::Error>> {
        let app_name = "graphql-backend-app";
        let app_dir = format!("{}/{}", self.config.test_app_dir, app_name);

        // Ensure clean test environment
        self.setup_test_environment(&app_dir).await?;

        let start_time = std::time::Instant::now();
        let mut stages = Vec::new();

        // Stage 1: Idea Breakdown
        let idea_breakdown_result = self.run_stage_with_retry(
            "Idea Breakdown",
            || self.run_idea_breakdown_stage(&app_dir, "backend"),
        ).await?;
        stages.push(idea_breakdown_result);

        // Stage 2: Feature Development
        let feature_dev_result = self.run_stage_with_retry(
            "Feature Development",
            || self.run_feature_development_stage(&app_dir, "backend"),
        ).await?;
        stages.push(feature_dev_result);

        // Stage 3: Code Review
        let code_review_result = self.run_stage_with_retry(
            "Code Review",
            || self.run_code_review_stage(&app_dir),
        ).await?;
        stages.push(code_review_result);

        // Stage 4: QA Testing
        let qa_result = self.run_stage_with_retry(
            "QA Testing",
            || self.run_qa_stage(&app_dir, "backend"),
        ).await?;
        stages.push(qa_result);

        // Stage 5: DevOps Deployment
        let devops_result = self.run_stage_with_retry(
            "DevOps Deployment",
            || self.run_devops_stage(&app_dir, "backend"),
        ).await?;
        stages.push(devops_result);

        let total_duration = start_time.elapsed().as_millis();
        let overall_success = stages.iter().all(|s| s.success);
        let final_app_status = self.assess_app_status(&app_dir, "backend").await?;

        Ok(PipelineResult {
            pipeline_type: "backend".to_string(),
            app_name: app_name.to_string(),
            stages,
            overall_success,
            total_duration_ms: total_duration,
            final_app_status,
        })
    }

    /// Run a stage with retry mechanism
    async fn run_stage_with_retry<F, Fut>(
        &self,
        stage_name: &str,
        stage_fn: F,
    ) -> Result<StageResult, Box<dyn std::error::Error>>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<StageResult, Box<dyn std::error::Error>>>,
    {
        let retry_strategy = ExponentialBackoff::from_millis(self.config.retry_delay_ms)
            .max_delay(std::time::Duration::from_secs(30))
            .take(self.config.max_retries);

        let result = Retry::spawn(retry_strategy, || async {
            println!("🔄 Running stage: {}", stage_name);
            stage_fn().await
        }).await;

        match result {
            Ok(stage_result) => {
                println!("✅ Stage '{}' completed successfully", stage_name);
                Ok(stage_result)
            }
            Err(e) => {
                println!("❌ Stage '{}' failed after {} retries: {}", stage_name, self.config.max_retries, e);
                Ok(StageResult {
                    stage_name: stage_name.to_string(),
                    success: false,
                    actions_executed: vec![],
                    errors: vec![format!("Stage failed: {}", e)],
                    duration_ms: 0,
                    retry_count: self.config.max_retries,
                })
            }
        }
    }
}
