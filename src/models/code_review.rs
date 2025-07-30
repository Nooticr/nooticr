use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Types of code review feedback
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewFeedbackType {
    /// Critical issue that must be fixed
    Issue,
    /// Suggestion for improvement
    Suggestion,
    /// Minor style or formatting issue
    Nitpick,
    /// Positive feedback or praise
    Praise,
    /// Question or request for clarification
    Question,
    /// Security concern
    Security,
    /// Performance concern
    Performance,
}

/// Severity level for review feedback
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewSeverity {
    /// Must be fixed before merge
    Critical,
    /// Should be fixed before merge
    Major,
    /// Nice to fix but not blocking
    Minor,
    /// Informational only
    Info,
}

/// Individual piece of feedback on specific code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewComment {
    pub id: Uuid,
    pub file_path: String,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub feedback_type: ReviewFeedbackType,
    pub severity: ReviewSeverity,
    pub message: String,
    pub suggested_change: Option<String>,
    pub code_snippet: Option<String>,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
}

/// Review summary with overall metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total_files_reviewed: u32,
    pub total_lines_reviewed: u32,
    pub issues_found: u32,
    pub suggestions_made: u32,
    pub security_concerns: u32,
    pub performance_concerns: u32,
    pub test_coverage_adequate: bool,
    pub overall_quality_score: u8, // 0-100
}

/// Simplified CodeReview structure for deserializing prompt outputs
/// This matches the format expected by code_review_user_prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewInput {
    pub id: Option<Uuid>, // Always null in prompt output
    pub pull_request_id: String,
    pub approved: bool,
    pub overall_comment: String,
    pub comments: Vec<ReviewCommentInput>,
    pub summary: ReviewSummary,
}

/// Input structure for individual review comments from prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCommentInput {
    pub file_path: String,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub feedback_type: ReviewFeedbackType,
    pub severity: ReviewSeverity,
    pub message: String,
    pub suggested_change: Option<String>,
    pub code_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReview {
    pub id: Uuid,
    pub pull_request_id: String,
    pub reviewer: String,
    pub approved: bool,
    pub overall_comment: String,
    pub comments: Vec<ReviewComment>,
    pub summary: ReviewSummary,
    pub files_reviewed: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ReviewComment {
    /// Create a new review comment
    pub fn new(
        file_path: impl Into<String>,
        line_start: Option<u32>,
        line_end: Option<u32>,
        feedback_type: ReviewFeedbackType,
        severity: ReviewSeverity,
        message: impl Into<String>,
        suggested_change: Option<String>,
        code_snippet: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            file_path: file_path.into(),
            line_start,
            line_end,
            feedback_type,
            severity,
            message: message.into(),
            suggested_change,
            code_snippet,
            resolved: false,
            created_at: Utc::now(),
        }
    }

    /// Mark this comment as resolved
    pub fn resolve(&mut self) {
        self.resolved = true;
    }

    /// Check if this comment is blocking (critical or major severity)
    pub fn is_blocking(&self) -> bool {
        matches!(self.severity, ReviewSeverity::Critical | ReviewSeverity::Major)
    }
}

