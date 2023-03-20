//! The entry points for every command is defined in this module

pub use init::{init, InitError, InitOptions};

pub mod common;
mod init;
