//! All errors of this crate are defined in this module

use std::io;

use thiserror::Error;

use crate::output::logging::MessageId;

/// The errors that can be returned from [crate::sync::compact]
#[derive(Error, Debug)]
pub enum CompactError {
    /// An unknown error occurred
    #[error("Unknown error occurred: {0}")]
    Unknown(String),
    /// Error while splitting the arguments
    #[error("error while splitting the arguments")]
    ShlexError,
    /// The command failed to execute
    #[error("The command failed to execute: {0}")]
    CommandFailed(#[from] io::Error),
    /// Invalid borg output found
    #[error("Error while deserializing borg output: {0}")]
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    #[error("Error while deserializing borg output: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Borg was terminated by a signal
    #[error("Borg was terminated by a signal")]
    TerminatedBySignal,
    /// Piping from stdout or stderr failed
    #[error("Piping from stdout or stderr failed")]
    PipeFailed,
    /// An unexpected message id was received
    #[error("An unexpected message id was received: {0}")]
    UnexpectedMessageId(MessageId),
}

/// The errors that can be returned from [crate::sync::prune]
#[derive(Debug, Error)]
pub enum PruneError {
    /// An unknown error occurred
    #[error("Unknown error occurred: {0}")]
    Unknown(String),
    /// Error while splitting the arguments
    #[error("error while splitting the arguments")]
    ShlexError,
    /// The command failed to execute
    #[error("The command failed to execute: {0}")]
    CommandFailed(#[from] io::Error),
    /// Invalid borg output found
    #[error("Error while deserializing borg output: {0}")]
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    #[error("Error while deserializing borg output: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Borg was terminated by a signal
    #[error("Borg was terminated by a signal")]
    TerminatedBySignal,
    /// Piping from stdout or stderr failed
    #[error("Piping from stdout or stderr failed")]
    PipeFailed,
    /// The specified archive name already exists
    #[error("The specified archive name already exists")]
    ArchiveAlreadyExists,
    /// An unexpected message id was received
    #[error("An unexpected message id was received: {0}")]
    UnexpectedMessageId(MessageId),
}

/// The errors that can be returned from [crate::sync::mount]
#[derive(Debug, Error)]
pub enum MountError {
    /// An unknown error occurred
    #[error("Unknown error occurred: {0}")]
    Unknown(String),
    /// Error while splitting the arguments
    #[error("error while splitting the arguments")]
    ShlexError,
    /// The command failed to execute
    #[error("The command failed to execute: {0}")]
    CommandFailed(#[from] io::Error),
    /// Invalid borg output found
    #[error("Error while deserializing borg output: {0}")]
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    #[error("Error while deserializing borg output: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Borg was terminated by a signal
    #[error("Borg was terminated by a signal")]
    TerminatedBySignal,
    /// An unexpected message id was received
    #[error("Failed to umount: {0}")]
    UMountError(String),
    /// An unexpected message id was received
    #[error("An unexpected message id was received: {0}")]
    UnexpectedMessageId(MessageId),
}

/// The errors that can be returned from [crate::sync::list]
#[derive(Error, Debug)]
pub enum ListError {
    /// An unknown error occurred
    #[error("Unknown error occurred: {0}")]
    Unknown(String),
    /// Error while splitting the arguments
    #[error("error while splitting the arguments")]
    ShlexError,
    /// The command failed to execute
    #[error("The command failed to execute: {0}")]
    CommandFailed(#[from] io::Error),
    /// Invalid borg output found
    #[error("Could not read borg output: {0}")]
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    #[error("Error while deserializing borg output: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Borg was terminated by a signal
    #[error("Borg was terminated by a signal")]
    TerminatedBySignal,
    /// Piping from stdout or stderr failed
    #[error("Piping from stdout or stderr failed")]
    PipeFailed,
    /// An unexpected message id was received
    #[error("An unexpected message id was received: {0}")]
    UnexpectedMessageId(MessageId),
    /// The specified repository does not exist
    #[error("The repository does not exist")]
    RepositoryDoesNotExist,
    /// The provided passphrase was incorrect
    #[error("The provided passphrase was incorrect")]
    PassphraseWrong,
}

/// The possible errors that can get returned from [crate::sync::init]
#[derive(Error, Debug)]
pub enum InitError {
    /// Error while splitting the arguments
    #[error("error while splitting the arguments")]
    ShlexError,
    /// The command failed to execute
    #[error("The command failed to execute: {0}")]
    CommandFailed(#[from] io::Error),
    /// Invalid borg output found
    #[error("Could not read borg output: {0}")]
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    #[error("Error while deserializing borg output: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Borg was terminated by a signal
    #[error("Borg was terminated by a signal")]
    TerminatedBySignal,
    /// The repository already exists
    #[error("Repository already exists")]
    RepositoryAlreadyExists,
    /// An unexpected message id was received
    #[error("Unexpected msg_id received: {0}")]
    UnexpectedMessageId(MessageId),
    /// Unknown error occurred
    #[error("Unknown error occurred: {0}")]
    Unknown(String),
}

/// The errors of a borg create command
#[derive(Debug, Error)]
pub enum CreateError {
    /// Piping from stdout or stderr failed
    #[error("Piping failed")]
    PipeFailed,
    /// The specified archive name already exists
    #[error("Archive already exists")]
    ArchiveAlreadyExists,
    /// The provided passphrase was incorrect
    #[error("Invalid passphrase")]
    PassphraseWrong,
    /// Error while splitting the arguments
    #[error("error while splitting the arguments")]
    ShlexError,
    /// The command failed to execute
    #[error("The command failed to execute: {0}")]
    CommandFailed(#[from] io::Error),
    /// Invalid borg output found
    #[error("Could not read borg output: {0}")]
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    #[error("Error while deserializing borg output: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Borg was terminated by a signal
    #[error("Borg was terminated by a signal")]
    TerminatedBySignal,
    /// An unexpected message id was received
    #[error("Unexpected msg_id received: {0}")]
    UnexpectedMessageId(MessageId),
    /// Unknown error occurred
    #[error("Unknown error occurred: {0}")]
    Unknown(String),
}
