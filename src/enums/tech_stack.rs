use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TechStack {
    #[default]
    Rust,
    Vue,
    React,
    Fullstack(Rust, Vue),
    Fullstack(Rust, React),
}