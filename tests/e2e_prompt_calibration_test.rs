/// Integration test for E2E Prompt Calibration
/// 
/// This test verifies that the E2E testing framework can be instantiated
/// and basic functionality works correctly.

use orchy::e2e_tests::{E2ETestRunner, E2ETestConfig};

#[tokio::test]
async fn test_e2e_framework_instantiation() {
    let config = E2ETestConfig {
        test_app_dir: "test_temp_apps".to_string(),
        max_retries: 1,
        retry_delay_ms: 100,
        timeout_seconds: 10,
        gemini_model: "gemini-2.5-flash".to_string(),
    };

    let runner = E2ETestRunner::new(config.clone());
    
    // Test that we can create the test environment
    let test_dir = "test_temp_apps/test_app";
    let result = runner.setup_test_environment(test_dir).await;
    
    assert!(result.is_ok(), "Should be able to set up test environment");
    
    // Verify directory was created
    assert!(std::path::Path::new(test_dir).exists(), "Test directory should exist");
    
    // Clean up
    if std::path::Path::new("test_temp_apps").exists() {
        std::fs::remove_dir_all("test_temp_apps").ok();
    }
}

#[tokio::test]
async fn test_project_structure_creation() {
    let config = E2ETestConfig::default();
    let runner = E2ETestRunner::new(config);
    
    let test_dir = "test_temp_apps/frontend_test";
    runner.setup_test_environment(test_dir).await.unwrap();
    
    // Test frontend project structure creation
    let tasks = vec![];
    let result = runner.create_project_structure(test_dir, "frontend", &tasks).await;
    
    assert!(result.is_ok(), "Should be able to create frontend project structure");
    
    // Verify package.json was created
    let package_json_path = format!("{}/package.json", test_dir);
    assert!(std::path::Path::new(&package_json_path).exists(), "package.json should exist");
    
    // Verify directory structure
    assert!(std::path::Path::new(&format!("{}/src/components", test_dir)).exists());
    assert!(std::path::Path::new(&format!("{}/src/stores", test_dir)).exists());
    assert!(std::path::Path::new(&format!("{}/tests/unit", test_dir)).exists());
    
    // Clean up
    if std::path::Path::new("test_temp_apps").exists() {
        std::fs::remove_dir_all("test_temp_apps").ok();
    }
}

#[tokio::test]
async fn test_backend_project_structure_creation() {
    let config = E2ETestConfig::default();
    let runner = E2ETestRunner::new(config);
    
    let test_dir = "test_temp_apps/backend_test";
    runner.setup_test_environment(test_dir).await.unwrap();
    
    // Test backend project structure creation
    let tasks = vec![];
    let result = runner.create_project_structure(test_dir, "backend", &tasks).await;
    
    assert!(result.is_ok(), "Should be able to create backend project structure");
    
    // Verify package.json was created
    let package_json_path = format!("{}/package.json", test_dir);
    assert!(std::path::Path::new(&package_json_path).exists(), "package.json should exist");
    
    // Verify directory structure
    assert!(std::path::Path::new(&format!("{}/src/resolvers", test_dir)).exists());
    assert!(std::path::Path::new(&format!("{}/src/models", test_dir)).exists());
    assert!(std::path::Path::new(&format!("{}/tests/integration", test_dir)).exists());
    
    // Clean up
    if std::path::Path::new("test_temp_apps").exists() {
        std::fs::remove_dir_all("test_temp_apps").ok();
    }
}

#[tokio::test]
async fn test_tech_stack_detection() {
    let config = E2ETestConfig::default();
    let runner = E2ETestRunner::new(config);
    
    let test_dir = "test_temp_apps/tech_stack_test";
    runner.setup_test_environment(test_dir).await.unwrap();
    
    // Create a Vue.js package.json
    let package_json = r#"{
  "name": "test-app",
  "dependencies": {
    "vue": "^3.4.0",
    "vue-router": "^4.2.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "vite": "^5.0.0"
  }
}"#;
    
    std::fs::write(format!("{}/package.json", test_dir), package_json).unwrap();
    
    let tech_stack = runner.detect_tech_stack(test_dir).await.unwrap();
    
    assert!(tech_stack.contains("Vue 3"), "Should detect Vue 3");
    assert!(tech_stack.contains("TypeScript"), "Should detect TypeScript");
    assert!(tech_stack.contains("Vite"), "Should detect Vite");
    
    // Clean up
    if std::path::Path::new("test_temp_apps").exists() {
        std::fs::remove_dir_all("test_temp_apps").ok();
    }
}

