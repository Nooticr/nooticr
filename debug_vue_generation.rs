use orchy::utils::cli_handlers::handle_create_project;
use clap::{Arg, ArgMatches, Command};
use std::path::PathBuf;
use tempfile::TempDir;
use tracing::debug;
use std::fs;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    debug!("🧪 Debug Vue.js Generation");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().join("debug-vue");
    
    let args = create_test_args(
        "debug-vue",
        "Create a minimal Vue.js application to debug file generation",
        project_path.to_str().unwrap(),
        "vue"
    );
    
    debug!("🚀 Creating Vue.js project...");
    match handle_create_project(&args).await {
        Ok(_) => {
            debug!("✅ Project creation completed");
            
            // Examine generated files
            debug!("🔍 Examining generated files...");
            
            let files_to_check = vec![
                "src/App.vue",
                "src/components/TodoList.vue",
                "src/main.js",
                "package.json",
                "vite.config.js"
            ];
            
            for file in &files_to_check {
                let file_path = project_path.join(file);
                if file_path.exists() {
                    debug!("📄 Found: {}", file);
                    if let Ok(content) = fs::read_to_string(&file_path) {
                        debug!("   Content ({} characters):", content.len());
                        debug!("   {}", "=".repeat(50));
                        debug!("{}", content);
                        debug!("   {}", "=".repeat(50));
                    } else {
                        debug!("   ❌ Could not read file content");
                    }
                } else {
                    debug!("❌ Missing: {}", file);
                }
                debug!(""); // Empty line for separation
            }
            
            // List all files in project
            debug!("📋 Project directory contents:");
            fn list_files_recursive(dir: &PathBuf, prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            debug!("{}📁 {}/", prefix, entry.file_name().to_string_lossy());
                            list_files_recursive(&path, &format!("{}  ", prefix))?;
                        } else {
                            let size = fs::metadata(&path)?.len();
                            debug!("{}📄 {} ({} bytes)", prefix, entry.file_name().to_string_lossy(), size);
                        }
                    }
                }
                Ok(())
            }
            
            list_files_recursive(&project_path, "")?;
            
        },
        Err(e) => {
            debug!("❌ Project creation failed: {}", e);
        }
    }
    
    debug!("🎉 Debug complete!");
    Ok(())
}