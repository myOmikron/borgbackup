//! # borgbackup
//!
//! Rust interface for communication with the borg backup utility
#![warn(missing_docs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]

#[cfg(feature = "tokio")]
pub mod asynchronous;
pub mod common;
pub mod errors;
pub mod output;
pub mod sync;
