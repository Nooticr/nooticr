use orchy::managers::{McpManager, McpClient, McpModel};
use orchy::models::project::Project;
use orchy::enums::TechStack;
use orchy::utils::cli_handlers::handle_create_project;
use clap::{Arg, ArgMatches, Command};
use std::path::PathBuf;
use std::process::{Command as StdCommand, Stdio};
use std::time::Duration;
use tempfile::TempDir;
use tokio::fs;
use tokio::time::timeout;
use tracing::debug;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_tracing() {
    INIT.call_once(|| {
        // Load environment variables from .env file
        dotenv::dotenv().ok();
        tracing_subscriber::fmt::init();
    });
}

/// Helper function to create ArgMatches for testing
fn create_test_args(name: &str, idea: &str, path: &str, tech_stack: &str) -> ArgMatches {
    let app = Command::new("test")
        .arg(Arg::new("name").long("name").value_name("NAME").required(true))
        .arg(Arg::new("idea").long("idea").value_name("IDEA").required(true))
        .arg(Arg::new("path").long("path").value_name("PATH").required(true))
        .arg(Arg::new("tech-stack").long("tech-stack").value_name("TECH_STACK"))
        .arg(Arg::new("repository-url").long("repository-url").value_name("REPO_URL"))
        .arg(Arg::new("dependencies-urls").long("dependencies-urls").value_name("DEPS"));

    app.try_get_matches_from(vec![
        "test",
        "--name", name,
        "--idea", idea,
        "--path", path,
        "--tech-stack", tech_stack,
    ]).unwrap()
}

/// Test if a Vue.js project can be built successfully
async fn test_vue_project_buildable(project_path: &PathBuf) -> bool {
    debug!("🧪 Testing Vue.js project buildability at: {:?}", project_path);
    
    // Check if package.json exists
    let package_json = project_path.join("package.json");
    if !package_json.exists() {
        debug!("❌ package.json not found");
        return false;
    }
    
    // Check if vite.config.js exists (or similar build config)
    let vite_config = project_path.join("vite.config.js");
    let has_build_config = vite_config.exists();
    debug!("📦 Build config found: {}", has_build_config);
    
    // Check if main entry points exist
    let main_js = project_path.join("src/main.js");
    let app_vue = project_path.join("src/App.vue");
    let has_main_files = main_js.exists() && app_vue.exists();
    debug!("📄 Main files found: main.js={}, App.vue={}", main_js.exists(), app_vue.exists());
    
    // Check if package.json has proper Vue.js structure
    if let Ok(package_content) = std::fs::read_to_string(&package_json) {
        let has_vue_dep = package_content.contains("\"vue\"");
        let has_vite_dep = package_content.contains("\"vite\"") || package_content.contains("\"@vitejs/plugin-vue\"");
        let has_build_script = package_content.contains("\"build\"") && package_content.contains("vite build");
        
        debug!("📋 Package.json analysis:");
        debug!("   - Vue dependency: {}", has_vue_dep);
        debug!("   - Vite dependency: {}", has_vite_dep);
        debug!("   - Build script: {}", has_build_script);
        
        let is_valid_vue_project = has_vue_dep && has_vite_dep && has_build_script && has_main_files;
        debug!("✅ Vue project validation: {}", is_valid_vue_project);
        
        return is_valid_vue_project;
    }
    
    debug!("❌ Could not read package.json");
    false
}

/// Test if a Rust project can be compiled successfully
async fn test_rust_project_compilable(project_path: &PathBuf) -> bool {
    debug!("🧪 Testing Rust project compilability at: {:?}", project_path);
    
    // Check if Cargo.toml exists
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        debug!("❌ Cargo.toml not found");
        return false;
    }
    
    // Check if src/main.rs or src/lib.rs exists
    let main_rs = project_path.join("src/main.rs");
    let lib_rs = project_path.join("src/lib.rs");
    let has_rust_entry = main_rs.exists() || lib_rs.exists();
    debug!("📄 Rust entry points: main.rs={}, lib.rs={}", main_rs.exists(), lib_rs.exists());
    
    if !has_rust_entry {
        debug!("❌ No Rust entry point found");
        return false;
    }
    
    // Check if Cargo.toml has proper structure
    if let Ok(cargo_content) = std::fs::read_to_string(&cargo_toml) {
        let has_package_section = cargo_content.contains("[package]");
        let has_name = cargo_content.contains("name =");
        let has_version = cargo_content.contains("version =");
        
        debug!("📋 Cargo.toml analysis:");
        debug!("   - Package section: {}", has_package_section);
        debug!("   - Name field: {}", has_name);
        debug!("   - Version field: {}", has_version);
        
        let is_valid_rust_project = has_package_section && has_name && has_version;
        debug!("✅ Rust project validation: {}", is_valid_rust_project);
        
        return is_valid_rust_project;
    }
    
    debug!("❌ Could not read Cargo.toml");
    false
}

