//! # borgbackup
//!
//! Rust interface for communication with the borg backup utility
#![warn(missing_docs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]

pub mod commands;
pub mod output;
