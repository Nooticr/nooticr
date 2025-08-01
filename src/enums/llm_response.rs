use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::enums::Action;
use crate::models::prompt_responses::*;
use crate::models::code_review::CodeReviewInput;
use crate::models::conflict_resolution::ConflictResolutionInput;
use crate::models::task::TaskInput;
use crate::models::agent_error_recovery::ErrorRecoveryResponse;

/// Represents a single failure instance for a todo
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TodoFailure {
    /// When the failure occurred
    pub timestamp: DateTime<Utc>,
    /// The error message or reason for failure
    pub error_message: String,
    /// Optional details about the failure (stack trace, context, etc.)
    pub details: Option<String>,
    /// Which action index failed (if applicable)
    pub failed_action_index: Option<usize>,
}

impl TodoFailure {
    /// Creates a new failure record
    pub fn new(error_message: String) -> Self {
        Self {
            timestamp: Utc::now(),
            error_message,
            details: None,
            failed_action_index: None,
        }
    }
    
    /// Creates a failure with full details
    pub fn with_details(
        error_message: String,
        details: Option<String>,
        failed_action_index: Option<usize>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            error_message,
            details,
            failed_action_index,
        }
    }
}

/// A todo item with a title and a set of actions to perform
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    /// The title/description of this todo item
    pub title: String,
    /// Optional description providing more context
    pub description: Option<String>,
    /// The actions to perform for this todo
    pub actions: Vec<Action>,
    /// Priority level (High, Medium, Low)
    pub priority: Option<String>,
    /// Estimated complexity or effort (Low, Medium, High)
    pub complexity: Option<String>,
    /// Whether this todo is completed or not
    pub done: bool,
    /// Number of times this todo has been retried
    pub retry_count: u32,
    /// History of failures for this todo
    pub failure_history: Vec<TodoFailure>,
    /// Maximum number of retries allowed (None = unlimited)
    pub max_retries: Option<u32>,
}

impl Todo {
    /// Creates a new todo with title and actions (defaults to not done)
    pub fn new(title: String, actions: Vec<Action>) -> Self {
        Self {
            title,
            description: None,
            actions,
            priority: None,
            complexity: None,
            done: false,
            retry_count: 0,
            failure_history: Vec::new(),
            max_retries: None,
        }
    }
    
    /// Creates a new todo with full details
    pub fn with_details(
        title: String,
        description: Option<String>,
        actions: Vec<Action>,
        priority: Option<String>,
        complexity: Option<String>,
        done: bool,
    ) -> Self {
        Self {
            title,
            description,
            actions,
            priority,
            complexity,
            done,
            retry_count: 0,
            failure_history: Vec::new(),
            max_retries: None,
        }
    }
    
