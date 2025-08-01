use serde::{Deserialize, Serialize};
use crate::enums::Action;
use crate::models::prompt_responses::*;
use crate::models::code_review::CodeReviewInput;
use crate::models::conflict_resolution::ConflictResolutionInput;
use crate::models::task::TaskInput;
use crate::models::agent_error_recovery::ErrorRecoveryResponse;

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
}

impl Todo {
    /// Creates a new todo with title and actions
    pub fn new(title: String, actions: Vec<Action>) -> Self {
        Self {
            title,
            description: None,
            actions,
            priority: None,
            complexity: None,
        }
    }
    
    /// Creates a new todo with full details
    pub fn with_details(
        title: String,
        description: Option<String>,
        actions: Vec<Action>,
        priority: Option<String>,
        complexity: Option<String>,
    ) -> Self {
        Self {
            title,
            description,
            actions,
            priority,
            complexity,
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
        
        let detailed_todo = Todo::with_details(
            "Detailed Todo".to_string(),
            Some("Description".to_string()),
            actions.clone(),
            Some("High".to_string()),
            Some("Medium".to_string()),
        );
        assert_eq!(detailed_todo.title, "Detailed Todo");
        assert_eq!(detailed_todo.description, Some("Description".to_string()));
        assert_eq!(detailed_todo.priority, Some("High".to_string()));
        assert_eq!(detailed_todo.complexity, Some("Medium".to_string()));
    }
}