use super::agent::Agent;
use crate::enums::{CommentType, Priority, TaskStatus};
#[cfg(test)]
use crate::enums::CodeStatus;
use crate::error::{OrchestratorError, Result};
use crate::models::comment::Comment;
use crate::models::pull_request::PullRequest;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Simplified Task structure for deserializing prompt outputs
/// This matches the format expected by idea_breakdown_user_prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInput {
    pub id: String, // Will be replaced with new UUID
    pub title: String,
    pub description: String,
    pub priority: String, // "High/Medium/Low/Critical" - will be parsed to Priority enum
    pub complexity: u8, // 1-10 scale
    pub agent_type: Option<String>,
    pub tags: Vec<String>,
    pub depends_on: Vec<String>, // Task IDs that will be mapped to UUIDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: Uuid,
    pub title: String,
    pub status: TaskStatus,
    pub assigned_to: Option<String>,
    pub priority: Priority,
    pub is_overdue: bool,
    pub comment_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub status_history: Vec<(TaskStatus, chrono::DateTime<chrono::Utc>)>,
    pub rapporter: Option<Agent>,
    pub assigned_to: Option<Agent>,
    pub priority: Priority,
    pub estimated_complexity: Option<u8>, // 1-10 scale
    pub estimated_duration: Option<u32>,  // in minutes
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub comments: Vec<Comment>,
    pub depends_on: Vec<Uuid>,
    pub pull_request: Option<PullRequest>,
}

