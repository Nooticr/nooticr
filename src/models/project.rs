use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::task::Task;
use chrono::{DateTime, Utc};
use super::issue::Issue;
use super::agent::Agent;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub repository_url: String,
    pub tasks: Vec<Task>,
    pub issues: Vec<Issue>,
    pub agents: Vec<Agent>,
    pub tasks_history: Vec<(Task, DateTime<Utc>)>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dependencies_urls: Option<Vec<String>>,
}