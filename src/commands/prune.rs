use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::num::NonZeroU16;
use std::process::{Command, Output};
use std::{env, io};

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};

use crate::commands::common::CommonOptions;
use crate::output::logging::{LevelName, LoggingMessage, MessageId};

/// The quantifier for [PruneWithin]
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum PruneWithinTime {
    /// Hour quantifier
    Hour,
    /// Day quantifier
    Day,
    /// Week quantifier (7 days)
    Week,
    /// Month quantifier (31 days)
    Month,
    /// Year quantifier
    Year,
}

impl Display for PruneWithinTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PruneWithinTime::Hour => write!(f, "H"),
            PruneWithinTime::Day => write!(f, "d"),
            PruneWithinTime::Week => write!(f, "w"),
            PruneWithinTime::Month => write!(f, "m"),
            PruneWithinTime::Year => write!(f, "y"),
        }
    }
}

/// The definition to specify an interval in which backups
/// should not be pruned.
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct PruneWithin {
    /// quantifier
    pub quantifier: NonZeroU16,
    /// The time quantifier
    pub time: PruneWithinTime,
}

impl Display for PruneWithin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.quantifier, self.time)
    }
}

/// Options for [prune]
///
/// A good procedure is to thin out more and more the older your backups get.
/// As an example, `keep_daily` 7 means to keep the latest backup on each day, up to 7 most
/// recent days with backups (days without backups do not count). The rules are applied from
/// secondly to yearly, and backups selected by previous rules do not count towards those of
/// later rules. The time that each backup starts is used for pruning purposes.
/// Dates and times are interpreted in the local timezone, and weeks go from Monday to Sunday.
/// Specifying a negative number of archives to keep means that there is no limit.
///
/// As of borg 1.2.0, borg will retain the oldest archive if any of the secondly, minutely,
/// hourly, daily, weekly, monthly, or yearly rules was not otherwise able to meet
/// its retention target. This enables the first chronological archive to continue aging until
/// it is replaced by a newer archive that meets the retention criteria.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PruneOptions {
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
    /// The archives kept with this option do not count towards the totals specified
    /// by any other options.
    pub keep_within: Option<PruneWithin>,
    /// number of secondly archives to keep
    pub keep_secondly: Option<NonZeroU16>,
    /// number of minutely archives to keep
    pub keep_minutely: Option<NonZeroU16>,
    /// number of hourly archives to keep
    pub keep_hourly: Option<NonZeroU16>,
    /// number of daily archives to keep
    pub keep_daily: Option<NonZeroU16>,
    /// number of weekly archives to keep
    pub keep_weekly: Option<NonZeroU16>,
    /// number of monthly archives to keep
    pub keep_monthly: Option<NonZeroU16>,
    /// number of yearly archives to keep
    pub keep_yearly: Option<NonZeroU16>,
    /// write checkpoint every SECONDS seconds (Default: 1800)
    pub checkpoint_interval: Option<NonZeroU16>,
    /// only consider archive names matching the glob.
    ///
    /// The pattern can use [crate::commands::common::Pattern::Shell]
    pub glob_archives: Option<String>,
}

impl PruneOptions {
    /// Create an new [PruneOptions]
    pub fn new(repository: String) -> Self {
        Self {
            repository,
            passphrase: None,
            keep_within: None,
            keep_secondly: None,
            keep_minutely: None,
            keep_hourly: None,
            keep_daily: None,
            keep_weekly: None,
            keep_monthly: None,
            keep_yearly: None,
            checkpoint_interval: None,
            glob_archives: None,
        }
    }
}

fn fmt_args(options: &PruneOptions, common_options: &CommonOptions) -> String {
    format!(
        "--log-json {common_options} prune{keep_within}{keep_secondly}{keep_minutely}{keep_hourly}{keep_daily}{keep_weekly}{keep_monthly}{keep_yearly} {repository}",
        keep_within = options.keep_within.as_ref().map_or("".to_string(), |x| format!(" --keep-within {x}")),
        keep_secondly = options.keep_secondly.as_ref().map_or("".to_string(), |x| format!(" --keep-secondly {x}")),
        keep_minutely = options.keep_minutely.map_or("".to_string(), |x| format!(" --keep-minutely {x}")),
        keep_hourly = options.keep_minutely.map_or("".to_string(), |x| format!(" --keep-hourly {x}")),
        keep_daily = options.keep_minutely.map_or("".to_string(), |x| format!(" --keep-daily {x}")),
        keep_weekly = options.keep_minutely.map_or("".to_string(), |x| format!(" --keep-weekly {x}")),
        keep_monthly = options.keep_minutely.map_or("".to_string(), |x| format!(" --keep-monthly {x}")),
        keep_yearly = options.keep_minutely.map_or("".to_string(), |x| format!(" --keep-yearly {x}")),
        repository = shlex::quote(&options.repository)
    )
}

fn parse_output(res: Output) -> Result<(), PruneError> {
    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(PruneError::TerminatedBySignal);
    };

    for line in BufRead::lines(res.stderr.as_slice()) {
        let line = line.map_err(PruneError::InvalidBorgOutput)?;

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
                    return Err(PruneError::UnexpectedMessageId(msg_id));
                }
            }
        }
    }

    if exit_code != 0 {
        return Err(PruneError::Unknown);
    }

    Ok(())
}

/// The entry point for the borg init command
///
/// **Parameter**:
/// - `options`: Reference to [PruneOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub fn prune(options: &PruneOptions, common_options: &CommonOptions) -> Result<(), PruneError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(PruneError::ShlexError)?;
    let res = Command::new(local_path).args(args).output()?;

    if options.passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    parse_output(res)?;

    info!("Finished pruning");

    Ok(())
}

/// The entry point for the borg init command
///
/// **Parameter**:
/// - `options`: Reference to [PruneOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
#[cfg(feature = "tokio")]
pub async fn prune_async(
    options: &PruneOptions,
    common_options: &CommonOptions,
) -> Result<(), PruneError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(PruneError::ShlexError)?;
    let res = tokio::process::Command::new(local_path)
        .args(args)
        .output()
        .await?;

    if options.passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    parse_output(res)?;

    info!("Finished pruning");

    Ok(())
}

/// The errors that can be returned from [prune]
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