    /// Gets the number of actions in this todo
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }
    
    /// Checks if this todo has any actions
    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }
    
    /// Marks this todo as done
    pub fn mark_done(&mut self) {
        self.done = true;
    }
    
    /// Marks this todo as not done
    pub fn mark_not_done(&mut self) {
        self.done = false;
    }
    
    /// Checks if this todo is completed
    pub fn is_done(&self) -> bool {
        self.done
    }
    
    /// Creates a new todo that is already marked as done
    pub fn new_done(title: String, actions: Vec<Action>) -> Self {
        Self {
            title,
            description: None,
            actions,
            priority: None,
            complexity: None,
            done: true,
            retry_count: 0,
            failure_history: Vec::new(),
            max_retries: None,
        }
    }
    
    /// Creates a new todo with retry limits
    pub fn with_retry_limit(title: String, actions: Vec<Action>, max_retries: u32) -> Self {
        Self {
            title,
            description: None,
            actions,
            priority: None,
            complexity: None,
            done: false,
            retry_count: 0,
            failure_history: Vec::new(),
            max_retries: Some(max_retries),
        }
    }
    
    /// Records a failure and increments retry count
    pub fn record_failure(&mut self, error_message: String) {
        let failure = TodoFailure::new(error_message);
        self.failure_history.push(failure);
        self.retry_count += 1;
    }
    
    /// Records a failure with detailed information
    pub fn record_failure_with_details(
        &mut self,
        error_message: String,
        details: Option<String>,
        failed_action_index: Option<usize>,
    ) {
        let failure = TodoFailure::with_details(error_message, details, failed_action_index);
        self.failure_history.push(failure);
        self.retry_count += 1;
    }
    
    /// Checks if this todo has reached its retry limit
    pub fn has_reached_retry_limit(&self) -> bool {
        if let Some(max) = self.max_retries {
            self.retry_count >= max
        } else {
            false // No limit set
        }
    }
    
    /// Gets the number of failures recorded
    pub fn failure_count(&self) -> usize {
        self.failure_history.len()
    }
    
    /// Gets the most recent failure, if any
    pub fn last_failure(&self) -> Option<&TodoFailure> {
        self.failure_history.last()
    }
    
    /// Checks if this todo has any failures
    pub fn has_failures(&self) -> bool {
        !self.failure_history.is_empty()
    }
    
    /// Gets the current retry count
    pub fn get_retry_count(&self) -> u32 {
        self.retry_count
    }
    
    /// Sets the maximum number of retries allowed
    pub fn set_max_retries(&mut self, max_retries: Option<u32>) {
        self.max_retries = max_retries;
    }
    
    /// Resets failure history and retry count (for fresh attempts)
    pub fn reset_failures(&mut self) {
        self.failure_history.clear();
        self.retry_count = 0;
    }
    
    /// Gets all failure messages as a vector of strings
    pub fn get_failure_messages(&self) -> Vec<String> {
        self.failure_history.iter()
            .map(|f| f.error_message.clone())
            .collect()
    }
}

/// Comprehensive enum representing all possible responses from LLMs for different prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "response_type", content = "data")]
pub enum LLMResponse {
    // === TODO-BASED RESPONSES (max 7 todos each) ===
    /// Feature development - returns organized todos with actions
    FeatureDevelopment {
        todos: Vec<Todo>,
    },
    
    /// Task development - returns organized todos with actions  
    TaskDevelopment {
        todos: Vec<Todo>,
    },
    
    /// Unit testing - returns organized todos for testing tasks
    UnitTesting {
        todos: Vec<Todo>,
    },
    
    /// Integration testing - returns organized todos for integration testing
    IntegrationTesting {
        todos: Vec<Todo>,
    },
    
    /// E2E testing - returns organized todos for E2E testing  
    E2ETesting {
        todos: Vec<Todo>,
    },
    
    /// Performance testing - returns organized todos for performance testing
    PerformanceTesting {
        todos: Vec<Todo>,
    },
    
    /// Error recovery - returns organized todos for error recovery
    ErrorRecovery {
        response: ErrorRecoveryResponse,
        todos: Vec<Todo>,
    },

    // === STRUCTURED DATA RESPONSES ===
    /// Idea breakdown - returns task breakdown
    IdeaBreakdown {
        tasks: Vec<TaskInput>,
    },
    
    /// CI/CD fix analysis and improvements - returns organized todos  
    CiCdFix {
        response: CiCdFixResponse,
        todos: Vec<Todo>,
    },
    
    /// Docker deployment configuration - returns organized todos
    DockerDeployment {
        response: DockerDeploymentResponse,
        todos: Vec<Todo>,
    },
    
    /// QA analysis comprehensive report
    QaAnalysis {
        response: QaAnalysisResponse,
    },
    
    /// API synchronization analysis and fixes
    ApiSynchronization {
        response: ApiSynchronizationResponse,
    },
    
    /// Performance optimization recommendations
    PerformanceOptimization {
        response: PerformanceOptimizationResponse,
    },

    // === AGENT RESPONSES (Accept/Reject) ===
    /// Code review agent response - returns organized todos if accepted
    CodeReviewAgent {
        result: AgentResult<Vec<Todo>>,
    },
    
    /// QA agent response - returns organized todos if accepted
    QaAgent {
        result: AgentResult<Vec<Todo>>,
    },
    
