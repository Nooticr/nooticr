use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::agent_status_change::AgentStatusChange;
use crate::enums::AgentStatus;
use crate::error::{Result, OrchestratorError};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub file_path: PathBuf,
    pub description: String,
    pub status: AgentStatus,
    pub status_history: Vec<AgentStatusChange>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub error_count: u32,
    pub total_tasks_completed: u64,
}

impl Agent {
    /// Create a new agent
    pub fn new(name: impl Into<String>, file_path: PathBuf, description: impl Into<String>) -> Self {
        let now = Utc::now();
        let initial_status = AgentStatus::default();
        
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            file_path,
            description: description.into(),
            status: initial_status,
            status_history: vec![AgentStatusChange {
                from: None,
                to: initial_status,
                reason: Some("Agent created".to_string()),
                timestamp: now,
            }],
            created_at: now,
            updated_at: now,
            last_active_at: None,
            error_count: 0,
            total_tasks_completed: 0,
        }
    }
    
    /// Transition the agent to a new status with validation
    pub fn transition_to(&mut self, new_status: AgentStatus, reason: Option<String>) -> Result<()> {
        // Validate the transition
        let next_status = self.status.transition_to(new_status)?;
        let now = Utc::now();

        // Apply business rules
        match (&self.status, &next_status) {
            // Track when agent becomes active
            (_, AgentStatus::Active | AgentStatus::Working) => {
                self.last_active_at = Some(now);
            }
            // Increment error count
            (_, AgentStatus::Error) => {
                self.error_count += 1;
            }
            // Reset error count after maintenance
            (AgentStatus::Maintenance, AgentStatus::Idle) => {
                self.error_count = 0;
            }
            _ => {}
        }

        // Record the change
        self.status_history.push(AgentStatusChange {
            from: Some(self.status),
            to: next_status,
            reason,
            timestamp: now,
        });

        self.status = next_status;
        self.updated_at = now;

        Ok(())
    }
    
    /// Start working on a task
    pub fn start_work(&mut self, task_description: impl Into<String>) -> Result<()> {
        if !self.status.is_available() {
            return Err(OrchestratorError::agent_constraint(
                format!("Agent is not available. Current status: {}", self.status)
            ));
        }

        self.transition_to(AgentStatus::Working, Some(format!("Started: {}", task_description.into())))
    }

    /// Complete current work and become active
    pub fn complete_work(&mut self) -> Result<()> {
        if self.status != AgentStatus::Working {
            return Err(OrchestratorError::agent_constraint(
                "Agent is not currently working"
            ));
        }

        self.total_tasks_completed += 1;
        self.transition_to(AgentStatus::Active, Some("Work completed".to_string()))
    }

    /// Mark agent as busy (handling multiple tasks)
    pub fn mark_busy(&mut self, reason: impl Into<String>) -> Result<()> {
        if !matches!(self.status, AgentStatus::Working | AgentStatus::Active) {
            return Err(OrchestratorError::agent_constraint(
                "Can only become busy from Working or Active state"
            ));
        }

        self.transition_to(AgentStatus::Busy, Some(reason.into()))
    }
    
    /// Report an error
    pub fn report_error(&mut self, error_description: impl Into<String>) -> Result<()> {
        if self.status == AgentStatus::Maintenance {
            return Err(OrchestratorError::agent_constraint(
                "Cannot report error during maintenance"
            ));
        }

        self.transition_to(AgentStatus::Error, Some(error_description.into()))
    }

    /// Start maintenance
    pub fn start_maintenance(&mut self, reason: impl Into<String>) -> Result<()> {
        if !matches!(self.status, AgentStatus::Idle | AgentStatus::Error) {
            return Err(OrchestratorError::agent_constraint(
                "Can only start maintenance from Idle or Error state"
            ));
        }

        self.transition_to(AgentStatus::Maintenance, Some(reason.into()))
    }

    /// Complete maintenance and return to idle
    pub fn complete_maintenance(&mut self) -> Result<()> {
        if self.status != AgentStatus::Maintenance {
            return Err(OrchestratorError::agent_constraint(
                "Agent is not in maintenance"
            ));
        }

        self.transition_to(AgentStatus::Idle, Some("Maintenance completed".to_string()))
    }

    /// Go idle
    pub fn go_idle(&mut self, reason: Option<String>) -> Result<()> {
        if !matches!(self.status, AgentStatus::Active | AgentStatus::Busy | AgentStatus::Error) {
            return Err(OrchestratorError::agent_constraint(
                "Cannot go idle from current state"
            ));
        }

        self.transition_to(AgentStatus::Idle, reason.or_else(|| Some("Going idle".to_string())))
    }
    
    /// Get uptime (time since creation)
    pub fn uptime(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }
    
    /// Get time in current status
    pub fn time_in_current_status(&self) -> chrono::Duration {
        if let Some(last_change) = self.status_history.last() {
            Utc::now() - last_change.timestamp
        } else {
            chrono::Duration::zero()
        }
    }
    
    /// Get total time in a specific status
    pub fn total_time_in_status(&self, status: AgentStatus) -> chrono::Duration {
        let mut total = chrono::Duration::zero();
        let mut in_status = false;
        let mut start_time = None;
        
        for change in &self.status_history {
            if change.to == status {
                in_status = true;
                start_time = Some(change.timestamp);
            } else if in_status && change.from == Some(status) {
                if let Some(start) = start_time {
                    total = total + (change.timestamp - start);
                }
                in_status = false;
                start_time = None;
            }
        }
        
        // If still in the status, add time until now
        if in_status {
            if let Some(start) = start_time {
                total = total + (Utc::now() - start);
            }
        }
        
        total
    }
    
    /// Get agent health score (0-100)
    pub fn health_score(&self) -> u8 {
        let base_score = 100u8;
        let error_penalty = (self.error_count * 10).min(50) as u8;
        let status_penalty = match self.status {
            AgentStatus::Error => 30,
            AgentStatus::Maintenance => 20,
            AgentStatus::Busy => 5,
            _ => 0,
        };
        
        base_score.saturating_sub(error_penalty).saturating_sub(status_penalty)
    }
    
    /// Save agent state to file
    pub async fn save_state(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(&self.file_path).await?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }
    
    /// Load agent state from file
    pub async fn load_state(file_path: PathBuf) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let contents = tokio::fs::read_to_string(&file_path).await?;
        let agent: Agent = serde_json::from_str(&contents)?;
        Ok(agent)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("TestAgent", PathBuf::from("/tmp/agent.json"), "Test agent");
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.error_count, 0);
        assert_eq!(agent.total_tasks_completed, 0);
    }
    
    #[test]
    fn test_normal_workflow() {
        let mut agent = Agent::new("Worker", PathBuf::from("/tmp/worker.json"), "Worker agent");

        // Start work
        agent.start_work("Process data").expect("Should be able to start work");
        assert_eq!(agent.status, AgentStatus::Working);
        assert!(agent.last_active_at.is_some());

        // Complete work
        agent.complete_work().expect("Should be able to complete work");
        assert_eq!(agent.status, AgentStatus::Active);
        assert_eq!(agent.total_tasks_completed, 1);

        // Go idle
        agent.go_idle(None).expect("Should be able to go idle");
        assert_eq!(agent.status, AgentStatus::Idle);
    }
    
    #[test]
    fn test_error_handling() {
        let mut agent = Agent::new("ErrorAgent", PathBuf::from("/tmp/error.json"), "Test error");

        agent.start_work("Risky task").expect("Should be able to start work");
        agent.report_error("Task failed").expect("Should be able to report error");
        assert_eq!(agent.status, AgentStatus::Error);
        assert_eq!(agent.error_count, 1);

        // Start maintenance
        agent.start_maintenance("Fixing errors").expect("Should be able to start maintenance");
        assert_eq!(agent.status, AgentStatus::Maintenance);

        // Complete maintenance
        agent.complete_maintenance().expect("Should be able to complete maintenance");
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.error_count, 0); // Reset after maintenance
    }
    
    #[test]
    fn test_busy_state() {
        let mut agent = Agent::new("BusyAgent", PathBuf::from("/tmp/busy.json"), "Busy agent");

        agent.start_work("Task 1").expect("Should be able to start work");
        agent.mark_busy("Handling multiple requests").expect("Should be able to mark busy");
        assert_eq!(agent.status, AgentStatus::Busy);

        // Can go back to active
        agent.transition_to(AgentStatus::Active, Some("Load reduced".to_string())).expect("Should be able to transition to active");
        assert_eq!(agent.status, AgentStatus::Active);
    }
    
    #[test]
    fn test_invalid_transitions() {
        let mut agent = Agent::new("TestAgent", PathBuf::from("/tmp/test.json"), "Test");
        
        // Cannot go from Idle to Active directly
        assert!(agent.transition_to(AgentStatus::Active, None).is_err());
        
        // Cannot go from Idle to Error
        assert!(agent.transition_to(AgentStatus::Error, None).is_err());
        
        // Cannot work during maintenance
        agent.start_maintenance("Updates").expect("Should be able to start maintenance");
        assert!(agent.start_work("Task").is_err());
    }
    
    #[test]
    fn test_health_score() {
        let mut agent = Agent::new("HealthAgent", PathBuf::from("/tmp/health.json"), "Health test");
        
        // Perfect health initially
        assert_eq!(agent.health_score(), 100);
        
        // Errors reduce health
        agent.start_work("Task").expect("Should be able to start work");
        agent.report_error("Failed").expect("Should be able to report error");
        agent.go_idle(None).expect("Should be able to go idle");
        agent.start_work("Task2").expect("Should be able to start work again");
        agent.report_error("Failed again").expect("Should be able to report error again");
        
        // 2 errors = -20, Error status = -30, Total = 50
        assert_eq!(agent.health_score(), 50);
    }
}