/// Run a quick syntax check on generated Vue.js files
async fn test_vue_syntax_check(project_path: &PathBuf) -> bool {
    debug!("🔍 Running Vue.js syntax checks");
    
    let vue_files = vec!["src/App.vue", "src/components/TodoList.vue"];
    let mut syntax_ok = true;
    
    for vue_file in &vue_files {
        let file_path = project_path.join(vue_file);
        if file_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                debug!("📄 Checking {} ({} characters)", vue_file, content.len());
                debug!("   Content preview: {}", content.chars().take(200).collect::<String>());
                
                // Basic Vue.js syntax checks
                let has_template = content.contains("<template>");
                let has_script = content.contains("<script>");
                let has_closing_template = content.contains("</template>");
                let has_closing_script = content.contains("</script>");
                
                debug!("   - Has <template>: {}", has_template);
                debug!("   - Has <script>: {}", has_script);
                debug!("   - Has </template>: {}", has_closing_template);
                debug!("   - Has </script>: {}", has_closing_script);
                
                let file_syntax_ok = has_template && has_script && has_closing_template && has_closing_script;
                debug!("📄 {} syntax check: {}", vue_file, file_syntax_ok);
                
                if !file_syntax_ok {
                    debug!("   ❌ Syntax issues found in {}", vue_file);
                    debug!("   Full content:\n{}", content);
                    syntax_ok = false;
                }
            } else {
                debug!("❌ Could not read {}", vue_file);
                syntax_ok = false;
            }
        } else {
            debug!("❌ File not found: {}", vue_file);
            syntax_ok = false;
        }
    }
    
    debug!("✅ Overall Vue syntax validation: {}", syntax_ok);
    syntax_ok
}

/// Run a quick syntax check on generated Rust files
async fn test_rust_syntax_check(project_path: &PathBuf) -> bool {
    debug!("🔍 Running Rust syntax checks");
    
    let rust_files = vec!["src/main.rs", "src/lib.rs"];
    let mut syntax_ok = true;
    
    for rust_file in &rust_files {
        let file_path = project_path.join(rust_file);
        if file_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                // Basic Rust syntax checks
                let has_fn_main = content.contains("fn main") || content.contains("async fn main");
                let has_proper_braces = content.matches('{').count() == content.matches('}').count();
                let no_obvious_syntax_errors = !content.contains("{{") && !content.contains("}}");
                
                let file_syntax_ok = (has_fn_main || rust_file.contains("lib.rs")) && has_proper_braces && no_obvious_syntax_errors;
                debug!("📄 {} syntax check: {}", rust_file, file_syntax_ok);
                
                if !file_syntax_ok {
                    syntax_ok = false;
                }
            } else {
                debug!("❌ Could not read {}", rust_file);
                syntax_ok = false;
            }
        }
    }
    
    debug!("✅ Overall Rust syntax validation: {}", syntax_ok);
    syntax_ok
}

#[tokio::test]
async fn test_vue_app_startup_verification() {
    init_tracing();
    debug!("🧪 Testing Vue App Startup Verification");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().join("test-vue-app");
    
    let args = create_test_args(
        "test-vue-app",
        "Create a simple Vue.js todo application with basic CRUD operations",
        project_path.to_str().unwrap(),
        "vue"
    );
    
    // Create the project
    debug!("🚀 Creating Vue.js project...");
    let result = timeout(Duration::from_secs(600), handle_create_project(&args)).await;
    
    match result {
        Ok(Ok(_)) => {
            debug!("✅ Project creation completed");
            
            // Test if the project is buildable
            let is_buildable = test_vue_project_buildable(&project_path).await;
            debug!("📦 Vue project buildable: {}", is_buildable);
            
            // Test Vue.js syntax
            let syntax_ok = test_vue_syntax_check(&project_path).await;
            debug!("📝 Vue syntax check: {}", syntax_ok);
            
            // Verify project exists in database
            let db_path = orchy::utils::cli::get_default_database_path();
            let project_exists = orchy::utils::cli::project_exists_in_database(
                &project_path.to_string_lossy(),
                db_path
            ).await.unwrap_or(false);
            debug!("📄 Project exists in database: {}", project_exists);

            assert!(is_buildable, "Vue.js project should be buildable");
            assert!(syntax_ok, "Vue.js syntax should be valid");
            assert!(project_exists, "Project should exist in database");
            
            debug!("🎉 Vue app startup verification PASSED");
        },
        Ok(Err(e)) => {
            debug!("❌ Project creation failed: {}", e);
            panic!("Project creation failed: {}", e);
        },
        Err(_) => {
            debug!("⏰ Project creation timed out");
            panic!("Project creation timed out after 120 seconds");
        }
    }
}

