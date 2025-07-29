use super::agent::Agent;
use super::issue::Issue;
use super::task::Task;
use crate::enums::{CodeStatus, CommentType, TaskStatus, TechStack};
use crate::error::{OrchestratorError, Result};
use crate::models::comment::Comment;
use crate::models::pull_request::PullRequest;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ProjectStatus {
    #[default]
    Planning,
    Active,
    OnHold,
    Completed,
    Cancelled,
    Archived,
}

impl ProjectStatus {
    /// Check if the project is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ProjectStatus::Completed | ProjectStatus::Cancelled | ProjectStatus::Archived
        )
    }

    /// Check if the project can have tasks executed
    pub fn can_execute_tasks(&self) -> bool {
        matches!(self, ProjectStatus::Active)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub idea: String,
    pub name: String,
    pub repository_url: Option<String>,
    pub project_path: String,
    pub status: ProjectStatus,
    pub tech_stack: TechStack,
    pub tasks: Vec<Task>,
    pub issues: Vec<Issue>,
    pub agents: Vec<Agent>,
    pub tasks_history: Vec<(Task, DateTime<Utc>)>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dependencies_urls: Option<Vec<String>>,
}

impl Project {
    /// Create a new project
    pub fn new(
        name: impl Into<String>,
        idea: impl Into<String>,
        project_path: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            idea: idea.into(),
            name: name.into(),
            repository_url: None,
            project_path: project_path.into(),
            status: ProjectStatus::default(),
            tech_stack: TechStack::default(),
            tasks: Vec::new(),
            issues: Vec::new(),
            agents: Vec::new(),
            tasks_history: Vec::new(),
            created_at: now,
            updated_at: now,
            dependencies_urls: None,
        }
    }

    /// Create a new project with tech stack
    pub fn new_with_tech_stack(
        name: impl Into<String>,
        idea: impl Into<String>,
        project_path: impl Into<String>,
        tech_stack: TechStack,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            idea: idea.into(),
            name: name.into(),
            repository_url: None,
            project_path: project_path.into(),
            status: ProjectStatus::default(),
            tech_stack,
            tasks: Vec::new(),
            issues: Vec::new(),
            agents: Vec::new(),
            tasks_history: Vec::new(),
            created_at: now,
            updated_at: now,
            dependencies_urls: None,
        }
    }

    /// Set the tech stack for the project
    pub fn set_tech_stack(&mut self, tech_stack: TechStack) {
        self.tech_stack = tech_stack;
        self.updated_at = Utc::now();
    }

    /// Get available agent types from the project's agents
    pub fn get_available_agent_types(&self) -> Vec<String> {
        self.agents
            .iter()
            .map(|agent| format!("{:?}", agent.agent_type))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Load agents from directory and add them to the project
    pub async fn load_agents_from_directory(&mut self, agents_dir: &str) -> Result<()> {
        use crate::models::agent::Agent;

        let agents = Agent::load_agents_from_directory(agents_dir).await?;
        for agent in agents {
            self.add_agent(agent)?;
        }
        Ok(())
    }

    /// Set the repository URL for the project
    pub fn set_repository_url(&mut self, repository_url: impl Into<String>) {
        self.repository_url = Some(repository_url.into());
        self.updated_at = Utc::now();
    }

    /// Transition project to a new status
    pub fn transition_to(&mut self, new_status: ProjectStatus) -> Result<()> {
        match (&self.status, &new_status) {
            // From Planning
            (
                ProjectStatus::Planning,
                ProjectStatus::Active | ProjectStatus::OnHold | ProjectStatus::Cancelled,
            ) => {
                self.status = new_status;
                self.updated_at = Utc::now();
                Ok(())
            }

            // From Active
            (
                ProjectStatus::Active,
                ProjectStatus::OnHold | ProjectStatus::Completed | ProjectStatus::Cancelled,
            ) => {
                self.status = new_status;
                self.updated_at = Utc::now();
                Ok(())
            }

            // From OnHold
            (ProjectStatus::OnHold, ProjectStatus::Active | ProjectStatus::Cancelled) => {
                self.status = new_status;
                self.updated_at = Utc::now();
                Ok(())
            }

            // From Completed
            (ProjectStatus::Completed, ProjectStatus::Archived) => {
                self.status = new_status;
                self.updated_at = Utc::now();
                Ok(())
            }

            // From Cancelled
            (ProjectStatus::Cancelled, ProjectStatus::Archived) => {
                self.status = new_status;
                self.updated_at = Utc::now();
                Ok(())
            }

            // Invalid transitions
            _ => Err(OrchestratorError::validation(format!(
                "Cannot transition project from {:?} to {:?}",
                self.status, new_status
            ))),
        }
    }

    /// Get tasks that are ready to be executed (no pending dependencies)
    pub fn get_ready_tasks(&self) -> Vec<&Task> {
        if !self.status.can_execute_tasks() {
            return Vec::new();
        }

        let completed_task_ids: HashSet<Uuid> = self
            .tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Completed)
            .map(|task| task.id)
            .collect();

        self.tasks
            .iter()
            .filter(|task| {
                // Task must not be completed or cancelled
                !matches!(task.status, TaskStatus::Completed | TaskStatus::Cancelled) &&
                // All dependencies must be completed
                task.depends_on.iter().all(|dep_id| completed_task_ids.contains(dep_id))
            })
            .collect()
    }

    /// Get tasks that can be started immediately (no dependencies and not started)
    pub fn get_independent_tasks(&self) -> Vec<&Task> {
        if !self.status.can_execute_tasks() {
            return Vec::new();
        }

        self.tasks
            .iter()
            .filter(|task| task.depends_on.is_empty() && task.status == TaskStatus::Pending)
            .collect()
    }

    /// Get tasks that are blocked by dependencies
    pub fn get_blocked_tasks(&self) -> Vec<&Task> {
        let completed_task_ids: HashSet<Uuid> = self
            .tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Completed)
            .map(|task| task.id)
            .collect();

        self.tasks
            .iter()
            .filter(|task| {
                !task.depends_on.is_empty()
                    && !task
                        .depends_on
                        .iter()
                        .all(|dep_id| completed_task_ids.contains(dep_id))
                    && !matches!(task.status, TaskStatus::Completed | TaskStatus::Cancelled)
            })
            .collect()
    }

    /// Validate task dependencies (detect cycles and missing dependencies)
    pub fn validate_dependencies(&self) -> Result<()> {
        let task_ids: HashSet<Uuid> = self.tasks.iter().map(|t| t.id).collect();

        // Check for missing dependencies
        for task in &self.tasks {
            for dep_id in &task.depends_on {
                if !task_ids.contains(dep_id) {
                    return Err(OrchestratorError::validation(format!(
                        "Task '{}' depends on non-existent task {}",
                        task.title, dep_id
                    )));
                }
            }
        }

        // Check for cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for task in &self.tasks {
            if !visited.contains(&task.id) {
                if self.has_cycle_dfs(task.id, &mut visited, &mut rec_stack)? {
                    return Err(OrchestratorError::validation(
                        "Circular dependency detected in project tasks",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Helper method for cycle detection using DFS
    fn has_cycle_dfs(
        &self,
        task_id: Uuid,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
    ) -> Result<bool> {
        visited.insert(task_id);
        rec_stack.insert(task_id);

        let task = self.tasks.iter().find(|t| t.id == task_id).ok_or_else(|| {
            OrchestratorError::validation("Task not found during cycle detection")
        })?;

        for &dep_id in &task.depends_on {
            if !visited.contains(&dep_id) {
                if self.has_cycle_dfs(dep_id, visited, rec_stack)? {
                    return Ok(true);
                }
            } else if rec_stack.contains(&dep_id) {
                return Ok(true);
            }
        }

        rec_stack.remove(&task_id);
        Ok(false)
    }

    /// Get topological order of tasks (dependency-respecting execution order)
    pub fn get_execution_order(&self) -> Result<Vec<Uuid>> {
        self.validate_dependencies()?;

        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut adj_list: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        // Initialize in-degree and adjacency list
        for task in &self.tasks {
            in_degree.insert(task.id, 0);
            adj_list.insert(task.id, Vec::new());
        }

        // Build the graph
        for task in &self.tasks {
            for &dep_id in &task.depends_on {
                adj_list.get_mut(&dep_id).unwrap().push(task.id);
                *in_degree.get_mut(&task.id).unwrap() += 1;
            }
        }

        // Kahn's algorithm for topological sorting
        let mut queue: VecDeque<Uuid> = VecDeque::new();
        let mut result = Vec::new();

        // Add all nodes with in-degree 0 to queue
        for (&task_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(task_id);
            }
        }

        while let Some(task_id) = queue.pop_front() {
            result.push(task_id);

            // Reduce in-degree of adjacent nodes
            if let Some(neighbors) = adj_list.get(&task_id) {
                for &neighbor in neighbors {
                    let degree = in_degree.get_mut(&neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        if result.len() != self.tasks.len() {
            return Err(OrchestratorError::validation(
                "Cannot create execution order due to circular dependencies",
            ));
        }

        Ok(result)
    }

    /// Add a task to the project
    pub fn add_task(&mut self, task: Task) -> Result<()> {
        // Validate that dependencies exist
        let task_ids: HashSet<Uuid> = self.tasks.iter().map(|t| t.id).collect();
        for dep_id in &task.depends_on {
            if !task_ids.contains(dep_id) {
                return Err(OrchestratorError::validation(format!(
                    "Cannot add task: dependency {} does not exist",
                    dep_id
                )));
            }
        }

        self.tasks.push(task);
        self.updated_at = Utc::now();

        // Validate no cycles were introduced
        self.validate_dependencies()?;

        Ok(())
    }

    /// Remove a task from the project
    pub fn remove_task(&mut self, task_id: Uuid) -> Result<Task> {
        // Check if any other tasks depend on this one
        let dependents: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|task| task.depends_on.contains(&task_id))
            .collect();

        if !dependents.is_empty() {
            let dependent_titles: Vec<String> =
                dependents.iter().map(|t| t.title.clone()).collect();
            return Err(OrchestratorError::validation(format!(
                "Cannot remove task: it is a dependency for tasks: {}",
                dependent_titles.join(", ")
            )));
        }

        let position = self
            .tasks
            .iter()
            .position(|t| t.id == task_id)
            .ok_or_else(|| OrchestratorError::validation("Task not found"))?;

        let removed_task = self.tasks.remove(position);
        self.updated_at = Utc::now();

        Ok(removed_task)
    }

    /// Add a dependency between two tasks
    pub fn add_dependency(&mut self, task_id: Uuid, depends_on_id: Uuid) -> Result<()> {
        if task_id == depends_on_id {
            return Err(OrchestratorError::validation(
                "Task cannot depend on itself",
            ));
        }

        // Check if dependency exists
        if !self.tasks.iter().any(|t| t.id == depends_on_id) {
            return Err(OrchestratorError::validation("Dependency task not found"));
        }

        // Find the task and check if dependency already exists
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| OrchestratorError::validation("Task not found"))?;

        if task.depends_on.contains(&depends_on_id) {
            return Err(OrchestratorError::validation("Dependency already exists"));
        }

        task.depends_on.push(depends_on_id);
        self.updated_at = Utc::now();

        // Validate no cycles were introduced
        self.validate_dependencies()?;

        Ok(())
    }

    /// Remove a dependency between two tasks
    pub fn remove_dependency(&mut self, task_id: Uuid, depends_on_id: Uuid) -> Result<()> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| OrchestratorError::validation("Task not found"))?;

        let position = task
            .depends_on
            .iter()
            .position(|&id| id == depends_on_id)
            .ok_or_else(|| OrchestratorError::validation("Dependency not found"))?;

        task.depends_on.remove(position);
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Get project completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.tasks.is_empty() {
            return 100.0;
        }

        let completed_count = self
            .tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Completed)
            .count();

        (completed_count as f64 / self.tasks.len() as f64) * 100.0
    }

    /// Check if project can be marked as completed
    pub fn can_complete(&self) -> bool {
        !self.tasks.is_empty()
            && self
                .tasks
                .iter()
                .all(|task| task.status == TaskStatus::Completed)
    }

    /// Auto-transition project status based on task states
    pub fn auto_update_status(&mut self) -> Result<()> {
        match self.status {
            ProjectStatus::Active => {
                if self.can_complete() {
                    self.transition_to(ProjectStatus::Completed)?;
                }
            }
            ProjectStatus::Planning => {
                // Auto-transition to Active if there are tasks that could be ready
                // (check for independent tasks or tasks with completed dependencies)
                if !self.tasks.is_empty() {
                    let has_executable_tasks = self.tasks.iter().any(|task| {
                        !matches!(task.status, TaskStatus::Completed | TaskStatus::Cancelled)
                            && (task.depends_on.is_empty() || {
                                let completed_task_ids: HashSet<Uuid> = self
                                    .tasks
                                    .iter()
                                    .filter(|t| t.status == TaskStatus::Completed)
                                    .map(|t| t.id)
                                    .collect();
                                task.depends_on
                                    .iter()
                                    .all(|dep_id| completed_task_ids.contains(dep_id))
                            })
                    });

                    if has_executable_tasks {
                        self.transition_to(ProjectStatus::Active)?;
                    }
                }
            }
            _ => {} // No auto-transitions for other states
        }

        Ok(())
    }

    /// Get project statistics
    pub fn get_statistics(&self) -> ProjectStatistics {
        let total_tasks = self.tasks.len();
        let completed_tasks = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let in_progress_tasks = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();
        let blocked_tasks = self.get_blocked_tasks().len();
        let ready_tasks = self.get_ready_tasks().len();
        let unassigned_tasks = self
            .tasks
            .iter()
            .filter(|t| t.assigned_to.is_none())
            .count();

        let total_issues = self.issues.len();
        let open_issues = self.get_open_issues().len();
        let unsynced_issues = self.get_unsynced_issues().len();
        let critical_issues = self
            .issues
            .iter()
            .filter(|i| i.is_active() && i.labels.contains(&"critical".to_string()))
            .count();

        let total_agents = self.agents.len();
        let available_agents = self.get_available_agents().len();
        let working_agents = self.get_working_agents().len();
        let error_agents = self
            .get_agents_by_status(crate::enums::AgentStatus::Error)
            .len();

        // Pull request metrics
        let total_pull_requests = self.get_all_pull_requests().len();
        let open_pull_requests = self.get_open_pull_requests().len();
        let merged_pull_requests = self.get_merged_pull_requests().len();
        let unsynced_pull_requests = self.get_unsynced_pull_requests().len();

        let dependency_urls_count = self
            .dependencies_urls
            .as_ref()
            .map(|urls| urls.len())
            .unwrap_or(0);

        ProjectStatistics {
            total_tasks,
            completed_tasks,
            in_progress_tasks,
            blocked_tasks,
            ready_tasks,
            unassigned_tasks,
            completion_percentage: self.completion_percentage(),
            total_issues,
            open_issues,
            unsynced_issues,
            critical_issues,
            total_agents,
            available_agents,
            working_agents,
            error_agents,
            total_pull_requests,
            open_pull_requests,
            merged_pull_requests,
            unsynced_pull_requests,
            health_score: self.get_health_score(),
            dependency_urls_count,
        }
    }

    // ===== ISSUE MANAGEMENT =====

    /// Add an issue to the project
    pub fn add_issue(&mut self, issue: Issue) -> Result<()> {
        // Check if issue ID already exists
        if self.issues.iter().any(|i| i.id == issue.id) {
            return Err(OrchestratorError::validation(
                "Issue with this ID already exists",
            ));
        }

        self.issues.push(issue);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove an issue from the project
    pub fn remove_issue(&mut self, issue_id: Uuid) -> Result<Issue> {
        let position = self
            .issues
            .iter()
            .position(|i| i.id == issue_id)
            .ok_or_else(|| OrchestratorError::validation("Issue not found"))?;

        let removed_issue = self.issues.remove(position);
        self.updated_at = Utc::now();
        Ok(removed_issue)
    }

    /// Get an issue by ID
    pub fn get_issue(&self, issue_id: Uuid) -> Option<&Issue> {
        self.issues.iter().find(|i| i.id == issue_id)
    }

    /// Get a mutable reference to an issue by ID
    pub fn get_issue_mut(&mut self, issue_id: Uuid) -> Option<&mut Issue> {
        self.issues.iter_mut().find(|i| i.id == issue_id)
    }

    /// Get all open issues
    pub fn get_open_issues(&self) -> Vec<&Issue> {
        self.issues.iter().filter(|i| i.is_active()).collect()
    }

    /// Get all closed issues
    pub fn get_closed_issues(&self) -> Vec<&Issue> {
        self.issues.iter().filter(|i| !i.is_active()).collect()
    }

    /// Get all unsynced issues
    pub fn get_unsynced_issues(&self) -> Vec<&Issue> {
        self.issues.iter().filter(|i| i.needs_sync()).collect()
    }

    /// Mark all issues as synced
    pub fn mark_all_issues_synced(&mut self) {
        for issue in &mut self.issues {
            issue.mark_synced();
        }
        self.updated_at = Utc::now();
    }

    // ===== AGENT MANAGEMENT =====

    /// Add an agent to the project
    pub fn add_agent(&mut self, agent: Agent) -> Result<()> {
        // Check if agent ID already exists
        if self.agents.iter().any(|a| a.id == agent.id) {
            return Err(OrchestratorError::validation(
                "Agent with this ID already exists",
            ));
        }

        self.agents.push(agent);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove an agent from the project
    pub fn remove_agent(&mut self, agent_id: Uuid) -> Result<Agent> {
        // Check if agent is assigned to any tasks
        let assigned_tasks: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|task| task.assigned_to.as_ref().map(|a| a.id) == Some(agent_id))
            .collect();

        if !assigned_tasks.is_empty() {
            let task_titles: Vec<String> = assigned_tasks.iter().map(|t| t.title.clone()).collect();
            return Err(OrchestratorError::validation(format!(
                "Cannot remove agent: assigned to tasks: {}",
                task_titles.join(", ")
            )));
        }

        let position = self
            .agents
            .iter()
            .position(|a| a.id == agent_id)
            .ok_or_else(|| OrchestratorError::validation("Agent not found"))?;

        let removed_agent = self.agents.remove(position);
        self.updated_at = Utc::now();
        Ok(removed_agent)
    }

    /// Get an agent by ID
    pub fn get_agent(&self, agent_id: Uuid) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == agent_id)
    }

    /// Get a mutable reference to an agent by ID
    pub fn get_agent_mut(&mut self, agent_id: Uuid) -> Option<&mut Agent> {
        self.agents.iter_mut().find(|a| a.id == agent_id)
    }

    /// Get all available agents (idle or active)
    pub fn get_available_agents(&self) -> Vec<&Agent> {
        self.agents
            .iter()
            .filter(|a| a.status.is_available())
            .collect()
    }

    /// Get all working agents (working or busy)
    pub fn get_working_agents(&self) -> Vec<&Agent> {
        self.agents
            .iter()
            .filter(|a| a.status.is_working())
            .collect()
    }

    /// Get agents by status
    pub fn get_agents_by_status(&self, status: crate::enums::AgentStatus) -> Vec<&Agent> {
        self.agents.iter().filter(|a| a.status == status).collect()
    }

    // ===== TASK ASSIGNMENT =====

    /// Assign a task to an agent
    pub fn assign_task_to_agent(&mut self, task_id: Uuid, agent_id: Uuid) -> Result<()> {
        // Find the agent and clone it
        let agent = self
            .agents
            .iter()
            .find(|a| a.id == agent_id)
            .ok_or_else(|| OrchestratorError::validation("Agent not found"))?
            .clone();

        // Check if agent is available
        if !agent.status.is_available() {
            return Err(OrchestratorError::validation(format!(
                "Agent '{}' is not available (status: {:?})",
                agent.name, agent.status
            )));
        }

        // Check if task is ready (dependencies completed) before finding the task
        let ready_task_ids: HashSet<Uuid> = self.get_ready_tasks().iter().map(|t| t.id).collect();
        let independent_task_ids: HashSet<Uuid> =
            self.get_independent_tasks().iter().map(|t| t.id).collect();

        if !ready_task_ids.contains(&task_id) && !independent_task_ids.contains(&task_id) {
            return Err(OrchestratorError::validation(
                "Task is not ready for assignment (dependencies not completed)",
            ));
        }

        // Find the task
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| OrchestratorError::validation("Task not found"))?;

        // Check if task is already assigned
        if task.assigned_to.is_some() {
            return Err(OrchestratorError::validation(
                "Task is already assigned to an agent",
            ));
        }

        // Assign the agent to the task
        task.assigned_to = Some(agent);
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Unassign a task from its current agent
    pub fn unassign_task(&mut self, task_id: Uuid) -> Result<()> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| OrchestratorError::validation("Task not found"))?;

        if task.assigned_to.is_none() {
            return Err(OrchestratorError::validation(
                "Task is not assigned to any agent",
            ));
        }

        task.assigned_to = None;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Reassign a task from one agent to another
    pub fn reassign_task(&mut self, task_id: Uuid, new_agent_id: Uuid) -> Result<()> {
        // Only unassign if the task is currently assigned
        let task = self
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| OrchestratorError::validation("Task not found"))?;

        if task.assigned_to.is_some() {
            self.unassign_task(task_id)?;
        }

        self.assign_task_to_agent(task_id, new_agent_id)?;
        Ok(())
    }

    /// Get all tasks assigned to a specific agent
    pub fn get_tasks_for_agent(&self, agent_id: Uuid) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| task.assigned_to.as_ref().map(|a| a.id) == Some(agent_id))
            .collect()
    }

    /// Get all unassigned tasks that are ready for assignment
    pub fn get_unassigned_ready_tasks(&self) -> Vec<&Task> {
        self.get_ready_tasks()
            .into_iter()
            .filter(|task| task.assigned_to.is_none())
            .collect()
    }

    /// Auto-assign ready tasks to available agents (simple round-robin)
    pub fn auto_assign_tasks(&mut self) -> Result<Vec<(Uuid, Uuid)>> {
        let mut assignments = Vec::new();
        let unassigned_tasks: Vec<Uuid> = self
            .get_unassigned_ready_tasks()
            .into_iter()
            .map(|t| t.id)
            .collect();

        let available_agents: Vec<Uuid> = self
            .get_available_agents()
            .into_iter()
            .map(|a| a.id)
            .collect();

        if available_agents.is_empty() {
            return Ok(assignments);
        }

        let mut agent_index = 0;
        for task_id in unassigned_tasks {
            let agent_id = available_agents[agent_index % available_agents.len()];

            if let Ok(()) = self.assign_task_to_agent(task_id, agent_id) {
                assignments.push((task_id, agent_id));
                agent_index += 1;
            }
        }

        Ok(assignments)
    }

    // ===== DEPENDENCY URL MANAGEMENT =====

    /// Add a dependency URL to the project
    pub fn add_dependency_url(&mut self, url: impl Into<String>) -> Result<()> {
        let url_string = url.into();

        // Validate URL format (basic check)
        if !url_string.starts_with("http://") && !url_string.starts_with("https://") {
            return Err(OrchestratorError::validation("Invalid URL format"));
        }

        if self.dependencies_urls.is_none() {
            self.dependencies_urls = Some(Vec::new());
        }

        let urls = self.dependencies_urls.as_mut().unwrap();

        // Check if URL already exists
        if urls.contains(&url_string) {
            return Err(OrchestratorError::validation(
                "Dependency URL already exists",
            ));
        }

        urls.push(url_string);
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Remove a dependency URL from the project
    pub fn remove_dependency_url(&mut self, url: &str) -> Result<()> {
        let urls = self
            .dependencies_urls
            .as_mut()
            .ok_or_else(|| OrchestratorError::validation("No dependency URLs configured"))?;

        let position = urls
            .iter()
            .position(|u| u == url)
            .ok_or_else(|| OrchestratorError::validation("Dependency URL not found"))?;

        urls.remove(position);
        self.updated_at = Utc::now();

        // Remove the Vec if it's empty
        if urls.is_empty() {
            self.dependencies_urls = None;
        }

        Ok(())
    }

    /// Update all dependency URLs (replace the entire list)
    pub fn update_dependency_urls(&mut self, urls: Vec<String>) -> Result<()> {
        // Validate all URLs
        for url in &urls {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(OrchestratorError::validation(format!(
                    "Invalid URL format: {}",
                    url
                )));
            }
        }

        // Check for duplicates
        let mut unique_urls = HashSet::new();
        for url in &urls {
            if !unique_urls.insert(url) {
                return Err(OrchestratorError::validation(format!(
                    "Duplicate URL found: {}",
                    url
                )));
            }
        }

        self.dependencies_urls = if urls.is_empty() { None } else { Some(urls) };
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Get all dependency URLs
    pub fn get_dependency_urls(&self) -> Vec<&String> {
        self.dependencies_urls
            .as_ref()
            .map(|urls| urls.iter().collect())
            .unwrap_or_default()
    }

    /// Check if a dependency URL exists
    pub fn has_dependency_url(&self, url: &str) -> bool {
        self.dependencies_urls
            .as_ref()
            .map(|urls| urls.contains(&url.to_string()))
            .unwrap_or(false)
    }

    // ===== UTILITY METHODS =====

    /// Get a task by ID
    pub fn get_task(&self, task_id: Uuid) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == task_id)
    }

    /// Get a mutable reference to a task by ID
    pub fn get_task_mut(&mut self, task_id: Uuid) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == task_id)
    }

    /// Get all tasks with a specific status
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.status == status).collect()
    }

    /// Get project workload distribution (tasks per agent)
    pub fn get_workload_distribution(&self) -> HashMap<Uuid, usize> {
        let mut distribution = HashMap::new();

        for task in &self.tasks {
            if let Some(agent) = &task.assigned_to {
                *distribution.entry(agent.id).or_insert(0) += 1;
            }
        }

        distribution
    }

    /// Get the most loaded agent (agent with most assigned tasks)
    pub fn get_most_loaded_agent(&self) -> Option<(&Agent, usize)> {
        let distribution = self.get_workload_distribution();

        distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .and_then(|(agent_id, count)| self.get_agent(*agent_id).map(|agent| (agent, *count)))
    }

    /// Get the least loaded agent (agent with fewest assigned tasks)
    pub fn get_least_loaded_agent(&self) -> Option<(&Agent, usize)> {
        let distribution = self.get_workload_distribution();

        // Include agents with no tasks
        let mut min_agent = None;
        let mut min_count = usize::MAX;

        for agent in &self.agents {
            let count = distribution.get(&agent.id).copied().unwrap_or(0);
            if count < min_count {
                min_count = count;
                min_agent = Some((agent, count));
            }
        }

        min_agent
    }

    /// Check if the project has any critical issues (issues labeled as critical)
    pub fn has_critical_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.is_active() && issue.labels.contains(&"critical".to_string()))
    }

    /// Get project health score (0-100 based on various metrics)
    pub fn get_health_score(&self) -> f64 {
        if self.tasks.is_empty() {
            return 100.0;
        }

        let mut score = 100.0;

        // Deduct points for blocked tasks
        let blocked_ratio = self.get_blocked_tasks().len() as f64 / self.tasks.len() as f64;
        score -= blocked_ratio * 30.0;

        // Deduct points for critical issues
        if self.has_critical_issues() {
            score -= 20.0;
        }

        // Deduct points for agents in error state
        let error_agents = self
            .get_agents_by_status(crate::enums::AgentStatus::Error)
            .len();
        if !self.agents.is_empty() {
            let error_ratio = error_agents as f64 / self.agents.len() as f64;
            score -= error_ratio * 25.0;
        }

        // Deduct points for unassigned ready tasks
        let unassigned_ready = self.get_unassigned_ready_tasks().len();
        if unassigned_ready > 0 && !self.agents.is_empty() {
            score -= (unassigned_ready as f64 / self.tasks.len() as f64) * 15.0;
        }

        // Bonus points for high completion
        let completion = self.completion_percentage();
        if completion > 80.0 {
            score += (completion - 80.0) * 0.5;
        }

        score.max(0.0).min(100.0)
    }

    // ===== COMMENT MANAGEMENT =====

    /// Get all comments from all tasks, issues, and pull requests
    pub fn get_all_comments(&self) -> Vec<&Comment> {
        let mut comments = Vec::new();

        // Collect task comments
        for task in &self.tasks {
            comments.extend(task.get_all_comments());
        }

        // Collect issue comments
        for issue in &self.issues {
            comments.extend(issue.comments.iter());
        }

        comments
    }

    /// Get all unsynced comments from tasks and issues
    pub fn get_all_unsynced_comments(&self) -> Vec<&Comment> {
        self.get_all_comments()
            .into_iter()
            .filter(|c| c.needs_sync())
            .collect()
    }

    /// Get comments by type
    pub fn get_comments_by_type(&self, comment_type: CommentType) -> Vec<&Comment> {
        self.get_all_comments()
            .into_iter()
            .filter(|c| c.comment_type == comment_type)
            .collect()
    }

    /// Get comments by author
    pub fn get_comments_by_author(&self, author: &str) -> Vec<&Comment> {
        self.get_all_comments()
            .into_iter()
            .filter(|c| c.author == author)
            .collect()
    }

    /// Mark all comments as synced across the entire project
    pub fn mark_all_comments_synced(&mut self) {
        for task in &mut self.tasks {
            task.mark_all_comments_synced();
            // Also mark PR comments as synced
            if let Some(pr) = &mut task.pull_request {
                pr.mark_all_comments_synced();
            }
        }

        for issue in &mut self.issues {
            issue.mark_all_comments_synced();
        }

        self.updated_at = Utc::now();
    }

    /// Mark everything as synced (issues, pull requests, and all comments)
    pub fn mark_all_synced(&mut self) {
        self.mark_all_issues_synced();
        self.mark_all_pull_requests_synced();
        self.mark_all_comments_synced();
        self.updated_at = Utc::now();
    }

    // ===== PULL REQUEST MANAGEMENT =====

    /// Get all pull requests from tasks
    pub fn get_all_pull_requests(&self) -> Vec<&PullRequest> {
        self.tasks
            .iter()
            .filter_map(|task| task.pull_request.as_ref())
            .collect()
    }

    /// Get all unsynced pull requests
    pub fn get_unsynced_pull_requests(&self) -> Vec<&PullRequest> {
        self.get_all_pull_requests()
            .into_iter()
            .filter(|pr| pr.needs_sync())
            .collect()
    }

    /// Mark all pull requests as synced
    pub fn mark_all_pull_requests_synced(&mut self) {
        for task in &mut self.tasks {
            if let Some(pr) = &mut task.pull_request {
                pr.mark_synced();
            }
        }
        self.updated_at = Utc::now();
    }

    /// Get pull requests by status
    pub fn get_pull_requests_by_status(&self, status: CodeStatus) -> Vec<&PullRequest> {
        self.get_all_pull_requests()
            .into_iter()
            .filter(|pr| pr.code_status == status)
            .collect()
    }

    /// Get open pull requests
    pub fn get_open_pull_requests(&self) -> Vec<&PullRequest> {
        self.get_all_pull_requests()
            .into_iter()
            .filter(|pr| pr.is_open())
            .collect()
    }

    /// Get merged pull requests
    pub fn get_merged_pull_requests(&self) -> Vec<&PullRequest> {
        self.get_all_pull_requests()
            .into_iter()
            .filter(|pr| pr.is_merged())
            .collect()
    }

    /// Get comment statistics
    pub fn get_comment_statistics(&self) -> CommentStatistics {
        let all_comments = self.get_all_comments();
        let total_comments = all_comments.len();
        let unsynced_comments = all_comments.iter().filter(|c| c.needs_sync()).count();

        let task_comments = all_comments
            .iter()
            .filter(|c| c.comment_type == CommentType::Task)
            .count();
        let issue_comments = all_comments
            .iter()
            .filter(|c| c.comment_type == CommentType::Issue)
            .count();
        let pr_comments = all_comments
            .iter()
            .filter(|c| c.comment_type == CommentType::PullRequest)
            .count();

        CommentStatistics {
            total_comments,
            unsynced_comments,
            task_comments,
            issue_comments,
            pr_comments,
            sync_percentage: if total_comments > 0 {
                ((total_comments - unsynced_comments) as f64 / total_comments as f64) * 100.0
            } else {
                100.0
            },
        }
    }

    /// Find comment by ID across all tasks and issues
    pub fn find_comment(&self, comment_id: Uuid) -> Option<(&Comment, CommentLocation)> {
        // Search in tasks
        for (task_index, task) in self.tasks.iter().enumerate() {
            if let Some(comment) = task.get_comment(comment_id) {
                return Some((comment, CommentLocation::Task(task_index, task.id)));
            }
        }

        // Search in issues
        for (issue_index, issue) in self.issues.iter().enumerate() {
            if let Some(comment) = issue.get_comment(comment_id) {
                return Some((comment, CommentLocation::Issue(issue_index, issue.id)));
            }
        }

        None
    }

    /// Update a comment anywhere in the project
    pub fn update_comment_anywhere(
        &mut self,
        comment_id: Uuid,
        new_content: impl Into<String>,
    ) -> Result<()> {
        // Try to find and update in tasks
        for task in &mut self.tasks {
            if task.get_comment(comment_id).is_some() {
                return task.update_comment(comment_id, new_content);
            }
        }

        // Try to find and update in issues
        for issue in &mut self.issues {
            if issue.get_comment(comment_id).is_some() {
                return issue.update_comment(comment_id, new_content);
            }
        }

        Err(OrchestratorError::validation(
            "Comment not found in project",
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub in_progress_tasks: usize,
    pub blocked_tasks: usize,
    pub ready_tasks: usize,
    pub unassigned_tasks: usize,
    pub completion_percentage: f64,
    pub total_issues: usize,
    pub open_issues: usize,
    pub unsynced_issues: usize,
    pub critical_issues: usize,
    pub total_agents: usize,
    pub available_agents: usize,
    pub working_agents: usize,
    pub error_agents: usize,
    pub total_pull_requests: usize,
    pub open_pull_requests: usize,
    pub merged_pull_requests: usize,
    pub unsynced_pull_requests: usize,
    pub health_score: f64,
    pub dependency_urls_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentStatistics {
    pub total_comments: usize,
    pub unsynced_comments: usize,
    pub task_comments: usize,
    pub issue_comments: usize,
    pub pr_comments: usize,
    pub sync_percentage: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommentLocation {
    Task(usize, Uuid),  // (index, task_id)
    Issue(usize, Uuid), // (index, issue_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Priority;

    fn create_test_project(name: &str) -> Project {
        Project::new(name, "Test project idea", "/test/project/path")
    }

    fn create_test_task(title: &str, depends_on: Vec<Uuid>) -> Task {
        let mut task = Task::new(title, "Test description", Priority::Medium);
        task.depends_on = depends_on;
        task
    }

    #[test]
    fn test_project_creation() {
        let mut project = Project::new("Test Project", "A test project idea", "/path/to/project");
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.idea, "A test project idea");
        assert_eq!(project.project_path, "/path/to/project");
        assert_eq!(project.repository_url, None);
        assert_eq!(project.status, ProjectStatus::Planning);
        assert!(project.tasks.is_empty());

        // Test setting repository URL
        project.set_repository_url("https://github.com/test/repo");
        assert_eq!(project.repository_url, Some("https://github.com/test/repo".to_string()));
    }

    #[test]
    fn test_project_status_transitions() {
        let mut project = create_test_project("Test");

        // Planning -> Active
        assert!(project.transition_to(ProjectStatus::Active).is_ok());
        assert_eq!(project.status, ProjectStatus::Active);

        // Active -> OnHold
        assert!(project.transition_to(ProjectStatus::OnHold).is_ok());
        assert_eq!(project.status, ProjectStatus::OnHold);

        // OnHold -> Active
        assert!(project.transition_to(ProjectStatus::Active).is_ok());
        assert_eq!(project.status, ProjectStatus::Active);

        // Active -> Completed
        assert!(project.transition_to(ProjectStatus::Completed).is_ok());
        assert_eq!(project.status, ProjectStatus::Completed);

        // Completed -> Archived
        assert!(project.transition_to(ProjectStatus::Archived).is_ok());
        assert_eq!(project.status, ProjectStatus::Archived);
    }

    #[test]
    fn test_invalid_project_transitions() {
        let mut project = create_test_project("Test");

        // Cannot go from Planning to Completed
        assert!(project.transition_to(ProjectStatus::Completed).is_err());

        // Cannot go from Archived to anything
        project.status = ProjectStatus::Archived;
        assert!(project.transition_to(ProjectStatus::Active).is_err());
        assert!(project.transition_to(ProjectStatus::Planning).is_err());
    }

    #[test]
    fn test_independent_tasks() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        let task1 = create_test_task("Task 1", vec![]);
        let task2 = create_test_task("Task 2", vec![]);

        project.add_task(task1).unwrap();
        project.add_task(task2).unwrap();

        let independent = project.get_independent_tasks();
        assert_eq!(independent.len(), 2);

        let ready = project.get_ready_tasks();
        assert_eq!(ready.len(), 2);
    }

    #[test]
    fn test_task_dependencies() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        let task1 = create_test_task("Task 1", vec![]);
        let task1_id = task1.id;
        let task2 = create_test_task("Task 2", vec![task1_id]);
        let task2_id = task2.id;

        project.add_task(task1).unwrap();
        project.add_task(task2).unwrap();

        // Only task1 should be ready initially
        let ready = project.get_ready_tasks();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, task1_id);

        // Task2 should be blocked
        let blocked = project.get_blocked_tasks();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].id, task2_id);

        // Complete task1
        let task1_mut = project.tasks.iter_mut().find(|t| t.id == task1_id).unwrap();
        task1_mut.status = TaskStatus::Completed;

        // Now task2 should be ready
        let ready = project.get_ready_tasks();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, task2_id);

        let blocked = project.get_blocked_tasks();
        assert_eq!(blocked.len(), 0);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut project = create_test_project("Test");

        let task1 = create_test_task("Task 1", vec![]);
        let task1_id = task1.id;
        let task2 = create_test_task("Task 2", vec![task1_id]);
        let task2_id = task2.id;

        project.add_task(task1).unwrap();
        project.add_task(task2).unwrap();

        // Try to create a cycle: task1 depends on task2
        assert!(project.add_dependency(task1_id, task2_id).is_err());
    }

    #[test]
    fn test_execution_order() {
        let mut project = create_test_project("Test");

        let task1 = create_test_task("Task 1", vec![]);
        let task1_id = task1.id;
        let task2 = create_test_task("Task 2", vec![task1_id]);
        let task2_id = task2.id;
        let task3 = create_test_task("Task 3", vec![task1_id, task2_id]);
        let task3_id = task3.id;

        project.add_task(task1).unwrap();
        project.add_task(task2).unwrap();
        project.add_task(task3).unwrap();

        let order = project.get_execution_order().unwrap();

        // Task1 should come before Task2 and Task3
        let task1_pos = order.iter().position(|&id| id == task1_id).unwrap();
        let task2_pos = order.iter().position(|&id| id == task2_id).unwrap();
        let task3_pos = order.iter().position(|&id| id == task3_id).unwrap();

        assert!(task1_pos < task2_pos);
        assert!(task1_pos < task3_pos);
        assert!(task2_pos < task3_pos);
    }

    #[test]
    fn test_task_removal_with_dependents() {
        let mut project = create_test_project("Test");

        let task1 = create_test_task("Task 1", vec![]);
        let task1_id = task1.id;
        let task2 = create_test_task("Task 2", vec![task1_id]);

        project.add_task(task1).unwrap();
        project.add_task(task2).unwrap();

        // Should not be able to remove task1 because task2 depends on it
        assert!(project.remove_task(task1_id).is_err());
    }

    #[test]
    fn test_completion_percentage() {
        let mut project = create_test_project("Test");

        let task1 = create_test_task("Task 1", vec![]);
        let task1_id = task1.id;
        let task2 = create_test_task("Task 2", vec![]);

        project.add_task(task1).unwrap();
        project.add_task(task2).unwrap();

        assert_eq!(project.completion_percentage(), 0.0);

        // Complete one task
        let task1_mut = project.tasks.iter_mut().find(|t| t.id == task1_id).unwrap();
        task1_mut.status = TaskStatus::Completed;

        assert_eq!(project.completion_percentage(), 50.0);
    }

    #[test]
    fn test_auto_status_update() {
        let mut project = create_test_project("Test");

        let task1 = create_test_task("Task 1", vec![]);
        let task1_id = task1.id;

        project.add_task(task1).unwrap();

        // Should auto-transition to Active when there are ready tasks
        project.auto_update_status().unwrap();
        assert_eq!(project.status, ProjectStatus::Active);

        // Complete the task
        let task1_mut = project.tasks.iter_mut().find(|t| t.id == task1_id).unwrap();
        task1_mut.status = TaskStatus::Completed;

        // Should auto-transition to Completed when all tasks are done
        project.auto_update_status().unwrap();
        assert_eq!(project.status, ProjectStatus::Completed);
    }

    #[test]
    fn test_project_statistics() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        let task1 = create_test_task("Task 1", vec![]);
        let task1_id = task1.id;
        let task2 = create_test_task("Task 2", vec![task1_id]);

        project.add_task(task1).unwrap();
        project.add_task(task2).unwrap();

        let stats = project.get_statistics();
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.completed_tasks, 0);
        assert_eq!(stats.ready_tasks, 1); // Only task1 is ready
        assert_eq!(stats.blocked_tasks, 1); // Task2 is blocked
        assert_eq!(stats.completion_percentage, 0.0);
    }

    #[test]
    fn test_missing_dependency_validation() {
        let mut project = create_test_project("Test");

        let non_existent_id = Uuid::new_v4();
        let task = create_test_task("Task 1", vec![non_existent_id]);

        // Should fail to add task with non-existent dependency
        assert!(project.add_task(task).is_err());
    }

    // ===== ISSUE MANAGEMENT TESTS =====

    fn create_test_issue(title: &str) -> Issue {
        let task = create_test_task("Related Task", vec![]);
        let mut issue = Issue::from_task(&task);
        issue.title = title.to_string();
        issue.body = "Test description".to_string();
        issue
    }

    #[test]
    fn test_add_remove_issues() {
        let mut project = create_test_project("Test");

        let issue = create_test_issue("Test Issue");
        let issue_id = issue.id;

        // Add issue
        assert!(project.add_issue(issue).is_ok());
        assert_eq!(project.issues.len(), 1);
        assert!(project.get_issue(issue_id).is_some());

        // Remove issue
        let removed = project.remove_issue(issue_id).unwrap();
        assert_eq!(removed.title, "Test Issue");
        assert_eq!(project.issues.len(), 0);
    }

    #[test]
    fn test_duplicate_issue_prevention() {
        let mut project = create_test_project("Test");

        let issue1 = create_test_issue("Issue 1");
        let mut issue2 = create_test_issue("Issue 2");
        issue2.id = issue1.id; // Same ID

        project.add_issue(issue1).unwrap();
        assert!(project.add_issue(issue2).is_err());
    }

    // ===== AGENT MANAGEMENT TESTS =====

    fn create_test_agent(name: &str) -> Agent {
        Agent::new(
            name,
            std::path::PathBuf::from("/tmp/test.json"),
            "Test agent",
        )
    }

    #[test]
    fn test_add_remove_agents() {
        let mut project = create_test_project("Test");

        let agent = create_test_agent("Test Agent");
        let agent_id = agent.id;

        // Add agent
        assert!(project.add_agent(agent).is_ok());
        assert_eq!(project.agents.len(), 1);
        assert!(project.get_agent(agent_id).is_some());

        // Remove agent
        let removed = project.remove_agent(agent_id).unwrap();
        assert_eq!(removed.name, "Test Agent");
        assert_eq!(project.agents.len(), 0);
    }

    #[test]
    fn test_agent_removal_with_assigned_tasks() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        let agent = create_test_agent("Test Agent");
        let agent_id = agent.id;
        let task = create_test_task("Task 1", vec![]);
        let task_id = task.id;

        project.add_agent(agent).unwrap();
        project.add_task(task).unwrap();
        project.assign_task_to_agent(task_id, agent_id).unwrap();

        // Should not be able to remove agent with assigned tasks
        assert!(project.remove_agent(agent_id).is_err());
    }

    // ===== TASK ASSIGNMENT TESTS =====

    #[test]
    fn test_task_assignment() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        let agent = create_test_agent("Test Agent");
        let agent_id = agent.id;
        let task = create_test_task("Task 1", vec![]);
        let task_id = task.id;

        project.add_agent(agent).unwrap();
        project.add_task(task).unwrap();

        // Assign task to agent
        assert!(project.assign_task_to_agent(task_id, agent_id).is_ok());

        let assigned_task = project.get_task(task_id).unwrap();
        assert!(assigned_task.assigned_to.is_some());
        assert_eq!(assigned_task.assigned_to.as_ref().unwrap().id, agent_id);

        // Get tasks for agent
        let agent_tasks = project.get_tasks_for_agent(agent_id);
        assert_eq!(agent_tasks.len(), 1);
        assert_eq!(agent_tasks[0].id, task_id);
    }

    #[test]
    fn test_task_assignment_validation() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        let agent = create_test_agent("Test Agent");
        let agent_id = agent.id;
        let task = create_test_task("Task 1", vec![]);
        let task_id = task.id;

        project.add_agent(agent).unwrap();
        project.add_task(task).unwrap();

        // Assign task
        project.assign_task_to_agent(task_id, agent_id).unwrap();

        // Try to assign already assigned task
        assert!(project.assign_task_to_agent(task_id, agent_id).is_err());

        // Try to assign to non-existent agent
        let fake_agent_id = Uuid::new_v4();
        let task2 = create_test_task("Task 2", vec![]);
        let task2_id = task2.id;
        project.add_task(task2).unwrap();
        assert!(
            project
                .assign_task_to_agent(task2_id, fake_agent_id)
                .is_err()
        );
    }

    #[test]
    fn test_task_unassignment_and_reassignment() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        let agent1 = create_test_agent("Agent 1");
        let agent1_id = agent1.id;
        let agent2 = create_test_agent("Agent 2");
        let agent2_id = agent2.id;
        let task = create_test_task("Task 1", vec![]);
        let task_id = task.id;

        project.add_agent(agent1).unwrap();
        project.add_agent(agent2).unwrap();
        project.add_task(task).unwrap();

        // Assign, unassign, reassign
        project.assign_task_to_agent(task_id, agent1_id).unwrap();
        project.unassign_task(task_id).unwrap();

        let task = project.get_task(task_id).unwrap();
        assert!(task.assigned_to.is_none());

        project.reassign_task(task_id, agent2_id).unwrap();
        let task = project.get_task(task_id).unwrap();
        assert_eq!(task.assigned_to.as_ref().unwrap().id, agent2_id);
    }

    #[test]
    fn test_auto_assignment() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        let agent1 = create_test_agent("Agent 1");
        let _agent1_id = agent1.id;
        let agent2 = create_test_agent("Agent 2");
        let _agent2_id = agent2.id;

        let task1 = create_test_task("Task 1", vec![]);
        let task1_id = task1.id;
        let task2 = create_test_task("Task 2", vec![]);
        let task2_id = task2.id;

        project.add_agent(agent1).unwrap();
        project.add_agent(agent2).unwrap();
        project.add_task(task1).unwrap();
        project.add_task(task2).unwrap();

        let assignments = project.auto_assign_tasks().unwrap();
        assert_eq!(assignments.len(), 2);

        // Check that tasks are assigned
        let task1 = project.get_task(task1_id).unwrap();
        let task2 = project.get_task(task2_id).unwrap();
        assert!(task1.assigned_to.is_some());
        assert!(task2.assigned_to.is_some());
    }

    // ===== DEPENDENCY URL TESTS =====

    #[test]
    fn test_dependency_url_management() {
        let mut project = create_test_project("Test");

        // Add URLs
        assert!(
            project
                .add_dependency_url("https://example.com/dep1")
                .is_ok()
        );
        assert!(
            project
                .add_dependency_url("https://example.com/dep2")
                .is_ok()
        );

        // Check URLs exist
        assert!(project.has_dependency_url("https://example.com/dep1"));
        assert!(project.has_dependency_url("https://example.com/dep2"));
        assert!(!project.has_dependency_url("https://example.com/dep3"));

        let urls = project.get_dependency_urls();
        assert_eq!(urls.len(), 2);

        // Remove URL
        assert!(
            project
                .remove_dependency_url("https://example.com/dep1")
                .is_ok()
        );
        assert!(!project.has_dependency_url("https://example.com/dep1"));
        assert_eq!(project.get_dependency_urls().len(), 1);
    }

    #[test]
    fn test_dependency_url_validation() {
        let mut project = create_test_project("Test");

        // Invalid URL format
        assert!(project.add_dependency_url("invalid-url").is_err());
        assert!(project.add_dependency_url("ftp://example.com").is_err());

        // Duplicate URL
        project.add_dependency_url("https://example.com").unwrap();
        assert!(project.add_dependency_url("https://example.com").is_err());
    }

    #[test]
    fn test_update_dependency_urls() {
        let mut project = create_test_project("Test");

        let urls = vec![
            "https://example.com/dep1".to_string(),
            "https://example.com/dep2".to_string(),
            "https://example.com/dep3".to_string(),
        ];

        assert!(project.update_dependency_urls(urls.clone()).is_ok());
        assert_eq!(project.get_dependency_urls().len(), 3);

        // Update with invalid URL
        let invalid_urls = vec!["invalid-url".to_string()];
        assert!(project.update_dependency_urls(invalid_urls).is_err());

        // Update with duplicates
        let duplicate_urls = vec![
            "https://example.com/dep1".to_string(),
            "https://example.com/dep1".to_string(),
        ];
        assert!(project.update_dependency_urls(duplicate_urls).is_err());

        // Clear URLs
        assert!(project.update_dependency_urls(vec![]).is_ok());
        assert_eq!(project.get_dependency_urls().len(), 0);
    }

    // ===== UTILITY METHOD TESTS =====

    #[test]
    fn test_workload_distribution() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        let agent1 = create_test_agent("Agent 1");
        let agent1_id = agent1.id;
        let agent2 = create_test_agent("Agent 2");
        let agent2_id = agent2.id;

        project.add_agent(agent1).unwrap();
        project.add_agent(agent2).unwrap();

        // Add tasks and assign them
        for i in 0..3 {
            let task = create_test_task(&format!("Task {}", i + 1), vec![]);
            let task_id = task.id;
            project.add_task(task).unwrap();

            let agent_id = if i < 2 { agent1_id } else { agent2_id };
            project.assign_task_to_agent(task_id, agent_id).unwrap();
        }

        let distribution = project.get_workload_distribution();
        assert_eq!(distribution.get(&agent1_id), Some(&2));
        assert_eq!(distribution.get(&agent2_id), Some(&1));

        // Test most/least loaded agents
        let (most_loaded, count) = project.get_most_loaded_agent().unwrap();
        assert_eq!(most_loaded.id, agent1_id);
        assert_eq!(count, 2);

        let (least_loaded, count) = project.get_least_loaded_agent().unwrap();
        assert_eq!(least_loaded.id, agent2_id);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_health_score() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        // Empty project should have perfect health
        assert_eq!(project.get_health_score(), 100.0);

        // Add some tasks and agents
        let agent = create_test_agent("Agent 1");
        let agent_id = agent.id;
        project.add_agent(agent).unwrap();

        let task1 = create_test_task("Task 1", vec![]);
        let task1_id = task1.id;
        let task2 = create_test_task("Task 2", vec![task1_id]);

        project.add_task(task1).unwrap();
        project.add_task(task2).unwrap();

        // Health should be lower due to unassigned tasks
        let health = project.get_health_score();
        assert!(health < 100.0);

        // Assign tasks to improve health
        project.assign_task_to_agent(task1_id, agent_id).unwrap();
        let new_health = project.get_health_score();
        assert!(new_health > health);
    }

    #[test]
    fn test_enhanced_statistics() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        // Add agents
        let agent1 = create_test_agent("Agent 1");
        let agent1_id = agent1.id;
        let mut agent2 = create_test_agent("Agent 2");
        agent2.status = crate::enums::AgentStatus::Error;

        project.add_agent(agent1).unwrap();
        project.add_agent(agent2).unwrap();

        // Add tasks
        let task = create_test_task("Task 1", vec![]);
        let task_id = task.id;
        project.add_task(task).unwrap();
        project.assign_task_to_agent(task_id, agent1_id).unwrap();

        // Add issues
        let mut issue = create_test_issue("Critical Issue");
        issue.labels.push("critical".to_string());
        project.add_issue(issue).unwrap();

        // Add dependency URLs
        project
            .add_dependency_url("https://example.com/dep1")
            .unwrap();
        project
            .add_dependency_url("https://example.com/dep2")
            .unwrap();

        let stats = project.get_statistics();

        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.unassigned_tasks, 0);
        assert_eq!(stats.total_agents, 2);
        assert_eq!(stats.available_agents, 1);
        assert_eq!(stats.error_agents, 1);
        assert_eq!(stats.total_issues, 1);
        assert_eq!(stats.dependency_urls_count, 2);
        assert!(stats.health_score > 0.0);
    }

    // ===== COMMENT MANAGEMENT TESTS =====

    #[test]
    fn test_project_comment_management() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        // Add task with comments
        let mut task = create_test_task("Task 1", vec![]);
        task.add_comment("user1", "Task comment 1");
        task.add_comment_with_sync("user2", "Task comment 2", true);
        project.add_task(task).unwrap();

        // Add issue with comments
        let mut issue = create_test_issue("Issue 1");
        issue.add_new_comment("user3", "Issue comment 1");
        issue.add_new_comment_with_sync("user4", "Issue comment 2", true);
        project.add_issue(issue).unwrap();

        // Test comment retrieval
        let all_comments = project.get_all_comments();
        assert_eq!(all_comments.len(), 4);

        let unsynced_comments = project.get_all_unsynced_comments();
        assert_eq!(unsynced_comments.len(), 2);

        let task_comments = project.get_comments_by_type(CommentType::Task);
        assert_eq!(task_comments.len(), 2);

        let issue_comments = project.get_comments_by_type(CommentType::Issue);
        assert_eq!(issue_comments.len(), 2);

        let user1_comments = project.get_comments_by_author("user1");
        assert_eq!(user1_comments.len(), 1);
    }

    #[test]
    fn test_project_comment_statistics() {
        let mut project = create_test_project("Test");

        // Add task with mixed sync status comments
        let mut task = create_test_task("Task 1", vec![]);
        task.add_comment("user1", "Unsynced comment");
        task.add_comment_with_sync("user2", "Synced comment", true);
        project.add_task(task).unwrap();

        let stats = project.get_comment_statistics();
        assert_eq!(stats.total_comments, 2);
        assert_eq!(stats.unsynced_comments, 1);
        assert_eq!(stats.task_comments, 2);
        assert_eq!(stats.issue_comments, 0);
        assert_eq!(stats.pr_comments, 0);
        assert_eq!(stats.sync_percentage, 50.0);
    }

    #[test]
    fn test_project_mark_all_comments_synced() {
        let mut project = create_test_project("Test");

        // Add task and issue with unsynced comments
        let mut task = create_test_task("Task 1", vec![]);
        task.add_comment("user1", "Task comment");
        project.add_task(task).unwrap();

        let mut issue = create_test_issue("Issue 1");
        issue.add_new_comment("user2", "Issue comment");
        project.add_issue(issue).unwrap();

        // Initially all comments should be unsynced
        assert_eq!(project.get_all_unsynced_comments().len(), 2);

        // Mark all as synced
        project.mark_all_comments_synced();

        // Now no comments should be unsynced
        assert_eq!(project.get_all_unsynced_comments().len(), 0);
    }

    #[test]
    fn test_project_find_and_update_comment() {
        let mut project = create_test_project("Test");

        let mut task = create_test_task("Task 1", vec![]);
        task.add_comment("user1", "Original content");
        let comment_id = task.comments[0].id;
        project.add_task(task).unwrap();

        // Find comment
        let (comment, location) = project.find_comment(comment_id).unwrap();
        assert_eq!(comment.content, "Original content");
        assert!(matches!(location, CommentLocation::Task(_, _)));

        // Update comment
        project
            .update_comment_anywhere(comment_id, "Updated content")
            .unwrap();

        // Verify update
        let (updated_comment, _) = project.find_comment(comment_id).unwrap();
        assert_eq!(updated_comment.content, "Updated content");
        assert!(updated_comment.needs_sync()); // Should be marked as unsynced
    }

    // ===== ISSUE SYNC TESTS =====

    #[test]
    fn test_issue_sync_management() {
        let mut project = create_test_project("Test");

        // Add issues
        let issue1 = create_test_issue("Issue 1");
        let mut issue2 = create_test_issue("Issue 2");

        // Mark one as synced
        issue2.mark_synced();

        project.add_issue(issue1).unwrap();
        project.add_issue(issue2).unwrap();

        // Test unsynced issues
        let unsynced = project.get_unsynced_issues();
        assert_eq!(unsynced.len(), 1);
        assert_eq!(unsynced[0].title, "Issue 1");

        // Mark all as synced
        project.mark_all_issues_synced();
        assert_eq!(project.get_unsynced_issues().len(), 0);
    }

    #[test]
    fn test_issue_modification_unsyncs() {
        let mut project = create_test_project("Test");

        let mut issue = create_test_issue("Test Issue");
        issue.mark_synced(); // Start as synced
        project.add_issue(issue).unwrap();

        let issue_id = project.issues[0].id;
        let issue = project.get_issue_mut(issue_id).unwrap();

        // Test various modifications unsync the issue
        assert!(!issue.needs_sync()); // Initially synced

        issue.update_title("New Title");
        assert!(issue.needs_sync());

        issue.mark_synced();
        issue.update_body("New Body");
        assert!(issue.needs_sync());

        issue.mark_synced();
        issue.update_assignee(Some("new_assignee".to_string()));
        assert!(issue.needs_sync());

        issue.mark_synced();
        issue.add_label("new_label");
        assert!(issue.needs_sync());

        issue.mark_synced();
        issue.remove_label("new_label");
        assert!(issue.needs_sync());

        issue.mark_synced();
        issue.set_github_issue_number(123);
        assert!(issue.needs_sync());
    }

    #[test]
    fn test_issue_status_change_unsyncs() {
        let mut project = create_test_project("Test");

        let mut issue = create_test_issue("Test Issue");
        issue.mark_synced();
        project.add_issue(issue).unwrap();

        let issue_id = project.issues[0].id;
        let issue = project.get_issue_mut(issue_id).unwrap();

        assert!(!issue.needs_sync()); // Initially synced

        // Assign the issue first
        issue.assignee = Some("user".to_string());

        // Status change should unsync
        issue
            .progress("user", Some("Starting work".to_string()))
            .unwrap();
        assert!(issue.needs_sync());
    }

    #[test]
    fn test_project_statistics_with_issue_sync() {
        let mut project = create_test_project("Test");

        // Add synced and unsynced issues
        let mut issue1 = create_test_issue("Synced Issue");
        issue1.mark_synced();
        let issue2 = create_test_issue("Unsynced Issue");

        project.add_issue(issue1).unwrap();
        project.add_issue(issue2).unwrap();

        let stats = project.get_statistics();
        assert_eq!(stats.total_issues, 2);
        assert_eq!(stats.unsynced_issues, 1);
    }

    #[test]
    fn test_project_mark_all_synced() {
        let mut project = create_test_project("Test");

        // Add unsynced issue and task with comments
        let mut issue = create_test_issue("Unsynced Issue");
        issue.add_new_comment("user", "Unsynced comment");
        project.add_issue(issue).unwrap();

        let mut task = create_test_task("Unsynced Task", vec![]);
        task.add_comment("user", "Unsynced comment");
        project.add_task(task).unwrap();

        // Initially everything should be unsynced
        assert_eq!(project.get_unsynced_issues().len(), 1);
        assert_eq!(project.get_all_unsynced_comments().len(), 2);

        // Mark everything as synced
        project.mark_all_synced();

        // Now everything should be synced
        assert_eq!(project.get_unsynced_issues().len(), 0);
        assert_eq!(project.get_all_unsynced_comments().len(), 0);
    }

    // ===== PULL REQUEST TESTS =====

    #[test]
    fn test_project_pull_request_management() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        // Create task with pull request
        let mut task = create_test_task("Task with PR", vec![]);
        let agent = create_test_agent("Agent 1");
        task.assigned_to = Some(agent);
        task.transition_task_status(TaskStatus::InProgress).unwrap();
        task.create_pull_request("Fix: Task", "Description", "feature/task", "main", "dev")
            .unwrap();

        project.add_task(task).unwrap();

        // Test PR retrieval
        let all_prs = project.get_all_pull_requests();
        assert_eq!(all_prs.len(), 1);

        let open_prs = project.get_open_pull_requests();
        assert_eq!(open_prs.len(), 1);

        let unsynced_prs = project.get_unsynced_pull_requests();
        assert_eq!(unsynced_prs.len(), 1);

        // Mark all PRs as synced
        project.mark_all_pull_requests_synced();
        assert_eq!(project.get_unsynced_pull_requests().len(), 0);
    }

    #[test]
    fn test_project_statistics_with_pull_requests() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        // Create task with pull request
        let mut task = create_test_task("Task with PR", vec![]);
        let agent = create_test_agent("Agent 1");
        task.assigned_to = Some(agent);
        task.transition_task_status(TaskStatus::InProgress).unwrap();
        task.create_pull_request("Fix: Task", "Description", "feature/task", "main", "dev")
            .unwrap();

        project.add_task(task).unwrap();

        let stats = project.get_statistics();
        assert_eq!(stats.total_pull_requests, 1);
        assert_eq!(stats.open_pull_requests, 1);
        assert_eq!(stats.merged_pull_requests, 0);
        assert_eq!(stats.unsynced_pull_requests, 1);
    }

    #[test]
    fn test_project_mark_all_synced_with_prs() {
        let mut project = create_test_project("Test");
        project.transition_to(ProjectStatus::Active).unwrap();

        // Create task with pull request and comments
        let mut task = create_test_task("Task with PR", vec![]);
        let agent = create_test_agent("Agent 1");
        task.assigned_to = Some(agent);
        task.transition_task_status(TaskStatus::InProgress).unwrap();
        task.create_pull_request("Fix: Task", "Description", "feature/task", "main", "dev")
            .unwrap();
        task.add_pr_comment("user", "Great work!").unwrap();

        project.add_task(task).unwrap();

        // Add unsynced issue
        let issue = create_test_issue("Unsynced Issue");
        project.add_issue(issue).unwrap();

        // Initially everything should be unsynced
        assert_eq!(project.get_unsynced_pull_requests().len(), 1);
        assert_eq!(project.get_unsynced_issues().len(), 1);
        assert_eq!(project.get_all_unsynced_comments().len(), 1);

        // Mark everything as synced
        project.mark_all_synced();

        // Now everything should be synced
        assert_eq!(project.get_unsynced_pull_requests().len(), 0);
        assert_eq!(project.get_unsynced_issues().len(), 0);
        assert_eq!(project.get_all_unsynced_comments().len(), 0);
    }
}
