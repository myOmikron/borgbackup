use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::process::Command;
#[cfg(feature = "tokio")]
use std::process::Stdio;
use std::{env, io};

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
#[cfg(feature = "tokio")]
use tokio::io::AsyncReadExt;
#[cfg(feature = "tokio")]
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::commands::common::{CommonOptions, CompressionMode};
use crate::output::create::Create;
use crate::output::logging::{LevelName, LoggingMessage, MessageId};

/// The options for a borg create command
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateOptions {
    /// Path to the repository
    ///
    /// Example values:
    /// - `/tmp/foo`
    /// - `user@example.com:/opt/repo`
    /// - `ssh://user@example.com:2323:/opt/repo`
    pub repository: String,
    /// Name of archive to create (must be also a valid directory name)
    ///
    /// The archive name needs to be unique.
    /// It must not end in ‘.checkpoint’ or ‘.checkpoint.N’ (with N being a number),
    /// because these names are used for checkpoints and treated in special ways.
    ///
    /// In the archive name, you may use the following placeholders:
    /// {now}, {utcnow}, {fqdn}, {hostname}, {user} and some others.
    ///
    /// See https://borgbackup.readthedocs.io/en/stable/usage/help.html#borg-patterns for further
    /// information.
    pub archive: String,
    /// The passphrase for the repository
    ///
    /// If using a repository with [crate::commands::common::EncryptionMode::None],
    /// you can leave this option empty
    pub passphrase: Option<String>,
    /// Add a comment text to the archive
    pub comment: Option<String>,
    /// Specify the compression mode that should be used.
    ///
    /// Defaults to [CompressionMode::Lz4].
    pub compression: Option<CompressionMode>,
    /// The paths to archive.
    ///
    /// All given paths will be recursively traversed.
    pub paths: Vec<String>,
}

fn fmt_args(options: &CreateOptions, common_options: &CommonOptions, progress: bool) -> String {
    format!(
        "--log-json{p} {common_options}create --json{comment}{compression} {repo}::{archive} {paths}",
        p = if progress { " --progress" } else { "" },
        comment = options.comment.as_ref().map_or("".to_string(), |x| format!(
            " --comment {}",
            shlex::quote(x)
        )),
        compression = options.compression.as_ref().map_or("".to_string(), |x| format!(" --compression {x}")),
        repo = shlex::quote(&options.repository),
        archive = shlex::quote(&options.archive),
        paths = options.paths.join(" "),
    )
}

/// This command creates a backup archive containing all files found
/// while recursively traversing all paths specified.
/// Paths are added to the archive as they are given,
/// that means if relative paths are desired, the command has to be run from the correct directory.
///
/// **Parameter**:
/// - `options`: Reference to [CreateOptions]
/// - `common_options`: Reference to [CommonOptions]
pub fn create(
    options: &CreateOptions,
    common_options: &CommonOptions,
) -> Result<Create, CreateError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = fmt_args(options, common_options, false);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(CreateError::ShlexError)?;
    let res = Command::new(local_path).args(args).output()?;

    if options.passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(CreateError::TerminatedBySignal);
    };

    for line in BufRead::lines(res.stderr.as_slice()) {
        let line = line.map_err(CreateError::InvalidBorgOutput)?;

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
                match msg_id {
                    MessageId::ArchiveAlreadyExists => {
                        return Err(CreateError::ArchiveAlreadyExists)
                    }
                    _ => {
                        if exit_code > 1 {
                            return Err(CreateError::UnexpectedMessageId(msg_id));
                        }
                    }
                }
            }
        }
    }

    if exit_code != 0 {
        return Err(CreateError::Unknown);
    }

    trace!("Parsing stats");
    let stats: Create = serde_json::from_slice(&res.stdout)?;

    info!("Finished creating archive");

    Ok(stats)
}

/// The progress of a borg create command.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg(feature = "tokio")]
pub enum CreateProgress {
    /// The current progress of the archive
    Progress {
        /// The size of the original files
        original_size: u64,
        /// The size of the compressed files
        compressed_size: u64,
        /// The size of the deduplicated files
        deduplicated_size: u64,
        /// The current number of files
        nfiles: u64,
        /// The path to the current file
        path: String,
    },
    /// Finished the creation of the archive
    Finished,
}

