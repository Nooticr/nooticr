use crate::error::{OrchestratorError, Result};
use tokio::sync::{mpsc, oneshot};
use std::time::Instant;

/// Common async patterns and utilities for managers
pub struct ManagerHelpers;

impl ManagerHelpers {
    /// Execute a command with oneshot channel pattern and error handling
    pub async fn execute_command<T, F>(
        command_tx: &mpsc::UnboundedSender<F>,
        command: F,
        operation_name: &str,
    ) -> Result<T>
    where
        F: Send + 'static,
    {
        command_tx.send(command)
            .map_err(|_| OrchestratorError::channel(format!("Failed to send {} command", operation_name)))?;
        
        // Note: This is a simplified version. In practice, you'd need to handle the response channel
        // This would be implemented differently based on the specific command pattern
        todo!("Implement based on specific command response pattern")
    }

    /// Execute an operation with timing and statistics tracking
    pub async fn execute_with_timing<T, F, Fut>(
        operation: F,
        operation_name: &str,
        update_stats: impl FnOnce(u64, bool),
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let start_time = Instant::now();
        
        let result = operation().await;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        let success = result.is_ok();
        
        update_stats(execution_time, success);
        
        result
    }

    /// Create a oneshot channel and handle the response with timeout
    pub async fn oneshot_with_timeout<T>(
        rx: oneshot::Receiver<T>,
        timeout_ms: u64,
        operation_name: &str,
    ) -> Result<T> {
        let timeout = tokio::time::Duration::from_millis(timeout_ms);
        
        tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| OrchestratorError::timeout(format!("{} operation timed out", operation_name)))?
            .map_err(|_| OrchestratorError::channel(format!("{} manager disconnected", operation_name)))
    }

    /// Handle command execution with automatic error logging
    pub async fn handle_command_with_logging<T, F, Fut>(
        command_name: &str,
        operation: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        tracing::debug!("🔄 Executing command: {}", command_name);
        
        match operation().await {
            Ok(result) => {
                tracing::debug!("✅ Command completed successfully: {}", command_name);
                Ok(result)
            }
            Err(e) => {
                tracing::error!("❌ Command failed: {} - {}", command_name, e);
                Err(e)
            }
        }
    }

    /// Batch process items with error collection
    pub async fn batch_process<T, R, F, Fut>(
        items: Vec<T>,
        processor: F,
        operation_name: &str,
    ) -> (Vec<R>, Vec<OrchestratorError>)
    where
        F: Fn(T) -> Fut,
        Fut: std::future::Future<Output = Result<R>>,
    {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for item in items {
            match processor(item).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::warn!("Failed to process item in {}: {}", operation_name, e);
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() {
            tracing::warn!("Batch operation {} completed with {} errors", operation_name, errors.len());
        }

        (results, errors)
    }

    /// Retry an operation with exponential backoff
    pub async fn retry_with_backoff<T, F, Fut>(
        operation: F,
        max_retries: u32,
        initial_delay_ms: u64,
        operation_name: &str,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut delay = initial_delay_ms;
        
        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        tracing::info!("✅ {} succeeded after {} retries", operation_name, attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if attempt == max_retries {
                        tracing::error!("❌ {} failed after {} retries: {}", operation_name, max_retries, e);
                        return Err(e);
                    }
                    
                    tracing::warn!("⚠️ {} attempt {} failed, retrying in {}ms: {}", 
                                 operation_name, attempt + 1, delay, e);
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }
        
        unreachable!()
    }
}

/// Trait for manager statistics to ensure consistency
pub trait ManagerStatistics {
    fn update_success(&mut self, execution_time_ms: u64);
    fn update_failure(&mut self, execution_time_ms: u64);
    fn get_success_rate(&self) -> f64;
    fn get_average_execution_time(&self) -> f64;
}

/// Common statistics implementation
#[derive(Debug, Clone, Default)]
pub struct CommonStatistics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub total_execution_time_ms: u64,
}

impl ManagerStatistics for CommonStatistics {
    fn update_success(&mut self, execution_time_ms: u64) {
        self.total_operations += 1;
        self.successful_operations += 1;
        self.total_execution_time_ms += execution_time_ms;
    }

    fn update_failure(&mut self, execution_time_ms: u64) {
        self.total_operations += 1;
        self.failed_operations += 1;
        self.total_execution_time_ms += execution_time_ms;
    }

    fn get_success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            (self.successful_operations as f64 / self.total_operations as f64) * 100.0
        }
    }

    fn get_average_execution_time(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.total_execution_time_ms as f64 / self.total_operations as f64
        }
    }
}

/// Macro to generate common manager command handling patterns
#[macro_export]
macro_rules! impl_manager_command_handler {
    ($manager:ty, $command:ty, $event:ty) => {
        impl $manager {
            pub async fn handle_command(&mut self, command: $command) -> crate::error::Result<()> {
                use crate::managers::common::ManagerHelpers;
                
                let command_name = std::any::type_name::<$command>();
                
                ManagerHelpers::handle_command_with_logging(command_name, || async {
                    self.process_command(command).await
                }).await
            }
            
            async fn process_command(&mut self, command: $command) -> crate::error::Result<()> {
                // This would be implemented by each specific manager
                todo!("Implement specific command processing")
            }
        }
    };
}

/// Trait for command validation
pub trait CommandValidator {
    type Command;
    
    fn validate_command(&self, command: &Self::Command) -> Result<()>;
}

/// Common command patterns
pub enum CommonCommand<T> {
    Execute { payload: T, respond_to: oneshot::Sender<Result<()>> },
    Query { payload: T, respond_to: oneshot::Sender<Result<String>> },
    Shutdown,
}

/// Helper for creating command clients
pub struct CommandClient<T> {
    command_tx: mpsc::UnboundedSender<T>,
}

impl<T> CommandClient<T> {
    pub fn new(command_tx: mpsc::UnboundedSender<T>) -> Self {
        Self { command_tx }
    }

    pub async fn send_command(&self, command: T) -> Result<()> {
        self.command_tx.send(command)
            .map_err(|_| OrchestratorError::channel("Failed to send command".to_string()))?;
        Ok(())
    }
}

/// Event broadcasting helper
pub struct EventBroadcaster<T> {
    event_tx: tokio::sync::broadcast::Sender<T>,
}

impl<T: Clone> EventBroadcaster<T> {
    pub fn new(capacity: usize) -> (Self, tokio::sync::broadcast::Receiver<T>) {
        let (event_tx, event_rx) = tokio::sync::broadcast::channel(capacity);
        (Self { event_tx }, event_rx)
    }

    pub fn broadcast(&self, event: T) -> Result<()> {
        self.event_tx.send(event)
            .map_err(|_| OrchestratorError::channel("Failed to broadcast event".to_string()))?;
        Ok(())
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<T> {
        self.event_tx.subscribe()
    }
}
