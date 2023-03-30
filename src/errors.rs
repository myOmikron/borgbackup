//! All errors of this crate are defined in this module

use std::fmt::{Display, Formatter};
use std::io;

use crate::output::logging::MessageId;

/// The errors that can be returned from [crate::sync::compact]
#[derive(Debug)]
pub enum CompactError {
    /// An unknown error occurred
    Unknown,
    /// Error while splitting the arguments
    ShlexError,
    /// The command failed to execute
    CommandFailed(io::Error),
    /// Invalid borg output found
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    DeserializeError(serde_json::Error),
    /// Borg was terminated by a signal
    TerminatedBySignal,
    /// Piping from stdout or stderr failed
    PipeFailed,
    /// An unexpected message id was received
    UnexpectedMessageId(MessageId),
}

impl Display for CompactError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CompactError::Unknown => write!(f, "Unknown error occurred"),
            CompactError::ShlexError => write!(f, "error while splitting the arguments"),
            CompactError::CommandFailed(err) => write!(f, "The command failed to execute: {err}"),
            CompactError::InvalidBorgOutput(err) => write!(f, "Could not read borg output: {err}"),
            CompactError::DeserializeError(err) => {
                write!(f, "Error while deserializing borg output: {err}")
            }
            CompactError::TerminatedBySignal => write!(f, "Borg was terminated by a signal"),
            CompactError::PipeFailed => write!(f, "Piping from stdout or stderr failed"),
            CompactError::UnexpectedMessageId(x) => {
                write!(f, "An unexpected message id was received: {x}")
            }
        }
    }
}

impl From<io::Error> for CompactError {
    fn from(value: io::Error) -> Self {
        Self::CommandFailed(value)
    }
}

impl From<serde_json::Error> for CompactError {
    fn from(value: serde_json::Error) -> Self {
        Self::DeserializeError(value)
    }
}

/// The errors that can be returned from [crate::sync::prune]
#[derive(Debug)]
pub enum PruneError {
    /// An unknown error occurred
    Unknown,
    /// Error while splitting the arguments
    ShlexError,
    /// The command failed to execute
    CommandFailed(io::Error),
    /// Invalid borg output found
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    DeserializeError(serde_json::Error),
    /// Borg was terminated by a signal
    TerminatedBySignal,
    /// Piping from stdout or stderr failed
    PipeFailed,
    /// The specified archive name already exists
    ArchiveAlreadyExists,
    /// An unexpected message id was received
    UnexpectedMessageId(MessageId),
}

impl Display for PruneError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PruneError::Unknown => write!(f, "Unknown error occurred"),
            PruneError::ShlexError => write!(f, "error while splitting the arguments"),
            PruneError::CommandFailed(err) => write!(f, "The command failed to execute: {err}"),
            PruneError::InvalidBorgOutput(err) => write!(f, "Could not read borg output: {err}"),
            PruneError::DeserializeError(err) => {
                write!(f, "Error while deserializing borg output: {err}")
            }
            PruneError::TerminatedBySignal => write!(f, "Borg was terminated by a signal"),
            PruneError::PipeFailed => write!(f, "Piping from stdout or stderr failed"),
            PruneError::ArchiveAlreadyExists => {
                write!(f, "The specified archive name already exists")
            }
            PruneError::UnexpectedMessageId(x) => {
                write!(f, "An unexpected message id was received: {x}")
            }
        }
    }
}

impl From<io::Error> for PruneError {
    fn from(value: io::Error) -> Self {
        Self::CommandFailed(value)
    }
}

impl From<serde_json::Error> for PruneError {
    fn from(value: serde_json::Error) -> Self {
        Self::DeserializeError(value)
    }
}

/// The errors that can be returned from [crate::sync::list]
#[derive(Debug)]
pub enum ListError {
    /// An unknown error occurred
    Unknown,
    /// Error while splitting the arguments
    ShlexError,
    /// The command failed to execute
    CommandFailed(io::Error),
    /// Invalid borg output found
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    DeserializeError(serde_json::Error),
    /// Borg was terminated by a signal
    TerminatedBySignal,
    /// Piping from stdout or stderr failed
    PipeFailed,
    /// An unexpected message id was received
    UnexpectedMessageId(MessageId),
    /// The specified repository does not exist
    RepositoryDoesNotExist,
}

