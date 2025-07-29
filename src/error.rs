//! Centralized error handling for the Orchy orchestrator
//!
//! This module provides a comprehensive error type using thiserror that consolidates
//! all error types from across the codebase.

use crate::enums::{AgentStatus, CodeStatus, IssueStatus, TaskStatus};
use thiserror::Error;

/// Result type alias for the orchestrator
pub type Result<T> = std::result::Result<T, OrchestratorError>;

/// Comprehensive error type for the orchestrator
#[derive(Error, Debug)]
pub enum OrchestratorError {
    /// IO errors (file operations, network, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// JSON parsing errors with context
    #[error("Failed to parse JSON from {context}: {source}")]
    JsonParsing {
        context: String,
        #[source]
        source: serde_json::Error,
    },

    /// Command execution errors
    #[error("Command execution failed: {message}")]
    CommandExecution { message: String },

    /// Process spawn errors
    #[error("Failed to spawn process: {command}")]
    ProcessSpawn { command: String },

    /// Agent status transition errors
    #[error("Invalid agent status transition from {from:?} to {to:?}")]
    AgentTransition { from: AgentStatus, to: AgentStatus },

    /// Agent constraint violation errors
    #[error("Agent constraint violation: {message}")]
    AgentConstraint { message: String },

    /// Agent already in terminal state
    #[error("Agent already in terminal state: {status:?}")]
    AgentTerminal { status: AgentStatus },

    /// Code status transition errors
    #[error("Invalid code status transition from {from:?} to {to:?}")]
    CodeTransition { from: CodeStatus, to: CodeStatus },

    /// Code status already terminal
    #[error("Code status already in terminal state: {status:?}")]
    CodeTerminal { status: CodeStatus },

    /// Code status requires external action
    #[error("Code status requires external action: {message}")]
    CodeExternalAction { message: String },

    /// Task status transition errors
    #[error("Invalid task status transition from {from:?} to {to:?}")]
    TaskTransition { from: TaskStatus, to: TaskStatus },

    /// Task status already terminal
    #[error("Task status already in terminal state: {status:?}")]
    TaskTerminal { status: TaskStatus },

    /// Issue status transition errors
    #[error("Invalid issue status transition from {from:?} to {to:?}")]
    IssueTransition { from: IssueStatus, to: IssueStatus },

    /// Issue status already terminal
    #[error("Issue status already in terminal state: {status:?}")]
    IssueTerminal { status: IssueStatus },

    /// Generic internal errors
    #[error("Internal error: {message}")]
    Internal { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Not found errors
    #[error("Not found: {resource}")]
    NotFound { resource: String },

    /// Already exists errors
    #[error("Already exists: {resource}")]
    AlreadyExists { resource: String },

    /// Permission denied errors
    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    /// Timeout errors
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Network errors
    #[error("Network error: {message}")]
    Network { message: String },

    /// Database errors
    #[error("Database error: {message}")]
    Database { message: String },

    /// External service errors
    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },
}

impl OrchestratorError {
    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create a JSON parsing error with context
    pub fn json_parsing(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::JsonParsing {
            context: context.into(),
            source,
        }
    }

    /// Create a command execution error
    pub fn command_execution(message: impl Into<String>) -> Self {
        Self::CommandExecution {
            message: message.into(),
        }
    }

    /// Create a process spawn error
    pub fn process_spawn(command: impl Into<String>) -> Self {
        Self::ProcessSpawn {
            command: command.into(),
        }
    }

    /// Create an agent transition error
    pub fn agent_transition(from: AgentStatus, to: AgentStatus) -> Self {
        Self::AgentTransition { from, to }
    }

    /// Create an agent constraint violation error
    pub fn agent_constraint(message: impl Into<String>) -> Self {
        Self::AgentConstraint {
            message: message.into(),
        }
    }

    /// Create an agent terminal state error
    pub fn agent_terminal(status: AgentStatus) -> Self {
        Self::AgentTerminal { status }
    }

    /// Create a code transition error
    pub fn code_transition(from: CodeStatus, to: CodeStatus) -> Self {
        Self::CodeTransition { from, to }
    }

    /// Create a code terminal state error
    pub fn code_terminal(status: CodeStatus) -> Self {
        Self::CodeTerminal { status }
    }

    /// Create a code external action error
    pub fn code_external_action(message: impl Into<String>) -> Self {
        Self::CodeExternalAction {
            message: message.into(),
        }
    }

    /// Create a task transition error
    pub fn task_transition(from: TaskStatus, to: TaskStatus) -> Self {
        Self::TaskTransition { from, to }
    }

    /// Create a task terminal state error
    pub fn task_terminal(status: TaskStatus) -> Self {
        Self::TaskTerminal { status }
    }

    /// Create an issue transition error
    pub fn issue_transition(from: IssueStatus, to: IssueStatus) -> Self {
        Self::IssueTransition { from, to }
    }

    /// Create an issue terminal state error
    pub fn issue_terminal(status: IssueStatus) -> Self {
        Self::IssueTerminal { status }
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create an already exists error
    pub fn already_exists(resource: impl Into<String>) -> Self {
        Self::AlreadyExists {
            resource: resource.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(operation: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create a database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    /// Create an external service error
    pub fn external_service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }
}
