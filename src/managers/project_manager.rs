use crate::models::project::Project;
use crate::models::task::Task;
use crate::models::agent::Agent;
use crate::models::issue::Issue;
use crate::enums::{TaskStatus, AgentStatus, IssueStatus, IssueType, CommentType};
use crate::error::{OrchestratorError, Result};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::{mpsc, RwLock, broadcast};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;

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

    // MCP Integration Events
    McpTaskExecutionStarted { task_id: Uuid, task_title: String },
    McpTaskExecutionCompleted { task_id: Uuid, success: bool },
    McpFeatureDevelopmentStarted { task_id: Uuid },
    McpFeatureDevelopmentCompleted { task_id: Uuid, actions_count: usize },

    // Statistics Events
    ProjectStatisticsUpdated { stats: ProjectStatistics },

    // Project Events
    ProjectStatusChanged { old_status: String, new_status: String },
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

    // MCP Integration
    ExecuteTaskWithMcp { task_id: Uuid, mcp_client: crate::managers::McpClient, respond_to: mpsc::UnboundedSender<Result<()>> },
    ProcessTaskCompletion { task_id: Uuid, mcp_client: crate::managers::McpClient, respond_to: mpsc::UnboundedSender<Result<()>> },
    
    // Status Transition Commands (queue-based)
    TransitionTaskStatus { task_id: Uuid, new_status: TaskStatus, reason: String },
    TransitionAgentStatus { agent_id: Uuid, new_status: AgentStatus, reason: String },
    TransitionIssueStatus { issue_id: Uuid, new_status: IssueStatus, reason: String },
    
    // Project Management
    CreateProject { 
        name: String, 
        idea: String, 
        path: String, 
        tech_stack: crate::enums::TechStack,
        repository_url: Option<String>,
        dependencies_urls: Option<Vec<String>>,
        mcp_client: crate::managers::McpClient,
        respond_to: mpsc::UnboundedSender<Result<Project>> 
    },
    SaveProject { respond_to: mpsc::UnboundedSender<Result<()>> },
    LoadProject { path: std::path::PathBuf, respond_to: mpsc::UnboundedSender<Result<Project>> },
    ExecuteAllTasksInOrder { mcp_client: crate::managers::McpClient, respond_to: mpsc::UnboundedSender<Result<()>> },

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
///     debug!("Received event: {:?}", event);
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

    /// Create a new project with full business logic
    pub async fn create_project(
        name: String,
        idea: String,
        path: String,
        tech_stack: crate::enums::TechStack,
        repository_url: Option<String>,
        dependencies_urls: Option<Vec<String>>,
        mcp_client: crate::managers::McpClient,
    ) -> Result<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Create a temporary project manager for the creation process
        let temp_project = Project::new_with_tech_stack(&name, &idea, &path, tech_stack.clone());
        let temp_manager = Self::new(temp_project);
        
        temp_manager.command_sender.send(ProjectCommand::CreateProject {
            name,
            idea,
            path,
            tech_stack,
            repository_url,
            dependencies_urls,
            mcp_client,
            respond_to: tx,
        }).map_err(|_| OrchestratorError::internal("Failed to send create project command"))?;

        let created_project = rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive project creation response"))??;
        
        // Shutdown temporary manager
        temp_manager.shutdown().await;
        
        // Return new manager with created project
        Ok(Self::new(created_project))
    }

    /// Save the current project state
    pub async fn save_project(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.command_sender.send(ProjectCommand::SaveProject {
            respond_to: tx
        }).map_err(|_| OrchestratorError::internal("Failed to send save project command"))?;

        rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive save response"))?
    }

    /// Load a project from the specified path
    pub async fn load_project(path: std::path::PathBuf) -> Result<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Create temporary manager for loading
        let temp_project = Project::new("temp", "temp", path.to_str().unwrap_or(""));
        let temp_manager = Self::new(temp_project);
        
        temp_manager.command_sender.send(ProjectCommand::LoadProject {
            path,
            respond_to: tx,
        }).map_err(|_| OrchestratorError::internal("Failed to send load project command"))?;

        let loaded_project = rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive load response"))??;
        
        // Shutdown temporary manager
        temp_manager.shutdown().await;
        
        // Return new manager with loaded project
        Ok(Self::new(loaded_project))
    }

    /// Execute all tasks in dependency order
    pub async fn execute_all_tasks_in_order(&self, mcp_client: crate::managers::McpClient) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.command_sender.send(ProjectCommand::ExecuteAllTasksInOrder {
            mcp_client,
            respond_to: tx
        }).map_err(|_| OrchestratorError::internal("Failed to send execute all tasks command"))?;

        rx.recv().await
            .ok_or_else(|| OrchestratorError::internal("Failed to receive execute response"))?
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

                ProjectCommand::ExecuteTaskWithMcp { task_id, mcp_client, respond_to } => {
                    let result = Self::handle_execute_task_with_mcp(
                        &project,
                        task_id,
                        mcp_client,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::ProcessTaskCompletion { task_id, mcp_client, respond_to } => {
                    let result = Self::handle_process_task_completion(
                        &project,
                        task_id,
                        mcp_client,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::CreateProject { 
                    name, idea, path, tech_stack, repository_url, dependencies_urls, mcp_client, respond_to 
                } => {
                    let result = Self::handle_create_project(
                        &project,
                        name,
                        idea,
                        path,
                        tech_stack,
                        repository_url,
                        dependencies_urls,
                        mcp_client,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::SaveProject { respond_to } => {
                    let result = Self::handle_save_project(&project).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::LoadProject { path, respond_to } => {
                    let result = Self::handle_load_project(path).await;
                    let _ = respond_to.send(result);
                }

                ProjectCommand::ExecuteAllTasksInOrder { mcp_client, respond_to } => {
                    let result = Self::handle_execute_all_tasks_in_order(
                        &project,
                        mcp_client,
                        &event_broadcaster
                    ).await;
                    let _ = respond_to.send(result);
                }

                // Status Transition Commands (queue-based streaming updates)
                ProjectCommand::TransitionTaskStatus { task_id, new_status, reason } => {
                    Self::handle_transition_task_status(
                        &project,
                        task_id,
                        new_status,
                        reason,
                        &event_broadcaster
                    ).await;
                }

                ProjectCommand::TransitionAgentStatus { agent_id, new_status, reason } => {
                    Self::handle_transition_agent_status(
                        &project,
                        agent_id,
                        new_status,
                        reason,
                        &event_broadcaster
                    ).await;
                }

                ProjectCommand::TransitionIssueStatus { issue_id, new_status, reason } => {
                    Self::handle_transition_issue_status(
                        &project,
                        issue_id,
                        new_status,
                        reason,
                        &event_broadcaster
                    ).await;
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
        let errors = 0;

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

    /// Handle task execution with MCP integration
    async fn handle_execute_task_with_mcp(
        project: &Arc<RwLock<Project>>,
        task_id: Uuid,
        mcp_client: crate::managers::McpClient,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<()> {
        debug!("🎯 Starting MCP task execution with proper history tracking for task: {}", task_id);
        
        // Step 1: Update task status to InProgress and extract task info
        let (task_title, task_description, task_complexity, task_priority, task_tags, existing_assignment) = {
            let mut project_guard = project.write().await;
            let task = project_guard.get_task_mut(task_id)
                .ok_or_else(|| OrchestratorError::validation("Task not found"))?;

            debug!("📋 Task found: {} - {}", task.title, task.description);

            // Transition task to InProgress with history tracking
            if let Err(e) = task.transition_task_status(TaskStatus::InProgress) {
                debug!("⚠️  Failed to transition task to InProgress: {}", e);
            } else {
                debug!("✅ Task '{}' transitioned to InProgress with history updated", task.title);
                
                // Emit task status change event
                let _ = event_broadcaster.send(ProjectEvent::TaskStatusChanged {
                    task_id,
                    old_status: TaskStatus::Pending,
                    new_status: TaskStatus::InProgress,
                });
            }

            // Extract task info and existing assignment status
            (
                task.title.clone(),
                task.description.clone(),
                task.estimated_complexity.unwrap_or(5),
                task.priority.clone(),
                task.tags.clone(),
                task.assigned_to.as_ref().map(|a| a.id)
            )
        };

        // Get tech stack separately to avoid borrowing conflicts
        let tech_stack = {
            let project_guard = project.read().await;
            project_guard.tech_stack.clone()
        };

        // Step 2: Handle agent assignment separately
        let assigned_agent_id = if existing_assignment.is_none() {
            let mut project_guard = project.write().await;
            
            // Find available agent
            if let Some(agent_index) = project_guard.agents.iter().position(|a| a.status.is_available()) {
                let agent = &mut project_guard.agents[agent_index];
                debug!("👤 Assigning task '{}' to agent: {}", task_title, agent.name);
                
                // Update agent status to Working with history tracking
                if let Err(e) = agent.start_work(&task_title) {
                    debug!("⚠️  Failed to start work on agent {}: {}", agent.name, e);
                    None
                } else {
                    debug!("✅ Agent {} started working on task with history updated", agent.name);
                    
                    let agent_id = agent.id;
                    let agent_name = agent.name.clone();
                    let agent_clone = agent.clone();
                    
                    // Emit agent status change event
                    let _ = event_broadcaster.send(ProjectEvent::AgentStatusChanged {
                        agent_id,
                        agent_name: agent_name.clone(),
                        old_status: AgentStatus::Idle,
                        new_status: AgentStatus::Working,
                    });
                    
                    // Emit agent started working event
                    let _ = event_broadcaster.send(ProjectEvent::AgentStartedWorking {
                        agent_id,
                        agent_name: agent_name.clone(),
                        task_id,
                        task_title: task_title.clone(),
                    });
                    
                    // Now assign the agent to the task
                    if let Some(task) = project_guard.get_task_mut(task_id) {
                        task.assigned_to = Some(agent_clone);
                        
                        // Emit task assigned event
                        let _ = event_broadcaster.send(ProjectEvent::TaskAssigned {
                            task_id,
                            agent_id,
                            agent_name,
                        });
                    }
                    
                    Some(agent_id)
                }
            } else {
                debug!("⚠️  No available agents found, proceeding without assignment");
                None
            }
        } else {
            debug!("✅ Task already assigned to agent");
            existing_assignment
        };


        // Emit start event
        let _ = event_broadcaster.send(ProjectEvent::McpTaskExecutionStarted {
            task_id,
            task_title: task_title.clone(),
        });

        // Collect existing files for context (simplified version)
        debug!("📂 Collecting existing project files for context...");
        let existing_files = {
            let project_guard = project.read().await;
            let project_path = PathBuf::from(&project_guard.project_path);
            debug!("📂 Collecting existing files from project: {:?}", project_path);
            collect_existing_files_for_context(&project_path).await.unwrap_or_else(|e| {
                debug!("⚠️  Failed to collect existing files: {}", e);
                Vec::new()
            })
        };
        
        // Build completed dependencies context
        let completed_dependency_names = {
            let project_guard = project.read().await;
            let task = project_guard.get_task(task_id).unwrap();
            task.depends_on.iter()
                .filter_map(|dep_id| {
                    project_guard.get_task(*dep_id)
                        .filter(|dep_task| dep_task.status == TaskStatus::Completed)
                        .map(|dep_task| dep_task.title.clone())
                })
                .collect::<Vec<_>>()
        };
        
        let acceptance_criteria = vec![task_description.clone()];
        let codebase_context = format!(
            "Project using {:?} technology stack. Task dependencies: {:?}",
            tech_stack, completed_dependency_names
        );

        debug!("🤖 Calling MCP task development with proper context...");
        debug!("📊 Task parameters:");
        debug!("   🎯 Title: {}", task_title);
        debug!("   📊 Complexity: {}/10", task_complexity);
        debug!("   🎚️  Priority: {:?}", task_priority);
        debug!("   🏷️  Tags: {:?}", task_tags);
        debug!("   📂 Context files: {}", existing_files.len());
        debug!("   ✅ Completed deps: {}", completed_dependency_names.len());

        // Execute task development with MCP using task_development method
        let _ = event_broadcaster.send(ProjectEvent::McpFeatureDevelopmentStarted { task_id });

        let result = mcp_client.task_development(
            task_title.clone(),
            task_description.clone(),
            task_complexity,
            format!("{:?}", task_priority),
            task_tags,
            format!("{:?}", tech_stack),
            existing_files,
            completed_dependency_names,
            acceptance_criteria,
            codebase_context,
            crate::managers::McpModel::Gemini,
        ).await;

        match result {
            Ok(response) => {
                debug!("✅ MCP task development successful! Received {} actions", response.actions.len());
                
                let _ = event_broadcaster.send(ProjectEvent::McpFeatureDevelopmentCompleted {
                    task_id,
                    actions_count: response.actions.len(),
                });

                // Execute the actions returned by MCP in the project directory
                let project_path = {
                    let project_guard = project.read().await;
                    PathBuf::from(&project_guard.project_path)
                };
                
                debug!("🔄 Executing {} actions in project directory: {:?}...", response.actions.len(), project_path);
                
                // Save current working directory
                let original_dir = std::env::current_dir().map_err(|e| {
                    debug!("⚠️  Failed to get current directory: {}", e);
                    e
                }).unwrap_or_else(|_| PathBuf::from("."));
                
                // Change to project directory for action execution
                if let Err(e) = std::env::set_current_dir(&project_path) {
                    debug!("⚠️  Failed to change to project directory {:?}: {}", project_path, e);
                } else {
                    debug!("📁 Changed working directory to: {:?}", project_path);
                }
                
                for (action_index, action) in response.actions.iter().enumerate() {
                    debug!("   🎬 Executing action {}/{}: {:?}", action_index + 1, response.actions.len(), action);
                    if let Err(e) = action.execute().await {
                        debug!("❌ Failed to execute action {}: {}", action_index + 1, e);
                        // Note: We continue with other actions even if one fails
                    } else {
                        debug!("✅ Action {} executed successfully", action_index + 1);
                    }
                }
                
                // Restore original working directory
                if let Err(e) = std::env::set_current_dir(&original_dir) {
                    debug!("⚠️  Failed to restore original directory {:?}: {}", original_dir, e);
                } else {
                    debug!("🔙 Restored working directory to: {:?}", original_dir);
                }

                // Update task status to completed with history tracking
                {
                    let mut project_guard = project.write().await;
                    if let Some(task) = project_guard.get_task_mut(task_id) {
                        if let Err(e) = task.transition_task_status(TaskStatus::Completed) {
                            debug!("⚠️  Failed to transition task to Completed: {}", e);
                        } else {
                            debug!("✅ Task '{}' transitioned to Completed with history updated", task.title);
                            
                            // Emit task status change event
                            let _ = event_broadcaster.send(ProjectEvent::TaskStatusChanged {
                                task_id,
                                old_status: TaskStatus::InProgress,
                                new_status: TaskStatus::Completed,
                            });
                            
                            // Emit task completed event
                            let _ = event_broadcaster.send(ProjectEvent::TaskCompleted {
                                task_id,
                                task_title: task.title.clone(),
                                completion_time: chrono::Utc::now(),
                            });
                        }
                    }
                    
                    // Update assigned agent status to completed work
                    if let Some(agent_id) = assigned_agent_id {
                        if let Some(agent) = project_guard.agents.iter_mut().find(|a| a.id == agent_id) {
                            if let Err(e) = agent.complete_work() {
                                debug!("⚠️  Failed to complete work on agent {}: {}", agent.name, e);
                            } else {
                                debug!("✅ Agent {} completed work with history updated", agent.name);
                                
                                // Emit agent finished working event
                                let _ = event_broadcaster.send(ProjectEvent::AgentFinishedWorking {
                                    agent_id: agent.id,
                                    agent_name: agent.name.clone(),
                                    task_id,
                                });
                                
                                // Emit agent status change event
                                let _ = event_broadcaster.send(ProjectEvent::AgentStatusChanged {
                                    agent_id: agent.id,
                                    agent_name: agent.name.clone(),
                                    old_status: AgentStatus::Working,
                                    new_status: AgentStatus::Active,
                                });
                            }
                        }
                    }
                }

                let _ = event_broadcaster.send(ProjectEvent::McpTaskExecutionCompleted {
                    task_id,
                    success: true,
                });

                debug!("🎉 Task '{}' completed successfully with full history tracking!", task_title);
                Ok(())
            }
            Err(e) => {
                debug!("❌ MCP task development failed: {}", e);
                
                // Update task status to failed with history tracking
                {
                    let mut project_guard = project.write().await;
                    if let Some(task) = project_guard.get_task_mut(task_id) {
                        if let Err(transition_err) = task.transition_task_status(TaskStatus::Failed) {
                            debug!("⚠️  Failed to transition task to Failed: {}", transition_err);
                        } else {
                            debug!("✅ Task '{}' transitioned to Failed with history updated", task.title);
                            
                            // Emit task status change event
                            let _ = event_broadcaster.send(ProjectEvent::TaskStatusChanged {
                                task_id,
                                old_status: TaskStatus::InProgress,
                                new_status: TaskStatus::Failed,
                            });
                        }
                    }
                    
                    // Update assigned agent to error state
                    if let Some(agent_id) = assigned_agent_id {
                        if let Some(agent) = project_guard.agents.iter_mut().find(|a| a.id == agent_id) {
                            if let Err(agent_err) = agent.report_error(format!("Task execution failed: {}", e)) {
                                debug!("⚠️  Failed to report error on agent {}: {}", agent.name, agent_err);
                            } else {
                                debug!("✅ Agent {} reported error with history updated", agent.name);
                                
                                // Emit agent status change event
                                let _ = event_broadcaster.send(ProjectEvent::AgentStatusChanged {
                                    agent_id: agent.id,
                                    agent_name: agent.name.clone(),
                                    old_status: AgentStatus::Working,
                                    new_status: AgentStatus::Error,
                                });
                            }
                        }
                    }
                }
                
                let _ = event_broadcaster.send(ProjectEvent::McpTaskExecutionCompleted {
                    task_id,
                    success: false,
                });
                
                Err(e)
            }
        }
    }

    /// Handle task completion processing with MCP
    async fn handle_process_task_completion(
        project: &Arc<RwLock<Project>>,
        task_id: Uuid,
        mcp_client: crate::managers::McpClient,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<()> {
        let project_guard = project.read().await;
        let task = project_guard.get_task(task_id)
            .ok_or_else(|| OrchestratorError::validation("Task not found"))?;

        // If task is completed, trigger next dependent tasks
        if task.status == TaskStatus::Completed {
            // Find tasks that depend on this completed task
            let dependent_tasks: Vec<Uuid> = project_guard.tasks.iter()
                .filter(|t| t.depends_on.contains(&task_id) && t.status == TaskStatus::Pending)
                .map(|t| t.id)
                .collect();

            drop(project_guard); // Release the read lock

            // Start dependent tasks
            for dependent_task_id in dependent_tasks {
                if let Err(e) = Self::handle_execute_task_with_mcp(
                    project,
                    dependent_task_id,
                    mcp_client.clone(),
                    event_broadcaster,
                ).await {
                    debug!("Failed to execute dependent task {}: {}", dependent_task_id, e);
                }
            }
        }

        Ok(())
    }

    /// Handle project creation with full business logic
    async fn handle_create_project(
        project: &Arc<RwLock<Project>>,
        name: String,
        idea: String,
        path: String,
        tech_stack: crate::enums::TechStack,
        repository_url: Option<String>,
        dependencies_urls: Option<Vec<String>>,
        mcp_client: crate::managers::McpClient,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<Project> {
        use std::fs;
        use std::path::PathBuf;
        
        debug!("🚀 ProjectManager: Creating project: {}", name);
        debug!("💡 ProjectManager: Idea: {}", idea);
        debug!("📁 ProjectManager: Path: {}", path);
        debug!("🔧 ProjectManager: Tech Stack: {:?}", tech_stack);

        // Create project directory if it doesn't exist
        let project_path = PathBuf::from(&path);
        if !project_path.exists() {
            fs::create_dir_all(&project_path)
                .map_err(OrchestratorError::Io)?;
            debug!("📂 ProjectManager: Created project directory: {}", path);
        }

        // Initialize context files using MCP
        debug!("📝 ProjectManager: Initializing context files...");
        if let Err(e) = mcp_client.initialize_context(project_path.clone(), tech_stack.clone()).await {
            debug!("Warning: Failed to initialize context files: {}", e);
        } else {
            debug!("✅ ProjectManager: Context files created (GEMINI.md, CLAUDE.md)");
        }

        // Update project with new details
        let mut project_guard = project.write().await;
        project_guard.name = name.clone();
        project_guard.idea = idea.clone();
        project_guard.project_path = path.clone();
        project_guard.tech_stack = tech_stack.clone();

        // Set optional repository URL
        if let Some(repo_url) = repository_url {
            project_guard.set_repository_url(&repo_url);
            debug!("🔗 ProjectManager: Repository URL: {}", repo_url);
        }

        // Set optional dependency URLs
        if let Some(deps) = dependencies_urls {
            for url in &deps {
                if let Err(e) = project_guard.add_dependency_url(url) {
                    debug!("Warning: Failed to add dependency URL '{}': {}", url, e);
                }
            }
            debug!("📦 ProjectManager: Dependencies: {:?}", deps);
        }

        // Load agents from the agents directory
        if let Err(e) = project_guard.load_agents_from_directory("agents").await {
            debug!("Warning: Failed to load agents from directory: {}", e);
        } else {
            debug!("🤖 ProjectManager: Loaded {} agents from agents directory", project_guard.agents.len());
        }

        // Prepare agent types for idea breakdown
        let available_agents: Vec<String> = project_guard.agents.iter()
            .map(|agent| agent.name.clone())
            .collect();

        // If no agents loaded, use default agent types based on tech stack
        let agent_types = if available_agents.is_empty() {
            match tech_stack {
                crate::enums::TechStack::Rust => vec!["BackendEngineerRust".to_string()],
                crate::enums::TechStack::Vue => vec!["FrontendEngineerVue".to_string()],
                crate::enums::TechStack::React => vec!["FrontendEngineerReact".to_string()],
                crate::enums::TechStack::FullstackRustVue => vec![
                    "BackendEngineerRust".to_string(),
                    "FrontendEngineerVue".to_string(),
                    "DevOpsEngineer".to_string(),
                ],
                crate::enums::TechStack::FullstackRustReact => vec![
                    "BackendEngineerRust".to_string(),
                    "FrontendEngineerReact".to_string(),
                    "DevOpsEngineer".to_string(),
                ],
            }
        } else {
            available_agents
        };

        // Save project configuration IMMEDIATELY after initial setup (before MCP call)
        let config_path = project_path.join("orchy.json");
        debug!("💾 ProjectManager: Saving initial project configuration to: {}", config_path.display());
        
        let initial_project_json = serde_json::to_string_pretty(&*project_guard)
            .map_err(OrchestratorError::Json)?;
        debug!("📄 ProjectManager: Initial project JSON size: {} bytes", initial_project_json.len());
        
        fs::write(&config_path, &initial_project_json)
            .map_err(OrchestratorError::Io)?;
        debug!("✅ ProjectManager: Initial project configuration saved successfully");

        drop(project_guard); // Release the write lock

        // Execute idea breakdown using MCP Manager
        debug!("🧠 ProjectManager: Breaking down idea into tasks using AI...");
        let context = format!("Project: {}\nTech Stack: {:?}\nPath: {}", name, tech_stack, path);

        debug!("🔗 ProjectManager: About to call MCP idea_breakdown with:");
        debug!("   - Idea: {}", idea);
        debug!("   - Context: {}", context);
        debug!("   - Agent types: {:?}", agent_types);
        debug!("   - Tech stack: {:?}", tech_stack);

        let breakdown_response = match tokio::time::timeout(
            std::time::Duration::from_secs(120),
            mcp_client.idea_breakdown(
                idea.clone(),
                context,
                agent_types,
                format!("{:?}", tech_stack),
                crate::managers::McpModel::Gemini,
            )
        ).await {
            Ok(Ok(response)) => {
                debug!("✅ ProjectManager: MCP idea_breakdown completed successfully");
                response
            },
            Ok(Err(e)) => {
                debug!("❌ ProjectManager: MCP idea_breakdown failed: {}", e);
                return Err(OrchestratorError::internal(format!("MCP integration failed: {}", e)));
            },
            Err(_) => {
                debug!("⏰ ProjectManager: MCP idea_breakdown timed out after 30 seconds");
                return Err(OrchestratorError::timeout("MCP idea_breakdown call"));
            }
        };

        debug!("✅ ProjectManager: Generated {} tasks from idea breakdown", breakdown_response.tasks.len());

        // Convert TaskInput to Task and add to project
        debug!("🔄 ProjectManager: Adding {} tasks to project", breakdown_response.tasks.len());
        let mut project_guard = project.write().await;
        for (index, task_input) in breakdown_response.tasks.iter().enumerate() {
            debug!("📝 ProjectManager: Processing task {}/{}: '{}'", index + 1, breakdown_response.tasks.len(), task_input.title);
            
            let task = Task::from_input(task_input.clone(), None);
            debug!("   - Generated Task ID: {}", task.id);
            
            if let Err(e) = project_guard.add_task(task.clone()) {
                debug!("❌ ProjectManager: Failed to add task '{}': {}", task_input.title, e);
            } else {
                debug!("✅ ProjectManager: Successfully added task '{}' to project", task_input.title);
            }
        }

        debug!("📊 ProjectManager: Project now has {} tasks total", project_guard.tasks.len());

        // Save project configuration IMMEDIATELY after tasks are added
        let config_path = project_path.join("orchy.json");
        debug!("💾 ProjectManager: Saving project configuration to: {}", config_path.display());
        
        let project_json = serde_json::to_string_pretty(&*project_guard)
            .map_err(OrchestratorError::Json)?;
        debug!("📄 ProjectManager: Project JSON size: {} bytes", project_json.len());
        
        fs::write(&config_path, &project_json)
            .map_err(OrchestratorError::Io)?;
        debug!("✅ ProjectManager: Project configuration saved successfully");

        // Emit project creation event
        let _ = event_broadcaster.send(ProjectEvent::ProjectStatusChanged {
            old_status: "Creating".to_string(),
            new_status: "Created".to_string(),
        });

        // Return the created project
        let result_project = project_guard.clone();
        drop(project_guard);

        debug!("✅ ProjectManager: Project '{}' created successfully!", name);
        debug!("📁 ProjectManager: Project saved to: {}", config_path.display());
        debug!("🆔 ProjectManager: Project ID: {}", result_project.id);
        debug!("📊 ProjectManager: Tasks created: {}", result_project.tasks.len());

        Ok(result_project)
    }

    /// Handle saving project to disk
    async fn handle_save_project(project: &Arc<RwLock<Project>>) -> Result<()> {
        use std::fs;
        use std::path::PathBuf;

        let project_guard = project.read().await;
        let project_path = PathBuf::from(&project_guard.project_path);
        let config_path = project_path.join("orchy.json");
        
        debug!("💾 ProjectManager: Saving project to: {}", config_path.display());
        
        let project_json = serde_json::to_string_pretty(&*project_guard)
            .map_err(OrchestratorError::Json)?;
        
        fs::write(&config_path, project_json)
            .map_err(OrchestratorError::Io)?;
        
        debug!("✅ ProjectManager: Project saved successfully");
        Ok(())
    }

    /// Handle loading project from disk
    async fn handle_load_project(path: std::path::PathBuf) -> Result<Project> {
        use std::fs;

        debug!("📂 ProjectManager: Loading project from: {:?}", path);
        let config_path = path.join("orchy.json");
        
        if !config_path.exists() {
            return Err(OrchestratorError::validation("Project configuration file (orchy.json) not found"));
        }

        let project_json = fs::read_to_string(&config_path)
            .map_err(OrchestratorError::Io)?;

        let project: Project = serde_json::from_str(&project_json)
            .map_err(OrchestratorError::Json)?;

        debug!("✅ ProjectManager: Successfully loaded project with {} tasks and {} agents",
               project.tasks.len(), project.agents.len());

        Ok(project)
    }

    /// Handle executing all tasks in dependency order
    async fn handle_execute_all_tasks_in_order(
        project: &Arc<RwLock<Project>>,
        mcp_client: crate::managers::McpClient,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) -> Result<()> {
        use crate::utils::dependency_resolver::DependencyResolver;
        use std::collections::HashSet;

        debug!("🎯 ProjectManager: Starting task-by-task development process");
        
        let sorted_tasks = {
            let project_guard = project.read().await;
            debug!("📊 ProjectManager: Total tasks to execute: {}", project_guard.tasks.len());
            debug!("🤖 ProjectManager: Total agents available: {}", project_guard.agents.len());

            // Validate dependencies first
            debug!("🔍 ProjectManager: Validating task dependencies...");
            DependencyResolver::validate_dependencies(&project_guard.tasks)
                .map_err(|e| OrchestratorError::validation(&format!("Dependency validation failed: {}", e)))?;
            debug!("✅ ProjectManager: All task dependencies are valid");

            // Sort tasks by dependencies
            debug!("📋 ProjectManager: Sorting tasks by dependency order...");
            DependencyResolver::sort_tasks_by_dependencies(project_guard.tasks.clone())
                .map_err(|e| OrchestratorError::validation(&format!("Task sorting failed: {}", e)))?
        };

        debug!("✅ ProjectManager: Tasks sorted successfully");

        // Track completed tasks
        let mut completed_tasks: HashSet<Uuid> = HashSet::new();
        let _task_name_mapping: std::collections::HashMap<Uuid, String> = sorted_tasks.iter()
            .map(|task| (task.id, task.title.clone()))
            .collect();

        debug!("🚀 ProjectManager: Beginning task execution in dependency order...");

        // Execute tasks one by one
        for (task_index, task) in sorted_tasks.iter().enumerate() {
            debug!("");
            debug!("{}", "=".repeat(80));
            debug!("🎯 ProjectManager: EXECUTING TASK {}/{}: {}", task_index + 1, sorted_tasks.len(), task.title);
            debug!("{}", "=".repeat(80));

            // Verify all dependencies are completed
            debug!("🔍 ProjectManager: Verifying dependencies are satisfied...");
            if !DependencyResolver::are_dependencies_satisfied(task, &completed_tasks) {
                debug!("⚠️  ProjectManager: Task '{}' has unsatisfied dependencies, skipping", task.title);
                debug!("🔄 ProjectManager: Marking task as failed due to unsatisfied dependencies");
                
                // Mark task as failed due to dependency issues
                {
                    let mut project_guard = project.write().await;
                    if let Some(mut_task) = project_guard.get_task_mut(task.id) {
                        if let Err(e) = mut_task.transition_task_status(TaskStatus::Failed) {
                            debug!("⚠️  Failed to transition task to Failed: {}", e);
                        } else {
                            debug!("✅ Task '{}' transitioned to Failed due to dependency issues", task.title);
                        }
                    }
                }
                
                // Emit failure event and continue with next task
                let _ = event_broadcaster.send(ProjectEvent::TaskStatusChanged {
                    task_id: task.id,
                    old_status: TaskStatus::Pending,
                    new_status: TaskStatus::Failed,
                });
                
                debug!("⏭️  ProjectManager: Skipping to next task");
                continue;
            }
            debug!("✅ ProjectManager: All dependencies satisfied, proceeding with task execution");

            // Execute task via MCP
            debug!("🤖 ProjectManager: Executing task via MCP...");
            match Self::handle_execute_task_with_mcp(
                project,
                task.id,
                mcp_client.clone(),
                event_broadcaster,
            ).await {
                Ok(_) => {
                    debug!("✅ ProjectManager: Task '{}' executed successfully!", task.title);
                    completed_tasks.insert(task.id);
                    debug!("✅ ProjectManager: Task '{}' marked as COMPLETED ({}/{} tasks done)",
                           task.title, completed_tasks.len(), sorted_tasks.len());
                }
                Err(e) => {
                    debug!("❌ ProjectManager: Task '{}' execution failed: {}", task.title, e);
                    debug!("🔄 ProjectManager: Continuing with next task (non-blocking failure)");
                    
                    // Emit failure event but continue processing other tasks
                    let _ = event_broadcaster.send(ProjectEvent::TaskStatusChanged {
                        task_id: task.id,
                        old_status: TaskStatus::InProgress,
                        new_status: TaskStatus::Failed,
                    });
                    
                    // DO NOT return error - continue with next task
                    // Tasks that depend on this failed task will be skipped naturally
                    debug!("⚠️  ProjectManager: Task '{}' failed but continuing execution", task.title);
                }
            }

            debug!("🎉 ProjectManager: Task '{}' completed successfully!", task.title);
            debug!("📊 ProjectManager: Progress: {}/{} tasks completed", completed_tasks.len(), sorted_tasks.len());
        }

        debug!("");
        debug!("🎉 ProjectManager: ALL TASKS COMPLETED SUCCESSFULLY!");
        debug!("📊 ProjectManager: Final statistics:");
        debug!("   ✅ Total tasks executed: {}", sorted_tasks.len());
        debug!("   ✅ All dependencies satisfied");
        debug!("   ✅ All actions executed successfully");

        // Save final project state
        debug!("📊 ProjectManager: Saving final project state...");
        if let Err(e) = Self::handle_save_project(project).await {
            debug!("⚠️  ProjectManager: Failed to save final project state: {}", e);
        } else {
            debug!("✅ ProjectManager: Final project state saved successfully");
        }

        debug!("🏁 ProjectManager: Task-by-task development process complete!");
        Ok(())
    }

    /// Handle task status transition with streaming updates and status history
    async fn handle_transition_task_status(
        project: &Arc<RwLock<Project>>,
        task_id: Uuid,
        new_status: TaskStatus,
        reason: String,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) {
        let mut project_guard = project.write().await;
        
        if let Some(task) = project_guard.get_task_mut(task_id) {
            let old_status = task.status.clone();
            
            // Transition the task status (this automatically updates status history)
            if let Err(e) = task.transition_task_status(new_status.clone()) {
                debug!("❌ Failed to transition task status: {}", e);
                return;
            }
            
            debug!("🔄 Task '{}' transitioned from {:?} to {:?}", task.title, old_status, new_status);
            
            // Emit streaming event
            let _ = event_broadcaster.send(ProjectEvent::TaskStatusChanged {
                task_id,
                old_status,
                new_status,
            });
            
            // Update project statistics
            let stats = Self::calculate_statistics_from_project(&*project_guard);
            let _ = event_broadcaster.send(ProjectEvent::ProjectStatisticsUpdated { stats });
        } else {
            debug!("⚠️  Task with ID {} not found for status transition", task_id);
        }
    }

    /// Handle agent status transition with streaming updates and status history
    async fn handle_transition_agent_status(
        project: &Arc<RwLock<Project>>,
        agent_id: Uuid,
        new_status: AgentStatus,
        reason: String,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) {
        let mut project_guard = project.write().await;
        
        if let Some(agent) = project_guard.get_agent_mut(agent_id) {
            let old_status = agent.status.clone();
            let agent_name = agent.name.clone();
            
            // Transition the agent status (this automatically updates status history)
            if let Err(e) = agent.transition_to(new_status.clone(), Some(reason)) {
                debug!("❌ Failed to transition agent status: {}", e);
                return;
            }
            
            debug!("🔄 Agent '{}' transitioned from {:?} to {:?}", agent_name, old_status, new_status);
            
            // Emit streaming event
            let _ = event_broadcaster.send(ProjectEvent::AgentStatusChanged {
                agent_id,
                agent_name,
                old_status,
                new_status,
            });
        } else {
            debug!("⚠️  Agent with ID {} not found for status transition", agent_id);
        }
    }

    /// Handle issue status transition with streaming updates and status history
    async fn handle_transition_issue_status(
        project: &Arc<RwLock<Project>>,
        issue_id: Uuid,
        new_status: IssueStatus,
        reason: String,
        event_broadcaster: &broadcast::Sender<ProjectEvent>,
    ) {
        let mut project_guard = project.write().await;
        
        if let Some(issue) = project_guard.get_issue_mut(issue_id) {
            let old_status = issue.status.clone();
            
            // Transition the issue status (this automatically updates status history)
            if let Err(e) = issue.transition_to(new_status.clone(), "ProjectManager", Some(reason)) {
                debug!("❌ Failed to transition issue status: {}", e);
                return;
            }
            
            debug!("🔄 Issue '{}' transitioned from {:?} to {:?}", issue.title, old_status, new_status);
            
            // Emit streaming event
            let _ = event_broadcaster.send(ProjectEvent::IssueStatusChanged {
                issue_id,
                old_status,
                new_status,
            });
            
            // Update project statistics
            let stats = Self::calculate_statistics_from_project(&*project_guard);
            let _ = event_broadcaster.send(ProjectEvent::ProjectStatisticsUpdated { stats });
        } else {
            debug!("⚠️  Issue with ID {} not found for status transition", issue_id);
        }
    }
}

/// Collect existing files in the project directory for context
async fn collect_existing_files_for_context(project_path: &PathBuf) -> Result<Vec<(String, String)>> {
    debug!("📂 Collecting existing files from: {:?}", project_path);
    let mut files = Vec::new();
    
    // Define file extensions we want to include for context
    let relevant_extensions = vec![
        "rs", "js", "ts", "jsx", "tsx", "vue", "py", "go", "java", "cpp", "c", "h",
        "json", "toml", "yaml", "yml", "md", "txt", "html", "css", "scss", "less",
        "php", "rb", "swift", "kt", "cs", "scala", "sh", "dockerfile", "xml"
    ];
    
    fn collect_files_recursive(
        dir: &PathBuf, 
        files: &mut Vec<(String, String)>, 
        relevant_extensions: &[&str],
        base_path: &PathBuf
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Skip hidden files and directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') {
                        continue;
                    }
                }
                
                // Skip common directories that usually don't contain source code
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if matches!(name, "node_modules" | "target" | "build" | "dist" | "__pycache__" | ".git") {
                            continue;
                        }
                    }
                    collect_files_recursive(&path, files, relevant_extensions, base_path)?;
                } else if path.is_file() {
                    // Check if file has a relevant extension
                    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                        if relevant_extensions.contains(&extension) {
                            // Get relative path from project root
                            let relative_path = path.strip_prefix(base_path)
                                .unwrap_or(&path)
                                .to_string_lossy()
                                .to_string();
                            
                            // Read file content (limit size to avoid overwhelming the AI)
                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    let truncated_content = if content.len() > 2000 {
                                        format!("{}...\n[Content truncated - {} total characters]", 
                                               &content[..2000], content.len())
                                    } else {
                                        content
                                    };
                                    files.push((relative_path, truncated_content));
                                },
                                Err(e) => {
                                    debug!("⚠️  Could not read file {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    collect_files_recursive(project_path, &mut files, &relevant_extensions, project_path)
        .map_err(|e| OrchestratorError::internal(format!("Failed to collect files: {}", e)))?;
    
    debug!("📋 Collected {} files for context:", files.len());
    for (path, content) in &files {
        debug!("   📄 {} ({} characters)", path, content.len());
    }
    
    Ok(files)
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
            debug!("Error updating task status: {:?}", e);
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
