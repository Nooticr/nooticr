use clap::{Arg, Command};
use orchy::utils::*;
use tracing::debug;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();
    
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("create", sub_matches)) => {
            if let Err(e) = handle_create_project(sub_matches).await {
                debug!("Error creating project: {}", e);
                std::process::exit(1);
            }
        }
        Some(("list-tasks", _)) => {
            if let Err(e) = handle_list_tasks().await {
                debug!("Error listing tasks: {}", e);
                std::process::exit(1);
            }
        }
        Some(("list-agents", _)) => {
            if let Err(e) = handle_list_agents().await {
                debug!("Error listing agents: {}", e);
                std::process::exit(1);
            }
        }
        Some(("list-issues", _)) => {
            if let Err(e) = handle_list_issues().await {
                debug!("Error listing issues: {}", e);
                std::process::exit(1);
            }
        }
        Some(("add-sample-data", sub_matches)) => {
            if let Err(e) = handle_add_sample_data(sub_matches).await {
                debug!("Error adding sample data: {}", e);
                std::process::exit(1);
            }
        }
        Some(("tui", _)) | Some(("ui", _)) => {
            if let Err(e) = run_tui().await {
                debug!("Error in TUI mode: {}", e);
                std::process::exit(1);
            }
        }
        Some(("add-agent", sub_matches)) => {
            if let Err(e) = handle_add_agent(sub_matches).await {
                debug!("Error adding agent: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            // Default to TUI mode when no subcommand is provided
            if let Err(e) = run_tui().await {
                debug!("Error in TUI mode: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// Build the CLI application
fn build_cli() -> Command {
    Command::new("orchy")
        .version("0.1.0")
        .author("Orchy Team")
        .about("A powerful project orchestration and management tool")
        .subcommand_required(false)
        .arg_required_else_help(false)
        .subcommand(
            Command::new("create")
                .about("Create a new project")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .help("Project name")
                        .required(true)
                )
                .arg(
                    Arg::new("idea")
                        .short('i')
                        .long("idea")
                        .value_name("IDEA")
                        .help("Project idea/description")
                        .required(true)
                )
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .value_name("PATH")
                        .help("Project path")
                        .required(true)
                )
                .arg(
                    Arg::new("repository-url")
                        .short('r')
                        .long("repository-url")
                        .value_name("URL")
                        .help("Optional repository URL")
                        .required(false)
                )
                .arg(
                    Arg::new("dependencies-urls")
                        .short('d')
                        .long("dependencies-urls")
                        .value_name("URLS")
                        .help("Optional dependency URLs (comma-separated)")
                        .required(false)
                )
                .arg(
                    Arg::new("tech-stack")
                        .short('t')
                        .long("tech-stack")
                        .value_name("TECH_STACK")
                        .help("Technology stack (rust, vue, react, fullstack-rust-vue, fullstack-rust-react)")
                        .required(false)
                )
        )
        .subcommand(
            Command::new("list-tasks")
                .about("List tasks for a project (interactive)")
        )
        .subcommand(
            Command::new("list-agents")
                .about("List all agents")
        )
        .subcommand(
            Command::new("list-issues")
                .about("List issues for a project (interactive)")
        )
        .subcommand(
            Command::new("add-sample-data")
                .about("Add sample tasks and agents to a project (for testing)")
                .arg(
                    Arg::new("project-name")
                        .short('p')
                        .long("project")
                        .value_name("NAME")
                        .help("Project name to add sample data to")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("tui")
                .about("Start modern TUI interface (default when no command given)")
                .alias("ui")
        )
        .subcommand(
            Command::new("add-agent")
                .about("Add a new agent to a project")
                .arg(
                    Arg::new("project-name")
                        .short('p')
                        .long("project")
                        .value_name("NAME")
                        .help("Project name")
                        .required(true)
                )
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .help("Agent name")
                        .required(true)
                )
                .arg(
                    Arg::new("description")
                        .short('d')
                        .long("description")
                        .value_name("DESC")
                        .help("Agent description")
                        .required(true)
                )
                .arg(
                    Arg::new("file-path")
                        .short('f')
                        .long("file-path")
                        .value_name("PATH")
                        .help("Agent configuration file path")
                        .required(true)
                )
        )
}
