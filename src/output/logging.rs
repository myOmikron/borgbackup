//! The logging definitions

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// All valid logging messages are defined here
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum LoggingMessage {
    /// Any regular log output invokes this type.
    /// Regular log options and filtering applies to these as well.
    #[serde(rename = "log_message")]
    LogMessage {
        /// Unix timestamp
        time: f64,
        /// The loglevel of borg
        #[serde(rename = "levelname")]
        level_name: LevelName,
        /// Name of the emitting entity
        name: String,
        /// Formatted log message
        message: String,
        /// Message ID.
        ///
        /// Specifies more information or error information
        #[serde(rename = "msgid")]
        msg_id: Option<MessageId>,
    },
    /// This is only output by borg create and borg recreate if --list is specified.
    /// The usual rules for the file listing applies, including the --filter option.
    #[serde(rename = "file_status")]
    FileStatus {
        /// Single-character status as for regular list output
        status: String,
        /// Path of the file system object
        path: String,
    },
    /// Absolute progress information with defined end/total and current value.
    #[serde(rename = "progress_percent")]
    ProgressPercent {
        /// unique, opaque integer ID of the operation
        operation: u64,
        /// Message ID of the operation
        #[serde(rename = "msgid")]
        msg_id: Option<MessageId>,
        /// Unix timestamp
        time: f64,
        /// Indicating whether the operation has finished,
        /// only the last object for an operation can have this property set to true.
        finished: bool,
        /// Current value
        ///
        /// absent for finished == true
        current: Option<u64>,
        /// Total value
        ///
        /// absent for finished == true
        total: Option<u64>,
        /// Array that describes the current item, may be null, contents depend on msg_id
        ///
        /// absent for finished == true
        info: Option<Vec<Value>>,
    },
    /// A message-based progress information with no concrete progress information,
    /// just a message saying what is currently being worked on.
    #[serde(rename = "progress_message")]
    ProgressMessage {
        /// unique, opaque integer ID of the operation
        operation: u64,
        /// Message ID of the operation
        #[serde(rename = "msgid")]
        msg_id: Option<MessageId>,
        /// Indicating whether the operation has finished,
        /// only the last object for an operation can have this property set to true.
        finished: bool,
        /// current progress message (may be empty/absent)
        message: Option<String>,
        /// Unix timestamp
        time: f64,
    },
    /// Output during operations creating archives (borg create and borg recreate)
    #[serde(rename = "archive_progress")]
    ArchiveProgress {
        /// Original size of data processed so far
        /// before compression and deduplication, may be empty/absent
        original_size: Option<u64>,
        /// Compressed size (may be empty/absent)
        compressed_size: Option<u64>,
        /// Deduplicated size (may be empty/absent)
        deduplicated_size: Option<u64>,
        /// Number of (regular) files processed so far (may be empty/absent)
        nfiles: Option<u64>,
        /// Current path (may be empty/absent)
        path: Option<String>,
        /// Unix timestamp
        time: f64,
        /// Indicating whether the operation has finished,
        /// only the last object for an operation can have this property set to true.
        finished: bool,
    },
    /// Due to a bug in borg, umount failures are directly reported from
    /// fusermount instead of being logged as json.
    UMountError(String),
}

impl LoggingMessage {
    /// Given a borg json log, attempt to parse it into a LogMessage
    pub(crate) fn from_str(log_message: &str) -> Result<Self, serde_json::Error> {
        // XXX: There's a bug in borg umount where fusermount errors are not
        //      logged as json so we need to catch them here.
        if log_message.starts_with("fusermount: entry for") {
            Ok(LoggingMessage::UMountError(log_message.to_string()))
        } else {
            serde_json::from_str(log_message)
        }
    }
}
/// The valid loglevel of borg
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LevelName {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
}

