use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum AgentType {
    BackendEngineerRust,
    BackendQARust,
    #[default]
    FrontendEngineerVue,
    FrontendQAVue,
    FrontendEngineerReact,
    FrontendQAReact,
    DevOps,
    PerformanceEngineer,
    SecurityEngineer,
    SecurityAuditor,
    CodeReviewEngineer,
    ReleaseManager,
}