impl Display for ListError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ListError::Unknown => write!(f, "Unknown error occurred"),
            ListError::ShlexError => write!(f, "error while splitting the arguments"),
            ListError::CommandFailed(err) => write!(f, "The command failed to execute: {err}"),
            ListError::InvalidBorgOutput(err) => write!(f, "Could not read borg output: {err}"),
            ListError::DeserializeError(err) => {
                write!(f, "Error while deserializing borg output: {err}")
            }
            ListError::TerminatedBySignal => write!(f, "Borg was terminated by a signal"),
            ListError::PipeFailed => write!(f, "Piping from stdout or stderr failed"),
            ListError::UnexpectedMessageId(x) => {
                write!(f, "An unexpected message id was received: {x}")
            }
            ListError::RepositoryDoesNotExist => write!(f, "The repository does not exist"),
        }
    }
}

impl From<io::Error> for ListError {
    fn from(value: io::Error) -> Self {
        Self::CommandFailed(value)
    }
}

impl From<serde_json::Error> for ListError {
    fn from(value: serde_json::Error) -> Self {
        Self::DeserializeError(value)
    }
}

/// The possible errors that can get returned from [crate::sync::init]
#[derive(Debug)]
pub enum InitError {
    /// Error while splitting the arguments
    ShlexError,
    /// The command failed to execute
    CommandFailed(io::Error),
    /// Invalid borg output found
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    DeserializeError(serde_json::Error),
    /// Borg was terminated by a signal
    TerminatedBySignal,
    /// The repository already exists
    RepositoryAlreadyExists,
    /// An unexpected message id was received
    UnexpectedMessageId(MessageId),
    /// Unknown error occurred
    Unknown,
}

impl Display for InitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::ShlexError => write!(f, "error while splitting the arguments"),
            InitError::CommandFailed(err) => write!(f, "The command failed to execute: {err}"),
            InitError::InvalidBorgOutput(err) => write!(f, "Could not read borg output: {err}"),
            InitError::DeserializeError(err) => {
                write!(f, "Error while deserializing borg output: {err}")
            }
            InitError::TerminatedBySignal => write!(f, "Borg was terminated by a signal"),
            InitError::Unknown => write!(f, "Unknown error occurred"),
            InitError::RepositoryAlreadyExists => write!(f, "Repository already exists"),
            InitError::UnexpectedMessageId(err) => write!(f, "Unexpected msg_id received: {err}"),
        }
    }
}

impl From<io::Error> for InitError {
    fn from(value: io::Error) -> Self {
        Self::CommandFailed(value)
    }
}

impl From<serde_json::Error> for InitError {
    fn from(value: serde_json::Error) -> Self {
        Self::DeserializeError(value)
    }
}

/// The errors of a borg create command
#[derive(Debug)]
pub enum CreateError {
    /// An unknown error occurred
    Unknown,
    /// Error while splitting the arguments
    ShlexError,
    /// The command failed to execute
    CommandFailed(io::Error),
    /// Invalid borg output found
    InvalidBorgOutput(io::Error),
    /// Error while deserializing output of borg
    DeserializeError(serde_json::Error),
    /// Borg was terminated by a signal
    TerminatedBySignal,
    /// Piping from stdout or stderr failed
    PipeFailed,
    /// The specified archive name already exists
    ArchiveAlreadyExists,
    /// An unexpected message id was received
    UnexpectedMessageId(MessageId),
}

impl Display for CreateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateError::Unknown => write!(f, "Unknown error occurred"),
            CreateError::ShlexError => write!(f, "error while splitting the arguments"),
            CreateError::CommandFailed(err) => write!(f, "The command failed to execute: {err}"),
            CreateError::InvalidBorgOutput(err) => write!(f, "Could not read borg output: {err}"),
            CreateError::DeserializeError(err) => {
                write!(f, "Error while deserializing borg output: {err}")
            }
            CreateError::TerminatedBySignal => write!(f, "Borg was terminated by a signal"),
            CreateError::PipeFailed => write!(f, "Piping from stdout or stderr failed"),
            CreateError::ArchiveAlreadyExists => {
                write!(f, "The specified archive name already exists")
            }
            CreateError::UnexpectedMessageId(x) => {
                write!(f, "An unexpected message id was received: {x}")
            }
        }
    }
}

impl From<io::Error> for CreateError {
    fn from(value: io::Error) -> Self {
        Self::CommandFailed(value)
    }
}

impl From<serde_json::Error> for CreateError {
    fn from(value: serde_json::Error) -> Self {
        Self::DeserializeError(value)
    }
}
