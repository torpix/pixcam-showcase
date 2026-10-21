#![forbid(unsafe_code)]
pub mod bridge;
pub mod coordinates;
pub mod core;
pub mod cycles;
pub mod local_server;
pub mod mcp;
pub mod nc;
pub mod recognition;
pub mod registry;
pub mod showcase;
pub mod solidcraft;
pub type Result<T> = std::result::Result<T, String>;