#[cfg(feature = "tokio")]
impl Display for CreateProgress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateProgress::Progress {
                original_size,
                compressed_size,
                deduplicated_size,
                nfiles,
                path,
            } => {
                write!(
                    f,
                    "O {original_size} C {compressed_size} D {deduplicated_size} N {nfiles} {path}",
                )
            }
            CreateProgress::Finished => write!(f, "Finished"),
        }
    }
}

/// This command creates a backup archive containing all files found
/// while recursively traversing all paths specified.
/// Paths are added to the archive as they are given,
/// that means if relative paths are desired, the command has to be run from the correct directory.
///
/// The progress will be sent back through the provided channel.
///
/// **Parameter**:
/// - `options`: Reference to [CreateOptions]
/// - `common_options`: Reference to [CommonOptions]
/// - `progress_channel`: A [tokio::sync::mpsc::Sender] of [CreateProgress]. On every progress
/// update, a message will be sent to this channel
#[cfg(feature = "tokio")]
pub async fn create_async_progress(
    options: &CreateOptions,
    common_options: &CommonOptions,
    progress_channel: tokio::sync::mpsc::Sender<CreateProgress>,
) -> Result<Create, CreateError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = fmt_args(options, common_options, true);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(CreateError::ShlexError)?;
    let mut child = tokio::process::Command::new(local_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let mut stdout = child.stdout.take().ok_or(CreateError::PipeFailed)?;
    let stderr = child.stderr.take().ok_or(CreateError::PipeFailed)?;

    let mut stderr_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            result = stderr_reader.next_line() => match result {
                Ok(Some(line)) => {
                    let res: LoggingMessage = serde_json::from_str(&line)?;

                    if let LoggingMessage::ArchiveProgress {
                        original_size,
                        compressed_size,
                        deduplicated_size,
                        nfiles,
                        path,
                        finished,
                        ..
                    } = res {
                        if finished {
                            trace!("Progress: finished");
                            if let Err(err) = progress_channel.send(CreateProgress::Finished).await {
                                error!("Could not send to progress channel: {err}");
                            }

                            continue;
                        }

                        if let Err(err) = progress_channel.send(CreateProgress::Progress {
                            original_size: original_size.unwrap(),
                            compressed_size: compressed_size.unwrap(),
                            deduplicated_size: deduplicated_size.unwrap(),
                            nfiles: nfiles.unwrap(),
                            path: path.unwrap(),
                        }).await {
                            error!("Could not send to progress channel: {err}");
                        }
                    } else if let LoggingMessage::LogMessage {
                        name,
                        message,
                        level_name,
                        time,
                        msg_id,
                    } = res {
                        match level_name {
                            LevelName::Debug => debug!("{time} {name}: {message}"),
                            LevelName::Info => info!("{time} {name}: {message}"),
                            LevelName::Warning => warn!("{time} {name}: {message}"),
                            LevelName::Error => error!("{time} {name}: {message}"),
                            LevelName::Critical => error!("{time} {name}: {message}"),
                        }

                        if let Some(MessageId::RepositoryAlreadyExists) = msg_id {
                            return Err(CreateError::ArchiveAlreadyExists);
                        }

                    }
                },
                Err(_) => break,
                _ => (),
            },
            result = child.wait() => {
                if let Ok(exit_code) = result {
                    debug!("Child process exited with {exit_code}");
                    if exit_code.code().unwrap() > 1 {
                        return Err(CreateError::Unknown);
                    }
                }
                break // child process exited
            }
        }
    }

    if options.passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    let mut stdout_str = String::new();
    stdout
        .read_to_string(&mut stdout_str)
        .await
        .map_err(CreateError::InvalidBorgOutput)?;

    trace!("Parsing stats: {stdout_str}");
    let stats: Create = serde_json::from_str(&stdout_str)?;

    info!("Finished creating archive");

    Ok(stats)
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
