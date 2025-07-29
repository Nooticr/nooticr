use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::enums::CommentType;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub content: String,
    pub author: String,
    pub comment_type: CommentType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub remotly_synced: bool,
}

impl Comment {
    /// Create a new comment
    pub fn new(
        author: impl Into<String>,
        content: impl Into<String>,
        comment_type: CommentType
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            content: content.into(),
            author: author.into(),
            comment_type,
            created_at: now,
            updated_at: now,
            remotly_synced: false,
        }
    }

    /// Create a new comment with sync status
    pub fn new_with_sync(
        author: impl Into<String>,
        content: impl Into<String>,
        comment_type: CommentType,
        remotly_synced: bool
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            content: content.into(),
            author: author.into(),
            comment_type,
            created_at: now,
            updated_at: now,
            remotly_synced,
        }
    }

    /// Mark comment as synced
    pub fn mark_synced(&mut self) {
        self.remotly_synced = true;
        self.updated_at = Utc::now();
    }

    /// Mark comment as not synced
    pub fn mark_unsynced(&mut self) {
        self.remotly_synced = false;
        self.updated_at = Utc::now();
    }

    /// Update comment content
    pub fn update_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when content changes
    }

    /// Check if comment needs syncing
    pub fn needs_sync(&self) -> bool {
        !self.remotly_synced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_creation() {
        let comment = Comment::new("author", "content", CommentType::Task);

        assert_eq!(comment.author, "author");
        assert_eq!(comment.content, "content");
        assert_eq!(comment.comment_type, CommentType::Task);
        assert!(!comment.remotly_synced);
        assert!(comment.needs_sync());
    }

    #[test]
    fn test_comment_creation_with_sync() {
        let comment = Comment::new_with_sync("author", "content", CommentType::Issue, true);

        assert_eq!(comment.author, "author");
        assert_eq!(comment.content, "content");
        assert_eq!(comment.comment_type, CommentType::Issue);
        assert!(comment.remotly_synced);
        assert!(!comment.needs_sync());
    }

    #[test]
    fn test_comment_sync_operations() {
        let mut comment = Comment::new("author", "content", CommentType::PullRequest);

        // Initially not synced
        assert!(comment.needs_sync());

        // Mark as synced
        comment.mark_synced();
        assert!(!comment.needs_sync());
        assert!(comment.remotly_synced);

        // Mark as unsynced
        comment.mark_unsynced();
        assert!(comment.needs_sync());
        assert!(!comment.remotly_synced);
    }

    #[test]
    fn test_comment_content_update() {
        let mut comment = Comment::new_with_sync("author", "original", CommentType::Task, true);

        // Initially synced
        assert!(!comment.needs_sync());

        // Update content should mark as unsynced
        comment.update_content("updated content");
        assert_eq!(comment.content, "updated content");
        assert!(comment.needs_sync());
        assert!(!comment.remotly_synced);
    }

    #[test]
    fn test_comment_type_default() {
        assert_eq!(CommentType::default(), CommentType::PullRequest);
    }
}