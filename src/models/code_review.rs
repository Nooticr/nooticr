use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Simplified CodeReview structure for deserializing prompt outputs
/// This matches the format expected by code_review_user_prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewInput {
    pub id: Option<Uuid>, // Always null in prompt output
    pub pull_request_id: String,
    pub approved: bool,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReview {
    pub id: Uuid,
    pub pull_request_id: String,
    pub reviewer: String,
    pub approved: bool,
    pub comments: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CodeReview {
    /// Create a new code review
    pub fn new(
        pull_request_id: impl Into<String>,
        reviewer: impl Into<String>,
        approved: bool,
        comments: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            pull_request_id: pull_request_id.into(),
            reviewer: reviewer.into(),
            approved,
            comments,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a CodeReview from a CodeReviewInput (from prompt output)
    pub fn from_input(
        input: CodeReviewInput,
        reviewer: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            pull_request_id: input.pull_request_id,
            reviewer: reviewer.into(),
            approved: input.approved,
            comments: input.comments,
            created_at: now,
            updated_at: now,
        }
    }

    /// Deserialize a JSON string from code_review_user_prompt output
    ///
    /// This parses the JSON output from the code review prompt and creates
    /// a CodeReview with the specified reviewer.
    ///
    /// # Example
    /// ```rust
    /// use orchy::models::code_review::CodeReview;
    ///
    /// let json = r#"{
    ///     "id": null,
    ///     "pull_request_id": "pr-123",
    ///     "approved": true,
    ///     "comments": ["Looks good!", "Minor style issue on line 42"]
    /// }"#;
    ///
    /// let review = CodeReview::from_json(json, "reviewer@example.com").unwrap();
    /// assert_eq!(review.pull_request_id, "pr-123");
    /// assert!(review.approved);
    /// assert_eq!(review.comments.len(), 2);
    /// ```
    pub fn from_json(
        json_str: &str,
        reviewer: impl Into<String>,
    ) -> Result<Self, serde_json::Error> {
        let input: CodeReviewInput = serde_json::from_str(json_str)?;
        Ok(Self::from_input(input, reviewer))
    }

    /// Update the review status
    pub fn update_approval(&mut self, approved: bool) {
        self.approved = approved;
        self.updated_at = Utc::now();
    }

    /// Add a comment to the review
    pub fn add_comment(&mut self, comment: impl Into<String>) {
        self.comments.push(comment.into());
        self.updated_at = Utc::now();
    }

    /// Remove a comment from the review
    pub fn remove_comment(&mut self, index: usize) -> Option<String> {
        if index < self.comments.len() {
            let removed = self.comments.remove(index);
            self.updated_at = Utc::now();
            Some(removed)
        } else {
            None
        }
    }

    /// Update a comment in the review
    pub fn update_comment(&mut self, index: usize, new_comment: impl Into<String>) -> bool {
        if index < self.comments.len() {
            self.comments[index] = new_comment.into();
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Get the age of the review
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_review_creation() {
        let review = CodeReview::new("pr-123", "reviewer@example.com", true, vec!["LGTM".to_string()]);

        assert_eq!(review.pull_request_id, "pr-123");
        assert_eq!(review.reviewer, "reviewer@example.com");
        assert!(review.approved);
        assert_eq!(review.comments.len(), 1);
        assert_eq!(review.comments[0], "LGTM");
    }

    #[test]
    fn test_code_review_from_input() {
        let input = CodeReviewInput {
            id: None,
            pull_request_id: "pr-456".to_string(),
            approved: false,
            comments: vec!["Needs changes".to_string(), "Fix the bug".to_string()],
        };

        let review = CodeReview::from_input(input, "senior_dev");

        assert_eq!(review.pull_request_id, "pr-456");
        assert_eq!(review.reviewer, "senior_dev");
        assert!(!review.approved);
        assert_eq!(review.comments.len(), 2);
        assert_eq!(review.comments[0], "Needs changes");
        assert_eq!(review.comments[1], "Fix the bug");
    }

    #[test]
    fn test_code_review_from_json() {
        let json = r#"{
            "id": null,
            "pull_request_id": "pr-789",
            "approved": true,
            "comments": ["Great work!", "Minor style issue on line 42", "Consider adding unit tests"]
        }"#;

        let review = CodeReview::from_json(json, "tech_lead@company.com").unwrap();

        assert_eq!(review.pull_request_id, "pr-789");
        assert_eq!(review.reviewer, "tech_lead@company.com");
        assert!(review.approved);
        assert_eq!(review.comments.len(), 3);
        assert_eq!(review.comments[0], "Great work!");
        assert_eq!(review.comments[1], "Minor style issue on line 42");
        assert_eq!(review.comments[2], "Consider adding unit tests");
    }

    #[test]
    fn test_code_review_from_json_rejected() {
        let json = r#"{
            "id": null,
            "pull_request_id": "pr-999",
            "approved": false,
            "comments": [
                "Memory leak in function process_data()",
                "Missing error handling for network requests",
                "Please add documentation for public APIs"
            ]
        }"#;

        let review = CodeReview::from_json(json, "security_reviewer").unwrap();

        assert_eq!(review.pull_request_id, "pr-999");
        assert_eq!(review.reviewer, "security_reviewer");
        assert!(!review.approved);
        assert_eq!(review.comments.len(), 3);
        assert!(review.comments[0].contains("Memory leak"));
        assert!(review.comments[1].contains("error handling"));
        assert!(review.comments[2].contains("documentation"));
    }

    #[test]
    fn test_code_review_from_json_empty_comments() {
        let json = r#"{
            "id": null,
            "pull_request_id": "pr-empty",
            "approved": true,
            "comments": []
        }"#;

        let review = CodeReview::from_json(json, "auto_reviewer").unwrap();

        assert_eq!(review.pull_request_id, "pr-empty");
        assert_eq!(review.reviewer, "auto_reviewer");
        assert!(review.approved);
        assert_eq!(review.comments.len(), 0);
    }

    #[test]
    fn test_code_review_from_json_invalid() {
        let invalid_json = r#"{"invalid": "structure"}"#;
        let result = CodeReview::from_json(invalid_json, "reviewer");
        assert!(result.is_err());
    }

    #[test]
    fn test_code_review_update_approval() {
        let mut review = CodeReview::new("pr-123", "reviewer", false, vec!["Needs work".to_string()]);
        let original_updated_at = review.updated_at;

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(1));

        review.update_approval(true);
        assert!(review.approved);
        assert!(review.updated_at > original_updated_at);
    }

    #[test]
    fn test_code_review_add_comment() {
        let mut review = CodeReview::new("pr-123", "reviewer", true, vec![]);

        review.add_comment("First comment");
        assert_eq!(review.comments.len(), 1);
        assert_eq!(review.comments[0], "First comment");

        review.add_comment("Second comment");
        assert_eq!(review.comments.len(), 2);
        assert_eq!(review.comments[1], "Second comment");
    }

    #[test]
    fn test_code_review_remove_comment() {
        let mut review = CodeReview::new(
            "pr-123",
            "reviewer",
            true,
            vec!["Comment 1".to_string(), "Comment 2".to_string(), "Comment 3".to_string()]
        );

        let removed = review.remove_comment(1);
        assert_eq!(removed, Some("Comment 2".to_string()));
        assert_eq!(review.comments.len(), 2);
        assert_eq!(review.comments[0], "Comment 1");
        assert_eq!(review.comments[1], "Comment 3");

        // Test removing invalid index
        let removed = review.remove_comment(10);
        assert_eq!(removed, None);
        assert_eq!(review.comments.len(), 2);
    }

    #[test]
    fn test_code_review_update_comment() {
        let mut review = CodeReview::new(
            "pr-123",
            "reviewer",
            true,
            vec!["Original comment".to_string()]
        );

        let success = review.update_comment(0, "Updated comment");
        assert!(success);
        assert_eq!(review.comments[0], "Updated comment");

        // Test updating invalid index
        let success = review.update_comment(10, "Invalid update");
        assert!(!success);
        assert_eq!(review.comments.len(), 1);
    }

    #[test]
    fn test_code_review_age() {
        let review = CodeReview::new("pr-123", "reviewer", true, vec![]);
        let age = review.age();

        // Age should be very small (just created)
        assert!(age.num_seconds() < 1);
    }
}