    /// DevOps agent response - returns organized todos if accepted
    DevOpsAgent {
        result: AgentResult<Vec<Todo>>,
    },

    // === INPUT VALIDATION RESPONSES ===
    /// Code review input validation
    CodeReviewInput {
        input: CodeReviewInput,
    },
    
    /// Conflict resolution input validation
    ConflictResolutionInput {
        input: ConflictResolutionInput,
    },

    // === RAW RESPONSES ===
    /// Raw JSON response when type cannot be determined
    Raw {
        content: String,
    },
    
    /// Plain text response
    Text {
        content: String,
    },
}

/// Result type for agent responses that can either accept or reject
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum AgentResult<T> {
    /// Agent accepts and provides response data
    Accept {
        data: T,
    },
    /// Agent rejects with reason and blocking issues
    Reject {
        reason: String,
        blocking_issues: Vec<String>,
    },
}

impl LLMResponse {
    /// Extracts all actions from todos in responses
    pub fn extract_actions(&self) -> Vec<Action> {
        let todos = self.extract_todos();
        todos.into_iter()
            .flat_map(|todo| todo.actions)
            .collect()
    }
    
    /// Extracts todos from responses that contain them
    pub fn extract_todos(&self) -> Vec<Todo> {
        match self {
            LLMResponse::FeatureDevelopment { todos } => todos.clone(),
            LLMResponse::TaskDevelopment { todos } => todos.clone(),
            LLMResponse::UnitTesting { todos } => todos.clone(),
            LLMResponse::IntegrationTesting { todos } => todos.clone(),
            LLMResponse::E2ETesting { todos } => todos.clone(),
            LLMResponse::PerformanceTesting { todos } => todos.clone(),
            LLMResponse::ErrorRecovery { todos, .. } => todos.clone(),
            LLMResponse::CiCdFix { todos, .. } => todos.clone(),
            LLMResponse::DockerDeployment { todos, .. } => todos.clone(),
            LLMResponse::CodeReviewAgent { result } => {
                match result {
                    AgentResult::Accept { data } => data.clone(),
                    AgentResult::Reject { .. } => vec![],
                }
            },
            LLMResponse::QaAgent { result } => {
                match result {
                    AgentResult::Accept { data } => data.clone(),
                    AgentResult::Reject { .. } => vec![],
                }
            },
            LLMResponse::DevOpsAgent { result } => {
                match result {
                    AgentResult::Accept { data } => data.clone(),
                    AgentResult::Reject { .. } => vec![],
                }
            },
            _ => vec![],
        }
    }
    
    /// Checks if this is a rejection response
    pub fn is_rejection(&self) -> bool {
        match self {
            LLMResponse::CodeReviewAgent { result } |
            LLMResponse::QaAgent { result } |
            LLMResponse::DevOpsAgent { result } => {
                matches!(result, AgentResult::Reject { .. })
            },
            _ => false,
        }
    }
    
    /// Gets rejection reason if this is a rejection
    pub fn get_rejection_reason(&self) -> Option<&str> {
        match self {
            LLMResponse::CodeReviewAgent { result } |
            LLMResponse::QaAgent { result } |
            LLMResponse::DevOpsAgent { result } => {
                match result {
                    AgentResult::Reject { reason, .. } => Some(reason),
                    _ => None,
                }
            },
            _ => None,
        }
    }
    
    /// Gets blocking issues if this is a rejection
    pub fn get_blocking_issues(&self) -> Vec<&str> {
        match self {
            LLMResponse::CodeReviewAgent { result } |
            LLMResponse::QaAgent { result } |
            LLMResponse::DevOpsAgent { result } => {
                match result {
                    AgentResult::Reject { blocking_issues, .. } => {
                        blocking_issues.iter().map(|s| s.as_str()).collect()
                    },
                    _ => vec![],
                }
            },
            _ => vec![],
        }
    }
    
