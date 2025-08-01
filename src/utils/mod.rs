pub mod cli;
pub mod cli_handlers;
pub mod tui;
pub mod dependency_resolver;

pub use cli::*;
pub use cli_handlers::*;
pub use tui::*;
pub use dependency_resolver::*;



pub fn extract_json_from_response(response: &str) -> String {
    let lines: Vec<&str> = response.lines().collect();
    let mut json_content = String::new();
    let mut in_json_block = false;

    for line in lines {
        if line.trim().starts_with("```json") {
            in_json_block = true;
            continue;
        }
        if line.trim() == "```" && in_json_block {
            break;
        }
        if in_json_block {
            json_content.push_str(line);
            json_content.push('\n');
        }
    }

    // If no JSON block found, try the entire response
    if json_content.trim().is_empty() {
        response.to_string()
    } else {
        json_content
    }
}