/// Message IDs are strings that essentially give a log message or operation a name,
/// without actually using the full text, since texts change more frequently.
///
/// Message IDs are unambiguous and reduce the need to parse log messages.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
pub enum MessageId {
    /// Archive {} already exists
    #[serde(rename = "Archive.AlreadyExists")]
    ArchiveAlreadyExists,
    /// Archive {} does not exist
    #[serde(rename = "Archive.DoesNotExist")]
    ArchiveDoesNotExist,
    /// Failed to encode filename “{}” into file system encoding “{}”.
    /// Consider configuring the LANG environment variable.
    #[serde(rename = "Archive.IncompatibleFilesystemEncodingError")]
    ArchiveIncompatibleFilesystemEncodingError,
    /// Obscure ENOENT error
    BackupFileNotFoundError,
    /// Cache initialization aborted
    #[serde(rename = "Cache.CacheInitAbortedError")]
    CacheCacheInitAbortedError,
    /// Repository encryption method changed since last access, refusing to continue
    #[serde(rename = "Cache.EncryptionMethodMismatch")]
    CacheEncryptionMethodMismatch,
    /// Repository access aborted
    #[serde(rename = "Cache.RepositoryAccessAborted")]
    CacheRepositoryAccessAborted,
    /// Cache is newer than repository - do you have multiple,
    /// independently updated repos with same ID?
    #[serde(rename = "Cache.RepositoryIDNotUnique")]
    CacheRepositoryIDNotUnique,
    /// Cache is newer than repository - this is either an attack
    /// or unsafe (multiple repos with same ID)
    #[serde(rename = "Cache.RepositoryReplay")]
    CacheRepositoryReplay,
    /// Requested buffer size {} is above the limit of {}.
    #[serde(rename = "Buffer.MemoryLimitExceeded")]
    BufferMemoryLimitExceeded,
    /// The Borg binary extension modules do not seem to be properly installed
    ExtensionModuleError,
    /// Data integrity error: {}
    IntegrityError,
    /// Repository has no manifest.
    NoManifestError,
    /// Formatting Error: “{}”.format({}): {}({})
    PlaceholderError,
    /// Invalid key file for repository {} found in {}.
    KeyfileInvalidError,
    /// Mismatch between repository {} and key file {}.
    KeyfileMismatchError,
    /// No key file for repository {} found in {}.
    KeyfileNotFoundError,
    /// passphrase supplied in BORG_PASSPHRASE is incorrect
    PassphraseWrong,
    /// exceeded the maximum password retries
    PasswordRetriesExceeded,
    /// No key entry found in the config of repository {}.
    RepoKeyNotFoundError,
    /// Unsupported manifest envelope. A newer version is required to access this repository.
    UnsupportedManifestError,
    /// Unsupported payload type {}. A newer version is required to access this repository.
    UnsupportedPayloadError,
    /// This file is not a borg key backup, aborting.
    NotABorgKeyFile,
    /// This key backup seems to be for a different backup repository, aborting.
    RepoIdMismatch,
    /// Key-management not available for unencrypted repositories.
    UnencryptedRepo,
    /// Keytype {0} is unknown.
    UnknownKeyType,
    /// Failed to acquire the lock {}.
    LockError,
    /// Failed to acquire the lock {}.
    LockErrorT,
    /// Connection closed by remote host
    ConnectionClosed,
    /// RPC method {} is not valid
    InvalidRPCMethod,
    /// Repository path not allowed
    PathNotAllowed,
    /// No passphrase was provided for an encrypted repository
    NoPassphraseFailure,
    /// Borg server is too old for {}. Required version {}
    #[serde(rename = "RemoteRepository.RPCServerOutdated")]
    RemoteRepositoryRPCServerOutdated,
    /// Borg {}: Got unexpected RPC data format from client.
    UnexpectedRPCDataFormatFromClient,
    /// Got unexpected RPC data format from server: {}
    UnexpectedRPCDataFormatFromServer,
    /// Repository {} already exists.
    #[serde(rename = "Repository.AlreadyExists")]
    RepositoryAlreadyExists,
    /// Inconsistency detected. Please run “borg check {}”.
    #[serde(rename = "Repository.CheckNeeded")]
    RepositoryCheckNeeded,
    /// Repository {} does not exist.
    #[serde(rename = "Repository.DoesNotExist")]
    RepositoryDoesNotExist,
    /// Insufficient free space to complete transaction (required: {}, available: {}).
    #[serde(rename = "Repository.InsufficientFreeSpaceError")]
    RepositoryInsufficientFreeSpaceError,
    /// {} is not a valid repository. Check repo config.
    #[serde(rename = "Repository.InvalidRepository")]
    RepositoryInvalidRepository,
    /// Attic repository detected. Please run “borg upgrade {}”.
    #[serde(rename = "Repository.AtticRepository")]
    RepositoryAtticRepository,
    /// Object with key {} not found in repository {}.
    #[serde(rename = "Repository.ObjectNotFound")]
    RepositoryObjectNotFound,
    /// Begin a cache transaction
    #[serde(rename = "cache.begin_transaction")]
    CacheBeginTransaction,
    /// appears with borg create --no-cache-sync
    #[serde(rename = "cache.download_chunks")]
    CacheDownloadChunks,
    /// TODO
    #[serde(rename = "cache.commit")]
    CacheCommit,
    /// info is one string element, the name of the archive currently synced.
    #[serde(rename = "cache.sync")]
    CacheSync,
    /// TODO
    #[serde(rename = "repository.compact_segments")]
    RepositoryCompactSegments,
    /// TODO
    #[serde(rename = "repository.replay_segments")]
    RepositoryReplaySegments,
    /// TODO
    #[serde(rename = "repository.check")]
    RepositoryCheck,
    /// TODO
    #[serde(rename = "check.verify_data")]
    CheckVerifyData,
    /// TODO
    #[serde(rename = "check.rebuild_manifest")]
    CheckRebuildManifest,
    /// info is one string element, the name of the path currently extracted.
    #[serde(rename = "extract")]
    Extract,
    /// TODO
    #[serde(rename = "extract.permissions")]
    ExtractPermissions,
    /// TODO
    #[serde(rename = "archive.delete")]
    ArchiveDelete,
    /// TODO
    #[serde(rename = "archive.calc_stats")]
    ArchiveCalcStats,
    /// TODO
    #[serde(rename = "prune")]
    Prune,
    /// TODO
    #[serde(rename = "upgrade.convert_segments")]
    UpgradeConvertSegments,
}