    /// Determines the response type for categorization
    pub fn response_type(&self) -> &'static str {
        match self {
            LLMResponse::FeatureDevelopment { .. } => "feature_development",
            LLMResponse::TaskDevelopment { .. } => "task_development", 
            LLMResponse::UnitTesting { .. } => "unit_testing",
            LLMResponse::IntegrationTesting { .. } => "integration_testing",
            LLMResponse::E2ETesting { .. } => "e2e_testing",
            LLMResponse::PerformanceTesting { .. } => "performance_testing",
            LLMResponse::ErrorRecovery { .. } => "error_recovery",
            LLMResponse::IdeaBreakdown { .. } => "idea_breakdown",
            LLMResponse::CiCdFix { .. } => "ci_cd_fix",
            LLMResponse::DockerDeployment { .. } => "docker_deployment",
            LLMResponse::QaAnalysis { .. } => "qa_analysis", 
            LLMResponse::ApiSynchronization { .. } => "api_synchronization",
            LLMResponse::PerformanceOptimization { .. } => "performance_optimization",
            LLMResponse::CodeReviewAgent { .. } => "code_review_agent",
            LLMResponse::QaAgent { .. } => "qa_agent",
            LLMResponse::DevOpsAgent { .. } => "devops_agent",
            LLMResponse::CodeReviewInput { .. } => "code_review_input",
            LLMResponse::ConflictResolutionInput { .. } => "conflict_resolution_input",
            LLMResponse::Raw { .. } => "raw",
            LLMResponse::Text { .. } => "text",
        }
    }
    
    /// Checks if response contains executable actions
    pub fn has_actions(&self) -> bool {
        !self.extract_actions().is_empty()
    }
    
    /// Attempts to parse raw JSON into specific response types
    pub fn from_raw_json(json_str: &str) -> Result<Self, serde_json::Error> {
        // Try to parse as different response types in order of likelihood
        
        // Try todos array first (new format)
        if let Ok(todos) = serde_json::from_str::<Vec<Todo>>(json_str) {
            return Ok(LLMResponse::FeatureDevelopment { todos });
        }
        
        // Try action array (legacy format)
        if let Ok(actions) = serde_json::from_str::<Vec<Action>>(json_str) {
            let todo = Todo::new("Feature Development".to_string(), actions);
            return Ok(LLMResponse::FeatureDevelopment { todos: vec![todo] });
        }
        
        // Try structured responses
        if let Ok(response) = serde_json::from_str::<CiCdFixResponse>(json_str) {
            return Ok(LLMResponse::CiCdFix { response, todos: vec![] });
        }
        
        if let Ok(response) = serde_json::from_str::<DockerDeploymentResponse>(json_str) {
            return Ok(LLMResponse::DockerDeployment { response, todos: vec![] });
        }
        
        if let Ok(response) = serde_json::from_str::<QaAnalysisResponse>(json_str) {
            return Ok(LLMResponse::QaAnalysis { response });
        }
        
        if let Ok(response) = serde_json::from_str::<ApiSynchronizationResponse>(json_str) {
            return Ok(LLMResponse::ApiSynchronization { response });
        }
        
        if let Ok(response) = serde_json::from_str::<PerformanceOptimizationResponse>(json_str) {
            return Ok(LLMResponse::PerformanceOptimization { response });
        }
        
        if let Ok(tasks) = serde_json::from_str::<Vec<TaskInput>>(json_str) {
            return Ok(LLMResponse::IdeaBreakdown { tasks });
        }
        
        if let Ok(input) = serde_json::from_str::<CodeReviewInput>(json_str) {
            return Ok(LLMResponse::CodeReviewInput { input });
        }
        
        if let Ok(input) = serde_json::from_str::<ConflictResolutionInput>(json_str) {
            return Ok(LLMResponse::ConflictResolutionInput { input });
        }
        
        // If none match, store as raw
        Ok(LLMResponse::Raw { 
            content: json_str.to_string() 
        })
    }
    
    /// Converts response to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    /// Creates a feature development response from actions
    pub fn feature_development(actions: Vec<Action>) -> Self {
        let todo = Todo::new("Feature Development".to_string(), actions);
        LLMResponse::FeatureDevelopment { todos: vec![todo] }
    }
    
    /// Creates a code review agent rejection
    pub fn code_review_rejection(reason: String, blocking_issues: Vec<String>) -> Self {
        LLMResponse::CodeReviewAgent {
            result: AgentResult::reject(reason, blocking_issues)
        }
    }
    
    /// Creates a code review agent acceptance
    pub fn code_review_acceptance(actions: Vec<Action>) -> Self {
        let todo = Todo::new("Code Review Actions".to_string(), actions);
        LLMResponse::CodeReviewAgent {
            result: AgentResult::accept(vec![todo])
        }
    }
    
    /// Creates a QA agent rejection
    pub fn qa_rejection(reason: String, blocking_issues: Vec<String>) -> Self {
        LLMResponse::QaAgent {
            result: AgentResult::reject(reason, blocking_issues)
        }
    }
    
    /// Creates a QA agent acceptance
    pub fn qa_acceptance(actions: Vec<Action>) -> Self {
        let todo = Todo::new("QA Actions".to_string(), actions);
        LLMResponse::QaAgent {
            result: AgentResult::accept(vec![todo])
        }
    }
    
    /// Creates a DevOps agent rejection
    pub fn devops_rejection(reason: String, blocking_issues: Vec<String>) -> Self {
        LLMResponse::DevOpsAgent {
            result: AgentResult::reject(reason, blocking_issues)
        }
    }
    
    /// Creates a DevOps agent acceptance
    pub fn devops_acceptance(actions: Vec<Action>) -> Self {
        let todo = Todo::new("DevOps Actions".to_string(), actions);
        LLMResponse::DevOpsAgent {
            result: AgentResult::accept(vec![todo])
        }
    }
    
    /// Creates a raw text response
    pub fn text(content: String) -> Self {
        LLMResponse::Text { content }
    }
    
    /// Creates a raw JSON response
    pub fn raw(content: String) -> Self {
        LLMResponse::Raw { content }
    }
    
    /// Gets the main data from structured responses
    pub fn get_structured_data(&self) -> Option<serde_json::Value> {
        match self {
            LLMResponse::CiCdFix { response, .. } => serde_json::to_value(response).ok(),
            LLMResponse::DockerDeployment { response, .. } => serde_json::to_value(response).ok(),
            LLMResponse::QaAnalysis { response } => serde_json::to_value(response).ok(),
            LLMResponse::ApiSynchronization { response } => serde_json::to_value(response).ok(),
            LLMResponse::PerformanceOptimization { response } => serde_json::to_value(response).ok(),
            _ => None,
        }
    }
    
    /// Check if this response requires follow-up actions
    pub fn requires_followup(&self) -> bool {
        match self {
            LLMResponse::CiCdFix { .. } |
            LLMResponse::DockerDeployment { .. } |
            LLMResponse::QaAnalysis { .. } |
            LLMResponse::ApiSynchronization { .. } |
            LLMResponse::PerformanceOptimization { .. } => true,
            _ => self.has_actions(),
        }
    }
}

