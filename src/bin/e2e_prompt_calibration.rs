/// E2E Prompt Calibration Test Runner
/// 
/// This binary runs comprehensive E2E tests to calibrate and validate the entire
/// development pipeline from idea breakdown to deployment.
/// 
/// Usage: cargo run --bin e2e_prompt_calibration

use orchy::e2e_tests::{E2ETestRunner, E2ETestConfig};
use std::env;
use std::process;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting E2E Prompt Calibration Tests");
    println!("{}", "=".repeat(60));

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let config = parse_config(&args);

    // Create test runner
    let runner = E2ETestRunner::new(config.clone());

    // Ensure testing directory exists
    if let Err(e) = std::fs::create_dir_all(&config.test_app_dir) {
        eprintln!("❌ Failed to create testing directory: {}", e);
        process::exit(1);
    }

    println!("📁 Testing directory: {}", config.test_app_dir);
    println!("🤖 Using Gemini model: {}", config.gemini_model);
    println!("🔄 Max retries: {}", config.max_retries);
    println!("⏱️  Timeout: {}s", config.timeout_seconds);
    println!();

    // Check Gemini availability
    match check_gemini_availability().await {
        Ok(true) => println!("✅ Gemini CLI is available"),
        Ok(false) => {
            eprintln!("❌ Gemini CLI is not available. Please install and configure it.");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("❌ Error checking Gemini availability: {}", e);
            process::exit(1);
        }
    }

    println!();
    println!("🧪 Running Complete E2E Test Suite...");
    println!();

    // Run the complete test suite
    match runner.run_complete_test_suite().await {
        Ok(results) => {
            println!();
            println!("🎉 E2E Test Suite Completed!");
            println!("{}", "=".repeat(60));

            // Print summary
            print_test_summary(&results);

            // Check if all tests passed
            let all_passed = results.iter().all(|r| r.overall_success);
            if all_passed {
                println!("✅ ALL TESTS PASSED - Prompts are well calibrated!");
                process::exit(0);
            } else {
                println!("❌ SOME TESTS FAILED - Prompts need calibration!");
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ E2E Test Suite failed: {}", e);
            process::exit(1);
        }
    }
}

/// Parse configuration from command line arguments
fn parse_config(args: &[String]) -> E2ETestConfig {
    let mut config = E2ETestConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--test-dir" => {
                if i + 1 < args.len() {
                    config.test_app_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("❌ --test-dir requires a value");
                    process::exit(1);
                }
            }
            "--max-retries" => {
                if i + 1 < args.len() {
                    config.max_retries = args[i + 1].parse().unwrap_or(3);
                    i += 2;
                } else {
                    eprintln!("❌ --max-retries requires a value");
                    process::exit(1);
                }
            }
            "--timeout" => {
                if i + 1 < args.len() {
                    config.timeout_seconds = args[i + 1].parse().unwrap_or(300);
                    i += 2;
                } else {
                    eprintln!("❌ --timeout requires a value");
                    process::exit(1);
                }
            }
            "--model" => {
                if i + 1 < args.len() {
                    config.gemini_model = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("❌ --model requires a value");
                    process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            _ => {
                eprintln!("❌ Unknown argument: {}", args[i]);
                print_help();
                process::exit(1);
            }
        }
    }

    config
}

/// Print help message
fn print_help() {
    println!("E2E Prompt Calibration Test Runner");
    println!();
    println!("USAGE:");
    println!("    cargo run --bin e2e_prompt_calibration [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --test-dir <DIR>      Directory for test applications (default: testing_apps)");
    println!("    --max-retries <NUM>   Maximum number of retries per stage (default: 3)");
    println!("    --timeout <SECONDS>   Timeout for each operation (default: 300)");
    println!("    --model <MODEL>       Gemini model to use (default: gemini-2.5-flash)");
    println!("    --help, -h            Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    cargo run --bin e2e_prompt_calibration");
    println!("    cargo run --bin e2e_prompt_calibration --test-dir my_tests --max-retries 5");
    println!("    cargo run --bin e2e_prompt_calibration --model gemini-2.5-pro --timeout 600");
}

/// Check if Gemini CLI is available
async fn check_gemini_availability() -> Result<bool, Box<dyn std::error::Error>> {
    use orchy::mcp::gemini::GeminiCLI;
    Ok(GeminiCLI::is_available().await)
}

/// Print test summary
fn print_test_summary(results: &[orchy::e2e_tests::PipelineResult]) {
    println!("📊 TEST SUMMARY");
    println!("{}", "-".repeat(40));

    for result in results {
        println!();
        println!("🔧 {} Pipeline:", result.pipeline_type.to_uppercase());
        println!("   App: {}", result.app_name);
        println!("   Status: {}", if result.overall_success { "✅ PASS" } else { "❌ FAIL" });
        println!("   Duration: {}ms", result.total_duration_ms);
        
        println!("   Stages:");
        for stage in &result.stages {
            let status_icon = if stage.success { "✅" } else { "❌" };
            println!("     {} {} ({}ms, {} retries)", 
                status_icon, stage.stage_name, stage.duration_ms, stage.retry_count);
            
            if !stage.errors.is_empty() {
                println!("       Errors: {}", stage.errors.len());
            }
        }

        println!("   Final App Status:");
        let status = &result.final_app_status;
        println!("     Builds: {}", if status.builds_successfully { "✅" } else { "❌" });
        println!("     Tests: {}", if status.tests_pass { "✅" } else { "❌" });
        println!("     Deploy Ready: {}", if status.deployment_ready { "✅" } else { "❌" });
        println!("     Functionality: {}", if status.functionality_works { "✅" } else { "❌" });
        println!("     Performance: {}", if status.performance_acceptable { "✅" } else { "❌" });
    }

    println!();
    println!("{}", "-".repeat(40));
    
    let total_pipelines = results.len();
    let successful_pipelines = results.iter().filter(|r| r.overall_success).count();
    let total_stages = results.iter().map(|r| r.stages.len()).sum::<usize>();
    let successful_stages = results.iter()
        .flat_map(|r| &r.stages)
        .filter(|s| s.success)
        .count();

    println!("📈 OVERALL STATISTICS:");
    println!("   Pipelines: {}/{} successful ({:.1}%)", 
        successful_pipelines, total_pipelines, 
        (successful_pipelines as f64 / total_pipelines as f64) * 100.0);
    println!("   Stages: {}/{} successful ({:.1}%)", 
        successful_stages, total_stages,
        (successful_stages as f64 / total_stages as f64) * 100.0);

    let total_duration: u128 = results.iter().map(|r| r.total_duration_ms).sum();
    println!("   Total Duration: {}ms ({:.1}s)", total_duration, total_duration as f64 / 1000.0);

    if successful_pipelines == total_pipelines {
        println!();
        println!("🎉 ALL PIPELINES SUCCESSFUL!");
        println!("   The prompts are well-calibrated and can handle:");
        println!("   ✅ Frontend development (Vue.js login page)");
        println!("   ✅ Backend development (GraphQL server)");
        println!("   ✅ Error recovery and edge cases");
        println!("   ✅ Complete development lifecycle");
    } else {
        println!();
        println!("⚠️  SOME PIPELINES FAILED!");
        println!("   Prompt calibration needed for:");
        
        for result in results {
            if !result.overall_success {
                println!("   ❌ {} pipeline", result.pipeline_type);
                for stage in &result.stages {
                    if !stage.success {
                        println!("      - {} stage", stage.stage_name);
                    }
                }
            }
        }
    }
}