impl Task {
    /// Create a new task
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        priority: Priority,
    ) -> Self {
        let now = Utc::now();
        let task_status = TaskStatus::default();

        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            status: task_status.clone(),
            status_history: vec![(task_status, now)],
            rapporter: None,
            assigned_to: None,
            priority,
            estimated_complexity: None,
            estimated_duration: None,
            created_at: Some(now),
            updated_at: Some(now),
            completed_at: None,
            due_date: None,
            tags: Vec::new(),
            comments: Vec::new(),
            depends_on: Vec::new(),
            pull_request: None,
        }
    }

    /// Parse priority string to Priority enum
    fn parse_priority(priority_str: &str) -> Priority {
        match priority_str.to_lowercase().as_str() {
            "critical" => Priority::Critical,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::Medium, // Default to Medium for unknown values
        }
    }

    /// Create a Task from a TaskInput (from prompt output)
    ///
    /// This converts the simplified TaskInput structure from idea_breakdown_user_prompt
    /// into a full Task with proper UUIDs and default values.
    ///
    /// # Arguments
    /// * `input` - The TaskInput from JSON deserialization
    /// * `id_mapping` - Optional mapping from old string IDs to new UUIDs for dependency resolution
    pub fn from_input(
        input: TaskInput,
        id_mapping: Option<&std::collections::HashMap<String, Uuid>>,
    ) -> Self {
        let now = Utc::now();
        let task_status = TaskStatus::default();
        let priority = Self::parse_priority(&input.priority);

        // Map dependency IDs to UUIDs if mapping is provided
        let depends_on = if let Some(mapping) = id_mapping {
            input.depends_on
                .iter()
                .filter_map(|dep_id| mapping.get(dep_id).copied())
                .collect()
        } else {
            Vec::new() // No dependencies if no mapping provided
        };

        Self {
            id: Uuid::new_v4(),
            title: input.title,
            description: input.description,
            status: task_status.clone(),
            status_history: vec![(task_status, now)],
            rapporter: None,
            assigned_to: None,
            priority,
            estimated_complexity: Some(input.complexity),
            estimated_duration: None,
            created_at: Some(now),
            updated_at: Some(now),
            completed_at: None,
            due_date: None,
            tags: input.tags,
            comments: Vec::new(),
            depends_on,
            pull_request: None,
        }
    }

    /// Deserialize a JSON array from idea_breakdown_user_prompt output
    ///
    /// This parses the JSON output from the idea breakdown prompt and creates
    /// a vector of Tasks with proper dependency relationships.
    ///
    /// # Example
    /// ```rust
    /// use orchy::models::task::Task;
    ///
    /// let json = r#"[
    ///     {
    ///         "id": "task-1",
    ///         "title": "Setup Database",
    ///         "description": "Create database schema",
    ///         "priority": "High",
    ///         "complexity": 5,
    ///         "tags": ["backend", "database"],
    ///         "depends_on": []
    ///     },
    ///     {
    ///         "id": "task-2",
    ///         "title": "Create API",
    ///         "description": "Build REST API",
    ///         "priority": "Medium",
    ///         "complexity": 7,
    ///         "tags": ["backend", "api"],
    ///         "depends_on": ["task-1"]
    ///     }
    /// ]"#;
    ///
    /// let tasks = Task::from_json_array(json).unwrap();
    /// assert_eq!(tasks.len(), 2);
    /// assert_eq!(tasks[1].depends_on.len(), 1); // task-2 depends on task-1
    /// ```
    pub fn from_json_array(json_str: &str) -> std::result::Result<Vec<Self>, serde_json::Error> {
        let inputs: Vec<TaskInput> = serde_json::from_str(json_str)?;

        // First pass: create ID mapping from old string IDs to new UUIDs
        let mut id_mapping = std::collections::HashMap::new();
        for input in &inputs {
            id_mapping.insert(input.id.clone(), Uuid::new_v4());
        }

        // Second pass: create tasks with proper dependencies
        let tasks = inputs
            .into_iter()
            .map(|input| {
                let mut task = Self::from_input(input.clone(), Some(&id_mapping));
                // Use the pre-generated UUID for this task
                if let Some(&uuid) = id_mapping.get(&input.id) {
                    task.id = uuid;
                }
                task
            })
            .collect();

        Ok(tasks)
    }

    /// Parse JSON and create tasks with automatic dependency resolution
    ///
    /// This is a convenience method that combines JSON parsing with dependency resolution.
    /// It's particularly useful for processing the output from idea_breakdown_user_prompt.
    ///
    /// # Example
    /// ```rust
    /// use orchy::models::task::Task;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let json_output = r#"[
    ///     {
    ///         "id": "setup",
    ///         "title": "Project Setup",
    ///         "description": "Initialize project structure",
    ///         "priority": "High",
    ///         "complexity": 3,
    ///         "tags": ["setup"],
    ///         "depends_on": []
    ///     },
    ///     {
    ///         "id": "development",
    ///         "title": "Core Development",
    ///         "description": "Implement core features",
    ///         "priority": "Medium",
    ///         "complexity": 8,
    ///         "tags": ["development"],
    ///         "depends_on": ["setup"]
    ///     }
    /// ]"#;
    ///
    /// let tasks = Task::parse_idea_breakdown(json_output)?;
    /// assert_eq!(tasks.len(), 2);
    /// assert_eq!(tasks[1].depends_on.len(), 1); // development depends on setup
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_idea_breakdown(json_str: &str) -> std::result::Result<Vec<Self>, Box<dyn std::error::Error>> {
        let tasks = Self::from_json_array(json_str)?;
        Ok(tasks)
    }

    /// Transition task status with validation
    pub fn transition_task_status(&mut self, next_status: TaskStatus) -> Result<()> {
        // Validate the transition
        let new_status = self.status.transition_to(next_status)?;

        // Apply business rules and constraints
        match (&self.status, &new_status) {
            // Can only start InProgress if assigned
            (TaskStatus::Pending, TaskStatus::InProgress) if self.assigned_to.is_none() => {
                return Err(OrchestratorError::validation(
                    "Task must be assigned before starting",
                ));
            }
            // Can only complete if code is merged (check PR status if exists)
            (TaskStatus::Testing, TaskStatus::Completed) => {
                if let Some(pr) = &self.pull_request {
                    if !pr.is_merged() {
                        return Err(OrchestratorError::validation(
                            "Pull request must be merged before completing task",
                        ));
                    }
                }
            }
            _ => {}
        }

        // Update status and history
        let now = Utc::now();
        self.status = new_status.clone();
        self.status_history.push((new_status.clone(), now));
        self.updated_at = Some(now);

        // Handle completion
        if new_status == TaskStatus::Completed {
            self.completed_at = Some(now);
        }

        Ok(())
    }



    /// Add a comment to the task
    pub fn add_comment(&mut self, author: impl Into<String>, content: impl Into<String>) {
        let comment = Comment::new(author, content, CommentType::Task);
        self.comments.push(comment);
        self.updated_at = Some(Utc::now());
    }

    /// Add a comment to the task with sync status
    pub fn add_comment_with_sync(
        &mut self,
        author: impl Into<String>,
        content: impl Into<String>,
        synced: bool,
    ) {
        let comment = Comment::new_with_sync(author, content, CommentType::Task, synced);
        self.comments.push(comment);
        self.updated_at = Some(Utc::now());
    }

    /// Get all unsynced comments
    pub fn get_unsynced_comments(&self) -> Vec<&Comment> {
        self.comments.iter().filter(|c| c.needs_sync()).collect()
    }

    /// Mark all comments as synced
    pub fn mark_all_comments_synced(&mut self) {
        for comment in &mut self.comments {
            comment.mark_synced();
        }
        self.updated_at = Some(Utc::now());
    }

    /// Get comment by ID
    pub fn get_comment(&self, comment_id: Uuid) -> Option<&Comment> {
        self.comments.iter().find(|c| c.id == comment_id)
    }

    /// Get mutable comment by ID
    pub fn get_comment_mut(&mut self, comment_id: Uuid) -> Option<&mut Comment> {
        self.comments.iter_mut().find(|c| c.id == comment_id)
    }

    /// Update a comment's content
    pub fn update_comment(
        &mut self,
        comment_id: Uuid,
        new_content: impl Into<String>,
    ) -> Result<()> {
        let comment = self
            .get_comment_mut(comment_id)
            .ok_or_else(|| OrchestratorError::validation("Comment not found"))?;

        comment.update_content(new_content);
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// Remove a comment
    pub fn remove_comment(&mut self, comment_id: Uuid) -> Result<Comment> {
        let position = self
            .comments
            .iter()
            .position(|c| c.id == comment_id)
            .ok_or_else(|| OrchestratorError::validation("Comment not found"))?;

        let removed_comment = self.comments.remove(position);
        self.updated_at = Some(Utc::now());
        Ok(removed_comment)
    }

    // ===== PULL REQUEST MANAGEMENT =====

    /// Create a pull request for this task
    pub fn create_pull_request(
        &mut self,
        title: impl Into<String>,
        description: impl Into<String>,
        source_branch: impl Into<String>,
        target_branch: impl Into<String>,
        author: impl Into<String>,
    ) -> Result<()> {
        if self.status != TaskStatus::InProgress {
            return Err(OrchestratorError::validation(
                "Can only create pull request when task is in progress",
            ));
        }

        if self.pull_request.is_some() {
            return Err(OrchestratorError::validation(
                "Task already has a pull request",
            ));
        }

        let pr = PullRequest::new(title, description, source_branch, target_branch, author);
        self.pull_request = Some(pr);
        self.updated_at = Some(Utc::now());

        Ok(())
    }

    /// Get the pull request if it exists
    pub fn get_pull_request(&self) -> Option<&PullRequest> {
        self.pull_request.as_ref()
    }

    /// Get mutable pull request if it exists
    pub fn get_pull_request_mut(&mut self) -> Option<&mut PullRequest> {
        self.pull_request.as_mut()
    }

    /// Add a comment to the pull request
    pub fn add_pr_comment(
        &mut self,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<()> {
        let pr = self
            .pull_request
            .as_mut()
            .ok_or_else(|| OrchestratorError::validation("Task has no pull request"))?;

        pr.add_comment(author, content);
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// Handle CI result for the pull request
    pub fn handle_pr_ci_result(&mut self, success: bool) -> Result<()> {
        let pr = self
            .pull_request
            .as_mut()
            .ok_or_else(|| OrchestratorError::validation("Task has no pull request"))?;

        pr.handle_ci_result(success)?;
        self.updated_at = Some(Utc::now());

        Ok(())
    }

    /// Get all pull request comments
    pub fn get_pr_comments(&self) -> Vec<&Comment> {
        if let Some(pr) = &self.pull_request {
            pr.comments.iter().collect()
        } else {
            Vec::new()
        }
    }

    /// Get comments by type (includes both task and PR comments)
    pub fn get_comments_by_type(&self, comment_type: CommentType) -> Vec<&Comment> {
        let mut comments = self
            .comments
            .iter()
            .filter(|c| c.comment_type == comment_type)
            .collect::<Vec<_>>();

        if comment_type == CommentType::PullRequest {
            if let Some(pr) = &self.pull_request {
                comments.extend(pr.comments.iter());
            }
        }

        comments
    }

    /// Get all comments (task + PR comments)
    pub fn get_all_comments(&self) -> Vec<&Comment> {
        let mut comments: Vec<&Comment> = self.comments.iter().collect();

        if let Some(pr) = &self.pull_request {
            comments.extend(pr.comments.iter());
        }

        comments
    }

    /// Assign task to an agent
    pub fn assign_to(&mut self, agent: Agent) -> Result<()> {
        if self.status.is_terminal() {
            return Err(OrchestratorError::validation(
                "Cannot assign completed or cancelled tasks",
            ));
        }

        self.assigned_to = Some(agent);
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// Assign task to an agent by name (convenience method)
    pub fn assign_to_by_name(&mut self, agent_name: impl Into<String>) -> Result<()> {
        if self.status.is_terminal() {
            return Err(OrchestratorError::validation(
                "Cannot assign completed or cancelled tasks",
            ));
        }

        // Create a minimal agent with just the name for assignment
        let agent = Agent::new(
            &agent_name.into(),
            std::path::PathBuf::from("/tmp/placeholder.json"),
            "Assigned agent",
        );

        self.assigned_to = Some(agent);
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// Check if task is overdue
    pub fn is_overdue(&self) -> bool {
        match self.due_date {
            Some(due) => Utc::now() > due && !self.status.is_terminal(),
            None => false,
        }
    }

    /// Get task duration in minutes
    pub fn get_duration_minutes(&self) -> Option<i64> {
        match (self.created_at, self.completed_at) {
            (Some(start), Some(end)) => Some((end - start).num_minutes()),
            _ => None,
        }
    }

    /// Get time in current status
    pub fn time_in_current_status(&self) -> chrono::Duration {
        if let Some((_, timestamp)) = self.status_history.last() {
            Utc::now() - *timestamp
        } else {
            chrono::Duration::zero()
        }
    }

    /// Get a summary of the task
    pub fn summary(&self) -> TaskSummary {
        TaskSummary {
            id: self.id,
            title: self.title.clone(),
            status: self.status.clone(),
            assigned_to: self.assigned_to.as_ref().map(|agent| agent.name.clone()),
            priority: self.priority.clone(),
            is_overdue: self.is_overdue(),
            comment_count: self.comments.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_comment_management() {
        let mut task = Task::new("Test Task", "Description", Priority::Medium);

        // Add regular comment
        task.add_comment("user1", "Regular comment");
        assert_eq!(task.comments.len(), 1);
        assert_eq!(task.comments[0].comment_type, CommentType::Task);
        assert!(task.comments[0].needs_sync());

        // Add synced comment
        task.add_comment_with_sync("user2", "Synced comment", true);
        assert_eq!(task.comments.len(), 2);
        assert!(!task.comments[1].needs_sync());

        // Test unsynced comments
        let unsynced = task.get_unsynced_comments();
        assert_eq!(unsynced.len(), 1);

        // Mark all as synced
        task.mark_all_comments_synced();
        assert_eq!(task.get_unsynced_comments().len(), 0);
    }

    #[test]
    fn test_task_pull_request_management() {
        let mut task = Task::new("Test Task", "Description", Priority::Medium);

        // Should fail to add PR comment when no PR exists
        assert!(task.add_pr_comment("user1", "PR comment").is_err());

        // Assign task to an agent first
        use crate::models::agent::Agent;
        let agent = Agent::new(
            "Test Agent",
            std::path::PathBuf::from("/tmp/test.json"),
            "Test agent",
        );
        task.assigned_to = Some(agent);

        // Transition task to InProgress first
        task.transition_task_status(TaskStatus::InProgress).unwrap();

        // Create a pull request
        assert!(
            task.create_pull_request(
                "Fix: Test Task",
                "This fixes the test task",
                "feature/test-task",
                "main",
                "developer"
            )
            .is_ok()
        );

        // Should now have a pull request and be in PR status
        assert!(task.pull_request.is_some());
        let pr = task.get_pull_request().unwrap();
        assert_eq!(pr.code_status, CodeStatus::PullRequest);

        // Now should be able to add PR comment
        assert!(task.add_pr_comment("user1", "PR comment").is_ok());
        let pr_comments = task.get_pr_comments();
        assert_eq!(pr_comments.len(), 1);
        assert_eq!(pr_comments[0].comment_type, CommentType::PullRequest);

        // Test CI handling
        assert!(task.handle_pr_ci_result(false).is_ok());
        let pr = task.get_pull_request().unwrap();
        assert_eq!(pr.code_status, CodeStatus::CIFailed);
        assert_eq!(pr.ci_attemps, 1);

        // Reset PR to PullRequest status for next CI test (go through Coded first)
        if let Some(pr) = task.get_pull_request_mut() {
            pr.transition_code_status(CodeStatus::Coded).unwrap();
            pr.transition_code_status(CodeStatus::PullRequest).unwrap();
        }

        // Test successful CI
        assert!(task.handle_pr_ci_result(true).is_ok());
        let pr = task.get_pull_request().unwrap();
        assert_eq!(pr.code_status, CodeStatus::CISuccessful);
        assert_eq!(pr.ci_attemps, 2);
    }

    #[test]
    fn test_task_comment_operations() {
        let mut task = Task::new("Test Task", "Description", Priority::Medium);

        task.add_comment("user1", "Original comment");
        let comment_id = task.comments[0].id;

        // Test comment retrieval
        assert!(task.get_comment(comment_id).is_some());
        assert!(task.get_comment_mut(comment_id).is_some());

        // Test comment update
        assert!(task.update_comment(comment_id, "Updated comment").is_ok());
        assert_eq!(
            task.get_comment(comment_id).unwrap().content,
            "Updated comment"
        );

        // Test comment removal
        let removed = task.remove_comment(comment_id).unwrap();
        assert_eq!(removed.content, "Updated comment");
        assert_eq!(task.comments.len(), 0);

        // Test operations on non-existent comment
        let fake_id = Uuid::new_v4();
        assert!(task.update_comment(fake_id, "test").is_err());
        assert!(task.remove_comment(fake_id).is_err());
    }

    #[test]
    fn test_task_from_input() {
        let input = TaskInput {
            id: "task-1".to_string(),
            title: "Setup Database".to_string(),
            description: "Create database schema and tables".to_string(),
            priority: "High".to_string(),
            complexity: 7,
            agent_type: Some("BackendEngineerRust".to_string()),
            tags: vec!["backend".to_string(), "database".to_string()],
            depends_on: vec![],
        };

        let task = Task::from_input(input, None);

        assert_eq!(task.title, "Setup Database");
        assert_eq!(task.description, "Create database schema and tables");
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.estimated_complexity, Some(7));
        assert_eq!(task.tags.len(), 2);
        assert!(task.tags.contains(&"backend".to_string()));
        assert!(task.tags.contains(&"database".to_string()));
        assert_eq!(task.depends_on.len(), 0);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_task_parse_priority() {
        assert_eq!(Task::parse_priority("Critical"), Priority::Critical);
        assert_eq!(Task::parse_priority("High"), Priority::High);
        assert_eq!(Task::parse_priority("Medium"), Priority::Medium);
        assert_eq!(Task::parse_priority("Low"), Priority::Low);
        assert_eq!(Task::parse_priority("critical"), Priority::Critical); // case insensitive
        assert_eq!(Task::parse_priority("unknown"), Priority::Medium); // default
    }

    #[test]
    fn test_task_from_json_array() {
        let json = r#"[
            {
                "id": "task-1",
                "title": "Setup Database",
                "description": "Create database schema",
                "priority": "High",
                "complexity": 5,
                "tags": ["backend", "database"],
                "depends_on": []
            },
            {
                "id": "task-2",
                "title": "Create API",
                "description": "Build REST API endpoints",
                "priority": "Medium",
                "complexity": 7,
                "tags": ["backend", "api"],
                "depends_on": ["task-1"]
            },
            {
                "id": "task-3",
                "title": "Frontend UI",
                "description": "Create user interface",
                "priority": "Low",
                "complexity": 6,
                "tags": ["frontend", "ui"],
                "depends_on": ["task-2"]
            }
        ]"#;

        let tasks = Task::from_json_array(json).unwrap();

        assert_eq!(tasks.len(), 3);

        // Test first task
        assert_eq!(tasks[0].title, "Setup Database");
        assert_eq!(tasks[0].priority, Priority::High);
        assert_eq!(tasks[0].estimated_complexity, Some(5));
        assert_eq!(tasks[0].depends_on.len(), 0);

        // Test second task
        assert_eq!(tasks[1].title, "Create API");
        assert_eq!(tasks[1].priority, Priority::Medium);
        assert_eq!(tasks[1].estimated_complexity, Some(7));
        assert_eq!(tasks[1].depends_on.len(), 1);
        assert_eq!(tasks[1].depends_on[0], tasks[0].id); // Should depend on first task's UUID

        // Test third task
        assert_eq!(tasks[2].title, "Frontend UI");
        assert_eq!(tasks[2].priority, Priority::Low);
        assert_eq!(tasks[2].estimated_complexity, Some(6));
        assert_eq!(tasks[2].depends_on.len(), 1);
        assert_eq!(tasks[2].depends_on[0], tasks[1].id); // Should depend on second task's UUID

        // Verify all tasks have unique UUIDs
        assert_ne!(tasks[0].id, tasks[1].id);
        assert_ne!(tasks[1].id, tasks[2].id);
        assert_ne!(tasks[0].id, tasks[2].id);
    }

    #[test]
    fn test_task_from_json_array_complex_dependencies() {
        let json = r#"[
            {
                "id": "planning",
                "title": "Project Planning",
                "description": "Plan the project architecture",
                "priority": "Critical",
                "complexity": 3,
                "tags": ["planning"],
                "depends_on": []
            },
            {
                "id": "backend",
                "title": "Backend Development",
                "description": "Develop backend services",
                "priority": "High",
                "complexity": 8,
                "tags": ["backend", "api"],
                "depends_on": ["planning"]
            },
            {
                "id": "frontend",
                "title": "Frontend Development",
                "description": "Develop user interface",
                "priority": "High",
                "complexity": 7,
                "tags": ["frontend", "ui"],
                "depends_on": ["planning"]
            },
            {
                "id": "integration",
                "title": "System Integration",
                "description": "Integrate frontend and backend",
                "priority": "Medium",
                "complexity": 5,
                "tags": ["integration", "testing"],
                "depends_on": ["backend", "frontend"]
            }
        ]"#;

        let tasks = Task::from_json_array(json).unwrap();

        assert_eq!(tasks.len(), 4);

        // Find tasks by title for easier testing
        let planning = tasks.iter().find(|t| t.title == "Project Planning").unwrap();
        let backend = tasks.iter().find(|t| t.title == "Backend Development").unwrap();
        let frontend = tasks.iter().find(|t| t.title == "Frontend Development").unwrap();
        let integration = tasks.iter().find(|t| t.title == "System Integration").unwrap();

        // Test dependencies
        assert_eq!(planning.depends_on.len(), 0);
        assert_eq!(backend.depends_on.len(), 1);
        assert!(backend.depends_on.contains(&planning.id));
        assert_eq!(frontend.depends_on.len(), 1);
        assert!(frontend.depends_on.contains(&planning.id));
        assert_eq!(integration.depends_on.len(), 2);
        assert!(integration.depends_on.contains(&backend.id));
        assert!(integration.depends_on.contains(&frontend.id));

        // Test priorities
        assert_eq!(planning.priority, Priority::Critical);
        assert_eq!(backend.priority, Priority::High);
        assert_eq!(frontend.priority, Priority::High);
        assert_eq!(integration.priority, Priority::Medium);
    }

    #[test]
    fn test_task_from_json_array_invalid_json() {
        let invalid_json = r#"[{"invalid": "structure"}]"#;
        let result = Task::from_json_array(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_task_from_json_array_empty() {
        let empty_json = "[]";
        let tasks = Task::from_json_array(empty_json).unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_task_from_input_with_dependencies() {
        let mut id_mapping = std::collections::HashMap::new();
        let dep_uuid = Uuid::new_v4();
        id_mapping.insert("dependency-task".to_string(), dep_uuid);

        let input = TaskInput {
            id: "main-task".to_string(),
            title: "Main Task".to_string(),
            description: "A task with dependencies".to_string(),
            priority: "Medium".to_string(),
            complexity: 4,
            agent_type: Some("DevOps".to_string()),
            tags: vec!["test".to_string()],
            depends_on: vec!["dependency-task".to_string()],
        };

        let task = Task::from_input(input, Some(&id_mapping));

        assert_eq!(task.depends_on.len(), 1);
        assert_eq!(task.depends_on[0], dep_uuid);
    }

    #[test]
    fn test_task_parse_idea_breakdown() {
        let json = r#"[
            {
                "id": "research",
                "title": "Research and Planning",
                "description": "Research requirements and plan architecture",
                "priority": "Critical",
                "complexity": 4,
                "tags": ["planning", "research"],
                "depends_on": []
            },
            {
                "id": "backend",
                "title": "Backend Development",
                "description": "Implement backend services and APIs",
                "priority": "High",
                "complexity": 8,
                "tags": ["backend", "api", "database"],
                "depends_on": ["research"]
            },
            {
                "id": "frontend",
                "title": "Frontend Development",
                "description": "Create user interface and user experience",
                "priority": "High",
                "complexity": 7,
                "tags": ["frontend", "ui", "ux"],
                "depends_on": ["research"]
            },
            {
                "id": "testing",
                "title": "Testing and QA",
                "description": "Comprehensive testing and quality assurance",
                "priority": "Medium",
                "complexity": 5,
                "tags": ["testing", "qa"],
                "depends_on": ["backend", "frontend"]
            }
        ]"#;

        let tasks = Task::parse_idea_breakdown(json).unwrap();

        assert_eq!(tasks.len(), 4);

        // Find tasks by title
        let research = tasks.iter().find(|t| t.title == "Research and Planning").unwrap();
        let backend = tasks.iter().find(|t| t.title == "Backend Development").unwrap();
        let frontend = tasks.iter().find(|t| t.title == "Frontend Development").unwrap();
        let testing = tasks.iter().find(|t| t.title == "Testing and QA").unwrap();

        // Verify dependencies are properly resolved
        assert_eq!(research.depends_on.len(), 0);
        assert_eq!(backend.depends_on.len(), 1);
        assert!(backend.depends_on.contains(&research.id));
        assert_eq!(frontend.depends_on.len(), 1);
        assert!(frontend.depends_on.contains(&research.id));
        assert_eq!(testing.depends_on.len(), 2);
        assert!(testing.depends_on.contains(&backend.id));
        assert!(testing.depends_on.contains(&frontend.id));

        // Verify all tasks have proper attributes
        assert_eq!(research.priority, Priority::Critical);
        assert_eq!(backend.priority, Priority::High);
        assert_eq!(frontend.priority, Priority::High);
        assert_eq!(testing.priority, Priority::Medium);

        // Verify complexity is set
        assert_eq!(research.estimated_complexity, Some(4));
        assert_eq!(backend.estimated_complexity, Some(8));
        assert_eq!(frontend.estimated_complexity, Some(7));
        assert_eq!(testing.estimated_complexity, Some(5));

        // Verify tags are preserved
        assert!(research.tags.contains(&"planning".to_string()));
        assert!(backend.tags.contains(&"api".to_string()));
        assert!(frontend.tags.contains(&"ui".to_string()));
        assert!(testing.tags.contains(&"qa".to_string()));
    }

    #[test]
    fn test_task_parse_idea_breakdown_invalid() {
        let invalid_json = r#"[{"invalid": "structure"}]"#;
        let result = Task::parse_idea_breakdown(invalid_json);
        assert!(result.is_err());
    }
}
