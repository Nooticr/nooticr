use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Priority {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}
