use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::process::{Command, Output};
use std::{env, io};

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};

use crate::commands::common::CommonOptions;
use crate::output::list::ListRepository;
use crate::output::logging::{LevelName, LoggingMessage, MessageId};

/// The options for the [list] command
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListOptions {
    /// Path to the repository
    ///
    /// Example values:
    /// - `/tmp/foo`
    /// - `user@example.com:/opt/repo`
    /// - `ssh://user@example.com:2323:/opt/repo`
    pub repository: String,
    /// The passphrase for the repository
    ///
    /// If using a repository with [crate::commands::common::EncryptionMode::None],
    /// you can leave this option empty
    pub passphrase: Option<String>,
}

fn fmt_args(options: &ListOptions, common_options: &CommonOptions) -> String {
    format!(
        "--log-json {common_options} list --json {repository}",
        repository = shlex::quote(&options.repository)
    )
}

fn parse_output(res: Output) -> Result<ListRepository, ListError> {
    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(ListError::TerminatedBySignal);
    };

    for line in BufRead::lines(res.stderr.as_slice()) {
        let line = line.map_err(ListError::InvalidBorgOutput)?;

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
                    return Err(ListError::UnexpectedMessageId(msg_id));
                }
            }
        }
    }

    if exit_code != 0 {
        return Err(ListError::Unknown);
    }

    trace!("Parsing output");
    let list_repo: ListRepository = serde_json::from_slice(&res.stdout)?;

    Ok(list_repo)
}

/// The entry point for the borg list command
///
/// **Parameter**:
/// - `options`: Reference to [ListOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub fn list(
    options: &ListOptions,
    common_options: &CommonOptions,
) -> Result<ListRepository, ListError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(ListError::ShlexError)?;
    let res = Command::new(local_path).args(args).output()?;

    if options.passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    let list_output = parse_output(res)?;

    info!("Finished listing repository");

    Ok(list_output)
}

/// The entry point for the borg list command
///
/// **Parameter**:
/// - `options`: Reference to [ListOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
#[cfg(feature = "tokio")]
pub async fn list_async(
    options: &ListOptions,
    common_options: &CommonOptions,
) -> Result<ListRepository, ListError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(ListError::ShlexError)?;
    let res = tokio::process::Command::new(local_path)
        .args(args)
        .output()
        .await?;

    if options.passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    let list_repo = parse_output(res)?;

    info!("Finished pruning");

    Ok(list_repo)
}

/// The errors that can be returned from [list]
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