impl<T> AgentResult<T> {
    /// Creates an accept result
    pub fn accept(data: T) -> Self {
        AgentResult::Accept { data }
    }
    
    /// Creates a reject result
    pub fn reject(reason: String, blocking_issues: Vec<String>) -> Self {
        AgentResult::Reject { reason, blocking_issues }
    }
    
    /// Checks if this is an accept result
    pub fn is_accept(&self) -> bool {
        matches!(self, AgentResult::Accept { .. })
    }
    
    /// Checks if this is a reject result  
    pub fn is_reject(&self) -> bool {
        matches!(self, AgentResult::Reject { .. })
    }
    
    /// Extracts data if accept, returns None if reject
    pub fn into_data(self) -> Option<T> {
        match self {
            AgentResult::Accept { data } => Some(data),
            AgentResult::Reject { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_result_creation() {
        let accept_result: AgentResult<Vec<Action>> = AgentResult::accept(vec![]);
        assert!(accept_result.is_accept());
        assert!(!accept_result.is_reject());
        
        let reject_result: AgentResult<Vec<Action>> = AgentResult::reject(
            "Test rejection".to_string(),
            vec!["Issue 1".to_string(), "Issue 2".to_string()]
        );
        assert!(reject_result.is_reject());
        assert!(!reject_result.is_accept());
    }
    
    #[test] 
    fn test_llm_response_actions_extraction() {
        let actions = vec![
            Action::Write {
                path: "test.txt".to_string(),
                content: "test content".to_string(),
            }
        ];
        
        let todo = Todo::new("Test Todo".to_string(), actions.clone());
        let response = LLMResponse::FeatureDevelopment {
            todos: vec![todo]
        };
        
        assert_eq!(response.extract_actions(), actions);
        assert!(response.has_actions());
        assert!(!response.is_rejection());
    }
    
    #[test]
    fn test_llm_response_rejection() {
        let response = LLMResponse::CodeReviewAgent {
            result: AgentResult::reject(
                "Code quality issues".to_string(),
                vec!["Syntax errors".to_string()]
            )
        };
        
        assert!(response.is_rejection());
        assert_eq!(response.get_rejection_reason(), Some("Code quality issues"));
        assert_eq!(response.get_blocking_issues(), vec!["Syntax errors"]);
        assert!(!response.has_actions());
    }
    
    #[test]
    fn test_response_type_identification() {
        let response = LLMResponse::FeatureDevelopment { todos: vec![] };
        assert_eq!(response.response_type(), "feature_development");
        
        let response = LLMResponse::Raw { content: "test".to_string() };
        assert_eq!(response.response_type(), "raw");
    }

    #[test]
    fn test_utility_constructors() {
        // Test feature development constructor
        let actions = vec![Action::Write {
            path: "test.txt".to_string(),
            content: "test".to_string(),
        }];
        let response = LLMResponse::feature_development(actions.clone());
        assert_eq!(response.extract_actions(), actions);
        
        // Test rejection constructors
        let rejection = LLMResponse::code_review_rejection(
            "Issues found".to_string(),
            vec!["Error 1".to_string()]
        );
        assert!(rejection.is_rejection());
        assert_eq!(rejection.get_rejection_reason(), Some("Issues found"));
        
        // Test text constructor
        let text_response = LLMResponse::text("Hello world".to_string());
        assert_eq!(text_response.response_type(), "text");
        
        // Test followup requirement
        let feature_response = LLMResponse::feature_development(actions);
        assert!(feature_response.requires_followup());
        
        let empty_response = LLMResponse::feature_development(vec![]);
        assert!(!empty_response.requires_followup());
    }
    
    #[test]
    fn test_todo_structure() {
        let actions = vec![
            Action::Write {
                path: "test.txt".to_string(),
                content: "test content".to_string(),
            }
        ];
        
        let todo = Todo::new("Test Todo".to_string(), actions.clone());
        assert_eq!(todo.title, "Test Todo");
        assert_eq!(todo.actions, actions);
        assert_eq!(todo.action_count(), 1);
        assert!(todo.has_actions());
        assert!(!todo.is_done()); // New todos default to not done
        
        let detailed_todo = Todo::with_details(
            "Detailed Todo".to_string(),
            Some("Description".to_string()),
            actions.clone(),
            Some("High".to_string()),
            Some("Medium".to_string()),
            true, // Mark as done
        );
        assert_eq!(detailed_todo.title, "Detailed Todo");
        assert_eq!(detailed_todo.description, Some("Description".to_string()));
        assert_eq!(detailed_todo.priority, Some("High".to_string()));
        assert_eq!(detailed_todo.complexity, Some("Medium".to_string()));
        assert!(detailed_todo.is_done()); // Should be done as specified
    }
    
    #[test]
    fn test_todo_done_state() {
        let actions = vec![
            Action::Write {
                path: "test.txt".to_string(),
                content: "test content".to_string(),
            }
        ];
        
        // Test new todo defaults to not done
        let mut todo = Todo::new("Test Todo".to_string(), actions.clone());
        assert!(!todo.is_done());
        
        // Test marking as done
        todo.mark_done();
        assert!(todo.is_done());
        
        // Test marking as not done
        todo.mark_not_done();
        assert!(!todo.is_done());
        
        // Test new_done constructor
        let done_todo = Todo::new_done("Done Todo".to_string(), actions);
        assert!(done_todo.is_done());
        assert_eq!(done_todo.title, "Done Todo");
    }
    
    #[test]
    fn test_todo_failure_tracking() {
        let actions = vec![
            Action::Write {
                path: "test.txt".to_string(),
                content: "test content".to_string(),
            }
        ];
        
        let mut todo = Todo::new("Test Todo".to_string(), actions);
        
        // Test initial state
        assert_eq!(todo.get_retry_count(), 0);
        assert_eq!(todo.failure_count(), 0);
        assert!(!todo.has_failures());
        assert!(todo.last_failure().is_none());
        
        // Test recording a failure
        todo.record_failure("First error".to_string());
        assert_eq!(todo.get_retry_count(), 1);
        assert_eq!(todo.failure_count(), 1);
        assert!(todo.has_failures());
        
        let last_failure = todo.last_failure().unwrap();
        assert_eq!(last_failure.error_message, "First error");
        assert!(last_failure.details.is_none());
        assert!(last_failure.failed_action_index.is_none());
        
        // Test recording a failure with details
        todo.record_failure_with_details(
            "Second error".to_string(),
            Some("Stack trace here".to_string()),
            Some(0),
        );
        assert_eq!(todo.get_retry_count(), 2);
        assert_eq!(todo.failure_count(), 2);
        
        let last_failure = todo.last_failure().unwrap();
        assert_eq!(last_failure.error_message, "Second error");
        assert_eq!(last_failure.details, Some("Stack trace here".to_string()));
        assert_eq!(last_failure.failed_action_index, Some(0));
        
        // Test getting all failure messages
        let messages = todo.get_failure_messages();
        assert_eq!(messages, vec!["First error", "Second error"]);
        
        // Test resetting failures
        todo.reset_failures();
        assert_eq!(todo.get_retry_count(), 0);
        assert_eq!(todo.failure_count(), 0);
        assert!(!todo.has_failures());
    }
    
    #[test]
    fn test_todo_retry_limits() {
        let actions = vec![
            Action::Write {
                path: "test.txt".to_string(),
                content: "test content".to_string(),
            }
        ];
        
        // Test todo with retry limit
        let mut todo = Todo::with_retry_limit("Limited Todo".to_string(), actions, 2);
        assert!(!todo.has_reached_retry_limit());
        
        // Test first failure
        todo.record_failure("Error 1".to_string());
        assert!(!todo.has_reached_retry_limit());
        assert_eq!(todo.get_retry_count(), 1);
        
        // Test second failure - should reach limit
        todo.record_failure("Error 2".to_string());
        assert!(todo.has_reached_retry_limit());
        assert_eq!(todo.get_retry_count(), 2);
        
        // Test setting max retries
        todo.set_max_retries(Some(5));
        assert!(!todo.has_reached_retry_limit()); // Now under new limit
        
        // Test unlimited retries
        todo.set_max_retries(None);
        assert!(!todo.has_reached_retry_limit()); // No limit
    }
    
    #[test]
    fn test_todo_failure_structure() {
        // Test TodoFailure creation
        let failure1 = TodoFailure::new("Simple error".to_string());
        assert_eq!(failure1.error_message, "Simple error");
        assert!(failure1.details.is_none());
        assert!(failure1.failed_action_index.is_none());
        
        let failure2 = TodoFailure::with_details(
            "Detailed error".to_string(),
            Some("More info".to_string()),
            Some(1),
        );
        assert_eq!(failure2.error_message, "Detailed error");
        assert_eq!(failure2.details, Some("More info".to_string()));
        assert_eq!(failure2.failed_action_index, Some(1));
        
        // Timestamps should be recent (within last minute)
        let now = Utc::now();
        let time_diff = now.signed_duration_since(failure1.timestamp);
        assert!(time_diff.num_seconds() < 60);
    }
}