impl Display for MessageId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageId::ArchiveAlreadyExists => write!(f, "Archive.AlreadyExists"),
            MessageId::ArchiveDoesNotExist => write!(f, "Archive.DoesNotExist"),
            MessageId::ArchiveIncompatibleFilesystemEncodingError => {
                write!(f, "Archive.IncompatibleFilesystemEncodingError")
            }
            MessageId::BackupFileNotFoundError => write!(f, "BackupFileNotFoundError"),
            MessageId::CacheCacheInitAbortedError => write!(f, "Cache.CacheInitAbortedError"),
            MessageId::CacheEncryptionMethodMismatch => write!(f, "Cache.EncryptionMethodMismatch"),
            MessageId::CacheRepositoryAccessAborted => write!(f, "Cache.RepositoryAccessAborted"),
            MessageId::CacheRepositoryIDNotUnique => write!(f, "Cache.RepositoryIDNotUnique"),
            MessageId::CacheRepositoryReplay => write!(f, "Cache.RepositoryReplay"),
            MessageId::BufferMemoryLimitExceeded => write!(f, "Buffer.MemoryLimitExceeded"),
            MessageId::ExtensionModuleError => write!(f, "ExtensionModuleError"),
            MessageId::IntegrityError => write!(f, "IntegrityError"),
            MessageId::NoManifestError => write!(f, "NoManifestError"),
            MessageId::PlaceholderError => write!(f, "PlaceholderError"),
            MessageId::KeyfileInvalidError => write!(f, "KeyfileInvalidError"),
            MessageId::KeyfileMismatchError => write!(f, "KeyfileMismatchError"),
            MessageId::KeyfileNotFoundError => write!(f, "KeyfileNotFoundError"),
            MessageId::PassphraseWrong => write!(f, "PassphraseWrong"),
            MessageId::PasswordRetriesExceeded => write!(f, "PasswordRetriesExceeded"),
            MessageId::RepoKeyNotFoundError => write!(f, "RepoKeyNotFoundError"),
            MessageId::UnsupportedManifestError => write!(f, "UnsupportedManifestError"),
            MessageId::UnsupportedPayloadError => write!(f, "UnsupportedPayloadError"),
            MessageId::NotABorgKeyFile => write!(f, "NotABorgKeyFile"),
            MessageId::RepoIdMismatch => write!(f, "RepoIdMismatch"),
            MessageId::UnencryptedRepo => write!(f, "UnencryptedRepo"),
            MessageId::UnknownKeyType => write!(f, "UnknownKeyType"),
            MessageId::LockError => write!(f, "LockError"),
            MessageId::LockErrorT => write!(f, "LockErrorT"),
            MessageId::ConnectionClosed => write!(f, "ConnectionClosed"),
            MessageId::InvalidRPCMethod => write!(f, "InvalidRPCMethod"),
            MessageId::PathNotAllowed => write!(f, "PathNotAllowed"),
            MessageId::NoPassphraseFailure => write!(f, "NoPassphraseFailure"),
            MessageId::RemoteRepositoryRPCServerOutdated => {
                write!(f, "RemoteRepository.RPCServerOutdated")
            }
            MessageId::UnexpectedRPCDataFormatFromClient => {
                write!(f, "UnexpectedRPCDataFormatFromClient")
            }
            MessageId::UnexpectedRPCDataFormatFromServer => {
                write!(f, "UnexpectedRPCDataFormatFromServer")
            }
            MessageId::RepositoryAlreadyExists => write!(f, "Repository.AlreadyExists"),
            MessageId::RepositoryCheckNeeded => write!(f, "Repository.CheckNeeded"),
            MessageId::RepositoryDoesNotExist => write!(f, "Repository.DoesNotExist"),
            MessageId::RepositoryInsufficientFreeSpaceError => {
                write!(f, "Repository.InsufficientFreeSpaceError")
            }
            MessageId::RepositoryInvalidRepository => write!(f, "Repository.InvalidRepository"),
            MessageId::RepositoryAtticRepository => write!(f, "Repository.AtticRepository"),
            MessageId::RepositoryObjectNotFound => write!(f, "Repository.ObjectNotFound"),
            MessageId::CacheBeginTransaction => write!(f, "cache.begin_transaction"),
            MessageId::CacheDownloadChunks => write!(f, "cache.download_chunks"),
            MessageId::CacheCommit => write!(f, "cache.commit"),
            MessageId::CacheSync => write!(f, "cache.sync"),
            MessageId::RepositoryCompactSegments => write!(f, "repository.compact_segments"),
            MessageId::RepositoryReplaySegments => write!(f, "repository.replay_segments"),
            MessageId::RepositoryCheck => write!(f, "repository.check"),
            MessageId::CheckVerifyData => write!(f, "check.verify_data"),
            MessageId::CheckRebuildManifest => write!(f, "check.rebuild_manifest"),
            MessageId::Extract => write!(f, "extract"),
            MessageId::ExtractPermissions => write!(f, "extract.permissions"),
            MessageId::ArchiveDelete => write!(f, "archive.delete"),
            MessageId::ArchiveCalcStats => write!(f, "archive.calc_stats"),
            MessageId::Prune => write!(f, "prune"),
            MessageId::UpgradeConvertSegments => write!(f, "upgrade.convert_segments"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LoggingMessage;

    #[test]
    fn test_log_message_parse_mount() {
        let message = "fusermount: entry for /home/david/fake-directory not found in /etc/mtab";
        let log = LoggingMessage::from_str(message);
        assert_eq!(
            LoggingMessage::UMountError(message.to_string()),
            log.expect("Expected LoggingMessage::UMountError to be parsed correctly")
        )
    }
}
