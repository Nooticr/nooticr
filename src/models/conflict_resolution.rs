use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Types of merge conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    /// Content conflict in the same lines
    ContentConflict,
    /// File was modified in one branch and deleted in another
    ModifyDelete,
    /// File was added in both branches with different content
    AddAdd,
    /// File was renamed in both branches to different names
    RenameRename,
    /// File was renamed in one branch and modified in another
    RenameModify,
    /// Binary file conflict
    BinaryConflict,
    /// Submodule conflict
    SubmoduleConflict,
}

/// Resolution strategy for conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResolutionStrategy {
    /// Take the version from the current branch (ours)
    TakeOurs,
    /// Take the version from the incoming branch (theirs)
    TakeTheirs,
    /// Manually merge both versions
    ManualMerge,
    /// Use a custom resolution
    Custom,
    /// Delete the file
    Delete,
    /// Rename the file
    Rename,
}

/// Individual conflict in a specific file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictDetail {
    pub id: Uuid,
    pub file_path: String,
    pub conflict_type: ConflictType,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub our_content: Option<String>,
    pub their_content: Option<String>,
    pub base_content: Option<String>,
    pub resolution_strategy: ResolutionStrategy,
    pub resolved_content: String,
    pub explanation: String,
    pub confidence_score: u8, // 0-100
    pub requires_testing: bool,
    pub created_at: DateTime<Utc>,
}

/// Summary of the conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionSummary {
    pub total_files_with_conflicts: u32,
    pub total_conflicts_resolved: u32,
    pub conflicts_by_type: std::collections::HashMap<String, u32>,
    pub resolution_strategies_used: std::collections::HashMap<String, u32>,
    pub high_risk_resolutions: u32,
    pub requires_manual_review: bool,
    pub estimated_test_time_minutes: u32,
    pub overall_confidence_score: u8, // 0-100
}

/// Input structure for conflict resolution from prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionInput {
    pub id: Option<Uuid>, // Always null in prompt output
    pub merge_commit_message: String,
    pub branch_info: BranchInfo,
    pub conflicts: Vec<ConflictDetailInput>,
    pub summary: ConflictResolutionSummary,
    pub post_resolution_actions: Vec<String>, // Commands to run after resolution
}

/// Input structure for individual conflicts from prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetailInput {
    pub file_path: String,
    pub conflict_type: ConflictType,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub resolution_strategy: ResolutionStrategy,
    pub resolved_content: String,
    pub explanation: String,
    pub confidence_score: u8,
    pub requires_testing: bool,
}

/// Information about the branches involved in the merge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub source_branch: String,
    pub target_branch: String,
    pub source_commit: String,
    pub target_commit: String,
    pub merge_base: Option<String>,
    pub source_author: Option<String>,
    pub target_author: Option<String>,
}

/// Complete conflict resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub id: Uuid,
    pub merge_commit_message: String,
    pub branch_info: BranchInfo,
    pub conflicts: Vec<ConflictDetail>,
    pub summary: ConflictResolutionSummary,
    pub post_resolution_actions: Vec<String>,
    pub resolver: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConflictDetail {
    /// Create a new conflict detail
    pub fn new(
        file_path: impl Into<String>,
        conflict_type: ConflictType,
        line_start: Option<u32>,
        line_end: Option<u32>,
        our_content: Option<String>,
        their_content: Option<String>,
        base_content: Option<String>,
        resolution_strategy: ResolutionStrategy,
        resolved_content: impl Into<String>,
        explanation: impl Into<String>,
        confidence_score: u8,
        requires_testing: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            file_path: file_path.into(),
            conflict_type,
            line_start,
            line_end,
            our_content,
            their_content,
            base_content,
            resolution_strategy,
            resolved_content: resolved_content.into(),
            explanation: explanation.into(),
            confidence_score,
            requires_testing,
            created_at: Utc::now(),
        }
    }

    /// Check if this conflict resolution is high risk
    pub fn is_high_risk(&self) -> bool {
        self.confidence_score < 70 ||
        matches!(self.conflict_type, ConflictType::BinaryConflict | ConflictType::SubmoduleConflict) ||
        matches!(self.resolution_strategy, ResolutionStrategy::Custom)
    }
}

