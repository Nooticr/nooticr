use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum CommentType {
    #[default]
    PullRequest,
    Task,
    Issue,
}
