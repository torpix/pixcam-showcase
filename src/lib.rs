#![forbid(unsafe_code)]
pub mod core;
pub mod mcp;
pub mod nc;
pub mod showcase;
pub type Result<T> = std::result::Result<T, String>;