impl ConflictResolution {
    /// Create a new conflict resolution
    pub fn new(
        merge_commit_message: impl Into<String>,
        branch_info: BranchInfo,
        conflicts: Vec<ConflictDetail>,
        summary: ConflictResolutionSummary,
        post_resolution_actions: Vec<String>,
        resolver: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            merge_commit_message: merge_commit_message.into(),
            branch_info,
            conflicts,
            summary,
            post_resolution_actions,
            resolver: resolver.into(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create from input (from prompt output)
    pub fn from_input(
        input: ConflictResolutionInput,
        resolver: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        
        // Convert input conflicts to ConflictDetail objects
        let conflicts: Vec<ConflictDetail> = input.conflicts
            .into_iter()
            .map(|conflict_input| ConflictDetail {
                id: Uuid::new_v4(),
                file_path: conflict_input.file_path,
                conflict_type: conflict_input.conflict_type,
                line_start: conflict_input.line_start,
                line_end: conflict_input.line_end,
                our_content: None, // Will be filled from git data
                their_content: None, // Will be filled from git data
                base_content: None, // Will be filled from git data
                resolution_strategy: conflict_input.resolution_strategy,
                resolved_content: conflict_input.resolved_content,
                explanation: conflict_input.explanation,
                confidence_score: conflict_input.confidence_score,
                requires_testing: conflict_input.requires_testing,
                created_at: now,
            })
            .collect();

        Self {
            id: Uuid::new_v4(),
            merge_commit_message: input.merge_commit_message,
            branch_info: input.branch_info,
            conflicts,
            summary: input.summary,
            post_resolution_actions: input.post_resolution_actions,
            resolver: resolver.into(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Parse JSON string from conflict resolution prompt output
    pub fn from_json(
        json_str: &str,
        resolver: impl Into<String>,
    ) -> Result<Self, serde_json::Error> {
        let input: ConflictResolutionInput = serde_json::from_str(json_str)?;
        Ok(Self::from_input(input, resolver))
    }

    /// Get conflicts by type
    pub fn get_conflicts_by_type(&self, conflict_type: ConflictType) -> Vec<&ConflictDetail> {
        self.conflicts.iter().filter(|c| c.conflict_type == conflict_type).collect()
    }

    /// Get high-risk conflicts
    pub fn get_high_risk_conflicts(&self) -> Vec<&ConflictDetail> {
        self.conflicts.iter().filter(|c| c.is_high_risk()).collect()
    }

    /// Get conflicts requiring testing
    pub fn get_conflicts_requiring_testing(&self) -> Vec<&ConflictDetail> {
        self.conflicts.iter().filter(|c| c.requires_testing).collect()
    }

    /// Check if any conflicts require manual review
    pub fn requires_manual_review(&self) -> bool {
        self.summary.requires_manual_review || self.get_high_risk_conflicts().len() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_conflict_detail_creation() {
        let conflict = ConflictDetail::new(
            "src/main.rs",
            ConflictType::ContentConflict,
            Some(10),
            Some(15),
            Some("our version".to_string()),
            Some("their version".to_string()),
            Some("base version".to_string()),
            ResolutionStrategy::ManualMerge,
            "resolved content",
            "Merged both versions carefully",
            85,
            true,
        );

        assert_eq!(conflict.file_path, "src/main.rs");
        assert_eq!(conflict.conflict_type, ConflictType::ContentConflict);
        assert_eq!(conflict.line_start, Some(10));
        assert_eq!(conflict.line_end, Some(15));
        assert_eq!(conflict.confidence_score, 85);
        assert!(conflict.requires_testing);
        assert!(!conflict.is_high_risk()); // 85 > 70
    }

    #[test]
    fn test_high_risk_detection() {
        let low_confidence = ConflictDetail::new(
            "src/lib.rs",
            ConflictType::ContentConflict,
            None,
            None,
            None,
            None,
            None,
            ResolutionStrategy::ManualMerge,
            "content",
            "explanation",
            50, // Low confidence
            false,
        );

        let binary_conflict = ConflictDetail::new(
            "assets/image.png",
            ConflictType::BinaryConflict,
            None,
            None,
            None,
            None,
            None,
            ResolutionStrategy::TakeOurs,
            "binary content",
            "explanation",
            90, // High confidence but binary
            false,
        );

        assert!(low_confidence.is_high_risk());
        assert!(binary_conflict.is_high_risk());
    }

    #[test]
    fn test_conflict_resolution_from_input() {
        let branch_info = BranchInfo {
            source_branch: "feature/auth".to_string(),
            target_branch: "main".to_string(),
            source_commit: "abc123".to_string(),
            target_commit: "def456".to_string(),
            merge_base: Some("xyz789".to_string()),
            source_author: Some("dev@company.com".to_string()),
            target_author: Some("maintainer@company.com".to_string()),
        };

        let conflict_input = ConflictDetailInput {
            file_path: "src/auth.rs".to_string(),
            conflict_type: ConflictType::ContentConflict,
            line_start: Some(42),
            line_end: Some(58),
            resolution_strategy: ResolutionStrategy::ManualMerge,
            resolved_content: "resolved auth code".to_string(),
            explanation: "Merged authentication methods".to_string(),
            confidence_score: 85,
            requires_testing: true,
        };

        let mut conflicts_by_type = HashMap::new();
        conflicts_by_type.insert("ContentConflict".to_string(), 1);

        let mut strategies_used = HashMap::new();
        strategies_used.insert("ManualMerge".to_string(), 1);

        let summary = ConflictResolutionSummary {
            total_files_with_conflicts: 1,
            total_conflicts_resolved: 1,
            conflicts_by_type,
            resolution_strategies_used: strategies_used,
            high_risk_resolutions: 0,
            requires_manual_review: false,
            estimated_test_time_minutes: 30,
            overall_confidence_score: 85,
        };

        let input = ConflictResolutionInput {
            id: None,
            merge_commit_message: "Merge feature/auth into main".to_string(),
            branch_info,
            conflicts: vec![conflict_input],
            summary,
            post_resolution_actions: vec!["cargo test".to_string()],
        };

        let resolution = ConflictResolution::from_input(input, "ConflictResolver");

        assert_eq!(resolution.merge_commit_message, "Merge feature/auth into main");
        assert_eq!(resolution.resolver, "ConflictResolver");
        assert_eq!(resolution.conflicts.len(), 1);
        assert_eq!(resolution.conflicts[0].file_path, "src/auth.rs");
        assert_eq!(resolution.summary.overall_confidence_score, 85);
        assert!(!resolution.requires_manual_review());
    }

    #[test]
    fn test_conflict_filtering() {
        let conflicts = vec![
            ConflictDetail::new(
                "src/main.rs",
                ConflictType::ContentConflict,
                Some(10),
                Some(15),
                None,
                None,
                None,
                ResolutionStrategy::ManualMerge,
                "content1",
                "explanation1",
                90,
                true,
            ),
            ConflictDetail::new(
                "src/lib.rs",
                ConflictType::ModifyDelete,
                None,
                None,
                None,
                None,
                None,
                ResolutionStrategy::TakeOurs,
                "content2",
                "explanation2",
                60, // High risk due to low confidence
                false,
            ),
            ConflictDetail::new(
                "assets/image.png",
                ConflictType::BinaryConflict,
                None,
                None,
                None,
                None,
                None,
                ResolutionStrategy::TakeTheirs,
                "binary",
                "explanation3",
                95, // High risk due to binary type
                true,
            ),
        ];

        let resolution = ConflictResolution::new(
            "Test merge",
            BranchInfo {
                source_branch: "feature".to_string(),
                target_branch: "main".to_string(),
                source_commit: "abc".to_string(),
                target_commit: "def".to_string(),
                merge_base: None,
                source_author: None,
                target_author: None,
            },
            conflicts,
            ConflictResolutionSummary {
                total_files_with_conflicts: 3,
                total_conflicts_resolved: 3,
                conflicts_by_type: HashMap::new(),
                resolution_strategies_used: HashMap::new(),
                high_risk_resolutions: 2,
                requires_manual_review: true,
                estimated_test_time_minutes: 60,
                overall_confidence_score: 75,
            },
            vec!["cargo test".to_string()],
            "TestResolver",
        );

        // Test filtering by type
        let content_conflicts = resolution.get_conflicts_by_type(ConflictType::ContentConflict);
        assert_eq!(content_conflicts.len(), 1);

        let binary_conflicts = resolution.get_conflicts_by_type(ConflictType::BinaryConflict);
        assert_eq!(binary_conflicts.len(), 1);

        // Test high-risk filtering
        let high_risk = resolution.get_high_risk_conflicts();
        assert_eq!(high_risk.len(), 2); // Low confidence + binary conflict

        // Test testing requirements
        let needs_testing = resolution.get_conflicts_requiring_testing();
        assert_eq!(needs_testing.len(), 2);

        // Test manual review requirement
        assert!(resolution.requires_manual_review());
    }
}
