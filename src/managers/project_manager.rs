use crate::models::project::Project;
use crate::models::task::Task;
use crate::models::agent::Agent;
use crate::models::issue::Issue;
use crate::enums::{TaskStatus, AgentStatus, IssueStatus, IssueType, CommentType};
use crate::error::{OrchestratorError, Result};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, broadcast};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Real-time events that can occur in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectEvent {
    // Task Events
    TaskCreated { task_id: Uuid, task_title: String, assigned_to: Option<String> },
    TaskAssigned { task_id: Uuid, agent_id: Uuid, agent_name: String },
    TaskStatusChanged { task_id: Uuid, old_status: TaskStatus, new_status: TaskStatus },
    TaskCompleted { task_id: Uuid, task_title: String, completion_time: DateTime<Utc> },
    
    // Agent Events
    AgentStartedWorking { agent_id: Uuid, agent_name: String, task_id: Uuid, task_title: String },
    AgentFinishedWorking { agent_id: Uuid, agent_name: String, task_id: Uuid },
    AgentStatusChanged { agent_id: Uuid, agent_name: String, old_status: AgentStatus, new_status: AgentStatus },
    
    // Comment Events
    CommentAdded { comment_id: Uuid, author: String, content: String, comment_type: CommentType, target_id: Uuid },
    CommentSynced { comment_id: Uuid, comment_type: CommentType },
    CommentUpdated { comment_id: Uuid, new_content: String },
    
    // Code Review Events
    CodeReviewAdded { review_id: Uuid, reviewer: String, approved: bool, pr_id: String },
    CodeReviewUpdated { review_id: Uuid, approved: bool },
    
    // Pull Request Events
    PullRequestCreated { pr_id: String, title: String, author: String, task_id: Uuid },
    PullRequestMerged { pr_id: String, task_id: Uuid },
    PullRequestCIStatusChanged { pr_id: String, success: bool, attempt: u32 },
    
    // Issue Events
    IssueCreated { issue_id: Uuid, title: String, issue_type: Option<IssueType> },
    IssueStatusChanged { issue_id: Uuid, old_status: IssueStatus, new_status: IssueStatus },
    IssueSynced { issue_id: Uuid },
    
    // Sync Events
    SyncStarted { sync_type: String, item_count: usize },
    SyncProgress { sync_type: String, completed: usize, total: usize },
    SyncCompleted { sync_type: String, success_count: usize, error_count: usize },
    SyncError { sync_type: String, error: String },
    
    // Project Events
    ProjectStatusChanged { old_status: String, new_status: String },
    ProjectStatisticsUpdated { stats: ProjectStatistics },
}

/// Project statistics for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub in_progress_tasks: usize,
    pub pending_tasks: usize,
    pub total_agents: usize,
    pub active_agents: usize,
    pub total_issues: usize,
    pub open_issues: usize,
    pub completion_percentage: f64,
    pub unsynced_items: usize,
}

/// Commands that can be sent to the project manager
#[derive(Debug)]
pub enum ProjectCommand {
    // Task Management
    CreateTask { task: Task, respond_to: mpsc::UnboundedSender<Result<Uuid>> },
    AssignTask { task_id: Uuid, agent_id: Uuid, respond_to: mpsc::UnboundedSender<Result<()>> },
    UpdateTaskStatus { task_id: Uuid, status: TaskStatus, respond_to: mpsc::UnboundedSender<Result<()>> },
    
    // Agent Management
    AddAgent { agent: Agent, respond_to: mpsc::UnboundedSender<Result<Uuid>> },
    UpdateAgentStatus { agent_id: Uuid, status: AgentStatus, respond_to: mpsc::UnboundedSender<Result<()>> },
    
    // Comment Management
    AddComment { target_id: Uuid, comment_type: CommentType, author: String, content: String, respond_to: mpsc::UnboundedSender<Result<Uuid>> },
    SyncComment { comment_id: Uuid, respond_to: mpsc::UnboundedSender<Result<()>> },
    