impl CodeReview {
    /// Create a new code review
    pub fn new(
        pull_request_id: impl Into<String>,
        reviewer: impl Into<String>,
        approved: bool,
        overall_comment: impl Into<String>,
        comments: Vec<ReviewComment>,
        summary: ReviewSummary,
        files_reviewed: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            pull_request_id: pull_request_id.into(),
            reviewer: reviewer.into(),
            approved,
            overall_comment: overall_comment.into(),
            comments,
            summary,
            files_reviewed,
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

        // Convert input comments to ReviewComment objects
        let comments: Vec<ReviewComment> = input.comments
            .into_iter()
            .map(|comment_input| ReviewComment {
                id: Uuid::new_v4(),
                file_path: comment_input.file_path,
                line_start: comment_input.line_start,
                line_end: comment_input.line_end,
                feedback_type: comment_input.feedback_type,
                severity: comment_input.severity,
                message: comment_input.message,
                suggested_change: comment_input.suggested_change,
                code_snippet: comment_input.code_snippet,
                resolved: false,
                created_at: now,
            })
            .collect();

        // Extract unique file paths from comments
        let files_reviewed: Vec<String> = comments
            .iter()
            .map(|c| c.file_path.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        Self {
            id: Uuid::new_v4(),
            pull_request_id: input.pull_request_id,
            reviewer: reviewer.into(),
            approved: input.approved,
            overall_comment: input.overall_comment,
            comments,
            summary: input.summary,
            files_reviewed,
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
    pub fn add_comment(&mut self, comment: ReviewComment) {
        self.comments.push(comment);
        self.updated_at = Utc::now();
    }

    /// Remove a comment from the review by ID
    pub fn remove_comment(&mut self, comment_id: Uuid) -> Option<ReviewComment> {
        if let Some(index) = self.comments.iter().position(|c| c.id == comment_id) {
            let removed = self.comments.remove(index);
            self.updated_at = Utc::now();
            Some(removed)
        } else {
            None
        }
    }

    /// Get comments by file path
    pub fn get_comments_for_file(&self, file_path: &str) -> Vec<&ReviewComment> {
        self.comments.iter().filter(|c| c.file_path == file_path).collect()
    }

    /// Get comments by severity
    pub fn get_comments_by_severity(&self, severity: ReviewSeverity) -> Vec<&ReviewComment> {
        self.comments.iter().filter(|c| c.severity == severity).collect()
    }

    /// Get comments by feedback type
    pub fn get_comments_by_type(&self, feedback_type: ReviewFeedbackType) -> Vec<&ReviewComment> {
        self.comments.iter().filter(|c| c.feedback_type == feedback_type).collect()
    }

    /// Get all blocking comments (critical and major severity)
    pub fn get_blocking_comments(&self) -> Vec<&ReviewComment> {
        self.comments.iter().filter(|c| c.is_blocking()).collect()
    }

    /// Check if there are any unresolved blocking comments
    pub fn has_unresolved_blocking_comments(&self) -> bool {
        self.comments.iter().any(|c| c.is_blocking() && !c.resolved)
    }

    /// Resolve a comment by ID
    pub fn resolve_comment(&mut self, comment_id: Uuid) -> bool {
        if let Some(comment) = self.comments.iter_mut().find(|c| c.id == comment_id) {
            comment.resolve();
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Get count of comments by severity
    pub fn count_by_severity(&self, severity: ReviewSeverity) -> usize {
        self.comments.iter().filter(|c| c.severity == severity).count()
    }

    /// Get count of comments by type
    pub fn count_by_type(&self, feedback_type: ReviewFeedbackType) -> usize {
        self.comments.iter().filter(|c| c.feedback_type == feedback_type).count()
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
    fn test_review_comment_creation() {
        let comment = ReviewComment::new(
            "src/main.rs",
            Some(42),
            Some(45),
            ReviewFeedbackType::Issue,
            ReviewSeverity::Major,
            "Potential null pointer dereference",
            Some("Add null check before dereferencing".to_string()),
            Some("let value = ptr.unwrap();".to_string()),
        );

        assert_eq!(comment.file_path, "src/main.rs");
        assert_eq!(comment.line_start, Some(42));
        assert_eq!(comment.line_end, Some(45));
        assert_eq!(comment.feedback_type, ReviewFeedbackType::Issue);
        assert_eq!(comment.severity, ReviewSeverity::Major);
        assert!(comment.is_blocking());
        assert!(!comment.resolved);
    }

    #[test]
    fn test_code_review_creation() {
        let summary = ReviewSummary {
            total_files_reviewed: 2,
            total_lines_reviewed: 150,
            issues_found: 1,
            suggestions_made: 3,
            security_concerns: 0,
            performance_concerns: 1,
            test_coverage_adequate: true,
            overall_quality_score: 85,
        };

        let comment = ReviewComment::new(
            "src/lib.rs",
            Some(10),
            None,
            ReviewFeedbackType::Suggestion,
            ReviewSeverity::Minor,
            "Consider using a more descriptive variable name",
            Some("let user_count = users.len();".to_string()),
            Some("let n = users.len();".to_string()),
        );

        let review = CodeReview::new(
            "pr-123",
            "reviewer@example.com",
            true,
            "Overall good implementation with minor suggestions",
            vec![comment],
            summary,
            vec!["src/lib.rs".to_string(), "src/main.rs".to_string()],
        );

        assert_eq!(review.pull_request_id, "pr-123");
        assert_eq!(review.reviewer, "reviewer@example.com");
        assert!(review.approved);
        assert_eq!(review.comments.len(), 1);
        assert_eq!(review.files_reviewed.len(), 2);
        assert_eq!(review.summary.overall_quality_score, 85);
    }

    #[test]
    fn test_code_review_from_input() {
        let comment_input = ReviewCommentInput {
            file_path: "src/auth.rs".to_string(),
            line_start: Some(42),
            line_end: Some(45),
            feedback_type: ReviewFeedbackType::Security,
            severity: ReviewSeverity::Critical,
            message: "SQL injection vulnerability".to_string(),
            suggested_change: Some("Use parameterized queries".to_string()),
            code_snippet: Some("SELECT * FROM users WHERE id = {}".to_string()),
        };

        let summary = ReviewSummary {
            total_files_reviewed: 1,
            total_lines_reviewed: 100,
            issues_found: 1,
            suggestions_made: 0,
            security_concerns: 1,
            performance_concerns: 0,
            test_coverage_adequate: false,
            overall_quality_score: 40,
        };

        let input = CodeReviewInput {
            id: None,
            pull_request_id: "pr-456".to_string(),
            approved: false,
            overall_comment: "Critical security issues found".to_string(),
            comments: vec![comment_input],
            summary,
        };

        let review = CodeReview::from_input(input, "security_reviewer");

        assert_eq!(review.pull_request_id, "pr-456");
        assert_eq!(review.reviewer, "security_reviewer");
        assert!(!review.approved);
        assert_eq!(review.comments.len(), 1);
        assert_eq!(review.comments[0].feedback_type, ReviewFeedbackType::Security);
        assert_eq!(review.comments[0].severity, ReviewSeverity::Critical);
        assert!(review.has_unresolved_blocking_comments());
    }

    #[test]
    fn test_code_review_filtering_methods() {
        let mut review = create_test_review_with_multiple_comments();

        // Test filtering by severity
        let critical_comments = review.get_comments_by_severity(ReviewSeverity::Critical);
        assert_eq!(critical_comments.len(), 1);

        let minor_comments = review.get_comments_by_severity(ReviewSeverity::Minor);
        assert_eq!(minor_comments.len(), 1);

        // Test filtering by type
        let security_comments = review.get_comments_by_type(ReviewFeedbackType::Security);
        assert_eq!(security_comments.len(), 1);

        let suggestion_comments = review.get_comments_by_type(ReviewFeedbackType::Suggestion);
        assert_eq!(suggestion_comments.len(), 1);

        // Test blocking comments
        let blocking_comments = review.get_blocking_comments();
        assert_eq!(blocking_comments.len(), 1); // Only critical severity is blocking

        // Test file filtering
        let auth_comments = review.get_comments_for_file("src/auth.rs");
        assert_eq!(auth_comments.len(), 1);

        // Test resolving comments
        let comment_id = review.comments[0].id;
        assert!(review.resolve_comment(comment_id));
        assert!(!review.has_unresolved_blocking_comments());
    }

    fn create_test_review_with_multiple_comments() -> CodeReview {
        let summary = ReviewSummary {
            total_files_reviewed: 2,
            total_lines_reviewed: 200,
            issues_found: 2,
            suggestions_made: 1,
            security_concerns: 1,
            performance_concerns: 0,
            test_coverage_adequate: true,
            overall_quality_score: 75,
        };

        let comments = vec![
            ReviewComment::new(
                "src/auth.rs",
                Some(42),
                Some(45),
                ReviewFeedbackType::Security,
                ReviewSeverity::Critical,
                "SQL injection vulnerability",
                Some("Use parameterized queries".to_string()),
                Some("SELECT * FROM users WHERE id = {}".to_string()),
            ),
            ReviewComment::new(
                "src/utils.rs",
                Some(10),
                None,
                ReviewFeedbackType::Suggestion,
                ReviewSeverity::Minor,
                "Consider using a more descriptive variable name",
                Some("let user_count = users.len();".to_string()),
                Some("let n = users.len();".to_string()),
            ),
        ];

        CodeReview::new(
            "pr-test",
            "test_reviewer",
            false,
            "Mixed review with security concerns",
            comments,
            summary,
            vec!["src/auth.rs".to_string(), "src/utils.rs".to_string()],
        )
    }

    #[test]
    fn test_code_review_from_json_empty_comments() {
        let json = r#"{
            "id": null,
            "pull_request_id": "pr-empty",
            "approved": true,
            "overall_comment": "Clean code, no issues found",
            "comments": [],
            "summary": {
                "total_files_reviewed": 1,
                "total_lines_reviewed": 50,
                "issues_found": 0,
                "suggestions_made": 0,
                "security_concerns": 0,
                "performance_concerns": 0,
                "test_coverage_adequate": true,
                "overall_quality_score": 95
            }
        }"#;

        let review = CodeReview::from_json(json, "auto_reviewer").unwrap();

        assert_eq!(review.pull_request_id, "pr-empty");
        assert_eq!(review.reviewer, "auto_reviewer");
        assert!(review.approved);
        assert_eq!(review.comments.len(), 0);
        assert_eq!(review.overall_comment, "Clean code, no issues found");
        assert_eq!(review.summary.overall_quality_score, 95);
    }

    #[test]
    fn test_code_review_from_json_invalid() {
        let invalid_json = r#"{"invalid": "structure"}"#;
        let result = CodeReview::from_json(invalid_json, "reviewer");
        assert!(result.is_err());
    }









    #[test]
    fn test_code_review_age() {
        let summary = ReviewSummary {
            total_files_reviewed: 1,
            total_lines_reviewed: 50,
            issues_found: 0,
            suggestions_made: 0,
            security_concerns: 0,
            performance_concerns: 0,
            test_coverage_adequate: true,
            overall_quality_score: 90,
        };

        let review = CodeReview::new(
            "pr-123",
            "reviewer",
            true,
            "Good work",
            vec![],
            summary,
            vec!["src/main.rs".to_string()]
        );
        let age = review.age();

        // Age should be very small (just created)
        assert!(age.num_seconds() < 1);
    }
}
