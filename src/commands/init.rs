use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::process::{Command, Output};
use std::{env, io};

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};

use crate::commands::common::{CommonOptions, EncryptionMode};
use crate::output::logging::{LevelName, LoggingMessage, MessageId};

/// The options to provide to the init command
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitOptions {
    /// Path to the repository
    ///
    /// Example values:
    /// - `/tmp/foo`
    /// - `user@example.com:/opt/repo`
    /// - `ssh://user@example.com:2323:/opt/repo`
    pub repository: String,
    /// The mode to use for encryption
    pub encryption_mode: EncryptionMode,
    /// Set the repository to append_only mode.
    /// Note that this only affects the low level structure of the repository,
    /// and running delete or prune will still be allowed.
    ///
    /// See [Append-only mode (forbid compaction)](https://borgbackup.readthedocs.io/en/stable/usage/notes.html#append-only-mode)
    /// in Additional Notes for more details.
    pub append_only: bool,
    /// Create the parent directories of the repo, if they are missing.
    ///
    /// Defaults to false
    pub make_parent_dirs: bool,
    /// Set storage quota of the new repository (e.g. 5G, 1.5T)
    ///
    /// No quota is set by default
    pub storage_quota: Option<String>,
}

impl InitOptions {
    /// Create new [InitOptions].
    ///
    /// `append_only`, `make_parent_dirs` and `storage_quota` are set to their defaults.
    pub fn new(repository: String, encryption_mode: EncryptionMode) -> Self {
        Self {
            repository,
            encryption_mode,
            append_only: false,
            make_parent_dirs: false,
            storage_quota: None,
        }
    }
}

fn fmt_args(options: &InitOptions, common_options: &CommonOptions) -> String {
    format!(
        "--log-json {common_options}init -e {e}{append_only}{make_parent_dirs}{storage_quota} {repository}",
        e = options.encryption_mode,
        append_only = if options.append_only {
            " --append-only"
        } else {
            ""
        },
        make_parent_dirs = if options.make_parent_dirs {
            " --make-parent-dirs"
        } else {
            ""
        },
        storage_quota = options
            .storage_quota
            .as_ref()
            .map_or("".to_string(), |x| format!(
                " --storage-quota {quota}",
                quota = shlex::quote(x)
            )),
        repository = shlex::quote(&options.repository),
    )
}

fn parse_result(res: Output) -> Result<(), InitError> {
    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(InitError::TerminatedBySignal);
    };

    for line in res.stderr.lines() {
        let line = line.map_err(InitError::InvalidBorgOutput)?;

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
                    MessageId::RepositoryAlreadyExists => {
                        return Err(InitError::RepositoryAlreadyExists)
                    }
                    _ => {
                        if exit_code > 1 {
                            return Err(InitError::UnexpectedMessageId(msg_id));
                        }
                    }
                }
            }

            if let Some(MessageId::RepositoryAlreadyExists) = msg_id {
                return Err(InitError::RepositoryAlreadyExists);
            } else if let Some(msg_id) = msg_id {
                return Err(InitError::UnexpectedMessageId(msg_id));
            }
        }
    }

    if exit_code != 0 {
        return Err(InitError::Unknown);
    }

    Ok(())
}

/// The entry point for the borg init command
///
/// **Parameter**:
/// - `options`: Reference to [InitOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
#[cfg(feature = "tokio")]
pub fn init(options: &InitOptions, common_options: &CommonOptions) -> Result<(), InitError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = fmt_args(options, common_options);
    let passphrase = options.encryption_mode.get_passphrase();

    if let Some(passphrase) = passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    debug!("Calling borg: {local_path} {args}");

    let args = shlex::split(&args).ok_or(InitError::ShlexError)?;
    let res = Command::new(local_path).args(args).output()?;

    if passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    parse_result(res)?;

    info!("Repository {} created", options.repository);

    Ok(())
}

/// The entry point for the borg init command
///
/// **Parameter**:
/// - `options`: Reference to [InitOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
#[cfg(feature = "tokio")]
pub async fn init_async(
    options: &InitOptions,
    common_options: &CommonOptions,
) -> Result<(), InitError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = fmt_args(options, common_options);
    let passphrase = options.encryption_mode.get_passphrase();

    if let Some(passphrase) = passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(InitError::ShlexError)?;
    let res = tokio::process::Command::new(local_path)
        .args(args)
        .output()
        .await?;

    if passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    parse_result(res)?;

    info!("Repository {} created", options.repository);

    Ok(())
}

/// The possible errors that can get returned from [init]
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

#[cfg(test)]
mod tests {
    use crate::commands::common::{CommonOptions, EncryptionMode};
    use crate::commands::{init, InitOptions};

    #[test]
    fn create_repo() {
        let dir = tempfile::tempdir().unwrap();

        let pw = String::from("pw");

        let init_options = [
            InitOptions::new(
                format!("{}", dir.path().join("none").display()),
                EncryptionMode::None,
            ),
            InitOptions::new(
                format!("{}", dir.path().join("auth").display()),
                EncryptionMode::Authenticated(pw.clone()),
            ),
            InitOptions::new(
                format!("{}", dir.path().join("authb2").display()),
                EncryptionMode::AuthenticatedBlake2(pw.clone()),
            ),
            InitOptions::new(
                format!("{}", dir.path().join("repo").display()),
                EncryptionMode::Repokey(pw.clone()),
            ),
            InitOptions::new(
                format!("{}", dir.path().join("key").display()),
                EncryptionMode::Keyfile(pw.clone()),
            ),
            InitOptions::new(
                format!("{}", dir.path().join("repob2").display()),
                EncryptionMode::RepokeyBlake2(pw.clone()),
            ),
            InitOptions::new(
                format!("{}", dir.path().join("keyb2").display()),
                EncryptionMode::KeyfileBlake2(pw),
            ),
        ];

        for init_options in &init_options {
            assert!(init(init_options, &CommonOptions::default()).is_ok());
        }

        dir.close().unwrap()
    }
}
