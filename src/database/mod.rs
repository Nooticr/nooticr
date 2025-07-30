pub mod schema;
pub mod models;
pub mod repository;
pub mod migrations;

use crate::error::{OrchestratorError, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Database manager for SQLite operations
#[derive(Debug, Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .map_err(|e| OrchestratorError::database(format!("Failed to open database: {}", e)))?;

        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| OrchestratorError::database(format!("Failed to enable foreign keys: {}", e)))?;

        let db = Self {
            connection: Arc::new(Mutex::new(conn)),
        };

        // Run migrations
        db.run_migrations()?;

        Ok(db)
    }

    /// Create an in-memory database for testing
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| OrchestratorError::database(format!("Failed to create in-memory database: {}", e)))?;

        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| OrchestratorError::database(format!("Failed to enable foreign keys: {}", e)))?;

        let db = Self {
            connection: Arc::new(Mutex::new(conn)),
        };

        // Run migrations
        db.run_migrations()?;

        Ok(db)
    }

    /// Get a connection to the database
    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.connection)
    }

    /// Run database migrations
    fn run_migrations(&self) -> Result<()> {
        let conn = self.connection.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;
        
        migrations::run_migrations(&conn)
    }

    /// Check if the database is healthy
    pub fn health_check(&self) -> Result<()> {
        let conn = self.connection.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;

        let _: i32 = conn.query_row("SELECT 1", [], |row| row.get(0))
            .map_err(|e| OrchestratorError::database(format!("Database health check failed: {}", e)))?;

        Ok(())
    }

    /// Get database version
    pub fn get_version(&self) -> Result<i32> {
        let conn = self.connection.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;
        
        let version: i32 = conn.query_row(
            "SELECT user_version FROM pragma_user_version",
            [],
            |row| row.get(0)
        ).map_err(|e| OrchestratorError::database(format!("Failed to get database version: {}", e)))?;
        
        Ok(version)
    }

    /// Set database version
    pub fn set_version(&self, version: i32) -> Result<()> {
        let conn = self.connection.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;
        
        conn.execute(&format!("PRAGMA user_version = {}", version), [])
            .map_err(|e| OrchestratorError::database(format!("Failed to set database version: {}", e)))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::new_in_memory().unwrap();
        assert!(db.health_check().is_ok());
    }

    #[test]
    fn test_database_version() {
        let db = Database::new_in_memory().unwrap();
        
        // Set version
        db.set_version(1).unwrap();
        
        // Get version
        let version = db.get_version().unwrap();
        assert_eq!(version, 1);
    }
}
