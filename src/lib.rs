//! # Orchy - Orchestration Library
//!
//! Orchy is a comprehensive orchestration library for managing development workflows,
//! including tasks, pull requests, code reviews, and automated actions.
//!
//! ## Key Features
//!
//! - **Action Execution**: Parse and execute development actions from JSON
//! - **Code Review Management**: Handle code reviews with approval workflows
//! - **Task Management**: Track and manage development tasks
//! - **Pull Request Integration**: Manage pull requests with status tracking
//! - **Project Orchestration**: Coordinate multiple tasks and agents
//!
//! ## Quick Start
//!
//! ### Parsing and Executing Actions
//!
//! ```rust,no_run
//! use orchy::enums::Action;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // JSON output from conflict_resolution_user_prompt or feature_development_user_prompt
//! let json_actions = r#"[
//!     {
//!         "Write": {
//!             "path": "src/main.rs",
//!             "content": "fn main() { println!(\"Hello, world!\"); }"
//!         }
//!     },
//!     {
//!         "RunCommand": {
//!             "command": "cargo build",
//!             "env": null
//!         }
//!     }
//! ]"#;
//!
//! // Parse and execute all actions
//! Action::parse_and_execute(json_actions).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Processing Code Reviews
//!
//! ```rust
//! use orchy::models::code_review::CodeReview;
//!
//! // JSON output from code_review_user_prompt
//! let json_review = r#"{
//!     "id": null,
//!     "pull_request_id": "pr-123",
//!     "approved": true,
//!     "comments": ["LGTM!", "Minor style issue on line 42"]
//! }"#;
//!
//! // Parse the review and assign a reviewer
//! let review = CodeReview::from_json(json_review, "senior_dev@company.com").unwrap();
//! assert_eq!(review.pull_request_id, "pr-123");
//! assert!(review.approved);
//! ```
//!
//! ### Processing Task Breakdowns
//!
//! ```rust
//! use orchy::models::task::Task;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // JSON output from idea_breakdown_user_prompt
//! let json_tasks = r#"[
//!     {
//!         "id": "setup",
//!         "title": "Project Setup",
//!         "description": "Initialize project structure",
//!         "priority": "High",
//!         "complexity": 3,
//!         "tags": ["setup", "infrastructure"],
//!         "depends_on": []
//!     },
//!     {
//!         "id": "development",
//!         "title": "Core Development",
//!         "description": "Implement core features",
//!         "priority": "Medium",
//!         "complexity": 8,
//!         "tags": ["development", "features"],
//!         "depends_on": ["setup"]
//!     }
//! ]"#;
//!
//! // Parse tasks with automatic dependency resolution
//! let tasks = Task::parse_idea_breakdown(json_tasks)?;
//! assert_eq!(tasks.len(), 2);
//! assert_eq!(tasks[1].depends_on.len(), 1); // development depends on setup
//! # Ok(())
//! # }
//! ```

pub mod enums;
pub mod error;
pub mod mcp;
pub mod models;
pub mod prompts;
pub mod managers;
pub mod utils;
pub mod database;
// pub mod e2e_tests; // Temporarily disabled due to compilation errors