    // Code Review Management
    AddCodeReview { pr_id: String, reviewer: String, approved: bool, comments: Vec<String>, respond_to: mpsc::UnboundedSender<Result<Uuid>> },
    
    // Issue Management
    CreateIssue { issue: Issue, respond_to: mpsc::UnboundedSender<Result<Uuid>> },
    UpdateIssueStatus { issue_id: Uuid, status: IssueStatus, respond_to: mpsc::UnboundedSender<Result<()>> },
    
    // Sync Operations
    SyncAll { respond_to: mpsc::UnboundedSender<Result<()>> },
    SyncTasks { respond_to: mpsc::UnboundedSender<Result<()>> },
    SyncIssues { respond_to: mpsc::UnboundedSender<Result<()>> },
    SyncComments { respond_to: mpsc::UnboundedSender<Result<()>> },
    
    // Query Operations
    GetProject { respond_to: mpsc::UnboundedSender<Project> },
    GetStatistics { respond_to: mpsc::UnboundedSender<ProjectStatistics> },
    GetTask { task_id: Uuid, respond_to: mpsc::UnboundedSender<Option<Task>> },
    GetAgent { agent_id: Uuid, respond_to: mpsc::UnboundedSender<Option<Agent>> },
    
    // Shutdown
    Shutdown,
}

/// Project Manager handles all project operations with real-time events
///
/// The ProjectManager provides a high-level interface for managing projects with real-time
/// event broadcasting. It uses tokio channels for communication and supports:
///
/// - Task creation, assignment, and status updates
/// - Agent management and status tracking
/// - Comment management with typed comment types (Task, Issue, PullRequest)
/// - Issue creation and management
/// - Real-time sync operations
/// - Statistics calculation and updates
///
/// # Example
///
/// ```rust
/// use orchy::managers::ProjectManager;
/// use orchy::models::project::Project;
/// use orchy::models::task::Task;
/// use orchy::enums::{Priority, CommentType};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a project
/// let project = Project::new("My Project", "A sample project", "/path/to/project");
/// let manager = ProjectManager::new(project);
///
/// // Subscribe to events
/// let mut events = manager.subscribe_to_events();
///
/// // Create a task
/// let task = Task::new("Implement feature", "Add new functionality", Priority::High);
/// let task_id = manager.create_task(task).await?;
///
/// // Add a comment to the task
/// let comment_id = manager.add_comment(
///     task_id,
///     CommentType::Task,
///     "developer".to_string(),
///     "Starting work on this task".to_string(),
/// ).await?;
///
/// // Listen for events
/// if let Ok(event) = events.recv().await {
///     println!("Received event: {:?}", event);
/// }
///
/// manager.shutdown().await;
/// # Ok(())
/// # }
/// ```
pub struct ProjectManager {
    project: Arc<RwLock<Project>>,
    command_sender: mpsc::UnboundedSender<ProjectCommand>,
    event_broadcaster: broadcast::Sender<ProjectEvent>,
}

impl ProjectManager {
    /// Create a new project manager
    pub fn new(project: Project) -> Self {
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_broadcaster, _) = broadcast::channel(1000);
        
        let project = Arc::new(RwLock::new(project));
        let manager = ProjectManager {
            project: project.clone(),
            command_sender,
            event_broadcaster: event_broadcaster.clone(),
        };
        
        // Spawn the background task to handle commands
        tokio::spawn(Self::command_handler(
            project,
            command_receiver,
            event_broadcaster,
        ));
        
