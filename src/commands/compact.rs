use std::fmt::{Display, Formatter};
use std::io;
use std::io::BufRead;
use std::process::{Command, Output};

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};

use crate::commands::common::CommonOptions;
use crate::output::logging::{LevelName, LoggingMessage, MessageId};

/// Options for [compact]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompactOptions {
    /// Path to the repository
    ///
    /// Example values:
    /// - `/tmp/foo`
    /// - `user@example.com:/opt/repo`
    /// - `ssh://user@example.com:2323:/opt/repo`
    pub repository: String,
}

fn fmt_args(options: &CompactOptions, common_options: &CommonOptions) -> String {
    format!(
        "--log-json {common_options}compact {repository}",
        repository = shlex::quote(&options.repository)
    )
}

fn parse_output(res: Output) -> Result<(), CompactError> {
    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(CompactError::TerminatedBySignal);
    };

    for line in BufRead::lines(res.stderr.as_slice()) {
        let line = line.map_err(CompactError::InvalidBorgOutput)?;

        trace!("borg output: {line}");

        let log_msg: LoggingMessage = serde_json::from_str(&line)?;

        if let LoggingMessage::LogMessage {
            name,
            message,
            level_name,
            time,
            msg_id,
        } = log_msg
        {
            match level_name {
                LevelName::Debug => debug!("{time} {name}: {message}"),
                LevelName::Info => info!("{time} {name}: {message}"),
                LevelName::Warning => warn!("{time} {name}: {message}"),
                LevelName::Error => error!("{time} {name}: {message}"),
                LevelName::Critical => error!("{time} {name}: {message}"),
            }

            if let Some(msg_id) = msg_id {
                if exit_code > 1 {
                    return Err(CompactError::UnexpectedMessageId(msg_id));
                }
            }
        }
    }

    if exit_code != 0 {
        return Err(CompactError::Unknown);
    }

    Ok(())
}

///This command frees repository space by compacting segments.
///
/// Use this regularly to avoid running out of space - you do not need to use this after each borg
/// command though. It is especially useful after deleting archives,
/// because only compaction will really free repository space.
///
/// compact does not need a key, so it is possible to
/// invoke it from the client or also from the server.
///
/// **Parameter**:
/// - `options`: Reference to [CreateOptions]
/// - `common_options`: Reference to [CommonOptions]
pub fn compact(
    options: &CompactOptions,
    common_options: &CommonOptions,
) -> Result<(), CompactError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(CompactError::ShlexError)?;
    let res = Command::new(local_path).args(args).output()?;

    parse_output(res)?;

    info!("Finished compacting");

    Ok(())
}

///This command frees repository space by compacting segments.
///
/// Use this regularly to avoid running out of space - you do not need to use this after each borg
/// command though. It is especially useful after deleting archives,
/// because only compaction will really free repository space.
///
/// compact does not need a key, so it is possible to
/// invoke it from the client or also from the server.
///
/// **Parameter**:
/// - `options`: Reference to [CreateOptions]
/// - `common_options`: Reference to [CommonOptions]
#[cfg(feature = "tokio")]
pub async fn compact_async(
    options: &CompactOptions,
    common_options: &CommonOptions,
) -> Result<(), CompactError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(CompactError::ShlexError)?;
    let res = tokio::process::Command::new(local_path)
        .args(args)
        .output()
        .await?;

    parse_output(res)?;

    info!("Finished compacting");

    Ok(())
}

/// The errors that can be returned from [compact]
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
