//! The entry points for every command is defined in this module

#[cfg(feature = "tokio")]
pub use init::init_async;
pub use init::{init, InitError, InitOptions};

pub mod common;
mod init;
