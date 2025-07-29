use crate::enums::{CodeStatus, CommentType};
use crate::error::{OrchestratorError, Result};
use super::comment::Comment;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::code_review::CodeReview;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: Uuid,
    pub github_pr_number: Option<u64>,
    pub title: String,
    pub description: String,
    pub source_branch: String,
    pub target_branch: String,
    pub author: String,
    pub assignees: Vec<String>,
    pub reviewers: Vec<String>,
    pub labels: Vec<String>,
    pub code_status: CodeStatus,
    pub code_status_history: Vec<(CodeStatus, DateTime<Utc>)>,
    pub ci_attemps: u32,
    pub comments: Vec<Comment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub remotly_synced: bool,
    pub code_reviews: Vec<CodeReview>,
}

impl PullRequest {
    /// Create a new pull request
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        source_branch: impl Into<String>,
        target_branch: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        let initial_status = CodeStatus::PullRequest;

        Self {
            id: Uuid::new_v4(),
            github_pr_number: None,
            title: title.into(),
            description: description.into(),
            source_branch: source_branch.into(),
            target_branch: target_branch.into(),
            author: author.into(),
            assignees: Vec::new(),
            reviewers: Vec::new(),
            labels: Vec::new(),
            code_status: initial_status,
            code_status_history: vec![(initial_status, now)],
            ci_attemps: 0,
            comments: Vec::new(),
            created_at: now,
            updated_at: now,
            merged_at: None,
            closed_at: None,
            remotly_synced: false,
            code_reviews: Vec::new(),
        }
    }

    /// Transition code status with validation
    pub fn transition_code_status(&mut self, next_status: CodeStatus) -> Result<()> {
        // Validate the transition
        let new_status = self.code_status.transition_to(next_status)?;

        // Apply business rules
        match (&self.code_status, &new_status) {
            // Track CI attempts
            (CodeStatus::PullRequest, CodeStatus::CIFailed | CodeStatus::CISuccessful) => {
                self.ci_attemps += 1;
            }
            // Track merging
            (CodeStatus::Mergeable, CodeStatus::Merged) => {
                self.merged_at = Some(Utc::now());
            }
            // Track abandonment
            (_, CodeStatus::Abandoned) => {
                self.closed_at = Some(Utc::now());
            }
            _ => {}
        }

        // Update status and history
        let now = Utc::now();
        self.code_status = new_status;
        self.code_status_history.push((new_status, now));
        self.updated_at = now;
        self.remotly_synced = false; // Mark as unsynced when status changes

        Ok(())
    }

    /// Handle CI result
    pub fn handle_ci_result(&mut self, success: bool) -> Result<()> {
        if self.code_status != CodeStatus::PullRequest {
            return Err(OrchestratorError::validation(
                "CI can only run on pull requests",
            ));
        }

        let next_status = if success {
            CodeStatus::CISuccessful
        } else {
            CodeStatus::CIFailed
        };

        self.transition_code_status(next_status)
    }

    /// Add a comment to the pull request
    pub fn add_comment(&mut self, author: impl Into<String>, content: impl Into<String>) {
        let comment = Comment::new(author, content, CommentType::PullRequest);
        self.comments.push(comment);
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when comments are added
    }

    /// Add a comment with sync status
    pub fn add_comment_with_sync(
        &mut self,
        author: impl Into<String>,
        content: impl Into<String>,
        synced: bool,
    ) {
        let comment = Comment::new_with_sync(author, content, CommentType::PullRequest, synced);
        self.comments.push(comment);
        self.updated_at = Utc::now();
        if !synced {
            self.remotly_synced = false;
        }
    }

    /// Get all unsynced comments
    pub fn get_unsynced_comments(&self) -> Vec<&Comment> {
        self.comments.iter().filter(|c| c.needs_sync()).collect()
    }

    /// Mark all comments as synced
    pub fn mark_all_comments_synced(&mut self) {
        for comment in &mut self.comments {
            comment.mark_synced();
        }
        self.updated_at = Utc::now();
    }

    /// Add a code review to the pull request
    pub fn add_code_review(
        &mut self,
        reviewer: impl Into<String>,
        approved: bool,
        comments: Vec<String>,
    ) -> Uuid {
        let review = CodeReview::new(self.id.to_string(), reviewer, approved, comments);
        let review_id = review.id;
        self.code_reviews.push(review);
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when reviews are added
        review_id
    }

    /// Get all code reviews
    pub fn get_code_reviews(&self) -> &Vec<CodeReview> {
        &self.code_reviews
    }

    /// Get approved reviews count
    pub fn get_approved_reviews_count(&self) -> usize {
        self.code_reviews.iter().filter(|r| r.approved).count()
    }

    /// Check if PR has any approved reviews
    pub fn has_approved_reviews(&self) -> bool {
        self.code_reviews.iter().any(|r| r.approved)
    }

    /// Check if PR has any rejected reviews
    pub fn has_rejected_reviews(&self) -> bool {
        self.code_reviews.iter().any(|r| !r.approved)
    }

    /// Get review by ID
    pub fn get_code_review(&self, review_id: Uuid) -> Option<&CodeReview> {
        self.code_reviews.iter().find(|r| r.id == review_id)
    }

    /// Get mutable review by ID
    pub fn get_code_review_mut(&mut self, review_id: Uuid) -> Option<&mut CodeReview> {
        self.code_reviews.iter_mut().find(|r| r.id == review_id)
    }

    /// Get reviews by reviewer
    pub fn get_reviews_by_reviewer(&self, reviewer: &str) -> Vec<&CodeReview> {
        self.code_reviews.iter().filter(|r| r.reviewer == reviewer).collect()
    }

    /// Update a code review
    pub fn update_code_review(
        &mut self,
        review_id: Uuid,
        approved: bool,
    ) -> Result<()> {
        let review = self
            .get_code_review_mut(review_id)
            .ok_or_else(|| OrchestratorError::validation("Code review not found"))?;

        review.update_approval(approved);
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when reviews are modified
        Ok(())
    }

    /// Remove a code review
    pub fn remove_code_review(&mut self, review_id: Uuid) -> Result<CodeReview> {
        let position = self
            .code_reviews
            .iter()
            .position(|r| r.id == review_id)
            .ok_or_else(|| OrchestratorError::validation("Code review not found"))?;

        let removed_review = self.code_reviews.remove(position);
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when reviews are removed
        Ok(removed_review)
    }

    /// Get comment by ID
    pub fn get_comment(&self, comment_id: Uuid) -> Option<&Comment> {
        self.comments.iter().find(|c| c.id == comment_id)
    }

    /// Get mutable comment by ID
    pub fn get_comment_mut(&mut self, comment_id: Uuid) -> Option<&mut Comment> {
        self.comments.iter_mut().find(|c| c.id == comment_id)
    }

    /// Update a comment's content
    pub fn update_comment(
        &mut self,
        comment_id: Uuid,
        new_content: impl Into<String>,
    ) -> Result<()> {
        let comment = self
            .get_comment_mut(comment_id)
            .ok_or_else(|| OrchestratorError::validation("Comment not found"))?;

        comment.update_content(new_content);
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when comments are modified
        Ok(())
    }

    /// Remove a comment
    pub fn remove_comment(&mut self, comment_id: Uuid) -> Result<Comment> {
        let position = self
            .comments
            .iter()
            .position(|c| c.id == comment_id)
            .ok_or_else(|| OrchestratorError::validation("Comment not found"))?;

        let removed_comment = self.comments.remove(position);
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when comments are removed
        Ok(removed_comment)
    }

    // ===== SYNC MANAGEMENT =====

    /// Mark pull request as synced with remote system
    pub fn mark_synced(&mut self) {
        self.remotly_synced = true;
        self.updated_at = Utc::now();
    }

    /// Mark pull request as not synced with remote system
    pub fn mark_unsynced(&mut self) {
        self.remotly_synced = false;
        self.updated_at = Utc::now();
    }

    /// Check if pull request needs syncing with remote system
    pub fn needs_sync(&self) -> bool {
        !self.remotly_synced
    }

    /// Update pull request title and mark as unsynced
    pub fn update_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.remotly_synced = false;
        self.updated_at = Utc::now();
    }

    /// Update pull request description and mark as unsynced
    pub fn update_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
        self.remotly_synced = false;
        self.updated_at = Utc::now();
    }

    /// Set GitHub PR number
    pub fn set_github_pr_number(&mut self, pr_number: u64) {
        self.github_pr_number = Some(pr_number);
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when GitHub number changes
    }

    /// Add an assignee
    pub fn add_assignee(&mut self, assignee: impl Into<String>) {
        let assignee = assignee.into();
        if !self.assignees.contains(&assignee) {
            self.assignees.push(assignee);
            self.updated_at = Utc::now();
            self.remotly_synced = false; // Mark as unsynced when assignees change
        }
    }

    /// Remove an assignee
    pub fn remove_assignee(&mut self, assignee: &str) {
        self.assignees.retain(|a| a != assignee);
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when assignees change
    }

    /// Add a reviewer
    pub fn add_reviewer(&mut self, reviewer: impl Into<String>) {
        let reviewer = reviewer.into();
        if !self.reviewers.contains(&reviewer) {
            self.reviewers.push(reviewer);
            self.updated_at = Utc::now();
            self.remotly_synced = false; // Mark as unsynced when reviewers change
        }
    }

    /// Remove a reviewer
    pub fn remove_reviewer(&mut self, reviewer: &str) {
        self.reviewers.retain(|r| r != reviewer);
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when reviewers change
    }

    /// Add a label
    pub fn add_label(&mut self, label: impl Into<String>) {
        let label = label.into();
        if !self.labels.contains(&label) {
            self.labels.push(label);
            self.updated_at = Utc::now();
            self.remotly_synced = false; // Mark as unsynced when labels change
        }
    }

    /// Remove a label
    pub fn remove_label(&mut self, label: &str) {
        self.labels.retain(|l| l != label);
        self.updated_at = Utc::now();
        self.remotly_synced = false; // Mark as unsynced when labels change
    }

    // ===== STATUS CHECKS =====

    /// Check if the pull request is open
    pub fn is_open(&self) -> bool {
        !matches!(self.code_status, CodeStatus::Merged | CodeStatus::Abandoned)
    }

    /// Check if the pull request is merged
    pub fn is_merged(&self) -> bool {
        self.code_status == CodeStatus::Merged
    }

    /// Check if the pull request is closed/abandoned
    pub fn is_closed(&self) -> bool {
        self.code_status == CodeStatus::Abandoned
    }

    /// Check if CI is passing
    pub fn is_ci_passing(&self) -> bool {
        self.code_status == CodeStatus::CISuccessful
    }

    /// Check if the pull request is ready to merge
    pub fn is_ready_to_merge(&self) -> bool {
        self.code_status == CodeStatus::Mergeable
    }

    /// Get the age of the pull request
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_request_creation() {
        let pr = PullRequest::new(
            "Fix: Test feature",
            "This fixes the test feature",
            "feature/test",
            "main",
            "developer",
        );

        assert_eq!(pr.title, "Fix: Test feature");
        assert_eq!(pr.description, "This fixes the test feature");
        assert_eq!(pr.source_branch, "feature/test");
        assert_eq!(pr.target_branch, "main");
        assert_eq!(pr.author, "developer");
        assert_eq!(pr.code_status, CodeStatus::PullRequest);
        assert!(!pr.remotly_synced);
        assert!(pr.needs_sync());
        assert_eq!(pr.ci_attemps, 0);
    }

    #[test]
    fn test_pull_request_sync_operations() {
        let mut pr = PullRequest::new("Test", "Description", "feature", "main", "author");

        // Initially unsynced
        assert!(pr.needs_sync());

        // Mark as synced
        pr.mark_synced();
        assert!(!pr.needs_sync());
        assert!(pr.remotly_synced);

        // Mark as unsynced
        pr.mark_unsynced();
        assert!(pr.needs_sync());
        assert!(!pr.remotly_synced);
    }

    #[test]
    fn test_pull_request_update_methods_unsync() {
        let mut pr = PullRequest::new("Test", "Description", "feature", "main", "author");

        // Mark as synced first
        pr.mark_synced();
        assert!(!pr.needs_sync());

        // Test title update
        pr.update_title("New Title");
        assert_eq!(pr.title, "New Title");
        assert!(pr.needs_sync());

        // Reset sync and test description update
        pr.mark_synced();
        pr.update_description("New Description");
        assert_eq!(pr.description, "New Description");
        assert!(pr.needs_sync());
    }

    #[test]
    fn test_pull_request_assignee_operations() {
        let mut pr = PullRequest::new("Test", "Description", "feature", "main", "author");
        pr.mark_synced();

        // Add assignee should unsync
        pr.add_assignee("assignee1");
        assert!(pr.assignees.contains(&"assignee1".to_string()));
        assert!(pr.needs_sync());

        // Reset sync and test remove assignee
        pr.mark_synced();
        pr.remove_assignee("assignee1");
        assert!(!pr.assignees.contains(&"assignee1".to_string()));
        assert!(pr.needs_sync());
    }

    #[test]
    fn test_pull_request_reviewer_operations() {
        let mut pr = PullRequest::new("Test", "Description", "feature", "main", "author");
        pr.mark_synced();

        // Add reviewer should unsync
        pr.add_reviewer("reviewer1");
        assert!(pr.reviewers.contains(&"reviewer1".to_string()));
        assert!(pr.needs_sync());

        // Reset sync and test remove reviewer
        pr.mark_synced();
        pr.remove_reviewer("reviewer1");
        assert!(!pr.reviewers.contains(&"reviewer1".to_string()));
        assert!(pr.needs_sync());
    }

    #[test]
    fn test_pull_request_label_operations() {
        let mut pr = PullRequest::new("Test", "Description", "feature", "main", "author");
        pr.mark_synced();

        // Add label should unsync
        pr.add_label("bug");
        assert!(pr.labels.contains(&"bug".to_string()));
        assert!(pr.needs_sync());

        // Reset sync and test remove label
        pr.mark_synced();
        pr.remove_label("bug");
        assert!(!pr.labels.contains(&"bug".to_string()));
        assert!(pr.needs_sync());
    }

    #[test]
    fn test_pull_request_ci_handling() {
        let mut pr = PullRequest::new("Test", "Description", "feature", "main", "author");

        assert_eq!(pr.ci_attemps, 0);
        assert_eq!(pr.code_status, CodeStatus::PullRequest); // Should start in PullRequest status

        // Test failed CI
        pr.handle_ci_result(false).unwrap();
        assert_eq!(pr.code_status, CodeStatus::CIFailed);
        assert_eq!(pr.ci_attemps, 1);

        // Reset to PullRequest status for next test (go through Coded first)
        pr.transition_code_status(CodeStatus::Coded).unwrap();
        pr.transition_code_status(CodeStatus::PullRequest).unwrap();

        // Test successful CI
        pr.handle_ci_result(true).unwrap();
        assert_eq!(pr.code_status, CodeStatus::CISuccessful);
        assert_eq!(pr.ci_attemps, 2);
    }

    #[test]
    fn test_pull_request_status_checks() {
        let mut pr = PullRequest::new("Test", "Description", "feature", "main", "author");

        // Initially open
        assert!(pr.is_open());
        assert!(!pr.is_merged());
        assert!(!pr.is_closed());

        // Transition to merged
        pr.transition_code_status(CodeStatus::CISuccessful).unwrap();
        pr.transition_code_status(CodeStatus::Mergeable).unwrap();
        pr.transition_code_status(CodeStatus::Merged).unwrap();

        assert!(!pr.is_open());
        assert!(pr.is_merged());
        assert!(!pr.is_closed());
        assert!(pr.merged_at.is_some());
    }

    #[test]
    fn test_pull_request_comment_management() {
        let mut pr = PullRequest::new("Test", "Description", "feature", "main", "author");

        // Add comment
        pr.add_comment("user1", "Great work!");
        assert_eq!(pr.comments.len(), 1);
        assert_eq!(pr.comments[0].comment_type, CommentType::PullRequest);
        assert!(pr.comments[0].needs_sync());

        // Add synced comment
        pr.add_comment_with_sync("user2", "LGTM", true);
        assert_eq!(pr.comments.len(), 2);
        assert!(!pr.comments[1].needs_sync());

        // Test unsynced comments
        let unsynced = pr.get_unsynced_comments();
        assert_eq!(unsynced.len(), 1);

        // Mark all as synced
        pr.mark_all_comments_synced();
        assert_eq!(pr.get_unsynced_comments().len(), 0);
    }

    #[test]
    fn test_pull_request_code_review_management() {
        let mut pr = PullRequest::new("Test", "Description", "feature", "main", "author");
        pr.mark_synced();

        // Initially no reviews
        assert_eq!(pr.get_code_reviews().len(), 0);
        assert!(!pr.has_approved_reviews());
        assert!(!pr.has_rejected_reviews());
        assert_eq!(pr.get_approved_reviews_count(), 0);

        // Add approved review
        let review1_id = pr.add_code_review("reviewer1", true, vec!["Looks good!".to_string()]);
        assert_eq!(pr.get_code_reviews().len(), 1);
        assert!(pr.has_approved_reviews());
        assert!(!pr.has_rejected_reviews());
        assert_eq!(pr.get_approved_reviews_count(), 1);
        assert!(pr.needs_sync()); // Should mark as unsynced

        // Add rejected review
        pr.mark_synced();
        let review2_id = pr.add_code_review("reviewer2", false, vec!["Needs changes".to_string(), "Fix the bug".to_string()]);
        assert_eq!(pr.get_code_reviews().len(), 2);
        assert!(pr.has_approved_reviews());
        assert!(pr.has_rejected_reviews());
        assert_eq!(pr.get_approved_reviews_count(), 1);
        assert!(pr.needs_sync()); // Should mark as unsynced again

        // Test getting reviews by ID
        let review1 = pr.get_code_review(review1_id).unwrap();
        assert!(review1.approved);
        assert_eq!(review1.comments.len(), 1);
        assert_eq!(review1.reviewer, "reviewer1");

        let review2 = pr.get_code_review(review2_id).unwrap();
        assert!(!review2.approved);
        assert_eq!(review2.comments.len(), 2);
        assert_eq!(review2.reviewer, "reviewer2");

        // Test getting reviews by reviewer
        let reviewer1_reviews = pr.get_reviews_by_reviewer("reviewer1");
        assert_eq!(reviewer1_reviews.len(), 1);
        assert!(reviewer1_reviews[0].approved);

        let reviewer2_reviews = pr.get_reviews_by_reviewer("reviewer2");
        assert_eq!(reviewer2_reviews.len(), 1);
        assert!(!reviewer2_reviews[0].approved);

        // Test updating review
        pr.mark_synced();
        assert!(pr.update_code_review(review2_id, true).is_ok());
        assert!(pr.needs_sync()); // Should mark as unsynced
        let updated_review = pr.get_code_review(review2_id).unwrap();
        assert!(updated_review.approved);

        // Test removing review
        pr.mark_synced();
        assert!(pr.remove_code_review(review1_id).is_ok());
        assert_eq!(pr.get_code_reviews().len(), 1);
        assert!(pr.needs_sync()); // Should mark as unsynced

        // Test non-existent review
        let fake_id = Uuid::new_v4();
        assert!(pr.get_code_review(fake_id).is_none());
        assert!(pr.update_code_review(fake_id, true).is_err());
        assert!(pr.remove_code_review(fake_id).is_err());
    }
}