        manager
    }
    
    /// Subscribe to project events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ProjectEvent> {
        self.event_broadcaster.subscribe()
    }
    
    /// Get a sender for project commands
    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<ProjectCommand> {
        self.command_sender.clone()
    }

    /// Create a task with real-time events
    pub async fn create_task(&self, task: Task) -> Result<Uuid> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.command_sender.send(ProjectCommand::CreateTask {
            task,
            respond_to: tx
        }).map_err(|_| OrchestratorError::internal("Failed to send command"))?;

        rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive response"))?
    }

    /// Assign a task to an agent with real-time events
    pub async fn assign_task(&self, task_id: Uuid, agent_id: Uuid) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.command_sender.send(ProjectCommand::AssignTask {
            task_id,
            agent_id,
            respond_to: tx
        }).map_err(|_| OrchestratorError::internal("Failed to send command"))?;

        rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive response"))?
    }

    /// Update task status with real-time events
    pub async fn update_task_status(&self, task_id: Uuid, status: TaskStatus) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.command_sender.send(ProjectCommand::UpdateTaskStatus {
            task_id,
            status,
            respond_to: tx
        }).map_err(|_| OrchestratorError::internal("Failed to send command"))?;

        rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive response"))?
    }

    /// Add an agent with real-time events
    pub async fn add_agent(&self, agent: Agent) -> Result<Uuid> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.command_sender.send(ProjectCommand::AddAgent {
            agent,
            respond_to: tx
        }).map_err(|_| OrchestratorError::internal("Failed to send command"))?;

        rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive response"))?
    }

    /// Create an issue with real-time events
    pub async fn create_issue(&self, issue: Issue) -> Result<Uuid> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.command_sender.send(ProjectCommand::CreateIssue {
            issue,
            respond_to: tx
        }).map_err(|_| OrchestratorError::internal("Failed to send command"))?;

        rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive response"))?
    }

    /// Add a comment with real-time events
    pub async fn add_comment(
        &self,
        target_id: Uuid,
        comment_type: CommentType,
        author: String,
        content: String
    ) -> Result<Uuid> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.command_sender.send(ProjectCommand::AddComment {
            target_id,
            comment_type,
            author,
            content,
            respond_to: tx
        }).map_err(|_| OrchestratorError::internal("Failed to send command"))?;

        rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive response"))?
    }

    /// Get current project statistics
    pub async fn get_statistics(&self) -> ProjectStatistics {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let _ = self.command_sender.send(ProjectCommand::GetStatistics { respond_to: tx });

        rx.recv().await.unwrap_or_else(|| ProjectStatistics {
            total_tasks: 0,
            completed_tasks: 0,
            in_progress_tasks: 0,
            pending_tasks: 0,
            total_agents: 0,
            active_agents: 0,
            total_issues: 0,
            open_issues: 0,
            completion_percentage: 0.0,
            unsynced_items: 0,
        })
    }

    /// Sync all project items
    pub async fn sync_all(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.command_sender.send(ProjectCommand::SyncAll {
            respond_to: tx
        }).map_err(|_| OrchestratorError::internal("Failed to send command"))?;

        rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive response"))?
    }

    /// Shutdown the project manager
    pub async fn shutdown(&self) {
        let _ = self.command_sender.send(ProjectCommand::Shutdown);
    }

    /// Background task to handle commands and emit events
    async fn command_handler(
        project: Arc<RwLock<Project>>,
        mut command_receiver: mpsc::UnboundedReceiver<ProjectCommand>,
        event_broadcaster: broadcast::Sender<ProjectEvent>,
    ) {
        while let Some(command) = command_receiver.recv().await {
            match command {
                ProjectCommand::CreateTask { task, respond_to } => {
                    let result = Self::handle_create_task(
                        &project,
                        task,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::AssignTask { task_id, agent_id, respond_to } => {
                    let result = Self::handle_assign_task(
                        &project,
                        task_id,
                        agent_id,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::UpdateTaskStatus { task_id, status, respond_to } => {
                    let result = Self::handle_update_task_status(
                        &project,
                        task_id,
                        status,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::AddAgent { agent, respond_to } => {
                    let result = Self::handle_add_agent(
                        &project,
                        agent,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::UpdateAgentStatus { agent_id, status, respond_to } => {
                    let result = Self::handle_update_agent_status(
                        &project,
                        agent_id,
                        status,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::AddComment { target_id, comment_type, author, content, respond_to } => {
                    let result = Self::handle_add_comment(
                        &project,
                        target_id,
                        comment_type,
                        author,
                        content,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::SyncAll { respond_to } => {
                    let result = Self::handle_sync_all(
                        &project,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::GetStatistics { respond_to } => {
                    let stats = Self::calculate_statistics(&project).await;
                    let _ = respond_to.send(stats);
                }

                ProjectCommand::CreateIssue { issue, respond_to } => {
                    let result = Self::handle_create_issue(
                        &project,
                        issue,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::GetProject { respond_to } => {
                    let project_clone = project.read().await.clone();
                    let _ = respond_to.send(project_clone);
                }

                ProjectCommand::Shutdown => {
                    break;
                }

                // Handle other commands...
                _ => {
                    // TODO: Implement remaining command handlers
                }
            }
        }
    }

    /// Handle task creation with events
    async fn handle_create_task(
        project: &Arc<RwLock<Project>>,
        task: Task,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<Uuid> {
        let mut project_guard = project.write().await;
        let task_id = task.id;
        let task_title = task.title.clone();
        let assigned_to = task.assigned_to.as_ref().map(|a| a.name.clone());

        project_guard.add_task(task)?;

        // Emit event
        let _ = event_broadcaster.send(ProjectEvent::TaskCreated {
            task_id,
            task_title,
            assigned_to,
        });

        // Update statistics
        let stats = Self::calculate_statistics_from_project(&*project_guard);
        let _ = event_broadcaster.send(ProjectEvent::ProjectStatisticsUpdated { stats });

        Ok(task_id)
    }

    /// Handle task assignment with events
    async fn handle_assign_task(
        project: &Arc<RwLock<Project>>,
        task_id: Uuid,
        agent_id: Uuid,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<()> {
        let mut project_guard = project.write().await;

        // Get agent name for event
        let agent_name = project_guard.get_agent(agent_id)
            .ok_or_else(|| OrchestratorError::validation("Agent not found"))?
            .name.clone();

        project_guard.reassign_task(task_id, agent_id)?;

        // Emit event
        let _ = event_broadcaster.send(ProjectEvent::TaskAssigned {
            task_id,
            agent_id,
            agent_name,
        });

        Ok(())
    }

    /// Handle task status update with events
    async fn handle_update_task_status(
        project: &Arc<RwLock<Project>>,
        task_id: Uuid,
        new_status: TaskStatus,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<()> {
        let mut project_guard = project.write().await;

        let task = project_guard.get_task_mut(task_id)
            .ok_or_else(|| OrchestratorError::validation("Task not found"))?;

        let old_status = task.status.clone();
        task.transition_task_status(new_status.clone())?;

        // Emit status change event
        let _ = event_broadcaster.send(ProjectEvent::TaskStatusChanged {
            task_id,
            old_status,
            new_status: new_status.clone(),
        });

        // If task is completed, emit completion event
        if new_status == TaskStatus::Completed {
            let _ = event_broadcaster.send(ProjectEvent::TaskCompleted {
                task_id,
                task_title: task.title.clone(),
                completion_time: Utc::now(),
            });
        }

        // Update statistics
        let stats = Self::calculate_statistics_from_project(&*project_guard);
        let _ = event_broadcaster.send(ProjectEvent::ProjectStatisticsUpdated { stats });

        Ok(())
    }

    /// Handle agent addition with events
    async fn handle_add_agent(
        project: &Arc<RwLock<Project>>,
        agent: Agent,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<Uuid> {
        let mut project_guard = project.write().await;
        let agent_id = agent.id;

        project_guard.add_agent(agent)?;

        // Update statistics
        let stats = Self::calculate_statistics_from_project(&*project_guard);
        let _ = event_broadcaster.send(ProjectEvent::ProjectStatisticsUpdated { stats });

        Ok(agent_id)
    }

    /// Handle agent status update with events
    async fn handle_update_agent_status(
        project: &Arc<RwLock<Project>>,
        agent_id: Uuid,
        new_status: AgentStatus,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<()> {
        let mut project_guard = project.write().await;

        let agent = project_guard.get_agent_mut(agent_id)
            .ok_or_else(|| OrchestratorError::validation("Agent not found"))?;

        let old_status = agent.status.clone();
        let agent_name = agent.name.clone();
        agent.status = new_status.clone();

        // Emit event
        let _ = event_broadcaster.send(ProjectEvent::AgentStatusChanged {
            agent_id,
            agent_name,
            old_status,
            new_status,
        });

        Ok(())
    }

    /// Handle issue creation with events
    async fn handle_create_issue(
        project: &Arc<RwLock<Project>>,
        issue: Issue,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<Uuid> {
        let mut project_guard = project.write().await;
        let issue_id = issue.id;
        let issue_title = issue.title.clone();
        let issue_type = issue.issue_type.clone();

        project_guard.add_issue(issue)?;

        // Emit event
        let _ = event_broadcaster.send(ProjectEvent::IssueCreated {
            issue_id,
            title: issue_title,
            issue_type,
        });

        // Update statistics
        let stats = Self::calculate_statistics_from_project(&*project_guard);
        let _ = event_broadcaster.send(ProjectEvent::ProjectStatisticsUpdated { stats });

        Ok(issue_id)
    }

    /// Handle comment addition with events
    async fn handle_add_comment(
        project: &Arc<RwLock<Project>>,
        target_id: Uuid,
        comment_type: CommentType,
        author: String,
        content: String,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<Uuid> {
        let mut project_guard = project.write().await;
        let comment_id = Uuid::new_v4();

        // Add comment based on comment type
        match comment_type {
            CommentType::Task => {
                let task = project_guard.get_task_mut(target_id)
                    .ok_or_else(|| OrchestratorError::validation("Task not found"))?;
                task.add_comment(&author, &content);
            }
            CommentType::Issue => {
                let issue = project_guard.get_issue_mut(target_id)
                    .ok_or_else(|| OrchestratorError::validation("Issue not found"))?;
                issue.add_new_comment(&author, &content);
            }
            CommentType::PullRequest => {
                // Find task with the pull request
                let task = project_guard.tasks.iter_mut()
                    .find(|t| t.pull_request.as_ref().map(|pr| pr.id.to_string() == target_id.to_string()).unwrap_or(false))
                    .ok_or_else(|| OrchestratorError::validation("Pull request not found"))?;

                if let Some(pr) = &mut task.pull_request {
                    pr.add_comment(&author, &content);
                } else {
                    return Err(OrchestratorError::validation("Pull request not found"));
                }
            }
        }

        // Emit event
        let _ = event_broadcaster.send(ProjectEvent::CommentAdded {
            comment_id,
            author,
            content,
            comment_type,
            target_id,
        });

        Ok(comment_id)
    }

    /// Handle sync all operation with events
    async fn handle_sync_all(
        project: &Arc<RwLock<Project>>,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<()> {
        let mut project_guard = project.write().await;

        // Count items to sync
        let unsynced_pull_requests = project_guard.get_unsynced_pull_requests().len();
        let unsynced_issues = project_guard.get_unsynced_issues().len();
        let unsynced_comments = project_guard.get_all_unsynced_comments().len();
        let total_items = unsynced_pull_requests + unsynced_issues + unsynced_comments;

        // Emit sync started event
        let _ = event_broadcaster.send(ProjectEvent::SyncStarted {
            sync_type: "all".to_string(),
            item_count: total_items,
        });

        let mut completed = 0;
        let mut errors = 0;

        // Sync pull requests (simulated - actual implementation would sync with remote)
        for _ in 0..unsynced_pull_requests {
            // TODO: Implement actual sync logic
            completed += 1;
            let _ = event_broadcaster.send(ProjectEvent::SyncProgress {
                sync_type: "all".to_string(),
                completed,
                total: total_items,
            });
        }

        // Mark all as synced for now
        project_guard.mark_all_synced();

        // Emit sync completed event
        let _ = event_broadcaster.send(ProjectEvent::SyncCompleted {
            sync_type: "all".to_string(),
            success_count: completed,
            error_count: errors,
        });

        Ok(())
    }

    /// Calculate statistics from project
    async fn calculate_statistics(project: &Arc<RwLock<Project>>) -> ProjectStatistics {
        let project_guard = project.read().await;
        Self::calculate_statistics_from_project(&*project_guard)
    }

    /// Calculate statistics from project reference
    fn calculate_statistics_from_project(project: &Project) -> ProjectStatistics {
        let total_tasks = project.tasks.len();
        let completed_tasks = project.tasks.iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let in_progress_tasks = project.tasks.iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();
        let pending_tasks = project.tasks.iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count();

        let total_agents = project.agents.len();
        let active_agents = project.agents.iter()
            .filter(|a| a.status == AgentStatus::Active)
            .count();

        let total_issues = project.issues.len();
        let open_issues = project.issues.iter()
            .filter(|i| i.status == IssueStatus::Open)
            .count();

        let completion_percentage = if total_tasks > 0 {
            (completed_tasks as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };

        let unsynced_items = project.get_unsynced_pull_requests().len() +
            project.get_unsynced_issues().len() +
            project.get_all_unsynced_comments().len();

        ProjectStatistics {
            total_tasks,
            completed_tasks,
            in_progress_tasks,
            pending_tasks,
            total_agents,
            active_agents,
            total_issues,
            open_issues,
            completion_percentage,
            unsynced_items,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{Priority, CommentType};
    use crate::models::project::ProjectStatus;
    use tokio::time::{timeout, Duration};

    fn create_test_project() -> Project {
        let mut project = Project::new("Test Project", "A test project for the manager", "/test/path");
        // Set project to Active status so tasks can be assigned
        project.transition_to(ProjectStatus::Active).unwrap();
        project
    }

    fn create_test_task(title: &str) -> Task {
        Task::new(title, "Test description", Priority::Medium)
    }

    fn create_test_agent(name: &str) -> Agent {
        Agent::new(name, std::path::PathBuf::from("/tmp/test.json"), "Test agent")
    }

    #[tokio::test]
    async fn test_project_manager_creation() {
        let project = create_test_project();
        let manager = ProjectManager::new(project);

        // Test that we can get a command sender
        let _sender = manager.get_command_sender();

        // Test that we can subscribe to events
        let _receiver = manager.subscribe_to_events();

        manager.shutdown().await;
    }

    #[tokio::test]
    async fn test_task_creation_with_events() {
        let project = create_test_project();
        let manager = ProjectManager::new(project);

        // Subscribe to events
        let mut event_receiver = manager.subscribe_to_events();

        // Create a task
        let task = create_test_task("Test Task");
        let task_id = task.id;
        let result = manager.create_task(task).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), task_id);

        // Check for events
        let event = timeout(Duration::from_millis(100), event_receiver.recv()).await;
        assert!(event.is_ok());

        if let Ok(Ok(ProjectEvent::TaskCreated { task_id: received_id, task_title, assigned_to })) = event {
            assert_eq!(received_id, task_id);
            assert_eq!(task_title, "Test Task");
            assert_eq!(assigned_to, None);
        } else {
            panic!("Expected TaskCreated event");
        }

        manager.shutdown().await;
    }

    #[tokio::test]
    async fn test_task_status_update_with_events() {
        let project = create_test_project();
        let manager = ProjectManager::new(project);

        // Subscribe to events
        let mut event_receiver = manager.subscribe_to_events();

        // Create a task first
        let task = create_test_task("Test Task");
        let task_id = task.id;
        manager.create_task(task).await.unwrap();

        // Create and add an agent
        let agent = create_test_agent("Test Agent");
        let agent_id = agent.id;
        manager.add_agent(agent).await.unwrap();

        // Assign the task to the agent
        manager.assign_task(task_id, agent_id).await.unwrap();

        // Clear the creation events
        let _ = event_receiver.recv().await; // Task created
        let _ = event_receiver.recv().await; // Statistics update
        let _ = event_receiver.recv().await; // Statistics update (agent added)
        let _ = event_receiver.recv().await; // Task assigned

        // Update task status
        let result = manager.update_task_status(task_id, TaskStatus::InProgress).await;
        if let Err(e) = &result {
            println!("Error updating task status: {:?}", e);
        }
        assert!(result.is_ok());

        // Check for status change event
        let event = timeout(Duration::from_millis(100), event_receiver.recv()).await;
        assert!(event.is_ok());

        if let Ok(Ok(ProjectEvent::TaskStatusChanged { task_id: received_id, old_status, new_status })) = event {
            assert_eq!(received_id, task_id);
            assert_eq!(old_status, TaskStatus::Pending);
            assert_eq!(new_status, TaskStatus::InProgress);
        } else {
            panic!("Expected TaskStatusChanged event");
        }

        manager.shutdown().await;
    }

    #[tokio::test]
    async fn test_agent_addition_with_events() {
        let project = create_test_project();
        let manager = ProjectManager::new(project);

        // Create an agent
        let agent = create_test_agent("Test Agent");
        let agent_id = agent.id;
        let result = manager.add_agent(agent).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), agent_id);

        manager.shutdown().await;
    }

    #[tokio::test]
    async fn test_statistics_calculation() {
        let project = create_test_project();
        let manager = ProjectManager::new(project);

        // Get initial statistics
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.total_agents, 0);
        assert_eq!(stats.completion_percentage, 0.0);

        // Add a task
        let task = create_test_task("Test Task");
        manager.create_task(task).await.unwrap();

        // Get updated statistics
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.pending_tasks, 1);
        assert_eq!(stats.completion_percentage, 0.0);

        manager.shutdown().await;
    }

    #[tokio::test]
    async fn test_comment_addition_with_events() {
        let project = create_test_project();
        let manager = ProjectManager::new(project);

        // Subscribe to events
        let mut event_receiver = manager.subscribe_to_events();

        // Create a task first
        let task = create_test_task("Test Task");
        let task_id = task.id;
        manager.create_task(task).await.unwrap();

        // Clear creation events
        let _ = event_receiver.recv().await;
        let _ = event_receiver.recv().await;

        // Add a comment
        let result = manager.add_comment(
            task_id,
            CommentType::Task,
            "test_user".to_string(),
            "This is a test comment".to_string(),
        ).await;

        assert!(result.is_ok());

        // Check for comment event
        let event = timeout(Duration::from_millis(100), event_receiver.recv()).await;
        assert!(event.is_ok());

        if let Ok(Ok(ProjectEvent::CommentAdded { author, content, comment_type, target_id, .. })) = event {
            assert_eq!(author, "test_user");
            assert_eq!(content, "This is a test comment");
            assert_eq!(comment_type, CommentType::Task);
            assert_eq!(target_id, task_id);
        } else {
            panic!("Expected CommentAdded event");
        }

        manager.shutdown().await;
    }

    #[tokio::test]
    async fn test_comment_types() {
        let project = create_test_project();
        let manager = ProjectManager::new(project);

        // Create a task
        let task = create_test_task("Test Task");
        let task_id = task.id;
        manager.create_task(task).await.unwrap();

        // Create an issue from a task
        let issue_task = create_test_task("Issue Task");
        let issue = Issue::from_task(&issue_task);
        let issue_id = issue.id;
        manager.create_issue(issue).await.unwrap();

        // Test different comment types
        let task_comment_result = manager.add_comment(
            task_id,
            CommentType::Task,
            "user1".to_string(),
            "Task comment".to_string(),
        ).await;
        assert!(task_comment_result.is_ok());

        let issue_comment_result = manager.add_comment(
            issue_id,
            CommentType::Issue,
            "user2".to_string(),
            "Issue comment".to_string(),
        ).await;
        assert!(issue_comment_result.is_ok());

        manager.shutdown().await;
    }
}
