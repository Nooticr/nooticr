use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TechStack {
    #[default]
    Rust,
    Vue,
    React,
    FullstackRustVue,
    FullstackRustReact,
}

// Re-export the variants for easier access
pub use TechStack::*;