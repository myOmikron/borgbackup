//! # borgbackup
//!
//! BorgBackup (short: Borg) is a deduplicating backup program.
//! Optionally, it supports compression and authenticated encryption.
//!
//! The main goal of Borg is to provide an efficient and secure way to backup data.
//! The data deduplication technique used makes Borg suitable for daily backups since only
//! changes are stored. The authenticated encryption technique makes it suitable for backups
//! to not fully trusted targets.
//!
//! ---
//!
//! This library provides a wrapper to call borg programmatically.
//! The output of borg is parsed according to the definitions in:
//! <https://borgbackup.readthedocs.io/en/stable/internals/frontends.html>
//!
//!
//! ## Features
//! - `tokio`: provides the [asynchronous] module
//!
#![warn(missing_docs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]

#[cfg(feature = "tokio")]
pub mod asynchronous;
pub mod common;
pub mod errors;
pub mod output;
pub mod sync;