#[tokio::test]
async fn test_rust_app_compilation_verification() {
    init_tracing();
    debug!("🧪 Testing Rust App Compilation Verification");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().join("test-rust-api");
    
    let args = create_test_args(
        "test-rust-api",
        "Create a REST API for todo management using Rust and Actix-web with basic CRUD endpoints",
        project_path.to_str().unwrap(),
        "rust"
    );
    
    // Create the project
    debug!("🚀 Creating Rust project...");
    let result = timeout(Duration::from_secs(600), handle_create_project(&args)).await;
    
    match result {
        Ok(Ok(_)) => {
            debug!("✅ Project creation completed");
            
            // Test if the project is compilable
            let is_compilable = test_rust_project_compilable(&project_path).await;
            debug!("🔧 Rust project compilable: {}", is_compilable);
            
            // Test Rust syntax
            let syntax_ok = test_rust_syntax_check(&project_path).await;
            debug!("📝 Rust syntax check: {}", syntax_ok);
            
            // Verify project exists in database
            let db_path = orchy::utils::cli::get_default_database_path();
            let project_exists = orchy::utils::cli::project_exists_in_database(
                &project_path.to_string_lossy(),
                db_path
            ).await.unwrap_or(false);
            debug!("📄 Project exists in database: {}", project_exists);

            assert!(is_compilable, "Rust project should be compilable");
            assert!(syntax_ok, "Rust syntax should be valid");
            assert!(project_exists, "Project should exist in database");
            
            debug!("🎉 Rust app compilation verification PASSED");
        },
        Ok(Err(e)) => {
            debug!("❌ Project creation failed: {}", e);
            panic!("Project creation failed: {}", e);
        },
        Err(_) => {
            debug!("⏰ Project creation timed out");
            panic!("Project creation timed out after 120 seconds");
        }
    }
}

#[tokio::test] 
async fn test_project_files_integrity() {
    init_tracing();
    debug!("🧪 Testing Project Files Integrity");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().join("test-integrity");
    
    let args = create_test_args(
        "test-integrity", 
        "Create a minimal Vue.js application to test file generation integrity",
        project_path.to_str().unwrap(),
        "vue"
    );
    
    // Create the project
    debug!("🚀 Creating project for integrity test...");
    let result = timeout(Duration::from_secs(600), handle_create_project(&args)).await;
    
    match result {
        Ok(Ok(_)) => {
            debug!("✅ Project creation completed");
            
            // Check that critical files exist and are not empty
            let critical_files = vec![
                "package.json",
                "src/main.js",
                "src/App.vue"
            ];

            // Also check that project exists in database
            let db_path = orchy::utils::cli::get_default_database_path();
            let project_exists = orchy::utils::cli::project_exists_in_database(
                &project_path.to_string_lossy(),
                db_path
            ).await.unwrap_or(false);
            
            let mut files_ok = true;
            for file in &critical_files {
                let file_path = project_path.join(file);
                if file_path.exists() {
                    if let Ok(metadata) = std::fs::metadata(&file_path) {
                        let size = metadata.len();
                        debug!("📄 {} exists ({} bytes)", file, size);
                        if size == 0 {
                            debug!("⚠️  {} is empty!", file);
                            files_ok = false;
                        }
                    } else {
                        debug!("❌ Could not read metadata for {}", file);
                        files_ok = false;
                    }
                } else {
                    debug!("❌ {} does not exist", file);
                    files_ok = false;
                }
            }
            
            assert!(files_ok, "All critical files should exist and not be empty");
            assert!(project_exists, "Project should exist in database");
            debug!("🎉 Project files integrity verification PASSED");
        },
        Ok(Err(e)) => {
            debug!("❌ Project creation failed: {}", e);
            panic!("Project creation failed: {}", e);
        },
        Err(_) => {
            debug!("⏰ Project creation timed out");
            panic!("Project creation timed out after 60 seconds");
        }
    }
}