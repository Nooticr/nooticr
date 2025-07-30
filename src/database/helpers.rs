use crate::error::{OrchestratorError, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde_json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Database operation helper functions to reduce code duplication
pub struct DatabaseHelpers;

impl DatabaseHelpers {
    /// Execute a database operation with automatic connection handling and error mapping
    pub fn with_connection<T, F>(
        connection: &Arc<Mutex<Connection>>,
        operation: F,
    ) -> Result<T>
    where
        F: FnOnce(&Connection) -> rusqlite::Result<T>,
    {
        let conn = connection.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;
        
        operation(&conn)
            .map_err(|e| OrchestratorError::database(format!("Database operation failed: {}", e)))
    }

    /// Execute a database operation within a transaction
    pub fn with_transaction<T, F>(
        connection: &Arc<Mutex<Connection>>,
        operation: F,
    ) -> Result<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<T>,
    {
        let conn = connection.lock()
            .map_err(|e| OrchestratorError::database(format!("Failed to acquire database lock: {}", e)))?;

        let tx = conn.unchecked_transaction()
            .map_err(|e| OrchestratorError::database(format!("Failed to start transaction: {}", e)))?;

        let result = operation(&tx)?;

        tx.commit()
            .map_err(|e| OrchestratorError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(result)
    }

    /// Parse UUID from string with proper error handling
    pub fn parse_uuid(uuid_str: &str, context: &str) -> Result<Uuid> {
        Uuid::parse_str(uuid_str)
            .map_err(|e| OrchestratorError::validation(format!("Invalid {} UUID: {}", context, e)))
    }

    /// Parse DateTime from RFC3339 string with proper error handling
    pub fn parse_datetime(date_str: &str, context: &str) -> Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(date_str)
            .map_err(|e| OrchestratorError::validation(format!("Invalid {} date: {}", context, e)))
            .map(|dt| dt.with_timezone(&Utc))
    }

    /// Parse optional DateTime from RFC3339 string
    pub fn parse_optional_datetime(date_str: Option<&str>, context: &str) -> Result<Option<DateTime<Utc>>> {
        match date_str {
            Some(s) => Ok(Some(Self::parse_datetime(s, context)?)),
            None => Ok(None),
        }
    }

    /// Serialize enum to JSON string with error handling
    pub fn serialize_enum<T: serde::Serialize>(value: &T, context: &str) -> Result<String> {
        serde_json::to_string(value)
            .map_err(|e| OrchestratorError::json_parsing(context, e))
    }

    /// Deserialize enum from JSON string with error handling
    pub fn deserialize_enum<T: serde::de::DeserializeOwned>(json_str: &str, context: &str) -> Result<T> {
        serde_json::from_str(json_str)
            .map_err(|e| OrchestratorError::json_parsing(context, e))
    }

    /// Execute INSERT OR REPLACE with error handling
    pub fn insert_or_replace(
        conn: &Connection,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        context: &str,
    ) -> Result<()> {
        conn.execute(sql, params)
            .map_err(|e| OrchestratorError::database(format!("Failed to {}: {}", context, e)))?;
        Ok(())
    }

    /// Execute a query that returns a single row
    pub fn query_single_row<T, F>(
        conn: &Connection,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        mapper: F,
        context: &str,
    ) -> Result<T>
    where
        F: FnOnce(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        conn.query_row(sql, params, mapper)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    OrchestratorError::not_found(context)
                }
                _ => OrchestratorError::database(format!("Failed to query {}: {}", context, e))
            })
    }

    /// Execute a query that returns multiple rows
    pub fn query_multiple_rows<T, F>(
        conn: &Connection,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        mapper: F,
        context: &str,
    ) -> Result<Vec<T>>
    where
        F: Fn(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let mut stmt = conn.prepare(sql)
            .map_err(|e| OrchestratorError::database(format!("Failed to prepare {} query: {}", context, e)))?;

        let rows = stmt.query_map(params, mapper)
            .map_err(|e| OrchestratorError::database(format!("Failed to execute {} query: {}", context, e)))?;

        let mut results = Vec::new();
        for row_result in rows {
            let row = row_result
                .map_err(|e| OrchestratorError::database(format!("Failed to parse {} row: {}", context, e)))?;
            results.push(row);
        }

        Ok(results)
    }

    /// Save a collection of items with a common pattern
    pub fn save_collection<T, F>(
        conn: &Connection,
        items: &[T],
        save_fn: F,
        context: &str,
    ) -> Result<()>
    where
        F: Fn(&Connection, &T) -> Result<()>,
    {
        for item in items {
            save_fn(conn, item)
                .map_err(|e| OrchestratorError::database(format!("Failed to save {} item: {}", context, e)))?;
        }
        Ok(())
    }

    /// Delete items by parent ID with a common pattern
    pub fn delete_by_parent_id(
        conn: &Connection,
        table: &str,
        parent_column: &str,
        parent_id: &str,
        context: &str,
    ) -> Result<()> {
        let sql = format!("DELETE FROM {} WHERE {} = ?1", table, parent_column);
        conn.execute(&sql, params![parent_id])
            .map_err(|e| OrchestratorError::database(format!("Failed to delete {}: {}", context, e)))?;
        Ok(())
    }

    /// Check if a record exists
    pub fn exists(
        conn: &Connection,
        table: &str,
        column: &str,
        value: &str,
        context: &str,
    ) -> Result<bool> {
        let sql = format!("SELECT COUNT(*) FROM {} WHERE {} = ?1", table, column);
        let count: i32 = conn.query_row(&sql, params![value], |row| row.get(0))
            .map_err(|e| OrchestratorError::database(format!("Failed to check {} existence: {}", context, e)))?;
        Ok(count > 0)
    }
}

/// Trait for database entities to reduce boilerplate
pub trait DatabaseEntity {
    type DbModel;
    
    /// Convert from domain model to database model
    fn to_db_model(&self) -> Self::DbModel;
    
    /// Convert from database model to domain model
    fn from_db_model(db_model: Self::DbModel) -> Result<Self>
    where
        Self: Sized;
}

/// Macro to generate common CRUD operations
#[macro_export]
macro_rules! impl_crud_operations {
    ($entity:ty, $db_model:ty, $table:literal, $id_column:literal) => {
        impl $entity {
            pub fn save(&self, conn: &Connection) -> Result<()> {
                let db_model = self.to_db_model();
                // Implementation would be generated based on the table schema
                todo!("Generated save implementation")
            }

            pub fn load(conn: &Connection, id: &str) -> Result<Self> {
                // Implementation would be generated based on the table schema
                todo!("Generated load implementation")
            }

            pub fn delete(conn: &Connection, id: &str) -> Result<()> {
                let sql = format!("DELETE FROM {} WHERE {} = ?1", $table, $id_column);
                DatabaseHelpers::insert_or_replace(conn, &sql, &[&id], "delete entity")?;
                Ok(())
            }
        }
    };
}
