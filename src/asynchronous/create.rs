use std::env;
use std::fmt::{Display, Formatter, Write};
use std::process::Stdio;

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

use crate::common::{create_fmt_args, create_parse_output, CommonOptions, CreateOptions};
use crate::errors::CreateError;
use crate::output::create::Create;
use crate::output::logging::{LevelName, LoggingMessage, MessageId};

/// This command creates a backup archive containing all files found
/// while recursively traversing all paths specified.
/// Paths are added to the archive as they are given,
/// that means if relative paths are desired, the command has to be run from the correct directory.
///
/// **Parameter**:
/// - `options`: Reference to [CreateOptions]
/// - `common_options`: Reference to [CommonOptions]
pub async fn create(
    options: &CreateOptions,
    common_options: &CommonOptions,
) -> Result<Create, CreateError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = create_fmt_args(options, common_options, false);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(CreateError::ShlexError)?;
    let res = tokio::process::Command::new(local_path)
        .args(args)
        .output()
        .await?;

    if options.passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    let stats = create_parse_output(res)?;

    info!("Finished creating archive");

    Ok(stats)
}

/// The progress of a borg create command.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
pub async fn create_progress(
    options: &CreateOptions,
    common_options: &CommonOptions,
    progress_channel: tokio::sync::mpsc::Sender<CreateProgress>,
) -> Result<Create, CreateError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = create_fmt_args(options, common_options, true);
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

    let mut output = String::new();

    loop {
        tokio::select! {
            result = stderr_reader.next_line() => match result {
                Ok(Some(line)) => {
                    writeln!(output, "{line}").unwrap();
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
                        return Err(CreateError::Unknown(output));
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