#[tokio::test]
async fn test_read_project_files() {
    let config = E2ETestConfig::default();
    let runner = E2ETestRunner::new(config);
    
    let test_dir = "test_temp_apps/read_files_test";
    runner.setup_test_environment(test_dir).await.unwrap();
    
    // Create some test files
    std::fs::create_dir_all(format!("{}/src", test_dir)).unwrap();
    std::fs::write(format!("{}/package.json", test_dir), r#"{"name": "test"}"#).unwrap();
    std::fs::write(format!("{}/src/main.ts", test_dir), "console.log('hello');").unwrap();
    
    let files = runner.read_project_files(test_dir).await.unwrap();
    
    assert!(!files.is_empty(), "Should read some files");
    assert!(files.iter().any(|(name, _)| name == "package.json"), "Should include package.json");
    assert!(files.iter().any(|(name, _)| name == "src/main.ts"), "Should include main.ts");
    
    // Clean up
    if std::path::Path::new("test_temp_apps").exists() {
        std::fs::remove_dir_all("test_temp_apps").ok();
    }
}

#[test]
fn test_config_parsing() {
    let config = E2ETestConfig::default();
    
    assert_eq!(config.test_app_dir, "testing_apps");
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.retry_delay_ms, 1000);
    assert_eq!(config.timeout_seconds, 300);
    assert_eq!(config.gemini_model, "gemini-2.5-flash");
}

#[test]
fn test_app_status_creation() {
    use orchy::e2e_tests::AppStatus;
    
    let status = AppStatus {
        builds_successfully: true,
        tests_pass: false,
        deployment_ready: true,
        functionality_works: true,
        performance_acceptable: true,
    };
    
    assert!(status.builds_successfully);
    assert!(!status.tests_pass);
    assert!(status.deployment_ready);
    assert!(status.functionality_works);
    assert!(status.performance_acceptable);
}

#[test]
fn test_stage_result_creation() {
    use orchy::e2e_tests::StageResult;
    use serde_json::json;
    
    let stage_result = StageResult {
        stage_name: "Test Stage".to_string(),
        success: true,
        actions_executed: vec![json!({"Write": {"path": "test.txt", "content": "test"}})],
        errors: vec![],
        duration_ms: 1000,
        retry_count: 0,
    };
    
    assert_eq!(stage_result.stage_name, "Test Stage");
    assert!(stage_result.success);
    assert_eq!(stage_result.actions_executed.len(), 1);
    assert!(stage_result.errors.is_empty());
    assert_eq!(stage_result.duration_ms, 1000);
    assert_eq!(stage_result.retry_count, 0);
}

#[test]
fn test_pipeline_result_creation() {
    use orchy::e2e_tests::{PipelineResult, StageResult, AppStatus};
    
    let app_status = AppStatus {
        builds_successfully: true,
        tests_pass: true,
        deployment_ready: true,
        functionality_works: true,
        performance_acceptable: true,
    };
    
    let stage = StageResult {
        stage_name: "Test Stage".to_string(),
        success: true,
        actions_executed: vec![],
        errors: vec![],
        duration_ms: 500,
        retry_count: 0,
    };
    
    let pipeline_result = PipelineResult {
        pipeline_type: "frontend".to_string(),
        app_name: "test-app".to_string(),
        stages: vec![stage],
        overall_success: true,
        total_duration_ms: 2000,
        final_app_status: app_status,
    };
    
    assert_eq!(pipeline_result.pipeline_type, "frontend");
    assert_eq!(pipeline_result.app_name, "test-app");
    assert_eq!(pipeline_result.stages.len(), 1);
    assert!(pipeline_result.overall_success);
    assert_eq!(pipeline_result.total_duration_ms, 2000);
    assert!(pipeline_result.final_app_status.builds_successfully);
}
